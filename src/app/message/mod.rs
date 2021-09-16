mod imp;

use glib::Object;
use gtk::glib;
use uuid::Uuid;

use std::convert::TryInto;
use crate::models::Message;

glib::wrapper! {
    pub struct MessageObject(ObjectSubclass<imp::MessageObject>);
}

impl MessageObject {
    pub fn new(msg: Message) -> Self {
        let mut mentions_string = String::new();
        let mut mentions_start_string = String::new();

        if msg.mentions.is_some() && msg.mentions_start.is_some() {
            let mentions = msg.mentions.unwrap();
            let mentions_start = msg.mentions_start.unwrap();

            for i in 0..mentions.len()/16 {
                if i > 0 {
                    mentions_string.push(',');
                    mentions_start_string.push(',');
                }
                mentions_string.push_str(
                    Uuid::from_slice(&mentions[i*16..i*16+16])
                        .unwrap()
                        .to_string()
                        .as_str()
                );

                mentions_start_string.push_str(
                    i32::from_le_bytes(mentions_start[i*4..i*4+4].try_into().unwrap())
                        .to_string()
                        .as_str()
                );
            }
        }

        Object::new(
            &[
                ("timestamp", &msg.timestamp),
                ("number", &msg.number),
                ("from-me", &msg.from_me),
                ("attachments", &msg.attachments),
                ("body", &msg.body),
                ("groupid", &msg.groupid),
                ("quote-timestamp", &msg.quote_timestamp.unwrap_or(-1i64)),
                ("quote-author", &msg.quote_author),
                ("mentions", &mentions_string),
                ("mentions-start", &mentions_start_string),
            ]
        ).expect("Failed to create MessageObject")
    }
}
