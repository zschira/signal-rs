use signald::Signald;
use signald::types::{JsonAttachmentV0, IncomingMessageV1, SignaldTypes};
use async_std::channel::{Receiver, Sender};
use uuid::Uuid;
use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;

use crate::database::establish_connection;
use crate::models::{NewAttachment, NewMessage};

use crate::schema::{attachments, messages};

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
    let attachments = msg.attachments.map(|attachments| {
        attachments.iter().fold(String::new(), |acc, attachment| {
            handle_attachment(db, attachment);
            format!("{}\n{}", acc, attachment.id.as_ref().unwrap())
        })
    });
    let body = msg.body.as_ref().unwrap();
    let groupid = msg.group_v_2.as_ref().map(|group| {
        group.id.as_ref().unwrap().as_str()
    });
    let quote_timestamp = msg.quote.as_ref().map(|quote| {
        quote.id.unwrap()
    });
    let quote_uuid = msg.quote.map(|quote| {
        Uuid::parse_str(
            quote.author
                .unwrap()
                .uuid
                .as_ref()
                .unwrap()
        ).unwrap()
    });
    let quote_uuid = quote_uuid.as_ref().map(|uuid| &uuid.as_bytes()[..]);
    let mentions = msg.mentions.as_ref().map(|mentions| {
        mentions.iter().fold(Vec::new(), |mut acc, mention| {
            acc.extend_from_slice(
                Uuid::parse_str(mention.uuid.as_ref().unwrap()).unwrap().as_bytes()
            );
            acc
        })
    });

    let msg = NewMessage {
        timestamp,
        number,
        attachments,
        body,
        groupid,
        quote_timestamp,
        quote_uuid,
        mentions
    };

    diesel::insert_into(messages::table)
        .values(&msg)
        .execute(db)
        .expect("Failed to insert message into db");
}

fn handle_reaction(_db: &SqliteConnection, _envelope: IncomingMessageV1) {
}

fn handle_attachment(db: &SqliteConnection, attachment: &JsonAttachmentV0) {
    let id = attachment.id.as_ref().unwrap().as_str();
    let blurhash = attachment.blurhash.as_ref().map(|blurhash| {
        blurhash.as_str()
    });
    let content_type = attachment.content_type.as_ref().unwrap().as_str();
    let filename = attachment.stored_filename.as_ref().map(|filename| {
        filename.as_str()
    });

    let attachment = NewAttachment {
        id,
        blurhash,
        content_type,
        filename
    };

    diesel::insert_into(attachments::table)
        .values(&attachment)
        .execute(db)
        .expect("Failed to insert attachment into db");
}
