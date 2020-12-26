table! {
    logs (id) {
        id -> Integer,
        probe -> Text,
        chip -> Text,
        os -> Text,
        commit_hash -> Text,
        timestamp -> Timestamp,
        kind -> Text,
        speed -> Integer,
    }
}
