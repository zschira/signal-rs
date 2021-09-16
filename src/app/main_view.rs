use gtk::prelude::*;
use gtk::{Button, Box as Box_, Label, HeaderBar, Orientation, ScrolledWindow, PolicyType};
use gtk::glib::clone;
use std::rc::Rc;

use crate::app::conversation::ConversationType;
use crate::app::App;

impl App {
    pub fn main_view_ui(self: Rc<App>) -> Rc<ScrolledWindow> {
        if self.main_view.borrow().is_some() {
            return self.main_view.borrow().as_ref().unwrap().clone();
        }

        let vbox = Box_::new(Orientation::Vertical, 5);

        let header = HeaderBar::builder()
            .name("Signal")
            .show_title_buttons(true)
            .build();

        vbox.append(&header);

        self.conversations.borrow().iter().for_each(|conversation| {
            let label = match &conversation.conversation_type {
                ConversationType::Individual(individual) => individual.name.as_ref().unwrap().clone(),
                ConversationType::Group(group) => group.title.as_ref().unwrap().clone()
            };

            let label = Label::builder()
                .label(&format!("{}", label))
                .css_classes(vec!["label1".to_owned()])
                .halign(gtk::Align::Start)
                .build();

            let conv_button = Button::builder()
                .child(&label)
                .build();

            let app = self.clone();
            conv_button.connect_clicked(clone!(@strong conversation => move |_| {
                &app.clone().conversation_ui(conversation.clone());
            }));

            vbox.append(&conv_button);
        });

        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .child(&vbox)
            .build();

        Rc::new(scroll)
    }
}

pub fn loading() -> Label {
    Label::builder()
        .label("Loading...")
        .css_classes(vec!["label1".to_owned()])
        .halign(gtk::Align::Center)
        .build()
}
