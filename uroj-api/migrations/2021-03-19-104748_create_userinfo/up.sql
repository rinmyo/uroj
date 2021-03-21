-- Your SQL goes here

CREATE TABLE userinfo (
    id          serial          primary key,
    password    varchar(30)     not null,
    username    varchar(20)     unique not null,
    email       text            unique not null,
    class_id    int             references classes(id) on delete set null,
    is_admin    boolean         not null default 'f',
    is_active   boolean         not null default 'f',
    date_joined timestamp       not null,
    last_login  timestamp
)