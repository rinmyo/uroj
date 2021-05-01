table! {
    classes (id) {
        id -> Int4,
        class_name -> Varchar,
    }
}

table! {
    exams (id) {
        id -> Int4,
        title -> Varchar,
        finish_at -> Timestamp,
    }
}

table! {
    executors (id) {
        id -> Int4,
        addr -> Varchar,
    }
}

table! {
    instance_questions (id) {
        id -> Int4,
        instance_id -> Uuid,
        question_id -> Int4,
        score -> Nullable<Int4>,
    }
}

table! {
    instances (id) {
        id -> Uuid,
        title -> Varchar,
        description -> Nullable<Text>,
        created_at -> Timestamp,
        creator_id -> Nullable<Varchar>,
        player_id -> Varchar,
        station_id -> Int4,
        curr_state -> Varchar,
        begin_at -> Timestamp,
        executor_id -> Int4,
        token -> Varchar,
    }
}

table! {
    questions (id) {
        id -> Int4,
        title -> Varchar,
        from_node -> Int4,
        to_node -> Int4,
        err_node -> Array<Int4>,
        err_sgn -> Bool,
        exam_id -> Int4,
        station_id -> Int4,
        score -> Int4,
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

joinable!(instance_questions -> instances (instance_id));
joinable!(instance_questions -> questions (question_id));
joinable!(instances -> executors (executor_id));
joinable!(instances -> stations (station_id));
joinable!(questions -> exams (exam_id));
joinable!(questions -> stations (station_id));
joinable!(stations -> users (author_id));
joinable!(users -> classes (class_id));

allow_tables_to_appear_in_same_query!(
    classes,
    exams,
    executors,
    instance_questions,
    instances,
    questions,
    stations,
    users,
);
