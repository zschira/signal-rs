use signald::types::{JsonAddressV1, ProfileV1};

pub trait ProfileV1Helpers {
    fn get_name(&self) -> String;
}

impl ProfileV1Helpers for ProfileV1 {
    fn get_name(&self) -> String {
        match self.name.as_ref().unwrap().is_empty() {
            false => self.name.as_ref().unwrap().clone(),
            true => {
                self.profile_name.as_ref().map(|name| {
                    name.clone()
                }).unwrap_or_default()
            }
        }
    }
}

pub trait JsonAddressV1OptionHelpers {
    fn get_number(&self) -> String;
}

pub trait JsonAddressV1Helpers {
    fn from_number(number: String) -> Option<Self> where Self: Sized;
}

impl JsonAddressV1OptionHelpers for Option<JsonAddressV1> {
    fn get_number(&self) -> String {
        self.as_ref()
            .unwrap()
            .number
            .as_ref()
            .unwrap()
            .clone()
    }
}

impl JsonAddressV1Helpers for JsonAddressV1 {
    fn from_number(number: String) -> Option<Self> {
        Some(
            JsonAddressV1 {
                number: Some(number),
                relay: None,
                uuid: None
            }
        )
    }
}

pub trait UnwrapClone<T> {
    fn unwrap_clone(&self) -> T;
}

impl<T: Clone> UnwrapClone<T> for Option<T> {
    fn unwrap_clone(&self) -> T {
        self.as_ref().unwrap().clone()
    }
}
