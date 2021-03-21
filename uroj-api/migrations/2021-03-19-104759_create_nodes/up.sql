-- Your SQL goes here
CREATE TABLE nodes(
    id               serial                 primary key,
    station_id       int                    not null references stations(id) on delete cascade,
    node_id          int                    not null,
    turnout_id       int[],
    track_id         varchar(10)            not null,
    adjoint_nodes    int[],
    conflicted_nodes int[],
    line_segment     double precision[2][2] not null,
    joint            boolean[2]             not null
)