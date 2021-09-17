use super::schema::{attachments, messages};

#[derive(Queryable, Default)]
pub struct Message {
    pub timestamp: i64,
    pub number: Option<String>,
    pub from_me: bool,
    pub is_read: bool,
    pub attachments: Option<String>,
    pub body: String,
    pub groupid: Option<String>,
    pub quote_timestamp: Option<i64>,
    pub quote_author: Option<String>,
    pub mentions: Option<Vec<u8>>,
    pub mentions_start: Option<Vec<u8>>,
    pub reaction_emojis: Option<String>,
    pub reaction_authors: Option<String>
}

#[derive(Insertable)]
#[table_name = "messages"]
pub struct NewMessage {
    pub timestamp: i64,
    pub number: Option<String>,
    pub from_me: bool,
    pub is_read: bool,
    pub attachments: Option<String>,
    pub body: String,
    pub groupid: Option<String>,
    pub quote_timestamp: Option<i64>,
    pub quote_author: Option<String>,
    pub mentions: Option<Vec<u8>>,
    pub mentions_start: Option<Vec<u8>>,
    pub reaction_emojis: Option<String>,
    pub reaction_authors: Option<String>
}

#[derive(Queryable)]
pub struct Attachment {
    pub id: String,
    pub blurhash: Option<String>,
    pub content_type: String,
    pub filename: Option<String>
}

#[derive(Insertable)]
#[table_name = "attachments"]
pub struct NewAttachment<'a> {
    pub id: &'a str,
    pub blurhash: Option<&'a str>,
    pub content_type: &'a str,
    pub filename: Option<&'a str>
}


