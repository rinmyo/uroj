-- Your SQL goes here
CREATE TABLE signals(
    id               serial               primary key,
    station_id       int                  not null references stations(id) on delete cascade,
    signal_id        varchar(10)          not null,
    pos              double precision[2]  not null,
    dir              varchar(10)          not null,
    sig_type         varchar(20)          not null,
    sig_mnt          varchar(20)          not null,
    protect_node_id  int                  not null
)