use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Widget};
use gtk::glib::{clone, MainContext};

use async_std::channel::{bounded, Sender};
use signald::types::SignaldTypes;
use std::rc::Rc;

use crate::signald_bridge::*;

pub struct App {
    window: ApplicationWindow,
    signald_sender: Sender<SignaldInteraction>,
}

impl App {
    pub fn new(application: &Application) -> Rc<Self> {
        let (sender, receiver) = bounded(10);
        let main_context = MainContext::default();

        main_context.spawn_local(async move {
            listen(receiver).await;
        });

        let app = Rc::new(App {
            window: ApplicationWindow::new(application),
            signald_sender: sender
        });

        application.connect_activate(clone!(@strong app => move |_| {
            app.window.show()
        }));

        app
    }


    pub async fn dispatch(&self, key: &'static str, msg: SignaldTypes) -> SignaldTypes {
        let (sender, receiver) = bounded(1);

        self.signald_sender.send(
            SignaldInteraction {
                key,
                msg,
                response_channel: Some(sender)
            }
        ).await.expect("Can't interact with signald bridge");

        receiver.recv().await
            .expect("Couldn't receive signald response").msg
    }

    pub fn update_ui<P: IsA<Widget>>(&self, child: &P) {
        self.window.set_child(Some(child));
    }
}
