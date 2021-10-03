use gtk::prelude::*;
use gtk::{Align, Button, Box as Box_, HeaderBar, Orientation, Label, ListView, 
          PolicyType, NoSelection, ScrolledWindow, SignalListItemFactory};
use gtk::gio;
use gtk::glib::{self, clone, MainContext};
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use diesel::sqlite::SqliteConnection;
use std::collections::HashMap;

use signald::types::{ProfileV1, JsonAddressV1, JsonGroupV2InfoV1, MarkReadRequestV1,
                     SignaldTypes};

use crate::app::App;
use crate::database;
use crate::app::message::MessageObject;
use crate::signal_type_utils::*;
use crate::models::NewMessage;

pub enum ConversationType {
    Individual(ProfileV1),
    Group(JsonGroupV2InfoV1)
}

pub struct Conversation {
    pub conversation_type: ConversationType,
    pub name: String,
    number: Option<String>,
    groupid: Option<String>,
    pub model: RefCell<Option<gio::ListStore>>,
    pub typing: RefCell<bool>,
    pub last_message_time: RefCell<i64>,
    pub is_active: RefCell<bool>,
    pub new_msgs: RefCell<usize>,
    pub unread: RefCell<HashMap<String, Vec<i64>>>
}

impl Conversation {
    pub fn new_individual(profile: ProfileV1, db: &SqliteConnection) -> Option<Self> {
        // Ask for profile from signal if known profile is incomplete
        let name = profile.get_name();
        let number = profile.address.get_number();
        let (new_msgs, unread) = database::get_unread(db, Some(&number), None);

        if name.is_empty() {
            None
        } else {
            Some(Conversation {
                conversation_type: ConversationType::Individual(profile),
                name,
                number: Some(number),
                groupid: None,
                model: RefCell::new(None),
                typing: RefCell::new(false),
                last_message_time: RefCell::new(i64::MIN),
                is_active: RefCell::new(false),
                new_msgs: RefCell::new(new_msgs),
                unread: RefCell::new(unread)
            })
        }
    }

    pub fn new_group(group: JsonGroupV2InfoV1, db: &SqliteConnection) -> Option<Self> {
        let name = group.title.unwrap_clone();
        let groupid = group.id.unwrap_clone();
        let (new_msgs, unread) = database::get_unread(db, None, Some(&groupid));

        if name.is_empty() {
            None
        } else {
            Some(Conversation {
                conversation_type: ConversationType::Group(group),
                name,
                number: None,
                groupid: Some(groupid),
                model: RefCell::new(None),
                typing: RefCell::new(false),
                last_message_time: RefCell::new(i64::MIN),
                is_active: RefCell::new(false),
                new_msgs: RefCell::new(new_msgs),
                unread: RefCell::new(unread)
            })
        }
    }

    pub fn set_last_message(&self, db: &SqliteConnection) {
        let msg = database::get_most_recent_message(
            db,
            &self.number,
            &self.groupid
        );
        if let Some(msg) = msg {
            self.last_message_time.replace(msg.timestamp);
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

    pub fn notify_msg(&self, msg: NewMessage) {
        if let Some(model) = &*self.model.borrow() {
            model.append(&MessageObject::new_sent(&msg));
        }

        if !(*self.is_active.borrow()) {
            self.new_msgs.replace_with(|&mut num_msgs| num_msgs + 1);
            let unread = &mut *self.unread.borrow_mut();
            let number = msg.number.unwrap();
            if let Some(timestamps) = unread.get_mut(&number) {
                timestamps.push(msg.timestamp);
            } else {
                unread.insert(number, vec![msg.timestamp]);
            }
        }
    }
}

impl App {
    pub fn conversation_ui(self: Rc<App>, conversation: Rc<Conversation>) {
        // Mark messages read
        MainContext::default().spawn_local(clone!(@strong self as app, @strong conversation => async move {
            app.read_messages(conversation).await;
        }));

        conversation.is_active.replace(true);
        let vbox = Box_::new(Orientation::Vertical, 5);

        let msg_box = self.clone().message_input_ui(conversation.clone());

        let header = HeaderBar::builder()
            .title_widget(&self.clone().get_header_widget(conversation.clone()))
            .show_title_buttons(true)
            .decoration_layout("icon,menu:close")
            .build();

        vbox.append(&header);
        vbox.append(&self.clone().get_messages(conversation.clone()));
        vbox.append(&msg_box);

        self.clone().update_ui(&vbox, "conversation");
    }

    async fn read_messages(self: Rc<App>, conversation: Rc<Conversation>) {
        conversation.new_msgs.replace(0);
        for (number, timestamps) in (*conversation.unread.borrow_mut()).drain() {
            database::read_msgs(&self.db.lock().unwrap(), &timestamps, &number);
            self.clone().dispatch(
                "mark_read",
                SignaldTypes::MarkReadRequestV1(
                    MarkReadRequestV1 {
                        account: Some(self.account.borrow().clone()),
                        timestamps: Some(timestamps),
                        to: JsonAddressV1::from_number(number),
                        when: Some(chrono::offset::Local::now().timestamp_millis())
                    }
                )
            ).await;
        }
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

            app.clone().message_ui(msg, msg_box);
        }));

        let selection_model = NoSelection::new(Some(&model));
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

    fn get_header_widget(self: Rc<App>, conversation: Rc<Conversation>) -> Box_ {
        let hbox = Box_::new(Orientation::Horizontal, 3);
        let back_button = Button::builder()
            .icon_name("go-previous")
            .build();

        back_button.connect_clicked(clone!(@strong self as app, @strong conversation => move |_| {
            conversation.is_active.replace(false);
            app.update_ui(&app.clone().main_view_ui(), "main_view");
        }));

        let name = Label::builder()
            .label(conversation.get_name())
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
