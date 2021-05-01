-- Your SQL goes here
CREATE TABLE exams (
        id             serial        primary key,
        title          varchar(250)  not null,
        finish_at      timestamp     not null
)