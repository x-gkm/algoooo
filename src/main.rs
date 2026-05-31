use std::env;
use std::sync::Mutex;

use algoleague::SubmissionStatus;
use futures::TryStreamExt;
use tokio::{fs::File, io::AsyncWriteExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    let client = algoleague::Client::login(
        &env::var("ALGOLEAGUE_USERNAME")?,
        env::var("ALGOLEAGUE_PASSWORD")?,
    )
    .await?;

    let signups_file = Mutex::new(File::create("signups.csv").await?);
    let solves_file = Mutex::new(File::create("solves.csv").await?);
    let contests_file = Mutex::new(File::create("contests.csv").await?);
    let problems_file = Mutex::new(File::create("problems.csv").await?);

    client
        .contests()
        .map_err(anyhow::Error::from)
        .try_for_each(async |contest| {
            contests_file
                .lock()
                .unwrap()
                .write_all(
                    format!("{:?}, {:?}\n", contest.slug, contest.participation_type).as_bytes(),
                )
                .await?;

            for problem in client.problems(&contest.id).await? {
                problems_file
                    .lock()
                    .unwrap()
                    .write_all(format!("{:?}, {:?}\n", contest.slug, problem.slug).as_bytes())
                    .await?;
            }

            if let Err(e) = client
                .submissions(contest.id)
                .map_err(anyhow::Error::from)
                .try_for_each(async |submission| {
                    if !submission.during_contest || submission.status != SubmissionStatus::Accepted
                    {
                        return Ok(());
                    }

                    solves_file
                        .lock()
                        .unwrap()
                        .write_all(
                            format!(
                                "{:?}, {:?}, {:?}\n",
                                submission.user_name, contest.slug, submission.problem_slug,
                            )
                            .as_bytes(),
                        )
                        .await?;

                    Ok(())
                })
                .await
            {
                eprintln!("{e}");
                return Ok(());
            }

            client
                .participants(&contest.slug)
                .map_err(anyhow::Error::from)
                .try_for_each(async |participant| {
                    if participant.creation_time > contest.end_date {
                        return Ok(());
                    }

                    signups_file
                        .lock()
                        .unwrap()
                        .write_all(
                            format!("{:?}, {:?}\n", participant.name, contest.slug).as_bytes(),
                        )
                        .await?;

                    Ok(())
                })
                .await?;
            Ok(())
        })
        .await?;

    Ok(())
}
