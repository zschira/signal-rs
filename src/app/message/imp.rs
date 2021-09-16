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
    groupid: RefCell<Option<String>>,
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
        ]});
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "timestamp" => { self.timestamp.replace(value.get().unwrap()); },
            "number" => { self.number.replace(value.get().unwrap()); },
            "from-me" => { self.from_me.replace(value.get().unwrap()); },
            "groupid" => { self.groupid.replace(value.get().unwrap()); },
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "timestamp" => self.timestamp.borrow().clone().to_value(),
            "number" => self.number.borrow().clone().to_value(),
            "from-me" => self.from_me.borrow().clone().to_value(),
            "groupid" => self.groupid.borrow().clone().to_value(),
            _ => unimplemented!(),
        }
    }
}
