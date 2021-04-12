-- Your SQL goes here
CREATE TABLE users (
    id          serial          primary key,
    hash_pwd    varchar(30)     not null,
    username    varchar(20)     unique not null,
    email       text            unique not null,
    class_id    int             references classes(id) on delete set null,
    user_role   varchar(20)     not null,
    is_active   boolean         not null default 'f',
    date_joined timestamp       not null,
    last_login  timestamp
)