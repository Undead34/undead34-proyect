use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Debug)]
pub struct PingResultV4 {
    pub ip: Ipv4Addr,
    pub status: u32,
    pub data_size: u16,
    pub round_trip_time: u32,
    pub ttl: u8,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct PingResultV6 {
    pub ip: Ipv6Addr,
    pub status: u32,
    pub round_trip_time: u32,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PingConfig {
    pub count: u32,
    pub size: u16,
    pub ttl: u8,
    pub timeout: u32,
}

#[derive(Debug)]
pub enum PingResult {
    V4(PingResultV4),
    V6(PingResultV6),
}

impl Default for PingConfig {
    fn default() -> Self {
        PingConfig {
            count: 1,
            size: 32,
            ttl: 128,
            timeout: 2000,
        }
    }
}

impl From<PingResultV4> for PingResult {
    fn from(result: PingResultV4) -> Self {
        PingResult::V4(result)
    }
}

impl From<PingResultV6> for PingResult {
    fn from(result: PingResultV6) -> Self {
        PingResult::V6(result)
    }
}

#[cfg(target_os = "windows")]
use crate::network::windows_specific::*;

#[cfg(target_os = "windows")]
pub fn ping(ip: IpAddr, config: PingConfig) -> Result<Vec<PingResult>, IcmpError> {
    let handle = create_handle(ip)?;
    let mut buffer = vec![0u8; config.size as usize];
    let mut reply_buffer = vec![0u8; 1024];
    let mut results = Vec::new();

    for _ in 0..config.count {
        results.push(send_ping(handle, ip, &config, &mut buffer, &mut reply_buffer));
    }

    unsafe { IcmpCloseHandle(handle as *mut _) };

    Ok(results)
}

#[cfg(not(target_os = "windows"))]
pub fn ping(_ip: IpAddr, _config: PingConfig) -> Result<Vec<PingResult>, IcmpError> {
    Err(IcmpError::UnsupportedPlatform)
}

