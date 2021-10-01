use gtk::prelude::*;
use gtk::{Align, Button, Box as Box_, HeaderBar, Orientation, Label, ListView, 
          PolicyType, NoSelection, ScrolledWindow, SignalListItemFactory};
use gtk::glib::clone;
use gtk::gio;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use diesel::sqlite::SqliteConnection;

use signald::types::{ProfileV1, JsonGroupV2InfoV1};

use crate::app::App;
use crate::database;
use crate::app::message::MessageObject;

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
}

impl Conversation {
    pub fn new_individual(profile: ProfileV1) -> Option<Self> {
        // Ask for profile from signal if known profile is incomplete
        let name = match profile.name.as_ref().unwrap().is_empty() {
            false => profile.name.as_ref().unwrap().clone(),
            true => {
                profile.profile_name.as_ref().map(|name| {
                    name.clone()
                }).unwrap_or_default()
            }
        };
        let number = profile.address.as_ref().unwrap().number.as_ref().unwrap().clone();

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
                last_message_time: RefCell::new(i64::MIN)
            })
        }
    }

    pub fn new_group(group: JsonGroupV2InfoV1) -> Option<Self> {
        let name = group.title.as_ref().unwrap().clone();
        let groupid = group.id.as_ref().unwrap().clone();

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
                last_message_time: RefCell::new(i64::MIN)
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

    fn get_header_widget(&self, app: Rc<App>) -> Box_ {
        let hbox = Box_::new(Orientation::Horizontal, 3);
        let back_button = Button::builder()
            .icon_name("go-previous")
            .build();

        back_button.connect_clicked(move |_| {
            app.update_ui(&app.clone().main_view_ui(), "main_view");
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

        let msg_box = self.clone().message_input_ui(conversation.clone());

        let header = HeaderBar::builder()
            .title_widget(&conversation.get_header_widget(self.clone()))
            .show_title_buttons(true)
            .decoration_layout("icon,menu:close")
            .build();

        vbox.append(&header);
        vbox.append(&self.clone().get_messages(conversation.clone()));
        vbox.append(&msg_box);

        self.clone().update_ui(&vbox, "conversation");

        // Indicate that current coversation is active
        self.active_conversation.replace(Some(conversation));
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

        //let reaction = EmojiChooser::new();

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

}
