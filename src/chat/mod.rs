use std::rc::Rc;
use signald::types::{SignaldTypes, ListAccountsRequestV1, ListContactsRequestV1,
                     ListGroupsRequestV1, RequestSyncRequestV1, SubscribeRequestV1};

use async_std::channel::bounded;

use crate::app::App;

pub mod link_device;
pub mod load_app;
pub mod main_view;
pub mod conversation;

pub struct Chat {
    app: Rc<App>,
    account: String,
    conversations: Vec<Rc<conversation::Conversation>>
}

impl Chat {
    pub async fn new(app: Rc<App>) -> Self {
        let account_list = app.dispatch(
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
                    app.update_ui(&link_device::build_ui(app.clone(), sender));
                    receiver.recv().await.expect("Can't get account number")
                }
            };

            let conversations = get_conversations(
                app.clone(),
                &account
            ).await;

            let chat = Chat {
                app: app.clone(),
                account: account.clone(),
                conversations
            };

            app.update_ui(&chat.main_view_ui());

            app.dispatch(
                "subscribe",
                SignaldTypes::SubscribeRequestV1(
                    SubscribeRequestV1 {
                        account: Some(account.clone())
                    }
                )
            ).await;

            app.dispatch(
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

            chat
        } else {
            panic!("Incorrect return type from list_accounts call");
        }
    }
}

async fn get_conversations(app: Rc<App>, account: &String) -> Vec<Rc<conversation::Conversation>> {
    let contacts = app.dispatch(
        "list_contacts",
        SignaldTypes::ListContactsRequestV1(
            ListContactsRequestV1 {
                account: Some(account.clone()),
                async_: Some(true)
            }
        )
    ).await;

    let mut conversations = get_profiles(contacts);

    let groups = app.dispatch(
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

fn get_profiles(contacts: SignaldTypes) -> Vec<Rc<conversation::Conversation>> {
    if let SignaldTypes::ProfileListV1(profiles) = contacts {
        profiles.profiles.unwrap().drain(..).map(|profile| {
            Rc::new(conversation::Conversation::Individual(profile))
        }).collect()
    } else if let SignaldTypes::GroupListV1(groups) = contacts {
        groups.groups.unwrap().drain(..).map(|group| {
            Rc::new(conversation::Conversation::Group(group))
        }).collect()
    } else {
        panic!("Wrong type");
    }
}
