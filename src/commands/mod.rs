use clap::Subcommand;
pub mod ping;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Rust native ping
    Ping(ping::PingCommand),
}
