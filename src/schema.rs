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
        is_read -> Bool,
        attachments -> Nullable<Text>,
        body -> Text,
        groupid -> Nullable<Text>,
        quote_timestamp -> Nullable<BigInt>,
        quote_author -> Nullable<Text>,
        mentions -> Nullable<Binary>,
        mentions_start -> Nullable<Binary>,
        reaction_emojis -> Nullable<Text>,
        reaction_authors -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    attachments,
    messages,
);
