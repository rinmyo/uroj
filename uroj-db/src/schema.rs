table! {
    classes (id) {
        id -> Int4,
        class_name -> Varchar,
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
        yaml -> Text,
    }
}

table! {
    users (id) {
        id -> Int4,
        hash_pwd -> Varchar,
        username -> Varchar,
        email -> Text,
        class_id -> Nullable<Int4>,
        user_role -> Varchar,
        is_active -> Bool,
        date_joined -> Timestamp,
        last_login -> Nullable<Timestamp>,
    }
}

joinable!(stations -> users (author_id));
joinable!(users -> classes (class_id));

allow_tables_to_appear_in_same_query!(
    classes,
    stations,
    users,
);
