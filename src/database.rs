use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use std::env;
use std::rc::Rc;
use uuid::Uuid;

use signald::types::{JsonAttachmentV0, JsonMentionV1};

use crate::models::{NewAttachment, NewMessage, Message};
use crate::schema::{attachments, messages};
use crate::app::conversation::ConversationType;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn store_message(db: &SqliteConnection, msg: NewMessage) {
    diesel::insert_into(messages::table)
        .values(&msg)
        .execute(db)
        .expect("Failed to insert message into db");
}

pub fn store_attachments(db: &SqliteConnection, attachments: Option<&Vec<JsonAttachmentV0>>) -> Option<String> {
    attachments.map(|attachments| {
        attachments.iter().fold(String::new(), |mut acc, attachment| {
            store_single_attachment(db, attachment);
            acc.push_str(attachment.id.as_ref().unwrap());
            acc.push('\n');
            acc
        })
    })
}

fn store_single_attachment(db: &SqliteConnection, attachment: &JsonAttachmentV0) {
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

pub fn convert_mentions(mentions: &Option<Vec<JsonMentionV1>>) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
    let mentions_id = mentions.as_ref().map(|mentions| {
        mentions.iter().fold(Vec::new(), |mut acc, mention| {
            acc.extend_from_slice(
                Uuid::parse_str(mention.uuid.as_ref().unwrap()).unwrap().as_bytes()
            );
            acc
        })
    });

    let mentions_start = mentions.as_ref().map(|mentions| {
        mentions.iter().fold(Vec::new(), |mut acc, mention| {
            acc.extend_from_slice(
                &mention.start.unwrap().to_le_bytes()
            );
            acc
        })
    });

    (mentions_id, mentions_start)
}

pub fn query_conversation(db: &SqliteConnection, conversation: &ConversationType) -> Vec<Message> {
    use crate::schema::messages::dsl::*;
    match conversation {
        ConversationType::Individual(profile) => {
            messages.filter(
                number.eq(
                    profile.address.as_ref().unwrap().number.as_ref().unwrap().as_str()
                )
            )
                .filter(groupid.is_null())
                .load(db)
                .expect("Failed to load messages")
        },
        ConversationType::Group(group) => {
            messages.filter(groupid.eq(group.id.as_ref().unwrap().as_str()))
                .load(db)
                .expect("Failed to load messages")
        }
    }
}
