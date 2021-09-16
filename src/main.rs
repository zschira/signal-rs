#[macro_use]
extern crate diesel;
extern crate dotenv;

use gtk::prelude::*;

use gtk::gdk::Display;
use gtk::{
    Application, CssProvider, StyleContext, STYLE_PROVIDER_PRIORITY_APPLICATION
};
use gtk::glib::{clone, MainContext};

mod signald_bridge;
//mod chat;
mod app;
mod database;
mod schema;
mod models;

use crate::app::App;

fn main() {
    let application = Application::new(Some("com.github.zschira.signalrs"), Default::default());
    application.connect_startup(|app| {
        // The CSS "magic" happens here.
        let provider = CssProvider::new();
        provider.load_from_data(include_bytes!("../style/style.css"));
        // We give the CssProvided to the default screen so the CSS rules we added
        // can be applied to our window.
        StyleContext::add_provider_for_display(
            &Display::default().expect("Error initializing gtk css provider."),
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // We build the application UI.
        App::new(app);
    });
    application.run();
}
