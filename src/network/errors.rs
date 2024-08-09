#[derive(Debug)]
pub enum IcmpError {
    CreateHandleError(u32),
    SendEchoError(u32),
    GeneralError(String),
}

impl std::fmt::Display for IcmpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IcmpError::CreateHandleError(err) => write!(f, "Unable to create ICMP handle: {}", err),
            IcmpError::SendEchoError(err) => write!(f, "Ping error: {}", format_error(*err)),
            IcmpError::GeneralError(err) => write!(f, "General error: {}", err),
        }
    }
}

impl std::error::Error for IcmpError {}

pub fn format_error(error: u32) -> String {
    match error {
        IP_REQ_TIMED_OUT => "Request timed out".to_string(),
        IP_DEST_HOST_UNREACHABLE => "Destination host unreachable".to_string(),
        IP_DEST_NET_UNREACHABLE => "Destination network unreachable".to_string(),
        IP_TTL_EXPIRED_TRANSIT => "TTL expired in transit".to_string(),
        IP_TTL_EXPIRED_REASSEM => "Reassembly time exceeded".to_string(),
        IP_PACKET_TOO_BIG => "Packet too big".to_string(),
        IP_BAD_ROUTE => "Bad route".to_string(),
        IP_GENERAL_FAILURE => "General failure".to_string(),
        NO_ERROR => "No error".to_string(),
        _ => format!("Unknown error: {}", error),
    }
}

const IP_REQ_TIMED_OUT: u32 = 11010;
const IP_DEST_HOST_UNREACHABLE: u32 = 11003;
const IP_DEST_NET_UNREACHABLE: u32 = 11002;
const IP_TTL_EXPIRED_TRANSIT: u32 = 11013;
const IP_TTL_EXPIRED_REASSEM: u32 = 11014;
const IP_PACKET_TOO_BIG: u32 = 11009;
const IP_BAD_ROUTE: u32 = 11012;
const IP_GENERAL_FAILURE: u32 = 11050;
const NO_ERROR: u32 = 0;
