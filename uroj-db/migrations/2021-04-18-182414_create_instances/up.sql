-- Your SQL goes here
CREATE TABLE instances (
    id             uuid          primary key  default gen_random_uuid(),
    title          varchar(250)  not null,
    description    text,
    created_at     timestamp     not null     default now(),
    creator_id     varchar(30)                references users(id) on delete set null,
    player_id      varchar(30)   not null     references users(id),    
    station_id     int           not null     references stations(id),
    curr_state     varchar(10)   not null,
    begin_at       timestamp     not null     default now(),
    executor_id    int           not null     references executors(id),
    token          varchar(6)    not null
)