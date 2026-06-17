use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Fetch data from algoleague.com
    Fetch,

    /// Update ratings
    Rate {
        /// Rating parameters
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,
    },

    /// Serve over HTTP
    Serve {
        /// Port to serve on
        #[arg(short, long, value_name = "PORT")]
        port: u16,
    },

    /// Run database migrations
    Migrate,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Fetch => {
            algoooo::fetch().await?;
        }
        Command::Rate { config } => {
            algoooo::rate(config).await?;
        }
        Command::Serve { port } => {
            algoooo::serve(port).await?;
        }
        Command::Migrate => {
            algoooo::migrate().await?;
        }
    }

    Ok(())
}
