use regex::Regex;
use lazy_static::lazy_static;

pub fn find_url<'r, 't>(text: &String) -> String {
    lazy_static! {
        static ref URL_RE: Regex = Regex::new(r"(?im)(?:(?:https?|ftp|file)://|www\.|ftp\.)(?:\([-A-Z0-9+&@#\\/%=~_|\$?!:,.]*\)|[-A-Z0-9+&@#/%=~_|\$?!:,.])*(?:\([-A-Z0-9+&@#/%=~_|\$?!:,.]*\)|[A-Z0-9+&@#/%=~_|\$])").unwrap();
    }

    let matches = URL_RE.find_iter(&text);
    
    matches.fold(text.clone(), |mut text, url_match| {
        let link = &text[url_match.start()..url_match.end()];
        let link_no_amp = link.replace("&", "&amp;");
        let link = format!("<a href=\"{}\">{}</a>", link_no_amp.as_str(), link_no_amp.as_str());
        text.replace_range(url_match.start()..url_match.end(), link.as_str());
        text
    })
}
