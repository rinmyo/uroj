-- Your SQL goes here
CREATE TABLE instances (
    id             uuid          primary key  default gen_random_uuid(),
    title          varchar(250)  not null,
    description    text,
    created_at     timestamp     not null     default now(),
    creator        varchar(30)                                  references users(id) on delete set null,
    player         varchar(30)   not null                       references users(id),    
    yaml           text          not null,
    curr_state     varchar(10)   not null,
    begin_at       timestamp     not null     default now(),
    executor_id    int           not null                       references executors(id),
    token          varchar(6)    not null
)