use gtk::prelude::*;
use gtk::{Align, Button, Box as Box_, Entry, HeaderBar, Justification, 
          Orientation, Label, ListView, PolicyType, SingleSelection, ScrolledWindow,
          SignalListItemFactory};
use gtk::glib::{self, clone, MainContext};
use gtk::gio;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};
use chrono;
use diesel::sqlite::SqliteConnection;

use signald::types::{ProfileV1, JsonGroupV2InfoV1, SendRequestV1, SignaldTypes};

use crate::app::App;
use crate::models::NewMessage;
use crate::database;
use crate::app::message::MessageObject;

pub enum ConversationType {
    Individual(ProfileV1),
    Group(JsonGroupV2InfoV1)
}

pub struct Conversation {
    pub conversation_type: ConversationType,
    pub model: RefCell<Option<gio::ListStore>>,
    pub typing: RefCell<bool>
}

impl Conversation {
    pub fn new_individual(profile: ProfileV1) -> Self {
        Conversation {
            conversation_type: ConversationType::Individual(profile),
            model: RefCell::new(None),
            typing: RefCell::new(false)
        }
    }

    pub fn new_group(group: JsonGroupV2InfoV1) -> Self {
        Conversation {
            conversation_type: ConversationType::Group(group),
            model: RefCell::new(None),
            typing: RefCell::new(false)
        }
    }

    fn get_name(&self) -> &str {
        match &self.conversation_type {
            ConversationType::Individual(profile) => {
                profile.name.as_ref().unwrap().as_str()
            },
            ConversationType::Group(group) => {
                group.title.as_ref().unwrap().as_str()
            }
        }
    }

    fn get_header_widget(&self, app: Rc<App>) -> Box_ {
        let hbox = Box_::new(Orientation::Horizontal, 3);
        let back_button = Button::builder()
            .icon_name("go-previous")
            .build();

        back_button.connect_clicked(move |_| {
            app.update_ui(app.clone().main_view_ui().as_ref());
            app.active_conversation.replace(None);
        });

        let name = Label::builder()
            .label(self.get_name())
            .halign(Align::Center)
            .build();

        let video_button = Button::builder()
            .halign(Align::End)
            .icon_name("camera-video")
            .build();

        let call_button = Button::builder()
            .halign(Align::End)
            .icon_name("phone")
            .build();

        let menu_button = Button::builder()
            .halign(Align::End)
            .icon_name("open-menu")
            .build();

        hbox.append(&back_button);
        hbox.append(&name);
        hbox.append(&video_button);
        hbox.append(&call_button);
        hbox.append(&menu_button);
        hbox.set_halign(Align::Start);
        hbox.set_hexpand(true);

        hbox
    }
}

impl App {
    pub fn conversation_ui(self: Rc<App>, conversation: Rc<Conversation>) {
        let vbox = Box_::new(Orientation::Vertical, 5);

        let msg_box = Box_::builder()
            .orientation(Orientation::Horizontal)
            .spacing(2)
            .halign(gtk::Align::Start)
            .build();

        let msg_entry = Entry::builder()
            .halign(gtk::Align::Start)
            .hexpand_set(true)
            .build();

        let send_button = Button::builder()
            .label("Send")
            .halign(gtk::Align::End)
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

        msg_box.append(&msg_entry);
        msg_box.append(&send_button);

        let header = HeaderBar::builder()
            .title_widget(&conversation.get_header_widget(self.clone()))
            .show_title_buttons(true)
            .decoration_layout("icon,menu:close")
            .build();

        vbox.append(&header);
        vbox.append(&self.get_messages(conversation.clone()));
        vbox.append(&msg_box);

        app.update_ui(&vbox);

        // Indicate that current coversation is active
        app.active_conversation.replace(Some(conversation));
    }

    fn get_messages(self: Rc<App>, conversation: Rc<Conversation>) -> ScrolledWindow {
        let model = gio::ListStore::new(MessageObject::static_type());
        let messages = database::query_conversation(&self.db.lock().unwrap(), &conversation.conversation_type);

        for message in messages {
            if !message.body.is_empty() {
                let msg = MessageObject::new(message);

                model.append(&msg);
            }
        }

        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            list_item.set_child(
                Some(
                    &Box_::builder()
                        .orientation(Orientation::Vertical)
                        .spacing(3)
                        .width_request(150)
                        .build()
                )
            );
        });

        factory.connect_bind(clone!(@strong self as app => move |_, list_item| {
            let msg = list_item
                .item()
                .expect("The item has to exist.")
                .downcast::<MessageObject>()
                .expect("The item has to be a MessageObject");

            // Get `Label` from `ListItem`
            let msg_box = list_item
                .child()
                .expect("The child has to exist.")
                .downcast::<Box_>()
                .expect("The child has to be a Box");

            message_ui(msg, app.db.clone(), msg_box);
        }));

        let selection_model = SingleSelection::new(Some(&model));
        let list_view = ListView::new(Some(&selection_model), Some(&factory));

        let window = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never) // Disable horizontal scrolling
            .child(&list_view)
            .vexpand(true)
            .vscrollbar_policy(PolicyType::Always)
            .build();

        let adj = window.vadjustment().unwrap();
        // Getting gtk to start scroll at bottom is weird so I used this hack
        let initialized = Rc::new(Cell::new(false));

        adj.connect_upper_notify(|adj| {
            adj.set_value(adj.upper() - adj.page_size());
        });
        adj.connect_value_changed(move |adj| {
            if !initialized.get() && 
                (adj.value() - adj.lower()).abs() < f64::EPSILON {
                adj.set_value(adj.upper() - adj.page_size());

                initialized.set(true);
            }
        });

        conversation.model.replace(Some(model));

        window
    }
}

fn message_ui(msg: MessageObject, db: Arc<Mutex<SqliteConnection>>, msg_box: Box_) {
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
        &db.lock().unwrap(),
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

    let label = Label::builder()
        .label(msg.body.as_str())
        .wrap(true)
        .css_classes(vec!["messageText".to_owned()])
        .justify(Justification::Left)
        .halign(Align::Start)
        .margin_bottom(5)
        .margin_top(5)
        .margin_start(5)
        .margin_end(5)
        .build();

    msg_box.append(&label);
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
