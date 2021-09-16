use glib::{ParamFlags, ParamSpec, Value};
use gtk::subclass::prelude::*;
use gtk::prelude::*;
use gtk::glib;
use once_cell::sync::Lazy;
use std::cell::RefCell;

#[derive(Default)]
pub struct MessageObject {
    timestamp: RefCell<i64>,
    number: RefCell<String>,
    from_me: RefCell<bool>,
    attachments: RefCell<Option<String>>,
    body: RefCell<String>,
    groupid: RefCell<Option<String>>,
    quote_timestamp: RefCell<i64>,
    quote_author: RefCell<Option<String>>,
    mentions: RefCell<String>,
    mentions_start: RefCell<String>
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for MessageObject {
    const NAME: &'static str = "SignalMessage";
    type Type = super::MessageObject;
    type ParentType = glib::Object;
}

// Trait shared by all GObjects
impl ObjectImpl for MessageObject {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {vec![
            ParamSpec::new_int64(
                // Name
                "timestamp",
                // Nickname
                "timestamp",
                // Short description
                "timestamp",
                // Minimum value
                0,
                // Maximum value
                i64::MAX,
                // Default value
                0,
                // The property can be read and written to
                ParamFlags::READWRITE,
            ),
            ParamSpec::new_string(
                // Name
                "number",
                // Nickname
                "number",
                // Short description
                "number",
                // Default value
                None,
                // The property can be read and written to
                ParamFlags::READWRITE,
            ),
            ParamSpec::new_boolean(
                // Name
                "from-me",
                // Nickname
                "from-me",
                // Short description
                "from-me",
                // Default value
                false,
                // The property can be read and written to
                ParamFlags::READWRITE,
            ),
            ParamSpec::new_string(
                // Name
                "attachments",
                // Nickname
                "attachments",
                // Short description
                "attachments",
                // Default value
                None,
                // The property can be read and written to
                ParamFlags::READWRITE,
            ),
            ParamSpec::new_string(
                // Name
                "body",
                // Nickname
                "body",
                // Short description
                "body",
                // Default value
                None,
                // The property can be read and written to
                ParamFlags::READWRITE,
            ),
            ParamSpec::new_string(
                // Name
                "groupid",
                // Nickname
                "groupid",
                // Short description
                "groupid",
                // Default value
                None,
                // The property can be read and written to
                ParamFlags::READWRITE,
            ),
            ParamSpec::new_int64(
                // Name
                "quote-timestamp",
                // Nickname
                "quote-timestamp",
                // Short description
                "quote-timestamp",
                // Minimum value
                -1,
                // Maximum value
                i64::MAX,
                // Default value
                -1,
                // The property can be read and written to
                ParamFlags::READWRITE,
            ),
            ParamSpec::new_string(
                // Name
                "quote-author",
                // Nickname
                "quote-author",
                // Short description
                "quote-author",
                // Default value
                None,
                // The property can be read and written to
                ParamFlags::READWRITE,
            ),
            ParamSpec::new_string(
                // Name
                "mentions",
                // Nickname
                "mentions",
                // Short description
                "mentions",
                // Default value
                None,
                // The property can be read and written to
                ParamFlags::READWRITE,
            ),
            ParamSpec::new_string(
                // Name
                "mentions-start",
                // Nickname
                "mentions-start",
                // Short description
                "mentions-start",
                // Default value
                None,
                // The property can be read and written to
                ParamFlags::READWRITE,
            ),
        ]});
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "timestamp" => { self.timestamp.replace(value.get().unwrap()); },
            "number" => { self.number.replace(value.get().unwrap()); },
            "from-me" => { self.from_me.replace(value.get().unwrap()); },
            "attachments" => { self.attachments.replace(value.get().unwrap()); },
            "body" => { self.body.replace(value.get().unwrap()); },
            "groupid" => { self.groupid.replace(value.get().unwrap()); },
            "quote-timestamp" => { self.quote_timestamp.replace(value.get().unwrap()); },
            "quote-author" => { self.quote_author.replace(value.get().unwrap()); },
            "mentions" => { self.mentions.replace(value.get().unwrap()); },
            "mentions-start" => { self.mentions_start.replace(value.get().unwrap()); },
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "timestamp" => self.timestamp.borrow().clone().to_value(),
            "number" => self.number.borrow().clone().to_value(),
            "from-me" => self.from_me.borrow().clone().to_value(),
            "attachments" => self.attachments.borrow().clone().to_value(),
            "body" => self.body.borrow().clone().to_value(),
            "groupid" => self.groupid.borrow().clone().to_value(),
            "quote-timestamp" => self.quote_timestamp.borrow().clone().to_value(),
            "quote-author" => self.quote_author.borrow().clone().to_value(),
            "mentions" => self.mentions.borrow().clone().to_value(),
            "mentions-start" => self.mentions_start.borrow().clone().to_value(),
            _ => unimplemented!(),
        }
    }
}
