-- Your SQL goes here
CREATE TABLE instance_questions (
    instance_id    uuid   not null references instances(id),
    question_id    int    not null references questions(id),
    score          int,
    PRIMARY KEY (instance_id, question_id)
)