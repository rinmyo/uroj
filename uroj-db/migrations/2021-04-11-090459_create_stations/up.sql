-- Your SQL goes here
CREATE TABLE stations (
    id             serial        primary key,
    title          varchar(250)  not null,
    description    text,
    created_at     timestamp     not null default now(),
    updated_at     timestamp     not null default now(),
    draft          boolean       not null default 'f',
    author_id      varchar(30)   references users(id) on delete set null,
    yaml           text          not null
)