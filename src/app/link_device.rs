use gtk::prelude::*;
use gtk::{Button, Box as Box_, Picture, Label, Entry, Orientation};
use gtk::glib::{self, clone, MainContext};
use async_std::channel::Sender;

use std::rc::Rc;

use qrcode::QrCode;
use image::Luma;

use signald::types::{SignaldTypes, FinishLinkRequestV1, GenerateLinkingURIRequestV1, LinkingURIV1};

use crate::app::App;

async fn handle_clicked(app: Rc<App>) -> Result<LinkingURIV1, &'static str> {
    let linking = app.dispatch(
        "generate_linking_uri",
        SignaldTypes::GenerateLinkingURIRequestV1(
            GenerateLinkingURIRequestV1::default()
        )
    ).await;


    if let SignaldTypes::LinkingURIV1(linking) = linking {
        Ok(linking)
    } else {
        Err("Error generating linking uri")
    }
}

pub fn build_ui(app: Rc<App>, sender: Sender<String>) -> Box_ {
    let vbox = Box_::new(Orientation::Vertical, 5);

    let label = Label::builder()
        .label("Welcome to Signal!")
        .css_classes(vec!["label1".to_owned()])
        .halign(gtk::Align::Start)
        .build();

    let hbox = Box_::builder()
        .orientation(Orientation::Horizontal)
        .spacing(3)
        .halign(gtk::Align::Center)
        .build();

    let button1 = Button::builder()
        .label("Register new number")
        .build();

    button1.connect_clicked(clone!(@strong app => move |_| {
        app.update_ui(&register_ui(app.clone()), "register");
    }));

    let button2 = Button::builder()
        .label("Link device")
        .build();

    button2.connect_clicked(clone!(@strong app, @weak button1 => 
        move |button| {
            let main_context = MainContext::default();

            main_context.spawn_local(clone!(@weak button, @weak button1, @strong app, @strong sender => 
                async move {
                    button.set_sensitive(false);
                    button1.set_sensitive(false);
                    let linking = handle_clicked(app.clone()).await.unwrap();
                    app.update_ui(&link_ui(app.clone(), linking, sender), "link");
                }
            ));
        }
    ));

    hbox.append(&button1);
    hbox.append(&button2);

    vbox.append(&label);
    vbox.append(&hbox);

    vbox
}

async fn finish_link(app: Rc<App>, session_id: String, sender: Sender<String>) {
    let account = app.dispatch(
        "finish_link",
        SignaldTypes::FinishLinkRequestV1(
            FinishLinkRequestV1 {
                session_id: Some(session_id),
                device_name: Some("signal-rs-test".to_owned())
            }
        )
    ).await;

    if let SignaldTypes::AccountV1(account) = account {
        sender.send(account.account_id.unwrap()).await.expect("Channel broken");
        //app.update_ui(&load_app::finish_link_ui(app.clone(), account));
    }
}

fn link_ui(app: Rc<App>, linking: LinkingURIV1, sender: Sender<String>) -> Box_ {
    let vbox = Box_::new(Orientation::Vertical, 5);

    let label = Label::builder()
        .label("Scan QR code on primary device")
        .css_classes(vec!["label1".to_owned()])
        .halign(gtk::Align::Center)
        .build();

    let code = QrCode::new(linking.uri.as_ref().unwrap()).unwrap();
    let image = code.render::<Luma<u8>>().build();
    image.save("/tmp/qrcode.png").unwrap();

    let image = Picture::for_filename("/tmp/qrcode.png");

    let main_context = MainContext::default();
    main_context.spawn_local(clone!(@strong app => async move {
        finish_link(app, linking.session_id.unwrap(), sender).await;
    }));

    vbox.append(&label);
    vbox.append(&image);

    vbox
}

fn register_ui(app: Rc<App>) -> Box_ {
    // The container container.
    let vbox = Box_::new(Orientation::Vertical, 5);

    let label = Label::builder()
        .label("Enter phone number to begin")
        .css_classes(vec!["label1".to_owned()])
        .halign(gtk::Align::Start)
        .build();

    let entry = Entry::builder()
        .build();

    let hbox = Box_::builder()
        .orientation(Orientation::Horizontal)
        .spacing(3)
        .halign(gtk::Align::End)
        .build();

    let button1 = Button::builder()
        .label("Back")
        .build();

    let button2 = Button::builder()
        .label("Next")
        .build();

    hbox.append(&button1);
    hbox.append(&button2);

    vbox.append(&label);
    vbox.append(&entry);
    vbox.append(&hbox);

    vbox
}
