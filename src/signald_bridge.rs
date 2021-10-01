use signald::Signald;
use signald::types::{IncomingMessageV1, JsonSyncMessageV1, SignaldTypes};
use async_std::channel::{Receiver, Sender};
use uuid::Uuid;
use diesel::sqlite::SqliteConnection;
use std::sync::{Arc, Mutex};

use gtk::glib::clone;

use crate::database;
use crate::models::NewMessage;
use crate::app::notifications::Notification;

pub struct SignaldInteraction {
    pub key: &'static str,
    pub msg: SignaldTypes,
    pub response_channel: Option<Sender<SignaldInteraction>>
}

pub async fn listen(db: Arc<Mutex<SqliteConnection>>, receiver: Receiver<SignaldInteraction>, sender: Sender<Notification>) {
    let mut signald = Signald::connect(
        "run/signald.sock",
        move |msg| {
            // Use async std runtime to manage future as that's what's being used
            // by the socket (also will eventually allow gtk main loop to fully
            // block while app is not open)
            async_std::task::spawn(clone!(@strong db, @strong sender => async move {
                message_handler(db, msg, sender).await;
            }));
        }
    ).await.expect("Failed to open socket to signald");

    loop {
        let request = receiver.recv().await.expect("Request channel not working");

        let response = signald.remote_call(
            request.key,
            Uuid::new_v4(),
            request.msg
        ).await;

        match response {
            Ok(response) => {
                if let Some(sender) = request.response_channel {
                    sender.send(
                        SignaldInteraction {
                            key: request.key,
                            msg: response,
                            response_channel: None
                        }
                    ).await.expect("Couldn't return signald response");
                }
            },
            Err(e) => println!("Signald error: {}", e)
        }
    }
}

async fn message_handler(db: Arc<Mutex<SqliteConnection>>, msg: IncomingMessageV1, sender: Sender<Notification>) {
    if msg.data_message.is_some() {
        handle_data_msg(db.clone(), msg.clone(), sender).await;
    } 
    if msg.sync_message.is_some() {
        println!("TYPE: {}", msg.type_.unwrap());
        handle_sync_message(db, msg.sync_message.unwrap());
    }
}

async fn handle_data_msg(db: Arc<Mutex<SqliteConnection>>, envelope: IncomingMessageV1, sender: Sender<Notification>) {
    // Check that message isn't just a reaction
    if envelope.data_message.as_ref().unwrap().reaction.is_some() {
        handle_reaction(db, envelope);
        return;
    }

    // Should only be called if it's determined to contain data_message
    let msg = envelope.data_message.unwrap();
    let timestamp = msg.timestamp.unwrap();
    let number = envelope.source.unwrap().number;
    let attachments = database::store_attachments(&db.lock().unwrap(), msg.attachments.as_ref());

    if !msg.body.is_some() {
        return;
    }

    let body = msg.body.unwrap();
    let groupid = msg.group_v_2.as_ref().map(|group| {
        group.id.as_ref().unwrap().clone()
    });
    let quote_timestamp = msg.quote.as_ref().map(|quote| {
        quote.id.unwrap()
    });
    let quote_author = msg.quote.as_ref().map(|quote| {
        quote.author.as_ref().unwrap().number.as_ref().unwrap().clone()
    });
    let (mentions, mentions_start) = database::convert_mentions(&msg.mentions);

    let msg = NewMessage {
        timestamp,
        number,
        from_me: false,
        is_read: false,
        attachments,
        body,
        groupid,
        quote_timestamp,
        quote_author,
        mentions,
        mentions_start,
        reaction_emojis: None,
        reaction_authors: None
    };

    database::store_message(&db.lock().unwrap(), &msg);
    sender.send(Notification::NewMessage(msg)).await.expect("Failed to send notification");
}

fn handle_sync_message(db: Arc<Mutex<SqliteConnection>>, msg: JsonSyncMessageV1) {
    if let Some(fetch_type) = msg.fetch_type {
        println!("Sync fetch type: {}", fetch_type);
    }

    if let Some(sent) = msg.sent {
        println!("THERE");
        let msg_packet = sent.message.unwrap();
        let destination = sent.destination.unwrap();
        let (mentions, mentions_start) = database::convert_mentions(&msg_packet.mentions);
        let msg = NewMessage {
            timestamp: sent.timestamp.unwrap(),
            number: destination.number,
            from_me: true,
            is_read: false,
            body: msg_packet.body.unwrap(),
            attachments: database::store_attachments(&db.lock().unwrap(), msg_packet.attachments.as_ref()),
            groupid: msg_packet.group_v_2.as_ref().map(|group| {
                group.id.as_ref().unwrap().clone()
            }),
            quote_timestamp: msg_packet.quote.as_ref().map(|quote| {
                quote.id.unwrap()
            }),
            quote_author: msg_packet.quote.as_ref().map(|quote| {
                quote.author.as_ref().unwrap().number.as_ref().unwrap().clone()
            }),
            mentions,
            mentions_start,
            reaction_emojis: None,
            reaction_authors: None
        };

        database::store_message(&db.lock().unwrap(), &msg);
    }
}

fn handle_reaction(_db: Arc<Mutex<SqliteConnection>>, _envelope: IncomingMessageV1) {
}
