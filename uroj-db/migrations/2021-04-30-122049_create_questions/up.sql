-- Your SQL goes here
CREATE TABLE questions (
        id             serial        primary key,
        title          varchar(250)  not null,
        from_node      int           not null,
        to_node        int           not null,
        err_node       int[]         not null, --故障点
        err_sgn        boolean       not null,       --需要引导接车
        exam_id        int           not null  references exams(id),
        station_id     int           not null  references stations(id),
        score          int           not null  --配点
)