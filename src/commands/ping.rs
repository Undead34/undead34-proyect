use anyhow::Result;
use clap::Args;
use std::io::{self, Write};
use std::net::IpAddr;

use super::Command;
use crate::network::icmp::{ping, PingConfig, PingResult};

#[derive(Args, Debug)]
pub struct PingCommand {
    #[arg(required = true)]
    pub ip_addresses: Vec<IpAddr>,

    #[arg(long, default_value_t = 1)]
    pub count: u32,

    #[arg(long, default_value_t = 56)]
    pub size: u16,

    #[arg(long, default_value_t = 5000)]
    pub timeout: u32,
}

impl Command for PingCommand {
    fn execute(&self) -> Result<()> {
        let config = PingConfig {
            count: self.count,
            size: self.size,
            timeout: self.timeout,
            ..PingConfig::default()
        };

        for ip in &self.ip_addresses {
            for _ in 0..self.count {
                if let Err(e) = self.ping_and_print(*ip, &config) {
                    eprintln!("Failed to ping {}: {}", ip, e);
                }
            }
        }

        Ok(())
    }
}

impl PingCommand {
    fn ping_and_print(
        &self,
        ip: IpAddr,
        config: &PingConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let results = ping(ip, config.clone())?;
        for result in results.iter() {
            self.print_ping_results(result);
        }

        Ok(())
    }

    fn print_ping_results(&self, result: &PingResult) {
        match result {
            PingResult::V4(result) => {
                if let Some(error) = &result.error {
                    println!("ip={}, error={}", result.ip, error);
                } else {
                    println!(
                        "ip={}, bytes={}, time={}ms, TTL={}",
                        result.ip, result.data_size, result.round_trip_time, result.ttl
                    );
                }
            }
            PingResult::V6(result) => {
                if let Some(error) = &result.error {
                    println!("ip={}, error={}", result.ip, error);
                } else {
                    println!("ip={}, time={}ms", result.ip, result.round_trip_time);
                }
            }
        }

        io::stdout().flush().unwrap();
    }
}
