use std::{cell::RefCell, env};

use algoleague::ContestParticipationType;
use futures::TryStreamExt;
use sqlx::query;

fn number_to_letter(mut n: isize) -> String {
    let mut result = String::new();

    while n >= 0 {
        result.push("abcdefghijklmnopqrstuvwxyz".as_bytes()[n as usize % 26] as char);
        n /= 26;
        n -= 1;
    }

    result.chars().rev().collect()
}

pub async fn fetch() -> anyhow::Result<()> {
    let db = crate::init().await?;

    let client = algoleague::Client::login(
        &env::var("ALGOLEAGUE_USERNAME")?,
        env::var("ALGOLEAGUE_PASSWORD")?,
    )
    .await?;

    client
        .contests()
        .map_err(anyhow::Error::from)
        .try_for_each(async |contest| {
            if contest.participation_type != ContestParticipationType::Individual {
                return Ok(())
            }

            let record = query!("SELECT FROM contests WHERE name = $1", contest.slug)
                .fetch_optional(&db)
                .await?;

            if record.is_some() {
                return Ok(());
            }

            let transaction = RefCell::new(db.begin().await?);

            let db_contest = query!(
                "INSERT INTO contests (name, start_date, end_date) VALUES ($1, $2, $3) RETURNING id",
                contest.slug,
                contest.start_date,
                contest.end_date
            )
            .fetch_one(&mut **transaction.borrow_mut())
            .await?;

            match client.problems(&contest.id).await {
                Err(e) => {
                    eprintln!("{e}");
                    return Ok(());
                }
                Ok(problems) => {
                    for (problem, letter) in problems
                        .into_iter()
                        .enumerate()
                        .map(|(index, problem)| (problem, number_to_letter(index as isize)))
                    {
                        let db_problem = query!(
                            "INSERT INTO problems (name) VALUES ($1)
                            ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
                            RETURNING id",
                            problem.slug,
                        )
                        .fetch_one(&mut **transaction.borrow_mut())
                        .await?;

                        query!(
                            "INSERT INTO contest_problems (contest_id, problem_id, letter) VALUES ($1, $2, $3)",
                            db_contest.id,
                            db_problem.id,
                            letter,
                        )
                        .execute(&mut **transaction.borrow_mut())
                        .await?;
                    }
                }
            }

            if let Err(e) = client
                .submissions(contest.id.clone())
                .map_err(anyhow::Error::from)
                .try_for_each(async |submission| {
                    if !submission.during_contest {
                        return Ok(())
                    }

                    if query!(
                        "SELECT FROM problems WHERE name = $1",
                        submission.problem_slug
                    )
                        .fetch_optional(&mut **transaction.borrow_mut())
                        .await?
                        .is_none() {
                            return Ok(());
                        }

                    query!(
                        "WITH inserted_user AS (
                            INSERT INTO users (name) VALUES ($1)
                            ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
                            RETURNING id
                        )
                        INSERT INTO submissions (
                            user_id,
                            problem_id,
                            contest_id,
                            status,
                            start_date,
                            end_date
                        ) VALUES (
                            (SELECT id FROM inserted_user),
                            (SELECT id FROM problems WHERE name = $2),
                            $3,
                            $4,
                            $5,
                            $6
                        )",
                        submission.user_name,
                        submission.problem_slug,
                        db_contest.id,
                        format!("{:?}", submission.status),
                        submission.start_date,
                        submission.end_date,
                    )
                    .execute(&mut **transaction.borrow_mut())
                    .await?;

                    Ok(())
                })
                .await {
                    eprintln!("{e}");
                    return Ok(());
                }

            transaction.into_inner().commit().await?;

            Ok(())
        })
        .await?;

    Ok(())
}
