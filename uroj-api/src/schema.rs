table! {
    class (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    node (id) {
        id -> Int4,
        station_id -> Int4,
        node_id -> Int4,
        turnout_id -> Nullable<Array<Int4>>,
        track_id -> Varchar,
        adjoint_nodes -> Nullable<Array<Int4>>,
        conflicted_nodes -> Nullable<Array<Int4>>,
        line -> Lseg,
        joint -> Array<Bool>,
    }
}

table! {
    signal (id) {
        id -> Int4,
        station_id -> Int4,
        signal_id -> Varchar,
        pos -> Point,
        dir -> Direction,
        sig_type -> Signal_type,
        sig_mnt -> Signal_mounting,
        protect_node_id -> Int4,
    }
}

table! {
    station (id) {
        id -> Int4,
        title -> Varchar,
        description -> Nullable<Text>,
        created -> Timestamp,
        updated -> Timestamp,
        draft -> Bool,
        author -> Nullable<Int4>,
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

joinable!(node -> station (station_id));
joinable!(signal -> station (station_id));
joinable!(station -> userinfo (author));
joinable!(userinfo -> class (class_id));

allow_tables_to_appear_in_same_query!(
    class,
    node,
    signal,
    station,
    userinfo,
);
