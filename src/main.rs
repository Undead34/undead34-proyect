use clap::Parser;
use undead34::commands::Commands;

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

    match app.commands {
        Commands::Ping(command) => {
            command.execute();
        }
    }
}

