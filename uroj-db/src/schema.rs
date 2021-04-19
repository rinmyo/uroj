table! {
    classes (id) {
        id -> Int4,
        class_name -> Varchar,
    }
}

table! {
    executors (id) {
        id -> Int4,
        addr -> Varchar,
    }
}

table! {
    instances (id) {
        id -> Uuid,
        title -> Varchar,
        description -> Nullable<Text>,
        created_at -> Timestamp,
        creator -> Nullable<Varchar>,
        player -> Varchar,
        yaml -> Text,
        curr_state -> Varchar,
        begin_at -> Timestamp,
        executor_id -> Int4,
        token -> Varchar,
    }
}

table! {
    stations (id) {
        id -> Int4,
        title -> Varchar,
        description -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        draft -> Bool,
        author_id -> Nullable<Varchar>,
        yaml -> Text,
    }
}

table! {
    users (id) {
        id -> Varchar,
        hash_pwd -> Varchar,
        email -> Text,
        class_id -> Nullable<Int4>,
        user_role -> Varchar,
        is_active -> Bool,
        joined_at -> Timestamp,
        last_login_at -> Nullable<Timestamp>,
    }
}

joinable!(instances -> executors (executor_id));
joinable!(stations -> users (author_id));
joinable!(users -> classes (class_id));

allow_tables_to_appear_in_same_query!(
    classes,
    executors,
    instances,
    stations,
    users,
);
