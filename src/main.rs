use clap::{Parser, Subcommand};
use colored::{ColoredString, Colorize};
use duration_str::parse;
use isdown::check::{CheckError, CheckOutcome, CheckStatus};
use isdown::planner::Planner;
use isdown::target::Target;
use isdown::watch::Watcher;
use serde::Serialize;
use std::process::exit;
use std::time::Duration;

const CLEAR: &str = "\x1B[2J";
const MAX_PROVIDER_LABEL_WIDTH: usize = 40;
const DEFAULT_WATCH_DURATION: Duration = Duration::from_secs(300);

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
    Watch {
        #[arg(required = true)]
        targets: Vec<String>,
        #[arg(short, long, default_value_t = 2)]
        interval: u64,
        #[arg(short, long, default_value_t = String::from("5m"))]
        duration: String,
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
        .build()
        .unwrap_or_else(|error| {
            println!("Error creating http client {}", error);
            exit(1);
        });

    let planner = Planner { http_client };

    match cli.command {
        Commands::Watch {
            targets,
            interval,
            duration,
        } => {
            watch_targets(targets, interval, duration, cli.timeout).await;
        }
        Commands::Check { targets, json } => {
            check_targets(&planner, targets, json).await;
        }
    }
}

async fn watch_targets(targets: Vec<String>, interval: u64, duration: String, http_timeout: u64) {
    let watch_interval = Duration::from_secs(interval);
    let watch_duration = parse(duration).unwrap_or(DEFAULT_WATCH_DURATION);
    let parsed_targets = parse_targets(&targets);

    let watcher = Watcher::new(parsed_targets, watch_interval, Some(watch_duration));
    let mut receiver = watcher.watch(http_timeout);

    while let Some(event) = receiver.recv().await {
        print!("{}", CLEAR);
        print_outcomes(&event.outcomes);
        println!(
            "Elapsed: {} seconds of {}",
            event.elapsed.as_secs(),
            watch_duration.as_secs()
        );
        println!("Press Ctrl+C to exit");
    }
}

async fn check_targets(planner: &Planner, targets: Vec<String>, json: bool) {
    let parsed_targets = parse_targets(&targets);
    let checks = planner.plan(&parsed_targets);
    let outcomes = planner.run(&checks).await;

    if json {
        print_json_outcomes(outcomes);
    } else {
        print_outcomes(&outcomes);
    }
}

fn parse_targets(targets: &[String]) -> Vec<Target> {
    targets
        .iter()
        .filter_map(|target| match Target::parse(target) {
            Some(parsed_target) => Some(parsed_target),
            None => {
                println!("{}: Invalid target - {}", "error".red().bold(), target);
                None
            }
        })
        .collect()
}

fn print_json_outcomes(outcomes: Vec<Result<CheckOutcome, CheckError>>) {
    let json_outcomes: Vec<JsonOutcome> = outcomes
        .into_iter()
        .map(|outcome| match outcome {
            Ok(outcome) => JsonOutcome::Success(outcome),
            Err(error) => JsonOutcome::Failure(error.to_string()),
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&json_outcomes).unwrap());
}

fn print_outcomes(outcomes: &[Result<CheckOutcome, CheckError>]) {
    let provider_label_width = outcomes
        .iter()
        .filter_map(|outcome| outcome.as_ref().ok())
        .map(|outcome| outcome.provider.len())
        .max()
        .unwrap_or(0)
        .min(MAX_PROVIDER_LABEL_WIDTH);

    for outcome in outcomes {
        match outcome {
            Ok(outcome) => print_successful_outcome(outcome, provider_label_width),
            Err(error) => println!("{}: {}", "error".red().bold(), error),
        }
    }
}

fn print_successful_outcome(outcome: &CheckOutcome, provider_label_width: usize) {
    let label = format!(
        "{:<width$}",
        format!("{}:", outcome.provider),
        width = provider_label_width + 1
    );

    println!("{} {}", label.bold(), format_status(&outcome.status));

    for cause in &outcome.causes {
        println!("  · {}", cause.dimmed());
    }
}

fn format_status(status: &CheckStatus) -> ColoredString {
    match status {
        CheckStatus::Up => "Up".green(),
        CheckStatus::Degraded => "Degraded".yellow(),
        CheckStatus::Down => "Down".red(),
    }
}
