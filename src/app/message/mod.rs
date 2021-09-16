mod imp;

use glib::Object;
use gtk::glib;
use crate::models::{NewMessage, Message};

glib::wrapper! {
    pub struct MessageObject(ObjectSubclass<imp::MessageObject>);
}

impl MessageObject {
    pub fn new(msg: Message) -> Self {
        Object::new(
            &[
                ("timestamp", &msg.timestamp),
                ("number", &msg.number),
                ("from-me", &msg.from_me),
                ("groupid", &msg.groupid),
            ]
        ).expect("Failed to create MessageObject")
    }
    
    pub fn new_sent(msg: &NewMessage) -> Self {
        Object::new(
            &[
                ("timestamp", &msg.timestamp),
                ("number", &msg.number),
                ("from-me", &msg.from_me),
                ("groupid", &msg.groupid),
            ]
        ).expect("Failed to create MessageObject")
    }
}
