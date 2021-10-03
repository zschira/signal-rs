use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::collections::HashMap;
use signald::types::{SignaldTypes, ListAccountsRequestV1, ListContactsRequestV1,
                     ListGroupsRequestV1, ProfileV1, RequestSyncRequestV1, 
                     SubscribeRequestV1};
use diesel::sqlite::SqliteConnection;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Widget};
use gtk::glib::{clone, MainContext};

use async_std::channel::{bounded, Sender, Receiver};

use crate::signald_bridge::{listen, SignaldInteraction};
use crate::database::establish_connection;
use crate::models::NewMessage;
use crate::signal_type_utils::*;

pub mod link_device;
pub mod load_app;
pub mod main_view;
pub mod conversation;
pub mod message;
pub mod notifications;
pub mod message_input;
mod media_viewer;

use notifications::Notification;
use message::MessageObject;

type ContactMap = HashMap<String, ProfileV1>;

pub struct App {
    account: RefCell<String>,
    window: ApplicationWindow,
    signald_sender: Sender<SignaldInteraction>,
    notification_receiver: Receiver<Notification>,
    conversations: RefCell<Vec<Rc<conversation::Conversation>>>,
    curr_view: RefCell<&'static str>,
    contacts: RefCell<ContactMap>,
    db: Arc<Mutex<SqliteConnection>>
}

impl App {
    pub fn new(application: &Application) -> Rc<Self> {
        let (msg_sender, msg_receiver) = bounded(10);
        let (notification_sender, notification_receiver) = bounded(10);
        let main_context = MainContext::default();
        let db = Arc::new(Mutex::new(establish_connection()));

        main_context.spawn_local(clone!(@strong db => async move {
            listen(db, msg_receiver, notification_sender).await;
        }));

        let app = Rc::new(App {
            account: RefCell::new(String::new()),
            window: ApplicationWindow::new(application),
            signald_sender: msg_sender,
            notification_receiver, 
            conversations: RefCell::new(Vec::new()),
            curr_view: RefCell::new("none"),
            contacts: RefCell::new(HashMap::new()),
            db
        });

        application.connect_activate(clone!(@strong app => move |_| {
            app.window.show()
        }));

        main_context.spawn_local(clone!(@strong app => async move {
            app.initialize().await;
        }));

        app
    }

    pub async fn initialize(self: Rc<App>) {
        let account_list = self.clone().dispatch(
            "list_accounts",
            SignaldTypes::ListAccountsRequestV1(
                ListAccountsRequestV1::default()
            )
        ).await;

        if let SignaldTypes::AccountListV1(account_list)  = account_list {
            let mut accounts = account_list.accounts.unwrap();

            // Assume at most one account is returned, because that's
            // all that's supported as of now
            let account = match accounts.pop() {
                Some(account) => account.account_id.unwrap(),
                None => {
                    let (sender, receiver) = bounded(1);
                    self.update_ui(
                        &link_device::build_ui(self.clone(), sender),
                        "new_device"
                    );
                    receiver.recv().await.expect("Can't get account number")
                }
            };

            *self.account.borrow_mut() = account.clone();

            *self.conversations.borrow_mut() = self.clone()
                .get_conversations(&account).await;

            self.clone().order_conversations();
            self.update_ui(&self.clone().main_view_ui(), "main_view");

            self.clone().dispatch(
                "subscribe",
                SignaldTypes::SubscribeRequestV1(
                    SubscribeRequestV1 {
                        account: Some(account.clone())
                    }
                )
            ).await;

            self.clone().dispatch(
                "request_sync",
                SignaldTypes::RequestSyncRequestV1(
                    RequestSyncRequestV1 {
                        account: Some(account),
                        groups: Some(true),
                        configuration: Some(true),
                        contacts: Some(true),
                        blocked: Some(true)
                    }
                )
            ).await;
        } else {
            panic!("Incorrect return type from list_accounts call");
        }

        self.handle_notifications().await;
    }

    async fn handle_notifications(self: Rc<App>) {
        loop {
            let notification = self.clone()
                .notification_receiver.recv()
                .await
                .expect("Failed to receive notification");
            
            match notification {
                Notification::NewMessage(msg) => {
                    self.clone().message_notification(msg).await;
                },
                Notification::Reaction(_reaction) => {
                }
            }
        }
    }

    async fn message_notification(self: Rc<App>, msg: NewMessage) {
        self.clone().order_conversations();
        (*self.conversations.borrow())[0].notify_msg(msg);

        // Redraw main view after adding notification
        if self.curr_view.borrow().eq("main_view") {
            self.clone().update_ui(&self.clone().main_view_ui(), "main_view");
        }

    }

    fn order_conversations(self: Rc<App>) {
        self.conversations.borrow().iter().for_each(|conv| {
            conv.set_last_message(&self.db.lock().unwrap());
        });

        self.conversations.borrow_mut().sort_by(|c1, c2| {
            c2.last_message_time.borrow().cmp(
                &c1.last_message_time.borrow()
            )
        });
    }

    pub async fn dispatch(self: Rc<App>, key: &'static str, msg: SignaldTypes) -> SignaldTypes {
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

    pub fn update_ui<P: IsA<Widget>>(&self, child: &P, view: &'static str) {
        self.curr_view.replace(view);
        self.window.set_child(Some(child));
    }

    async fn get_conversations(self: Rc<App>, account: &String) -> Vec<Rc<conversation::Conversation>> {
        let contacts = self.clone().dispatch(
            "list_contacts",
            SignaldTypes::ListContactsRequestV1(
                ListContactsRequestV1 {
                    account: Some(account.clone()),
                    async_: Some(true)
                }
            )
        ).await;

        let mut conversations = self.clone().get_profiles(
            contacts,
            &mut *self.contacts.borrow_mut()
        );

        let groups = self.clone().dispatch(
            "list_groups",
            SignaldTypes::ListGroupsRequestV1(
                ListGroupsRequestV1 {
                    account: Some(account.clone()),
                }
            )
        ).await;

        conversations.append(&mut self.get_groups(groups));

        conversations
    }

    pub fn get_name(self: Rc<App>, number: &str) -> Option<String> {
        (*self.contacts.borrow()).get(number).as_ref().map(|profile| {
            profile.get_name()
        })
    }

    fn get_profiles(self: Rc<App>, contacts: SignaldTypes, profiles: &mut ContactMap) -> Vec<Rc<conversation::Conversation>> {
        if let SignaldTypes::ProfileListV1(profile_list) = contacts {
            profile_list.profiles.unwrap().drain(..).filter_map(|profile| {
                let number = profile.address.get_number();
                profiles.insert(number, profile.clone());

                let db = self.db.lock().unwrap();
                conversation::Conversation::new_individual(profile, &db).map(|conv| {
                    Rc::new(conv)
                })
            }).collect()
        } else {
            panic!("Wrong type");
        }
    }

    fn get_groups(self: Rc<App>, groups: SignaldTypes) -> Vec<Rc<conversation::Conversation>> {
        if let SignaldTypes::GroupListV1(groups) = groups {
            groups.groups.unwrap().drain(..).filter_map(|group| {
                let db = self.db.lock().unwrap();
                conversation::Conversation::new_group(group, &db).map(|conv| {
                    Rc::new(conv)
                })
            }).collect()
        } else {
            panic!("Wrong type");
        }
    }
}
