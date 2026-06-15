CREATE TABLE submissions (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) NOT NULL,
    problem_id INTEGER NOT NULL,
    contest_id INTEGER NOT NULL,
    status TEXT NOT NULL,
    start_date TIMESTAMP WITH TIME ZONE NOT NULL,
    end_date TIMESTAMP WITH TIME ZONE NOT NULL,
    FOREIGN KEY(contest_id, problem_id) REFERENCES contest_problems(contest_id, problem_id)
);
