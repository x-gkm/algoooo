CREATE TABLE contest_problems (
    contest_id INTEGER REFERENCES contests(id) NOT NULL,
    problem_id INTEGER REFERENCES problems(id) NOT NULL,
    letter CHARACTER(1) NOT NULL,
    PRIMARY KEY(contest_id, problem_id)
);
