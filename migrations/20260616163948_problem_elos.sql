CREATE TABLE problem_elos (
    problem_id INTEGER PRIMARY KEY REFERENCES problems(id),
    elo DOUBLE PRECISION NOT NULL
);
