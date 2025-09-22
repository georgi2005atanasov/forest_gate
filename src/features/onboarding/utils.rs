use std::net::IpAddr;

use actix_web::HttpRequest;

pub fn parse_ip(s: &str) -> Option<IpAddr> {
    s.trim().parse::<IpAddr>().ok()
}

pub fn get_client_ip(req: &HttpRequest) -> Option<IpAddr> {
    // Prefer Forwarded: for=...
    if let Some(h) = req.headers().get("forwarded") {
        if let Ok(v) = h.to_str() {
            for part in v.split(';').flat_map(|x| x.split(',')) {
                let p = part.trim();
                if let Some(val) = p.strip_prefix("for=") {
                    let val = val.trim_matches('"');
                    if let Some(ip) = parse_ip(val) {
                        return Some(ip);
                    }
                }
            }
        }
    }
    if let Some(h) = req.headers().get("x-forwarded-for") {
        if let Ok(v) = h.to_str() {
            if let Some(first) = v.split(',').next() {
                if let Some(ip) = parse_ip(first) {
                    return Some(ip);
                }
            }
        }
    }
    req.peer_addr().map(|p| p.ip())
}

pub fn ip_to_bucket(ip: &IpAddr) -> String {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            format!("{}.{}.{}.0/24", o[0], o[1], o[2])
        }
        IpAddr::V6(v6) => {
            let s = v6.segments();
            format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
        }
    }
}

pub fn sha256_hex(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(input.as_bytes());
    hex::encode(h.finalize())
}
