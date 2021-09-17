use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use signald::types::{SignaldTypes, ListAccountsRequestV1, ListContactsRequestV1,
                     ListGroupsRequestV1, RequestSyncRequestV1, SubscribeRequestV1};
use diesel::sqlite::SqliteConnection;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, ScrolledWindow, Widget};
use gtk::glib::{clone, MainContext};

use async_std::channel::{bounded, Sender, Receiver};

use crate::signald_bridge::{listen, SignaldInteraction};
use crate::database::establish_connection;
use crate::models::NewMessage;

pub mod link_device;
pub mod load_app;
pub mod main_view;
pub mod conversation;
pub mod message;
pub mod notifications;

use notifications::Notification;
use conversation::ConversationType;
use message::MessageObject;

pub struct App {
    account: RefCell<String>,
    window: ApplicationWindow,
    signald_sender: Sender<SignaldInteraction>,
    notification_receiver: Receiver<Notification>,
    conversations: RefCell<Vec<Rc<conversation::Conversation>>>,
    active_conversation: RefCell<Option<Rc<conversation::Conversation>>>,
    curr_view: RefCell<&'static str>,
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
            active_conversation: RefCell::new(None),
            curr_view: RefCell::new("none"),
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
        if self.curr_view.borrow().eq("main_view") {
            self.clone().update_ui(&self.clone().main_view_ui(), "main_view");
        }

        if let Some(conv) = self.active_conversation.borrow().as_ref() {
            let notify_ui = match &conv.as_ref().conversation_type {
                ConversationType::Individual(profile) => {
                    let number = profile.address
                        .as_ref()
                        .unwrap()
                        .number
                        .as_ref()
                        .unwrap();

                    let msg_number = msg.number.as_ref()
                        .map(|msg| msg.as_str())
                        .unwrap_or_default();

                    number.as_str() == msg_number
                },
                ConversationType::Group(group) => {
                    let groupid = group.id.as_ref().unwrap();
                    let msg_groupid = msg.groupid.as_ref()
                        .map(|groupid| groupid.as_str())
                        .unwrap_or_default();

                    groupid.as_str() == msg_groupid
                }
            };

            if notify_ui {
                conv.as_ref()
                    .model
                    .borrow_mut()
                    .as_ref()
                    .unwrap()
                    .append(&MessageObject::new_sent(&msg));
            }
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
