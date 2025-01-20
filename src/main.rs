use anyhow::Result;
use clap::Parser;
use undead34::commands::*;

#[derive(Parser, Debug)]
#[command(version = "v0.0.1")]
#[command(about = "Ethical hacking utilities by Undead34")]
pub struct App {
    #[clap(subcommand)]
    pub commands: Commands,
}

#[tokio::main]
async fn main() {
    let app = App::parse();

    if let Err(e) = run(app).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run(app: App) -> Result<()> {
    match app.commands {
        Commands::Ping(command) => command.execute(),
        Commands::Init(command) => command.execute(),
        Commands::PassiveEnumeration(command) => command.execute(),
    }
}
