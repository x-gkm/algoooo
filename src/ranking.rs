use std::{cell::RefCell, collections::HashMap, fs::File, path::PathBuf};

use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::query;

fn get_expected_score(rating_a: f64, rating_b: f64) -> f64 {
    1.0 / (1.0 + 10.0f64.powf((rating_b - rating_a) / 400.0))
}

fn get_dynamic_k(n0: f64, k_bounds: KBounds, submission_count: i32) -> f64 {
    let power = 0.1 * (submission_count as f64 - n0);
    if power > 100.0 {
        return k_bounds.min;
    }
    k_bounds.min + (k_bounds.max - k_bounds.min) / (1.0 + power.exp())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Parameters {
    n0: f64,
    average_elo: f64,
    user_k_bounds: KBounds,
    problem_k_bounds: KBounds,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct KBounds {
    min: f64,
    max: f64,
}

pub async fn rate(config: PathBuf) -> anyhow::Result<()> {
    let db = crate::init().await?;

    let parameters: Parameters = yaml_serde::from_reader(File::open(config)?)?;

    let transaction = RefCell::new(db.begin().await?);

    query!(
        "INSERT INTO user_elos (user_id, elo)
        SELECT id, $1 FROM users
        ON CONFLICT (user_id) DO UPDATE SET elo = $1",
        parameters.average_elo,
    )
    .execute(&mut **transaction.borrow_mut())
    .await?;

    query!(
        "INSERT INTO problem_elos (problem_id, elo)
        SELECT id, $1 FROM problems
        ON CONFLICT (problem_id) DO UPDATE SET elo = $1",
        parameters.average_elo,
    )
    .execute(&mut **transaction.borrow_mut())
    .await?;

    let user_submission_counts: RefCell<HashMap<i32, i32>> = RefCell::new(HashMap::new());
    let problem_submission_counts: RefCell<HashMap<i32, i32>> = RefCell::new(HashMap::new());

    query!(
        r#"WITH triples AS (
            SELECT DISTINCT s.contest_id, s.user_id, cp.problem_id
            FROM submissions s
            JOIN contest_problems cp
                ON cp.contest_id = s.contest_id
        ),
        accepted AS (
            SELECT * FROM submissions WHERE status = 'Accepted'
        ),
        solved_states AS (
            SELECT
                t.contest_id,
                t.user_id,
                t.problem_id,
                ac.problem_id IS NOT NULL as is_solved
            FROM triples t
            LEFT JOIN accepted ac
                ON ac.contest_id = t.contest_id
                AND ac.user_id = t.user_id
                AND ac.problem_id = t.problem_id
        )
        SELECT
            contest_id AS id,
            array_agg((
                user_id,
                problem_id,
                is_solved
            )) AS "results!: Vec<(i32, i32, bool)>"
        FROM solved_states
        GROUP BY contest_id"#
    )
    .fetch(&db)
    .map_err(anyhow::Error::from)
    .try_for_each(async |contest| {
        let mut user_deltas = HashMap::<i32, f64>::new();
        let mut problem_deltas = HashMap::<i32, f64>::new();

        for (user_id, problem_id, is_solved) in contest.results {
            let problem = query!(
                r#"SELECT elo FROM problem_elos WHERE problem_id = $1"#,
                problem_id
            )
            .fetch_one(&mut **transaction.borrow_mut())
            .await?;

            let user = query!(r#"SELECT elo FROM user_elos WHERE user_id = $1"#, user_id)
                .fetch_one(&mut **transaction.borrow_mut())
                .await?;

            let expected_user_score = get_expected_score(user.elo, problem.elo);
            let expected_problem_score = 1.0 - expected_user_score;

            let actual_user_score = is_solved as i32 as f64;
            let actual_problem_score = 1.0 - actual_user_score;

            let k_user = get_dynamic_k(
                parameters.n0,
                parameters.user_k_bounds,
                user_submission_counts
                    .borrow()
                    .get(&user_id)
                    .copied()
                    .unwrap_or_default(),
            );

            let k_problem = get_dynamic_k(
                parameters.n0,
                parameters.problem_k_bounds,
                problem_submission_counts
                    .borrow()
                    .get(&problem_id)
                    .copied()
                    .unwrap_or_default(),
            );

            *user_deltas.entry(user_id).or_default() +=
                k_user * (actual_user_score - expected_user_score);
            *problem_deltas.entry(problem_id).or_default() +=
                k_problem * (actual_problem_score - expected_problem_score);

            *user_submission_counts
                .borrow_mut()
                .entry(user_id)
                .or_default() += 1;
            *problem_submission_counts
                .borrow_mut()
                .entry(problem_id)
                .or_default() += 1;
        }

        for (user_id, user_delta) in user_deltas {
            query!(
                "UPDATE user_elos SET elo = elo + $2 WHERE user_id = $1",
                user_id,
                user_delta,
            )
            .execute(&mut **transaction.borrow_mut())
            .await?;
        }

        for (problem_id, problem_delta) in problem_deltas {
            query!(
                "UPDATE problem_elos SET elo = elo + $2 WHERE problem_id = $1",
                problem_id,
                problem_delta,
            )
            .execute(&mut **transaction.borrow_mut())
            .await?;
        }

        Ok(())
    })
    .await?;

    transaction.into_inner().commit().await?;

    Ok(())
}
