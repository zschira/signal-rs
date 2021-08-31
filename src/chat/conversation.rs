use gtk::prelude::*;
use gtk::{Button, Box as Box_, Entry, Orientation};
use gtk::glib::{self, clone, MainContext};
use std::rc::Rc;
use chrono;

use signald::types::{ProfileV1, JsonGroupV2InfoV1, SendRequestV1, SignaldTypes};

use crate::app::App;

pub enum Conversation {
    Individual(ProfileV1),
    Group(JsonGroupV2InfoV1)
}

pub fn conversation_ui(app: Rc<App>, username: String, conversation: Rc<Conversation>) -> Box_ {
    let vbox = Box_::new(Orientation::Vertical, 5);

    let msg_box = Box_::builder()
        .orientation(Orientation::Horizontal)
        .spacing(2)
        .valign(gtk::Align::End)
        .build();

    let msg_entry = Entry::builder()
        .valign(gtk::Align::End)
        .build();

    let send_button = Button::builder()
        .label("Send")
        .halign(gtk::Align::End)
        .build();

    send_button.connect_clicked(clone!(@weak msg_entry, @strong conversation =>
        move |_| {
            let main_context = MainContext::default();
            let msg_body = msg_entry.text().to_string();
            msg_entry.delete_text(0, -1);

            let msg = construct_message(&username, conversation.clone(), msg_body);

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

    msg_box.append(&msg_entry);
    msg_box.append(&send_button);

    vbox.append(&msg_box);

    vbox
}

fn construct_message(username: &String, conversation: Rc<Conversation>, body: String) -> SendRequestV1 {
    SendRequestV1 {
        username: Some(username.clone()),
        recipient_address: match conversation.as_ref() {
            Conversation::Individual(conv) => conv.address.clone(),
            Conversation::Group(_) => None
        },
        recipient_group_id: match conversation.as_ref() {
            Conversation::Group(group) => group.id.clone(),
            Conversation::Individual(_) => None
        },
        message_body: Some(body),
        attachments: None,
        quote: None,
        timestamp: Some(chrono::offset::Local::now().timestamp_millis()),
        mentions: None
    }
}
