use regex::Regex;
use lazy_static::lazy_static;
use gtk::glib;

pub fn find_url<'r, 't>(text: &str) -> String {
    lazy_static! {
        static ref URL_RE: Regex = Regex::new(r"(?im)(?:(?:https?|ftp|file)://|www\.|ftp\.)(?:\([-A-Z0-9+&@#\\/%=~_|\$?!:,.]*\)|[-A-Z0-9+&@#/%=~_|\$?!:,.])*(?:\([-A-Z0-9+&@#/%=~_|\$?!:,.]*\)|[A-Z0-9+&@#/%=~_|\$])").unwrap();
    }

    let matches = URL_RE.find_iter(text);

    let mut text = text.to_owned();
    let mut previous = 0;
    let matches: Vec<(usize, usize)> = matches.map(|url_match| {
        let indices = (url_match.start(), url_match.end());
        let escaped_text = glib::markup_escape_text(
            &text[previous..indices.0]
        );

        text.replace_range(previous..indices.0, &escaped_text);

        // Will always be positive or 0 because escaped text can
        // only increase in length
        let length_diff = escaped_text.len() - (indices.0 - previous);
        let indices = (indices.0 + length_diff, indices.1 + length_diff);
        previous = indices.1;
        indices
    }).collect();

    // If no matches escape all of the text
    if matches.is_empty() {
        return glib::markup_escape_text(&text).to_string();
    }
    
    matches.iter().fold(text, |mut text, url_match| {
        let link = &text[url_match.0..url_match.1];
        let link_no_amp = link.replace("&", "&amp;");
        let link = format!("<a href=\"{}\">{}</a>", link_no_amp.as_str(), link_no_amp.as_str());
        text.replace_range(url_match.0..url_match.1, link.as_str());
        text
    })
}
