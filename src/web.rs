use askama::Template;
use sqlx::query_as;
use static_dir::static_dir;
use warp::{
    Filter,
    http::StatusCode,
    reply::{Reply, Response},
};

#[derive(askama::Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    problem_letters: Vec<String>,
    contests: Vec<Contest>,
    leaderboard: Vec<User>,
}

struct Contest {
    name: String,
    problems: Vec<Problem>,
}

#[derive(sqlx::Type)]
struct Problem {
    name: String,
    letter: String,
    elo: Option<f64>,
}

struct User {
    name: String,
    elo: f64,
}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("could not render template: {0}")]
    Render(#[from] askama::Error),

    #[error("db error: {0}")]
    Db(#[from] sqlx::Error),
}

impl warp::Reply for AppError {
    fn into_response(self) -> Response {
        eprintln!("{self}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}


pub async fn serve(port: u16) -> anyhow::Result<()> {
    let db = crate::init().await?;

    let with_db = warp::any().map(move || db.clone());

    let index = warp::path!()
        .and(warp::get())
        .and(with_db)
        .then(async |db| -> Result<Response, AppError> {
            let contests = query_as!(
                Contest,
                r#"SELECT
                    c.name,
                    COALESCE(
                        array_agg((
                            p.name,
                            cp.letter,
                            pe.elo
                        )),
                        ARRAY[]::RECORD[]
                    ) AS "problems!: Vec<Problem>"
                FROM contests c
                JOIN contest_problems cp
                    ON cp.contest_id = c.id
                JOIN problems p
                    ON p.id = cp.problem_id
                LEFT JOIN problem_elos pe
                    ON pe.problem_id = p.id
                GROUP BY c.id
                ORDER BY c.start_date DESC"#
            )
            .fetch_all(&db)
            .await?;

            let problem_letters = contests
                .iter()
                .max_by_key(|contest| contest.problems.len())
                .iter()
                .flat_map(|contest| {
                    contest
                        .problems
                        .iter()
                        .map(|problem| problem.letter.to_owned())
                })
                .collect();

            let leaderboard = query_as!(
                User,
                "SELECT u.name, ue.elo
                FROM users u
                JOIN user_elos ue
                    ON ue.user_id = u.id
                ORDER BY ue.elo DESC"
            )
            .fetch_all(&db)
            .await?;

            let result = IndexTemplate {
                problem_letters,
                contests,
                leaderboard,
            }
            .render()?;

            Ok(warp::reply::html(result).into_response())
        });

    let static_dir = warp::path("static").and(static_dir!("static"));

    let app = index.or(static_dir);

    warp::serve(app).run(([0, 0, 0, 0], port)).await;

    Ok(())
}
