-- Your SQL goes here
CREATE TABLE stations (
    id             serial        primary key,
    title          varchar(250)  not null,
    description    text,
    created        timestamp     not null,
    updated        timestamp     not null,
    draft          boolean       not null default 'f',
    author_id      int           references users(id) on delete set null,
    yaml           text          not null
)