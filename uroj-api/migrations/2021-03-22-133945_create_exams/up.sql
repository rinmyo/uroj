-- Your SQL goes here
CREATE TABLE exams (
    id           serial        primary key,
    instance_id  int not null  references instances(id) on delete cascade,
    
)