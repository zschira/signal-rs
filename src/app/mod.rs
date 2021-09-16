use std::rc::Rc;
use std::cell::RefCell;
use signald::types::{SignaldTypes, ListAccountsRequestV1, ListContactsRequestV1,
                     ListGroupsRequestV1, RequestSyncRequestV1, SubscribeRequestV1};
use diesel::sqlite::SqliteConnection;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, ScrolledWindow, Widget};
use gtk::glib::{clone, MainContext};

use async_std::channel::{bounded, Sender};

use crate::signald_bridge::{listen, SignaldInteraction};
use crate::database::establish_connection;

pub mod link_device;
pub mod load_app;
pub mod main_view;
pub mod conversation;
pub mod message;

pub struct App {
    account: RefCell<String>,
    window: ApplicationWindow,
    signald_sender: Sender<SignaldInteraction>,
    conversations: RefCell<Vec<Rc<conversation::Conversation>>>,
    main_view: RefCell<Option<Rc<ScrolledWindow>>>,
    db: SqliteConnection
}

impl App {
    pub fn new(application: &Application) -> Rc<Self> {
        let (sender, receiver) = bounded(10);
        let main_context = MainContext::default();

        main_context.spawn_local(async move {
            listen(receiver).await;
        });

        let app = Rc::new(App {
            account: RefCell::new(String::new()),
            window: ApplicationWindow::new(application),
            signald_sender: sender,
            conversations: RefCell::new(Vec::new()),
            main_view: RefCell::new(None),
            db: establish_connection()
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
                    self.update_ui(&link_device::build_ui(self.clone(), sender));
                    receiver.recv().await.expect("Can't get account number")
                }
            };

            *self.account.borrow_mut() = account.clone();

            *self.conversations.borrow_mut() = self.clone()
                .get_conversations(&account).await;

            self.update_ui(self.clone().main_view_ui().as_ref());

            self.clone().dispatch(
                "subscribe",
                SignaldTypes::SubscribeRequestV1(
                    SubscribeRequestV1 {
                        account: Some(account.clone())
                    }
                )
            ).await;

            self.dispatch(
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

    pub fn update_ui<P: IsA<Widget>>(&self, child: &P) {
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

        let mut conversations = get_profiles(contacts);

        let groups = self.dispatch(
            "list_groups",
            SignaldTypes::ListGroupsRequestV1(
                ListGroupsRequestV1 {
                    account: Some(account.clone()),
                }
            )
        ).await;

        conversations.append(&mut get_profiles(groups));

        conversations
    }
}

fn get_profiles(contacts: SignaldTypes) -> Vec<Rc<conversation::Conversation>> {
    if let SignaldTypes::ProfileListV1(profiles) = contacts {
        profiles.profiles.unwrap().drain(..).map(|profile| {
            Rc::new(conversation::Conversation::new_individual(profile))
        }).collect()
    } else if let SignaldTypes::GroupListV1(groups) = contacts {
        groups.groups.unwrap().drain(..).map(|group| {
            Rc::new(conversation::Conversation::new_group(group))
        }).collect()
    } else {
        panic!("Wrong type");
    }
}
