use super::schema::{attachments, messages};

#[derive(Queryable)]
pub struct Message {
    pub timestamp: i64,
    pub number: String,
    pub attachments: Option<String>,
    pub body: String,
    pub groupid: Option<String>,
    pub quote_timestamp: Option<i64>,
    pub quote_uuid: Option<Vec<u8>>,
    pub mentions: Option<Vec<u8>>
}

#[derive(Insertable)]
#[table_name = "messages"]
pub struct NewMessage<'a> {
    pub timestamp: i64,
    pub number: String,
    pub attachments: Option<String>,
    pub body: &'a str,
    pub groupid: Option<&'a str>,
    pub quote_timestamp: Option<i64>,
    pub quote_uuid: Option<&'a [u8]>,
    pub mentions: Option<Vec<u8>>
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
