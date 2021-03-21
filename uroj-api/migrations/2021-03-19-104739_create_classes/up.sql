-- Your SQL goes here
CREATE TABLE classes(
    id   serial       primary key,
    name varchar(50)  unique not null
)