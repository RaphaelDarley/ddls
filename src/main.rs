use clap::{Parser, Subcommand};
use ddls::{start::start, update::update};
use tracing::info;

#[derive(Parser, Debug)]
// #[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: SubCommands,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    Start,
    Update {
        #[arg(long, short, default_value_t = {"127.0.0.1".to_string()})]
        endpoint: String,
        #[arg(long, short, default_value_t = 6615)]
        port: u16,
        #[arg(long)]
        target: String,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    info!(cli = ?cli);

    match cli.command {
        SubCommands::Start => start().await,
        SubCommands::Update {
            endpoint,
            port,
            target,
        } => update(endpoint, port, target).await,
    }
}
