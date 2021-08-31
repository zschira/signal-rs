table! {
    attachments (id) {
        id -> Text,
        blurhash -> Nullable<Text>,
        content_type -> Text,
        filename -> Nullable<Text>,
    }
}

table! {
    messages (timestamp, number, groupid) {
        timestamp -> BigInt,
        number -> Text,
        attachments -> Nullable<Text>,
        body -> Text,
        groupid -> Nullable<Text>,
        quote_timestamp -> Nullable<BigInt>,
        quote_uuid -> Nullable<Binary>,
        mentions -> Nullable<Binary>,
    }
}

allow_tables_to_appear_in_same_query!(
    attachments,
    messages,
);
