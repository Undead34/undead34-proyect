use clap::Parser;
use std::net::IpAddr;
use undead34::icmp::{ping_ipv4, ping_ipv6, PingConfig, PingResult};

#[derive(Parser)]
#[command(name = "Undead34")]
#[command(version = "v0.0.1")]
#[command(about = "Ethical hacking utilities by Undead34")]
pub struct App {
    /// Print "Hello, world!"
    #[arg(long)]
    pub hello_world: bool,

    /// IP addresses for quick system check
    #[arg(long, alias("fwsys"), num_args = 1..)]
    pub fwhich_system: Option<Vec<IpAddr>>,
}

fn print_ping_results(result: PingResult) {
    match result {
        PingResult::V4(result) => {
            if let Some(error) = &result.error {
                println!("ip={}, error={}", result.ip, error);
            } else {
                let os = match result.ttl {
                    0..=64 => "Linux/Unix",
                    65..=128 => "Windows",
                    129..=255 => "Other/Undetermined",
                };

                println!(
                    "ip={}, bytes={}, time={}ms, TTL={}, OS={}",
                    result.ip, result.data_size, result.round_trip_time, result.ttl, os
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
}

fn ping_ip(ip: IpAddr, config: PingConfig) -> Result<(), Box<dyn std::error::Error>> {
    match ip {
        IpAddr::V4(ipv4) => {
            let results = ping_ipv4(ipv4, config)?;
            for result in results {
                print_ping_results(result);
            }
        }
        IpAddr::V6(ipv6) => {
            let results = ping_ipv6(ipv6, config)?;
            for result in results {
                print_ping_results(result);
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::parse();

    if app.hello_world {
        println!("Hello, world!");
    }

    if let Some(ip_addresses) = app.fwhich_system {
        for ip in ip_addresses {
            let config = PingConfig {
                count: 1,
                ..Default::default()
            };

            if let Err(e) = ping_ip(ip, config) {
                eprintln!("Failed to ping {}: {}", ip, e);
            }
        }
    }

    Ok(())
}
