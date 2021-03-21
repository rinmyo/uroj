-- Your SQL goes here
CREATE TABLE stations
(
    id             serial        primary key,
    title          varchar(250)  not null,
    description    text,
    created        timestamp     not null,
    updated        timestamp     not null,
    draft          boolean       not null default 'f',
    author_id      int           references userinfo(id) on delete set null
)