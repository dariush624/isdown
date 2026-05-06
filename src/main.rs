mod check;
mod planner;
mod registry;
mod target;

use crate::planner::Planner;
use crate::target::Target;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "isdown")]
#[command(about = "Detect downtime on any service", long_about = "")]
struct Cli {
    #[arg(short, long, default_value_t = 10)]
    timeout: u64,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Check {
        #[arg(required = true)]
        targets: Vec<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let planner = Planner {
        http_client: reqwest::Client::builder()
            .timeout(Duration::from_secs(cli.timeout))
            .build()
            .unwrap(),
    };
    match cli.command {
        Commands::Check { targets } => {
            let checks = planner.plan(
                &targets
                    .iter()
                    .map(|target| Target::parse(target).unwrap())
                    .collect::<Vec<_>>(),
            );
            let outcomes = planner.run(&checks).await;
            for outcome in outcomes.iter() {
                match outcome {
                    Ok(o) => {
                        use crate::check::CheckStatus;
                        let status = match o.status {
                            CheckStatus::Up => "Up".green(),
                            CheckStatus::Degraded => "Degraded".yellow(),
                            CheckStatus::Down => "Down".red(),
                        };
                        println!("{}: {}", o.provider.bold(), status);
                        for cause in &o.causes {
                            println!("  · {}", cause.dimmed());
                        }
                    }
                    Err(e) => println!("{}: {}", "error".red().bold(), e),
                }
            }
        }
    }
}
