use signald::Signald;
use signald::types::{IncomingMessageV1, JsonSyncMessageV1, SignaldTypes};
use async_std::channel::{Receiver, Sender};
use uuid::Uuid;
use diesel::sqlite::SqliteConnection;

use crate::database::{self, establish_connection};
use crate::models::NewMessage;

pub struct SignaldInteraction {
    pub key: &'static str,
    pub msg: SignaldTypes,
    pub response_channel: Option<Sender<SignaldInteraction>>
}

pub async fn listen(receiver: Receiver<SignaldInteraction>) {
    let db = establish_connection();

    let mut signald = Signald::connect(
        "run/signald.sock",
        move |msg| {
            message_handler(&db, msg);
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

fn message_handler(db: &SqliteConnection, msg: IncomingMessageV1) {
    if msg.data_message.is_some() {
        handle_data_msg(db, msg);
    } else if msg.sync_message.is_some() {
        handle_sync_message(db, msg.sync_message.unwrap());
    }
}

fn handle_data_msg(db: &SqliteConnection, envelope: IncomingMessageV1) {
    // Check that message isn't just a reaction
    if envelope.data_message.as_ref().unwrap().reaction.is_some() {
        handle_reaction(db, envelope);
        return;
    }

    // Should only be called if it's determined to contain data_message
    let msg = envelope.data_message.unwrap();
    let timestamp = msg.timestamp.unwrap();
    let number = envelope.source.unwrap().number.unwrap();
    let attachments = database::store_attachments(db, msg.attachments.as_ref());

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

    database::store_message(db, msg);
}

fn handle_sync_message(_db: &SqliteConnection, _msg: JsonSyncMessageV1) {
}

fn handle_reaction(_db: &SqliteConnection, _envelope: IncomingMessageV1) {
}
