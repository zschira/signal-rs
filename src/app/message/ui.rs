use super::MessageObject;
use crate::app::App;
use gtk::prelude::*;
use gtk::{Align, Box as Box_, EmojiChooser, GestureClick, Justification, Label, Popover};
use gtk::glib::{self, clone, MainContext};

use signald::types::{JsonAddressV1, JsonReactionV1, MarkReadRequestV1,
                     ReactRequestV1, SignaldTypes};

use std::rc::Rc;

use crate::database;
use crate::models::Message;
use crate::app::message::url_detect::find_url;

impl App {
    pub fn message_ui(self: Rc<App>, msg: MessageObject, msg_box: Box_) {
        let number = msg
            .property("number")
            .expect("The property needs to exist and be readable.")
            .get::<Option<String>>()
            .expect("The property needs to be of type String");

        let from_me = msg
            .property("from-me")
            .expect("The property needs to exist and be readable.")
            .get::<bool>()
            .expect("The property needs to be of type bool");

        let timestamp = msg
            .property("timestamp")
            .expect("The property needs to exist and be readable.")
            .get::<i64>()
            .expect("The property needs to be of type bool");

        let groupid = msg
            .property("groupid")
            .expect("The property needs to exist and be readable.")
            .get::<Option<String>>()
            .expect("The property needs to be of type bool");

        let msg = database::get_message(
            &self.db.lock().unwrap(),
            timestamp,
            number,
            from_me,
            groupid
        );

        if from_me {
            msg_box.set_halign(Align::End);
            msg_box.set_css_classes(&["messageSent"]);
            msg_box.set_margin_start(200);
        } else {
            msg_box.set_halign(Align::Start);
            msg_box.set_css_classes(&["messageReceived"]);
            msg_box.set_margin_end(200);
        }

        if let Some(attachments) = &msg.attachments {
            msg_box.append(&self.clone().new_media_viewer(attachments));
        }

        let text = find_url(&msg.body);

        let label = Label::builder()
            .wrap(true)
            .css_classes(vec!["messageText".to_owned()])
            .justify(Justification::Left)
            .halign(Align::Start)
            .margin_bottom(5)
            .margin_top(5)
            .margin_start(5)
            .margin_end(5)
            .build();

        label.set_markup(text.as_str());

        msg_box.append(&label);

        self.clone().mark_read(&msg);

        let right_click = GestureClick::builder()
            .button(3)
            .build();

        msg_box.add_controller(&right_click);

        let reaction = self.clone().get_reaction_menu(&msg_box, msg);

        right_click.connect_pressed(clone!(@weak msg_box, @weak reaction => 
            move|_,_,_,_| {
                reaction.popup();
            }
        ));
    }

    fn get_reaction_menu(self: Rc<App>, msg_box: &Box_, msg: Message) -> EmojiChooser {
        let reaction_selector = EmojiChooser::builder()
            .position(gtk::PositionType::Bottom)
            .build();

        let reactions = Box_::new(gtk::Orientation::Horizontal, 3);
        let reaction = Popover::builder()
            .autohide(false)
            .child(&reactions)
            .has_arrow(false)
            .position(gtk::PositionType::Bottom)
            .halign(Align::End)
            .build();

        reaction.set_parent(msg_box);

        reaction_selector.connect_emoji_picked(clone!(@strong self as app, @weak reactions, @weak reaction, @weak msg_box =>
            move |selector, emoji| {
                let label = Label::new(Some(emoji));
                reactions.append(&label);
                reaction.popup();

                selector.popdown();
                msg_box.set_margin_bottom(35);

                app.clone().add_reaction(&msg, emoji);
            }
        ));

        reaction_selector.set_parent(msg_box);

        reaction_selector
    }

    fn add_reaction(self: Rc<App>, msg: &Message, emoji: &str) {
        let reaction_request = create_react_request(msg, &self.account.borrow(), emoji);
        MainContext::default().spawn_local(clone!(@strong self as app => async move {
            app.clone().dispatch(
                "react",
                SignaldTypes::ReactRequestV1(
                    reaction_request
                )
            ).await;
        }));
    }

    fn mark_read(self: Rc<App>, msg: &Message) {
        if !msg.is_read && !msg.from_me {
            let timestamp = msg.timestamp;
            let number = msg.number.as_ref().unwrap().clone();
            MainContext::default().spawn_local(clone!(@strong self as app => async move {
                app.clone().dispatch(
                    "mark_read",
                    SignaldTypes::MarkReadRequestV1(
                        MarkReadRequestV1 {
                            account: Some(app.account.borrow().clone()),
                            timestamps: Some(vec![timestamp]),
                            to: Some(JsonAddressV1 {
                                number: Some(number),
                                relay: None,
                                uuid: None
                            }),
                            when: Some(
                                chrono::offset::Local::now().timestamp_millis()
                            ),
                        }
                    )
                ).await;
            }));

            database::read_message(
                &self.db.lock().unwrap(), 
                msg.timestamp,
                msg.number.as_ref().unwrap().clone()
            );
        }
    }
}

fn create_react_request(msg: &Message, account: &String, emoji: &str) -> ReactRequestV1 {
    ReactRequestV1 {
        reaction: Some(JsonReactionV1 {
            emoji: Some(emoji.to_owned()),
            remove: Some(false),
            target_author: Some(JsonAddressV1 {
                number: msg.number.clone().or(
                    Some(account.clone())
                ),
                uuid: None,
                relay: None
            }),
            target_sent_timestamp: Some(msg.timestamp)
        }),
        recipient_address: msg.number.as_ref().map(|number| {
            JsonAddressV1 {
                number: Some(number.clone()),
                uuid: None,
                relay: None
            }
        }),
        recipient_group_id: msg.groupid.clone(),
        timestamp: Some(chrono::offset::Local::now().timestamp_millis()),
        username: Some(account.clone())

    }
}
