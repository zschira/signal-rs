use gtk::prelude::*;
use gtk::Label;
use gtk::glib::{self, clone, MainContext};

use signald::types::{AccountV1, SignaldTypes, SubscribeRequestV1};

use crate::app::App;
use std::rc::Rc;

pub fn finish_link_ui(app: Rc<App>, account: AccountV1) -> Label {
    let label = Label::builder()
        .label("Finished linking!")
        .css_classes(vec!["label1".to_owned()])
        .halign(gtk::Align::Center)
        .build();

    let main_context = MainContext::default();
    main_context.spawn_local(clone!(@strong app => async move {
        app.dispatch(
            "subscribe",
            SignaldTypes::SubscribeRequestV1(
                SubscribeRequestV1 {
                    account: Some(account.address.unwrap().number.unwrap())
                }
            )
        ).await;
    }));

    label
}
