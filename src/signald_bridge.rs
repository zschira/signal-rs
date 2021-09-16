use signald::Signald;
use signald::types::{IncomingMessageV1, JsonSyncMessageV1, SignaldTypes};
use async_std::channel::{Receiver, Sender};
use uuid::Uuid;
use diesel::sqlite::SqliteConnection;
use std::sync::{Arc, Mutex};

use crate::database;
use crate::models::NewMessage;

pub struct SignaldInteraction {
    pub key: &'static str,
    pub msg: SignaldTypes,
    pub response_channel: Option<Sender<SignaldInteraction>>
}

pub async fn listen(db: Arc<Mutex<SqliteConnection>>, receiver: Receiver<SignaldInteraction>) {
    let mut signald = Signald::connect(
        "run/signald.sock",
        move |msg| {
            message_handler(db.clone(), msg);
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

fn message_handler(db: Arc<Mutex<SqliteConnection>>, msg: IncomingMessageV1) {
    if msg.data_message.is_some() {
        handle_data_msg(db, msg);
    } else if msg.sync_message.is_some() {
        handle_sync_message(db, msg.sync_message.unwrap());
    }
}

fn handle_data_msg(db: Arc<Mutex<SqliteConnection>>, envelope: IncomingMessageV1) {
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

    let body = msg.body.as_ref().unwrap();
    let groupid = msg.group_v_2.as_ref().map(|group| {
        group.id.as_ref().unwrap().as_str()
    });
    let quote_timestamp = msg.quote.as_ref().map(|quote| {
        quote.id.unwrap()
    });
    let quote_author = msg.quote.as_ref().map(|quote| {
        quote.author.as_ref().unwrap().number.as_ref().unwrap().as_str()
    });
    let (mentions, mentions_start) = database::convert_mentions(&msg.mentions);

    let msg = NewMessage {
        timestamp,
        number,
        from_me: false,
        attachments,
        body,
        groupid,
        quote_timestamp,
        quote_author,
        mentions,
        mentions_start
    };

    database::store_message(&db.lock().unwrap(), &msg);
}

fn handle_sync_message(db: Arc<Mutex<SqliteConnection>>, msg: JsonSyncMessageV1) {
    if let Some(sent) = msg.sent {
        let msg_packet = sent.message.unwrap();
        let destination = sent.destination.unwrap();
        let (mentions, mentions_start) = database::convert_mentions(&msg_packet.mentions);
        let msg = NewMessage {
            timestamp: sent.timestamp.unwrap(),
            number: destination.number,
            from_me: true,
            body: msg_packet.body.as_ref().unwrap().as_str(),
            attachments: database::store_attachments(&db.lock().unwrap(), msg_packet.attachments.as_ref()),
            groupid: msg_packet.group_v_2.as_ref().map(|group| {
                group.id.as_ref().unwrap().as_str()
            }),
            quote_timestamp: msg_packet.quote.as_ref().map(|quote| {
                quote.id.unwrap()
            }),
            quote_author: msg_packet.quote.as_ref().map(|quote| {
                quote.author.as_ref().unwrap().number.as_ref().unwrap().as_str()
            }),
            mentions,
            mentions_start
        };

        database::store_message(&db.lock().unwrap(), &msg);
    }
}

fn handle_reaction(_db: Arc<Mutex<SqliteConnection>>, _envelope: IncomingMessageV1) {
}
