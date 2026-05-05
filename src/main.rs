mod target;

use clap::{Parser, Subcommand};
#[derive(Parser, Debug)]
#[command(name="isdown")]
#[command(about="Detect downtime on any service", long_about="")]
struct Cli {
    #[command(subcommand)]
    command: Commands, 
}

#[derive(Subcommand, Debug)]
enum Commands {
    Check {
        #[arg(required = true)]
        targets: Vec<String>
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Check { targets } => {
            for target in targets.iter() {
                println!("{}", target);
            }
        }
    }
}
