pub use crate::network::errors::{format_error, IcmpError};
pub use winapi::ctypes::c_void;
pub use winapi::shared::minwindef::{DWORD, LPVOID};
pub use winapi::shared::ntdef::NULL;
pub use winapi::shared::ws2def::AF_INET6;
pub use winapi::shared::ws2ipdef::SOCKADDR_IN6;
pub use winapi::um::errhandlingapi::GetLastError;
pub use winapi::um::handleapi::INVALID_HANDLE_VALUE;
pub use winapi::um::icmpapi::{
    Icmp6CreateFile, Icmp6SendEcho2, IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho,
};
pub use winapi::um::ipexport::{ICMPV6_ECHO_REPLY, ICMP_ECHO_REPLY};

use crate::network::icmp::{PingConfig, PingResult, PingResultV4, PingResultV6};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub fn create_handle(ip: IpAddr) -> Result<*mut c_void, IcmpError> {
    match ip {
        IpAddr::V4(_) => create_handle_v4(),
        IpAddr::V6(_) => create_handle_v6(),
    }
}

fn create_handle_v4() -> Result<*mut c_void, IcmpError> {
    let handle = unsafe { IcmpCreateFile() };
    if handle == INVALID_HANDLE_VALUE {
        Err(IcmpError::CreateHandleError(unsafe { GetLastError() }))
    } else {
        Ok(handle as *mut c_void)
    }
}

fn create_handle_v6() -> Result<*mut c_void, IcmpError> {
    let handle = unsafe { Icmp6CreateFile() };
    if handle == INVALID_HANDLE_VALUE {
        Err(IcmpError::CreateHandleError(unsafe { GetLastError() }))
    } else {
        Ok(handle as *mut c_void)
    }
}

pub fn send_ping(
    handle: *mut c_void,
    ip: IpAddr,
    config: &PingConfig,
    buffer: &mut [u8],
    reply_buffer: &mut [u8],
) -> PingResult {
    match ip {
        IpAddr::V4(ipv4) => send_ping_v4(handle, ipv4, config, buffer, reply_buffer).into(),
        IpAddr::V6(ipv6) => send_ping_v6(handle, ipv6, config, buffer, reply_buffer).into(),
    }
}

fn send_ping_v4(
    handle: *mut c_void,
    ip: Ipv4Addr,
    config: &PingConfig,
    buffer: &mut [u8],
    reply_buffer: &mut [u8],
) -> PingResultV4 {
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
            error: if reply.Status == IP_SUCCESS {
                None
            } else {
                Some(format_error(reply.Status))
            },
        }
    }
}

fn send_ping_v6(
    handle: *mut c_void,
    ip: Ipv6Addr,
    config: &PingConfig,
    buffer: &mut [u8],
    reply_buffer: &mut [u8],
) -> PingResultV6 {
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
            error: if reply.Status == IP_SUCCESS {
                None
            } else {
                Some(format_error(reply.Status))
            },
        }
    }
}

const IP_SUCCESS: u32 = 0;
