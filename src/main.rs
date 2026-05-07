use clap::{Parser, Subcommand};
use colored::Colorize;
use isdown::check::{CheckOutcome, CheckStatus};
use isdown::planner::Planner;
use isdown::target::Target;
use serde::Serialize;
use std::process::exit;
use std::time::Duration;

const CLEAR: &str = "\x1B[2J";

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
        #[arg(short, long, default_value_t = false)]
        json: bool,
    },
}

#[derive(Serialize)]
enum JsonOutcome {
    Success(CheckOutcome),
    Failure(String),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(cli.timeout))
        .build();

    if let Err(e) = http_client {
        println!("Error creating http client {}", e);
        exit(1);
    }

    let planner = Planner {
        http_client: http_client.unwrap(),
    };
    match cli.command {
        Commands::Check { targets, json } => {
            let mut parsed_targets: Vec<Target> = vec![];
            for target in targets.iter() {
                match Target::parse(target) {
                    Some(t) => {
                        parsed_targets.push(t);
                    }
                    _ => {
                        println!("{}: Invalid target - {}", "error".red().bold(), target);
                        continue;
                    }
                }
            }

            let checks = planner.plan(&parsed_targets);
            let outcomes = planner.run(&checks).await;

            if json {
                let json_outcomes: Vec<JsonOutcome> = outcomes
                    .into_iter()
                    .map(|o| match o {
                        Ok(outcome) => JsonOutcome::Success(outcome),
                        Err(e) => JsonOutcome::Failure(e.to_string()),
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&json_outcomes).unwrap());
            } else {
                let mut max_width = outcomes
                    .iter()
                    .filter_map(|o| o.as_ref().ok())
                    .map(|o| o.provider.len())
                    .max()
                    .unwrap_or(0);
                max_width = max_width.min(40);

                for outcome in outcomes.iter() {
                    match outcome {
                        Ok(o) => {
                            let status = match o.status {
                                CheckStatus::Up => "Up".green(),
                                CheckStatus::Degraded => "Degraded".yellow(),
                                CheckStatus::Down => "Down".red(),
                            };
                            let label = format!(
                                "{:<width$}",
                                format!("{}:", o.provider),
                                width = max_width + 1
                            );
                            println!("{} {}", label.bold(), status);
                            for cause in &o.causes {
                                println!("  · {}", cause.dimmed());
                            }
                            println!("\x1B[2J")
                        }
                        Err(e) => println!("{}: {}", "error".red().bold(), e),
                    }
                }
            }
        }
    }
}
