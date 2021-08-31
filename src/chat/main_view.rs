use gtk::prelude::*;
use gtk::{Button, Box as Box_, Picture, Label, HeaderBar, Orientation};
use gtk::glib::{self, clone, MainContext};

use signald::types::{AccountV1, SignaldTypes, ProfileV1, SubscribeRequestV1};

use crate::chat::Chat;
use crate::chat::conversation::{conversation_ui, Conversation};

impl Chat {
    pub fn main_view_ui(&self) -> Box_ {
        let vbox = Box_::new(Orientation::Vertical, 5);

        let header = HeaderBar::builder()
            .name("Signal")
            .show_title_buttons(true)
            .build();

        vbox.append(&header);

        self.conversations.iter().for_each(|conversation| {
            let label = match conversation.as_ref() {
                Conversation::Individual(individual) => individual.name.as_ref().unwrap(),
                Conversation::Group(group) => group.title.as_ref().unwrap()
            };

            let label = Label::builder()
                .label(&format!("{}", label))
                .css_classes(vec!["label1".to_owned()])
                .halign(gtk::Align::Start)
                .build();

            let conv_button = Button::builder()
                .child(&label)
                .build();

            let app = self.app.clone();
            let account = self.account.clone();
            conv_button.connect_clicked(clone!(@strong conversation => move |_| {
                app.update_ui(&conversation_ui(app.clone(), account.clone(), conversation.clone()));
            }));

            vbox.append(&conv_button);
        });

        vbox
    }
}

pub fn loading() -> Label {
    Label::builder()
        .label("Loading...")
        .css_classes(vec!["label1".to_owned()])
        .halign(gtk::Align::Center)
        .build()
}
