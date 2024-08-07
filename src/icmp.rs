use std::net::{Ipv4Addr, Ipv6Addr};

#[cfg(target_os = "windows")]
mod windows_specific {
    pub use winapi::um::icmpapi::{IcmpCreateFile, IcmpSendEcho, IcmpCloseHandle, Icmp6CreateFile, Icmp6SendEcho2};
    pub use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    pub use winapi::um::errhandlingapi::GetLastError;
    pub use winapi::shared::ws2ipdef::SOCKADDR_IN6;
    pub use winapi::shared::ws2def::AF_INET6;
    pub use winapi::um::ipexport::{ICMPV6_ECHO_REPLY, ICMP_ECHO_REPLY};
    pub use winapi::shared::minwindef::{DWORD, LPVOID};
    pub use winapi::shared::ntdef::NULL;
    pub use winapi::ctypes::c_void;
    pub use crate::errors::{IcmpError, format_error};
}

#[cfg(target_os = "windows")]
use windows_specific::*;

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

#[derive(Debug)]
pub struct PingConfig {
    pub count: u32,
    pub size: u16,
    pub ttl: u8,
    pub timeout: u32,
    pub dont_fragment: bool,
    pub tos: u8,
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
            dont_fragment: false,
            tos: 0,
        }
    }
}

#[cfg(target_os = "windows")]
fn create_handle_v4() -> Result<*mut c_void, IcmpError> {
    let handle = unsafe { IcmpCreateFile() };
    if handle == INVALID_HANDLE_VALUE {
        Err(IcmpError::CreateHandleError(unsafe { GetLastError() }))
    } else {
        Ok(handle as *mut c_void)
    }
}

#[cfg(target_os = "windows")]
fn create_handle_v6() -> Result<*mut c_void, IcmpError> {
    let handle = unsafe { Icmp6CreateFile() };
    if handle == INVALID_HANDLE_VALUE {
        Err(IcmpError::CreateHandleError(unsafe { GetLastError() }))
    } else {
        Ok(handle as *mut c_void)
    }
}

#[cfg(target_os = "windows")]
fn send_ping_v4(handle: *mut c_void, ip: Ipv4Addr, config: &PingConfig, buffer: &mut [u8], reply_buffer: &mut [u8]) -> PingResultV4 {
    let ret = unsafe {
        IcmpSendEcho(
            handle as *mut _,
            u32::from_ne_bytes(ip.octets()),
            buffer.as_mut_ptr() as LPVOID,
            buffer.len() as u16,
            NULL as *mut _,
            reply_buffer.as_mut_ptr() as LPVOID,
            reply_buffer.len() as DWORD,
            config.timeout,
        )
    };

    if ret == 0 {
        let error = unsafe { GetLastError() };
        PingResultV4 {
            ip,
            status: 0,
            data_size: 0,
            round_trip_time: 0,
            ttl: 0,
            error: Some(format_error(error)),
        }
    } else {
        let reply = unsafe { &*(reply_buffer.as_ptr() as *const ICMP_ECHO_REPLY) };
        PingResultV4 {
            ip,
            status: reply.Status,
            data_size: reply.DataSize,
            round_trip_time: reply.RoundTripTime,
            ttl: reply.Options.Ttl,
            error: if reply.Status == IP_SUCCESS { None } else { Some(format_error(reply.Status)) },
        }
    }
}

#[cfg(target_os = "windows")]
fn send_ping_v6(handle: *mut c_void, ip: Ipv6Addr, config: &PingConfig, buffer: &mut [u8], reply_buffer: &mut [u8]) -> PingResultV6 {
    let mut src_saddr = SOCKADDR_IN6::default();
    let mut dst_saddr = SOCKADDR_IN6::default();

    dst_saddr.sin6_family = AF_INET6 as u16;
    dst_saddr.sin6_addr = unsafe { std::mem::transmute(ip.octets()) };

    let ret = unsafe {
        Icmp6SendEcho2(
            handle as *mut _,
            NULL as *mut _,
            NULL as *mut _,
            NULL as *mut _,
            &mut src_saddr,
            &mut dst_saddr,
            buffer.as_mut_ptr() as LPVOID,
            buffer.len() as u16,
            NULL as *mut _,
            reply_buffer.as_mut_ptr() as LPVOID,
            reply_buffer.len() as DWORD,
            config.timeout,
        )
    };

    if ret == 0 {
        let error = unsafe { GetLastError() };
        PingResultV6 {
            ip,
            status: 0,
            round_trip_time: 0,
            error: Some(format_error(error)),
        }
    } else {
        let reply = unsafe { &*(reply_buffer.as_ptr() as *const ICMPV6_ECHO_REPLY) };
        PingResultV6 {
            ip,
            status: reply.Status,
            round_trip_time: reply.RoundTripTime as u32,
            error: if reply.Status == IP_SUCCESS { None } else { Some(format_error(reply.Status)) },
        }
    }
}

#[cfg(target_os = "windows")]
pub fn ping_ipv4(ip: Ipv4Addr, config: PingConfig) -> Result<Vec<PingResult>, IcmpError> {
    let handle = create_handle_v4()?;

    let mut buffer = vec![0u8; config.size as usize];
    let mut reply_buffer = vec![0u8; 1024];

    let mut results = Vec::new();

    for _ in 0..config.count {
        results.push(PingResult::V4(send_ping_v4(handle, ip, &config, &mut buffer, &mut reply_buffer)));
    }

    unsafe { IcmpCloseHandle(handle as *mut _) };

    Ok(results)
}

#[cfg(target_os = "windows")]
pub fn ping_ipv6(ip: Ipv6Addr, config: PingConfig) -> Result<Vec<PingResult>, IcmpError> {
    let handle = create_handle_v6()?;

    let mut buffer = vec![0u8; config.size as usize];
    let mut reply_buffer = vec![0u8; 1024];

    let mut results = Vec::new();

    for _ in 0..config.count {
        results.push(PingResult::V6(send_ping_v6(handle, ip, &config, &mut buffer, &mut reply_buffer)));
    }

    unsafe { IcmpCloseHandle(handle as *mut _) };

    Ok(results)
}

#[cfg(target_os = "windows")]
const IP_SUCCESS: u32 = 0;

// Aquí es donde puedes agregar el código específico para Linux en el futuro.

#[cfg(not(target_os = "windows"))]
pub fn ping_ipv4(_ip: Ipv4Addr, _config: PingConfig) -> Result<Vec<PingResult>, IcmpError> {
    Err(IcmpError::UnsupportedPlatform)
}

#[cfg(not(target_os = "windows"))]
pub fn ping_ipv6(_ip: Ipv6Addr, _config: PingConfig) -> Result<Vec<PingResult>, IcmpError> {
    Err(IcmpError::UnsupportedPlatform)
}
