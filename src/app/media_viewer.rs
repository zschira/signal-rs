use gtk::prelude::*;
use gtk::{Box as Box_, Orientation, Picture, Video};
use std::rc::Rc;

use crate::models::Attachment;
use crate::app::App;
use crate::database;

impl App {
    pub fn new_media_viewer(self: Rc<App>, attachment_list: &String) -> Box_ {
        let media_box = Box_::new(Orientation::Vertical, 3);
        attachment_list.split('\n').for_each(|id| {
            let attachment = database::get_attachment(&self.db.lock().unwrap(), id);
            
            if let Some(attachment) = attachment {
                if attachment.content_type.starts_with("image/") {
                    media_box.append(&get_picture(attachment));
                }
            }
        });

        media_box
    }
}

fn get_picture(attachment: Attachment) -> Picture {
    Picture::for_filename(format!("run/attachments/{}", attachment.id))
}
