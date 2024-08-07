use clap::Parser;
use std::net::IpAddr;
use undead34::icmp::{ping_ipv4, PingConfig};

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

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::parse();

    if app.hello_world {
        println!("Hello, world!");
    }

    if let Some(ip_addresses) = app.fwhich_system {
        
        for ip in ip_addresses {
            let mut config = PingConfig::default();
            config.count = 1;

            if ip.is_ipv4() {
                let results = ping_ipv4(ip, config).await.unwrap();

                for result in results {
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
            } else {
                eprintln!("IPv6 is not supported")
            }
        }
    }

    Ok(())
}
