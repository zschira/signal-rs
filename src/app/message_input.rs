use gtk::prelude::*;
use gtk::{Entry, Box as Box_, Button, Orientation};
use gtk::glib::{self, clone, MainContext};

use std::rc::Rc;
use std::sync::{Arc, Mutex};
use chrono;
use diesel::sqlite::SqliteConnection;

use signald::types::{SendRequestV1, SignaldTypes};

use crate::models::NewMessage;
use crate::app::conversation::{Conversation, ConversationType};
use crate::app::App;
use crate::database;
use crate::app::MessageObject;

impl App {
    pub fn message_input_ui(self: Rc<App>, conversation: Rc<Conversation>) -> Box_ {
        let hbox = Box_::builder()
            .orientation(Orientation::Horizontal)
            .build();

        let msg_entry = Entry::builder()
            .hexpand(true)
            .show_emoji_icon(true)
            .build();

        let send_button = Button::builder()
            .icon_name("mail-send")
            .build();

        let app = self.clone();
        send_button.connect_clicked(clone!(@weak msg_entry, @strong conversation, @strong app =>
            move |_| {
                let main_context = MainContext::default();
                let msg_body = msg_entry.text().to_string();
                msg_entry.delete_text(0, -1);

                let msg = construct_message(&app.account.borrow(), conversation.clone(), msg_body);
                store_message(app.db.clone(), &msg, conversation.clone());

                main_context.spawn_local(clone!(@weak msg_entry, @strong app =>
                    async move {
                        app.dispatch(
                            "send",
                            SignaldTypes::SendRequestV1(msg)
                        ).await;
                    }
                ));
            }
        ));

        hbox.append(&msg_entry);
        hbox.append(&send_button);
        hbox
    }
}

fn construct_message(username: &String, conversation: Rc<Conversation>, body: String) -> SendRequestV1 {
    SendRequestV1 {
        username: Some(username.clone()),
        recipient_address: match &conversation.conversation_type {
            ConversationType::Individual(conv) => conv.address.clone(),
            ConversationType::Group(_) => None
        },
        recipient_group_id: match &conversation.conversation_type {
            ConversationType::Group(group) => group.id.clone(),
            ConversationType::Individual(_) => None
        },
        message_body: Some(body),
        attachments: None,
        quote: None,
        timestamp: Some(chrono::offset::Local::now().timestamp_millis()),
        mentions: None
    }
}

fn store_message(db: Arc<Mutex<SqliteConnection>>, msg: &SendRequestV1, conversation: Rc<Conversation>) {
    let (mentions, mentions_start) = database::convert_mentions(&msg.mentions);
    let msg = NewMessage {
        timestamp: msg.timestamp.unwrap(),
        number: msg.recipient_address.as_ref().map(|address| {
            address.number.as_ref().unwrap().clone()
        }),
        from_me: true,
        is_read: false,
        attachments: database::store_attachments(&db.lock().unwrap(), msg.attachments.as_ref()),
        body: msg.message_body.as_ref().unwrap().clone(),
        groupid: msg.recipient_group_id.as_ref().map(|id| id.clone()),
        quote_timestamp: msg.quote.as_ref().map(|quote| quote.id.unwrap()),
        quote_author: msg.quote.as_ref().map(|quote| {
            quote.author.as_ref().unwrap().number.as_ref().unwrap().clone()
        }),
        mentions,
        mentions_start,
        reaction_emojis: None,
        reaction_authors: None
    };

    database::store_message(&db.lock().unwrap(), &msg);

    conversation.model
        .borrow_mut()
        .as_ref()
        .unwrap()
        .append(&MessageObject::new_sent(&msg));
}
