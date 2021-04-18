-- Your SQL goes here
CREATE TABLE executors (
    id             serial        primary key,
    host           varchar(50)   not null,
    port           int           not null
)