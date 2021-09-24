use gtk::prelude::*;
use gtk::{Button, Box as Box_, Label, HeaderBar, Orientation, ScrolledWindow, Picture, PolicyType};
use gtk::glib::clone;
use adw::Avatar;
use std::rc::Rc;

use crate::app::App;
use crate::app::conversation::ConversationType;

impl App {
    pub fn main_view_ui(self: Rc<App>) -> ScrolledWindow {
        let vbox = Box_::new(Orientation::Vertical, 5);

        let header = HeaderBar::builder()
            .name("Signal")
            .show_title_buttons(true)
            .build();

        vbox.append(&header);

        self.conversations.borrow().iter().for_each(|conversation| {
            let label = Label::builder()
                .label(&conversation.name)
                .css_classes(vec!["label1".to_owned()])
                .halign(gtk::Align::Start)
                .build();

            let msg_box = Box_::new(Orientation::Horizontal, 15);
            let avatar = Avatar::builder()
                .text(conversation.name.as_str())
                .size(35)
                .icon_name("face-cool-symbolic")
                .build();

            let pic = match &conversation.conversation_type {
                ConversationType::Individual(individual) => {
                    avatar.set_show_initials(true);
                    individual.avatar.as_ref().map(|avatar| {
                        Picture::for_filename(avatar)
                    })
                },
                ConversationType::Group(group) => {
                    avatar.set_show_initials(false);
                    group.avatar.as_ref().map(|avatar| {
                        Picture::for_filename(avatar)
                    })
                }
            };

            avatar.set_custom_image(pic.map(|pic| {
                pic.paintable()
            }).unwrap_or(None).as_ref());

            msg_box.append(&avatar);
            msg_box.append(&label);

            let conv_button = Button::builder()
                .child(&msg_box)
                .build();

            let app = self.clone();
            conv_button.connect_clicked(clone!(@strong conversation => move |_| {
                &app.clone().conversation_ui(conversation.clone());
            }));

            vbox.append(&conv_button);
        });

        ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .child(&vbox)
            .build()
    }
}

pub fn loading() -> Label {
    Label::builder()
        .label("Loading...")
        .css_classes(vec!["label1".to_owned()])
        .halign(gtk::Align::Center)
        .build()
}
