table! {
    classes (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    nodes (id) {
        id -> Int4,
        station_id -> Int4,
        node_id -> Int4,
        turnout_id -> Nullable<Array<Int4>>,
        track_id -> Varchar,
        adjoint_nodes -> Nullable<Array<Int4>>,
        conflicted_nodes -> Nullable<Array<Int4>>,
        line_segment -> Array<Float8>,
        joint -> Array<Bool>,
    }
}

table! {
    signals (id) {
        id -> Int4,
        station_id -> Int4,
        signal_id -> Varchar,
        pos -> Array<Float8>,
        dir -> Varchar,
        sig_type -> Varchar,
        sig_mnt -> Varchar,
        protect_node_id -> Int4,
    }
}

table! {
    stations (id) {
        id -> Int4,
        title -> Varchar,
        description -> Nullable<Text>,
        created -> Timestamp,
        updated -> Timestamp,
        draft -> Bool,
        author_id -> Nullable<Int4>,
    }
}

table! {
    userinfo (id) {
        id -> Int4,
        password -> Varchar,
        username -> Varchar,
        email -> Text,
        class_id -> Nullable<Int4>,
        is_admin -> Bool,
        is_active -> Bool,
        date_joined -> Timestamp,
        last_login -> Nullable<Timestamp>,
    }
}

joinable!(nodes -> stations (station_id));
joinable!(signals -> stations (station_id));
joinable!(stations -> userinfo (author_id));
joinable!(userinfo -> classes (class_id));

allow_tables_to_appear_in_same_query!(
    classes,
    nodes,
    signals,
    stations,
    userinfo,
);
