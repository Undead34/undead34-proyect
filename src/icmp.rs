use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use winapi::um::ipexport::IP_OPTION_INFORMATION;
use winapi::um::icmpapi::{IcmpCreateFile, IcmpSendEcho, IcmpCloseHandle, Icmp6CreateFile, Icmp6SendEcho2};
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::errhandlingapi::GetLastError;
use winapi::shared::ws2ipdef::SOCKADDR_IN6;
use windows::Win32::Networking::WinSock::SOCKADDR_IN6;
use crate::errors::{IcmpError, format_error};

#[derive(Debug)]
pub struct PingResult {
    pub ip: Ipv4Addr,
    pub status: u32,
    pub data_size: u16,
    pub round_trip_time: u32,
    pub ttl: u8,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct PingConfig {
    pub count: u32,          // -n count
    pub size: u16,           // -l size
    pub ttl: u8,             // -i TTL
    pub timeout: u32,        // -w timeout
    pub dont_fragment: bool, // -f
    pub tos: u8,             // -v TOS
}

impl Default for PingConfig {
    fn default() -> Self {
        PingConfig {
            count: 4,
            size: 32,
            ttl: 128,
            timeout: 1000,
            dont_fragment: false,
            tos: 0,
            // Valores predeterminados para otros parámetros
        }
    }
}

pub async fn ping_ipv4(ip: Ipv4Addr, config: PingConfig) -> Result<Vec<PingResult>, IcmpError> {
    let handle = unsafe { IcmpCreateFile() };
    
    if handle == INVALID_HANDLE_VALUE {
        return Err(IcmpError::CreateHandleError(unsafe { GetLastError() }));
    }

    let mut buffer = vec![0u8; config.size as usize]; // Request buffer
    let mut reply_buffer = vec![0u8; 1024]; // Reply buffer

    let mut results = Vec::new();

    for _ in 0..config.count {
        let ret = unsafe {
            IcmpSendEcho(
                handle,
                u32::from_ne_bytes(ip.octets()),
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u16,
                std::ptr::null_mut(),
                reply_buffer.as_mut_ptr() as *mut _,
                reply_buffer.len() as u32,
                config.timeout,
            )
        };

        if ret == 0 {
            let error = unsafe { GetLastError() };
            results.push(PingResult {
                ip,
                status: 0,
                data_size: 0,
                round_trip_time: 0,
                ttl: 0,
                error: Some(format_error(error)),
            });
        } else {
            let reply = unsafe { &*(reply_buffer.as_ptr() as *const ICMP_ECHO_REPLY) };
            results.push(PingResult {
                ip,
                status: reply.Status,
                data_size: reply.DataSize,
                round_trip_time: reply.RoundTripTime,
                ttl: reply.Options.Ttl,
                error: if reply.Status == IP_SUCCESS { None } else { Some(format_error(reply.Status)) },
            });
        }
    }

    unsafe { IcmpCloseHandle(handle) };

    Ok(results)
}

pub async fn ping_ipv6(ip: Ipv6Addr, config: PingConfig) -> Result<Vec<PingResult>, IcmpError> {
    let handle = unsafe { Icmp6CreateFile() };
    
    if handle == INVALID_HANDLE_VALUE {
        return Err(IcmpError::CreateHandleError(unsafe { GetLastError() }));
    }

    let mut buffer = vec![0u8; config.size as usize]; // Request buffer
    let mut reply_buffer = vec![0u8; 1024]; // Reply buffer

    let mut results = Vec::new();

    let mut src_saddr = SOCKADDR_IN6::default();
    let mut dst_saddr = SOCKADDR_IN6::default();

    src_saddr.sin6_addr = "";
    dst_saddr.sin6_addr = ip.octets();

    for _ in 0..config.count {
        let ret = unsafe {
            Icmp6SendEcho2(
                handle,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut src_saddr,
                &mut dst_saddr,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u16,
                std::ptr::null_mut(),
                reply_buffer.as_mut_ptr() as *mut _,
                reply_buffer.len() as u32,
                config.timeout,
            )
        };

        if ret == 0 {
            let error = unsafe { GetLastError() };
            results.push(PingResult {
                ip,
                status: 0,
                data_size: 0,
                round_trip_time: 0,
                ttl: 0,
                error: Some(format_error(error)),
            });
        } else {
            let reply = unsafe { &*(reply_buffer.as_ptr() as *const ICMP_ECHO_REPLY) };
            results.push(PingResult {
                ip,
                status: reply.Status,
                data_size: reply.DataSize,
                round_trip_time: reply.RoundTripTime,
                ttl: reply.Options.Ttl,
                error: if reply.Status == IP_SUCCESS { None } else { Some(format_error(reply.Status)) },
            });
        }
    }

    unsafe { IcmpCloseHandle(handle) };

    Ok(results)
}

fn ip_to_u32(ip: IpAddr) -> Result<u32, IcmpError> {
    match ip {
        IpAddr::V4(ipv4) => Ok(u32::from_ne_bytes(ipv4.octets())),
        IpAddr::V6(_) => Err(IcmpError::GeneralError("IPv6 is not supported".to_string())),
    }
}

#[allow(non_snake_case, non_camel_case_types)]
#[repr(C)]
struct ICMP_ECHO_REPLY {
    Address: u32,
    Status: u32,
    RoundTripTime: u32,
    DataSize: u16,
    Reserved: u16,
    Data: *mut u8,
    Options: IP_OPTION_INFORMATION,
    DataPtr: [u8; 1], // This field is actually a variable-length array
}

const IP_SUCCESS: u32 = 0;
