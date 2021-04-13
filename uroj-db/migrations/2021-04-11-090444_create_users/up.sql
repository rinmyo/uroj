-- Your SQL goes here
CREATE TABLE users (
    id          varchar(30)     primary key,
    hash_pwd    varchar(60)     not null,
    email       text            not null,
    class_id    int             references classes(id) on delete set null,
    user_role   varchar(20)     not null,
    is_active   boolean         not null default 'f',
    date_joined timestamp       not null,
    last_login  timestamp
)