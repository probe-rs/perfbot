table! {
    logs (id) {
        id -> Integer,
        probe -> Text,
        chip -> Text,
        os -> Text,
        protocol -> Text,
        protocol_speed -> Integer,
        commit_hash -> Text,
        timestamp -> Timestamp,
        kind -> Text,
        read_speed -> Integer,
        write_speed -> Integer,
    }
}
