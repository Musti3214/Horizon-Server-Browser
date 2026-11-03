use crate::{Error, Result};
use std::net::{IpAddr, Ipv6Addr};

/// Validate without DNS or network traffic. Brackets are mandatory around IPv6
/// when a port is supplied. Bare IPv6 always means the default port.
pub fn parse(input: &str) -> Result<(String, u16)> {
    let input = input.trim();
    if input.is_empty()
        || input.chars().any(|c| c.is_whitespace() || c.is_control())
        || input.contains(['/', '\\', '@', '?', '#'])
    {
        return Err(Error::Invalid("invalid server address".into()));
    }
    let (host, port) = horizon_protocol::servers::parse_address(input)
        .ok_or_else(|| Error::Invalid("invalid port or IPv6 brackets".into()))?;
    if port == 0 || host.is_empty() || host.len() > 253 {
        return Err(Error::Invalid("invalid host or port".into()));
    }
    if host.contains(':') || input.starts_with('[') {
        host.parse::<Ipv6Addr>()
            .map_err(|_| Error::Invalid("invalid IPv6 address".into()))?;
    } else if host.parse::<IpAddr>().is_err() {
        let domain = host.trim_end_matches('.');
        if domain.is_empty()
            || domain.split('.').any(|part| {
                part.is_empty()
                    || part.len() > 63
                    || part.starts_with('-')
                    || part.ends_with('-')
                    || !part
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            })
        {
            return Err(Error::Invalid("invalid host name".into()));
        }
    }
    Ok((host, port))
}

pub fn key(input: &str) -> Result<String> {
    let (host, port) = parse(input)?;
    let host = match host.parse::<IpAddr>() {
        Ok(ip) => ip.to_string(),
        Err(_) => host.trim_end_matches('.').to_lowercase(),
    };
    Ok(if host.contains(':') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_and_normalizes_without_looking_up_hosts() {
        for address in [
            "localhost",
            "127.0.0.1:12",
            "[2001:db8::1]:25565",
            "::1",
            "play.example.org.",
        ] {
            assert!(parse(address).is_ok(), "{address}");
        }
        assert_eq!(
            key("PLAY.Example.org.").unwrap(),
            key("play.example.org:25565").unwrap()
        );
        assert_eq!(key("[0:0:0:0:0:0:0:1]").unwrap(), "[::1]:25565");
        for address in [
            "",
            ":25565",
            "a:0",
            "a:65536",
            "[hello]",
            "[::1]x",
            "a b",
            "https://a",
            "a@b",
            "a..b",
            "a\n",
        ] {
            // Leading/trailing whitespace is deliberately accepted like paste in vanilla.
            if address != "a\n" {
                assert!(parse(address).is_err(), "{address}");
            }
        }
    }
}
