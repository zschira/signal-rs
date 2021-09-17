use crate::models::NewMessage;

pub struct Reaction {
    emoji: String,
    author: String,
    groupid: String,
    message_timestamp: String
}

pub enum Notification {
    NewMessage(NewMessage),
    Reaction(Reaction)
}
