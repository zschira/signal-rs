table! {
    attachments (id) {
        id -> Text,
        blurhash -> Nullable<Text>,
        content_type -> Text,
        filename -> Nullable<Text>,
    }
}

table! {
    messages (timestamp, number, from_me, groupid) {
        timestamp -> BigInt,
        number -> Nullable<Text>,
        from_me -> Bool,
        attachments -> Nullable<Text>,
        body -> Text,
        groupid -> Nullable<Text>,
        quote_timestamp -> Nullable<BigInt>,
        quote_author -> Nullable<Text>,
        mentions -> Nullable<Binary>,
        mentions_start -> Nullable<Binary>,
    }
}

allow_tables_to_appear_in_same_query!(
    attachments,
    messages,
);
