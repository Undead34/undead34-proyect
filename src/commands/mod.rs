use clap::Subcommand;

pub mod init;
pub mod passive_enumeration;
pub mod ping;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Rust native ping
    Ping(ping::PingCommand),
    /// Init folder workspace
    Init(init::InitCommand),
    /// Passive Enumeration
    PassiveEnumeration(passive_enumeration::PassiveEnumerationCommand),
}

pub trait Command {
    fn execute(&self) -> anyhow::Result<()>;
}
