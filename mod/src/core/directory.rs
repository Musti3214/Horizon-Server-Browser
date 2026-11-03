//! Providers share pagination without exposing private saved data to a service.
//! HTTP is injected by the client, keeping fixtures and cache tests offline.
use crate::{
    Error, Result, address,
    model::{Health, Server, Status},
    persistence,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Mutex,
};

pub const ORIGIN: &str = "https://minecraftserve.rs";
const TTL: u64 = 300;
static REQUEST_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Request {
    pub page: u32,
    pub tag: String,
    pub version: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Page {
    pub servers: Vec<Server>,
    pub statuses: HashMap<String, Status>,
    pub page: u32,
    pub pages: u32,
    pub total: u64,
    #[serde(default)]
    pub partial: bool,
}
pub trait ServerDirectory {
    fn page(&self, request: &Request) -> Result<Page>;
}

pub struct LocalDirectory {
    pub servers: Vec<Server>,
}
impl ServerDirectory for LocalDirectory {
    fn page(&self, request: &Request) -> Result<Page> {
        let page = request.page.max(1);
        let total = self.servers.len();
        let start = (page as usize - 1).saturating_mul(50);
        Ok(Page {
            servers: self.servers.iter().skip(start).take(50).cloned().collect(),
            page,
            pages: total.div_ceil(50).max(1) as u32,
            total: total as u64,
            ..Page::default()
        })
    }
}
pub struct MockDirectory {
    pub pages: Vec<Page>,
}
impl ServerDirectory for MockDirectory {
    fn page(&self, request: &Request) -> Result<Page> {
        self.pages
            .iter()
            .find(|p| p.page == request.page.max(1))
            .cloned()
            .ok_or_else(|| Error::Invalid("fixture page not found".into()))
    }
}

#[derive(Serialize, Deserialize)]
struct Cache {
    schema: u32,
    expires: u64,
    page: Page,
}
#[derive(Serialize, Deserialize)]
struct Rate {
    schema: u32,
    next: u64,
}
pub struct PublicDirectory<F> {
    pub consent: bool,
    pub home: PathBuf,
    pub fetch: F,
}
impl<F: Fn(&str) -> Result<Vec<u8>>> ServerDirectory for PublicDirectory<F> {
    fn page(&self, request: &Request) -> Result<Page> {
        self.page_at(request, crate::unix_seconds())
    }
}
impl<F: Fn(&str) -> Result<Vec<u8>>> PublicDirectory<F> {
    pub fn page_at(&self, request: &Request, now: u64) -> Result<Page> {
        if self.consent
            && let Some(page) = exhausted_page(&self.home, request, &url)
        {
            return Ok(page);
        }
        cached_page(
            self.consent,
            &self.home,
            &url(request),
            &self.fetch,
            parse_page,
            now,
        )
    }
}

// A shorter source ends cleanly while the other source still has more pages.
// Its last known page count remains valid for this pagination session; refreshing
// page one obtains a new count. No out-of-range HTTP request is made.
fn exhausted_page(
    home: &Path,
    request: &Request,
    make_url: &impl Fn(&Request) -> String,
) -> Option<Page> {
    if request.page <= 1 {
        return None;
    }
    let first = Request {
        page: 1,
        ..request.clone()
    };
    let path = home
        .join("server-directory-cache")
        .join(format!("{:016x}.json", fingerprint(&make_url(&first))));
    let cache: Cache = serde_json::from_slice(&persistence::read_bounded(&path).ok()?).ok()?;
    if cache.schema != 1 || request.page <= cache.page.pages {
        return None;
    }
    Some(Page {
        page: request.page,
        pages: cache.page.pages,
        total: cache.page.total,
        ..Page::default()
    })
}

fn cached_page<F, P>(
    consent: bool,
    home: &Path,
    url: &str,
    fetch: &F,
    parse: P,
    now: u64,
) -> Result<Page>
where
    F: Fn(&str) -> Result<Vec<u8>>,
    P: Fn(&[u8]) -> Result<Page>,
{
    if !consent {
        return Err(Error::Invalid("online directory consent required".into()));
    }
    let _guard = REQUEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = home.join("server-directory-cache");
    let cache_path = home.join(format!("{:016x}.json", fingerprint(url)));
    if let Ok(bytes) = persistence::read_bounded(&cache_path)
        && let Ok(cache) = serde_json::from_slice::<Cache>(&bytes)
        && cache.schema == 1
        && cache.expires > now
    {
        return Ok(cache.page);
    }
    let rate_path = home.join("rate.json");
    if let Ok(bytes) = persistence::read_bounded(&rate_path)
        && let Ok(rate) = serde_json::from_slice::<Rate>(&bytes)
        && rate.next > now
    {
        return Err(Error::RateLimited(rate.next - now));
    }
    // Persist BEFORE HTTP so crashing/restarting cannot hammer the service.
    write_rate(&rate_path, now.saturating_add(60))?;
    let bytes = match fetch(url) {
        Err(Error::RateLimited(seconds)) => {
            write_rate(&rate_path, now.saturating_add(seconds.max(60)))?;
            return Err(Error::RateLimited(seconds.max(60)));
        }
        other => other?,
    };
    let page = parse(&bytes)?;
    persistence::atomic_write(
        &cache_path,
        &serde_json::to_vec(&Cache {
            schema: 1,
            expires: now.saturating_add(TTL),
            page: page.clone(),
        })?,
    )?;
    Ok(page)
}

/// Independent public API integration based on Modrinth's published data format.
/// It never downloads images, mods, modpacks or executable content.
pub struct ModrinthDirectory<F> {
    pub consent: bool,
    pub home: PathBuf,
    pub fetch: F,
}
impl<F: Fn(&str) -> Result<Vec<u8>>> ServerDirectory for ModrinthDirectory<F> {
    fn page(&self, request: &Request) -> Result<Page> {
        self.page_at(request, crate::unix_seconds())
    }
}
impl<F: Fn(&str) -> Result<Vec<u8>>> ModrinthDirectory<F> {
    pub fn page_at(&self, request: &Request, now: u64) -> Result<Page> {
        if self.consent
            && let Some(page) = exhausted_page(&self.home.join("modrinth"), request, &modrinth_url)
        {
            return Ok(page);
        }
        cached_page(
            self.consent,
            &self.home.join("modrinth"),
            &modrinth_url(request),
            &self.fetch,
            |bytes| parse_modrinth(bytes, request.page.max(1)),
            now,
        )
    }
}
pub fn modrinth_url(request: &Request) -> String {
    let mut facets = vec![vec!["project_types:minecraft_java_server".to_owned()]];
    if !request.tag.trim().is_empty() {
        facets.push(vec![format!(
            "categories:{}",
            request.tag.chars().take(128).collect::<String>()
        )]);
    }
    // Supported Minecraft versions are not inferred from unrelated project fields.
    format!(
        "https://api.modrinth.com/v3/search?limit=50&offset={}&index=relevance&facets={}",
        request.page.clamp(1, 200).saturating_sub(1) * 50,
        encode(&serde_json::to_string(&facets).expect("string arrays serialize"))
    )
}
fn public_address(value: &str) -> Result<String> {
    let (host, _) = address::parse(value)?;
    let allowed = match host.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(ip)) => {
            !ip.is_private()
                && !ip.is_loopback()
                && !ip.is_link_local()
                && !ip.is_unspecified()
                && !ip.is_multicast()
                && !ip.is_broadcast()
                && !(ip.octets()[0] == 100 && (64..=127).contains(&ip.octets()[1]))
        }
        Ok(std::net::IpAddr::V6(ip)) => {
            !ip.is_loopback()
                && !ip.is_unspecified()
                && !ip.is_multicast()
                && !ip.is_unique_local()
                && !ip.is_unicast_link_local()
                && ip.to_ipv4_mapped().is_none()
        }
        Err(_) => {
            let name = host.trim_end_matches('.').to_ascii_lowercase();
            name.contains('.')
                && ![".local", ".localhost", ".internal", ".home.arpa"]
                    .iter()
                    .any(|suffix| name.ends_with(suffix))
        }
    };
    if !allowed {
        return Err(Error::Invalid("non-public directory address".into()));
    }
    address::key(value)
}
pub fn parse_modrinth(bytes: &[u8], number: u32) -> Result<Page> {
    if bytes.len() > 8_388_608 {
        return Err(Error::Invalid("Modrinth response too large".into()));
    }
    let json: serde_json::Value = serde_json::from_slice(bytes)?;
    let hits = json["hits"]
        .as_array()
        .ok_or_else(|| Error::Invalid("missing Modrinth hits".into()))?;
    if hits.len() > 50 {
        return Err(Error::Invalid("Modrinth page exceeds 50 entries".into()));
    }
    let total = json["total_hits"]
        .as_u64()
        .ok_or_else(|| Error::Invalid("missing Modrinth total".into()))?
        .min(10_000);
    let mut page = Page {
        page: number,
        pages: total.div_ceil(50).max(1) as u32,
        total,
        ..Page::default()
    };
    let mut seen = std::collections::HashSet::new();
    let clean = |s: &str, n: usize| {
        s.chars()
            .filter(|c| !c.is_control())
            .take(n)
            .collect::<String>()
    };
    for hit in hits {
        if !hit["project_types"].as_array().is_some_and(|types| {
            types
                .iter()
                .any(|kind| kind.as_str() == Some("minecraft_java_server"))
        }) {
            continue;
        }
        let Some(raw) = hit["minecraft_java_server"]["address"].as_str() else {
            continue;
        };
        let Ok(address) = public_address(raw) else {
            continue;
        };
        if !seen.insert(address.clone()) {
            continue;
        }
        let mut server = Server {
            name: clean(hit["name"].as_str().unwrap_or(raw), 256),
            address,
            ..Server::default()
        };
        server.metadata.tags = hit["categories"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str())
            .take(31)
            .map(|s| clean(s, 64))
            .collect();
        if hit["minecraft_java_server"]["content"]["kind"].as_str() == Some("modpack")
            && !server.metadata.tags.iter().any(|tag| tag == "modded")
        {
            server.metadata.tags.push("modded".into());
        }
        server.metadata.group = "Modrinth".into();
        server.metadata.region =
            clean(hit["minecraft_server"]["region"].as_str().unwrap_or(""), 64);
        server.metadata.language = clean(
            hit["minecraft_server"]["languages"]
                .as_array()
                .and_then(|v| v.first())
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            32,
        );
        // Actual ping, compatibility, player count and authentication remain unknown
        // until our existing direct status checks provide them.
        page.servers.push(server);
    }
    Ok(page)
}

pub fn combine_pages(first: Result<Page>, second: Result<Page>, number: u32) -> Result<Page> {
    let (mut page, other) = match (first, second) {
        (Ok(a), Ok(b)) => (a, Some(b)),
        (Ok(mut page), Err(_)) | (Err(_), Ok(mut page)) => {
            page.partial = true;
            (page, None)
        }
        (Err(Error::RateLimited(a)), Err(Error::RateLimited(b))) => {
            return Err(Error::RateLimited(a.min(b)));
        }
        (Err(error), Err(_)) => return Err(error),
    };
    page.page = number;
    if let Some(other) = other {
        let mut seen: std::collections::HashSet<_> = page
            .servers
            .iter()
            .filter_map(|s| address::key(&s.address).ok())
            .collect();
        let mut duplicates = 0;
        for server in other.servers {
            if let Ok(key) = address::key(&server.address)
                && seen.insert(key)
            {
                page.servers.push(server);
            } else {
                duplicates += 1;
            }
        }
        page.statuses.extend(other.statuses);
        page.pages = page.pages.max(other.pages);
        page.total = page
            .total
            .saturating_add(other.total)
            .saturating_sub(duplicates);
        page.partial |= other.partial;
    }
    Ok(page)
}
fn write_rate(path: &Path, next: u64) -> Result<()> {
    persistence::atomic_write(path, &serde_json::to_vec(&Rate { schema: 1, next })?)
}
fn fingerprint(value: &str) -> u64 {
    value.bytes().fold(0xcbf29ce484222325, |hash, b| {
        (hash ^ u64::from(b)).wrapping_mul(0x100000001b3)
    })
}
fn encode(value: &str) -> String {
    let mut result = String::new();
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            result.push(char::from(b));
        } else {
            result.push_str(&format!("%{b:02X}"));
        }
    }
    result
}
pub fn url(request: &Request) -> String {
    let mut url = format!(
        "{ORIGIN}/api/servers?edition=java&per=50&page={}",
        request.page.clamp(1, 10000)
    );
    for (key, value) in [("tag", &request.tag), ("version", &request.version)] {
        if !value.is_empty() {
            url.push_str(&format!(
                "&{key}={}",
                encode(&value.chars().take(128).collect::<String>())
            ));
        }
    }
    url
}
pub fn parse_page(bytes: &[u8]) -> Result<Page> {
    if bytes.len() > 2_097_152 {
        return Err(Error::Invalid("directory response too large".into()));
    }
    let json: serde_json::Value = serde_json::from_slice(bytes)?;
    let entries = json["servers"]
        .as_array()
        .ok_or_else(|| Error::Invalid("missing directory servers".into()))?;
    if entries.len() > 50 {
        return Err(Error::Invalid("directory page exceeds 50 entries".into()));
    }
    let mut page = Page {
        page: json["page"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .unwrap_or(1)
            .max(1),
        pages: json["pages"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .unwrap_or(1)
            .max(1),
        total: json["total"].as_u64().unwrap_or(entries.len() as u64),
        ..Page::default()
    };
    for value in entries {
        if value["edition"].as_str() != Some("java") {
            continue;
        }
        let Some(host) = value["host"].as_str() else {
            continue;
        };
        let port = value["port"]
            .as_u64()
            .and_then(|n| u16::try_from(n).ok())
            .unwrap_or(25565);
        let address = if host.contains(':') {
            format!("[{host}]:{port}")
        } else {
            format!("{host}:{port}")
        };
        let Ok(key) = address::key(&address) else {
            continue;
        };
        if page
            .servers
            .iter()
            .any(|s| address::key(&s.address).is_ok_and(|k| k == key))
        {
            continue;
        }
        let mut server = Server {
            name: value["name"]
                .as_str()
                .unwrap_or(host)
                .chars()
                .take(256)
                .collect(),
            address: address.clone(),
            ..Server::default()
        };
        server.metadata.tags = value["tags"]
            .as_array()
            .into_iter()
            .flatten()
            .take(32)
            .filter_map(|v| v.as_str())
            .map(|s| s.chars().take(64).collect())
            .collect();
        let mut status=crate::service::parse_response(&serde_json::to_vec(&serde_json::json!({"description":value["motd"],"players":value["players"],"version":{"name":value["version"]}}))?)
            .map_err(|_|Error::Invalid("invalid directory status".into()))?;
        status.health = match value["online"].as_bool() {
            Some(true) => Health::Online,
            Some(false) => Health::Offline,
            None => Health::Pinging,
        };
        // Remote icon URLs are deliberately not fetched. They may point at an
        // unrelated origin; runtime resource-pack defaults are always available.
        page.statuses.insert(address, status);
        page.servers.push(server);
    }
    Ok(page)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    const FIXTURE:&[u8]=br#"{"servers":[{"name":"Local fixture","host":"localhost","port":25565,"edition":"java","online":true,"motd":"Hello","version":"26.3","players":{"online":2,"max":10},"tags":["survival"]},{"host":"bedrock.example","edition":"bedrock"}],"page":1,"pages":2,"total":51}"#;
    const MODRINTH:&[u8]=br#"{"hits":[{"project_types":["minecraft_java_server"],"name":"Public fixture","minecraft_java_server":{"address":"PLAY.EXAMPLE.ORG.","content":{"kind":"modpack"},"ping":{"data":{"players_online":9999,"version_protocol":777}}},"categories":["smp"],"minecraft_server":{"region":"europe","languages":["de"]}},{"project_types":["minecraft_java_server"],"minecraft_java_server":{"address":"play.example.org:25565"}},{"project_types":["minecraft_bedrock_server"],"minecraft_java_server":{"address":"bedrock.example.org"}},{"project_types":["minecraft_java_server"],"minecraft_java_server":{"address":"127.0.0.1:25565"}}],"total_hits":201}"#;
    #[test]
    fn modrinth_data_is_bounded_deduplicated_and_not_a_live_status() {
        let page = parse_modrinth(MODRINTH, 2).unwrap();
        assert_eq!((page.page, page.pages, page.total), (2, 5, 201));
        assert_eq!(page.servers.len(), 1);
        assert_eq!(page.servers[0].address, "play.example.org:25565");
        assert_eq!(page.servers[0].metadata.group, "Modrinth");
        assert_eq!(page.servers[0].metadata.language, "de");
        assert!(
            page.servers[0]
                .metadata
                .tags
                .iter()
                .any(|tag| tag == "modded")
        );
        assert_eq!(
            page.servers[0].metadata.auth,
            crate::model::AuthMode::Unknown
        );
        assert!(page.statuses.is_empty());
        for value in [
            "localhost",
            "192.168.1.1",
            "[::1]",
            "[::ffff:127.0.0.1]",
            "router.home.arpa",
            "host.local",
            "https://example.org",
        ] {
            assert!(public_address(value).is_err(), "{value}");
        }
        assert!(parse_modrinth(br#"{"hits":null,"total_hits":1}"#, 1).is_err());
        assert!(parse_modrinth(&vec![b' '; 8_388_609], 1).is_err());
        let oversized = serde_json::to_vec(
            &serde_json::json!({"hits":vec![serde_json::json!({});51],"total_hits":51}),
        )
        .unwrap();
        assert!(parse_modrinth(&oversized, 1).is_err());
    }
    #[test]
    fn sources_have_independent_persistent_limits_and_merge_partial_failures() {
        let home =
            std::env::temp_dir().join(format!("browser-two-directories-{}", std::process::id()));
        let calls = AtomicUsize::new(0);
        let first = PublicDirectory {
            consent: true,
            home: home.clone(),
            fetch: |_: &str| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(FIXTURE.to_vec())
            },
        };
        let second = ModrinthDirectory {
            consent: true,
            home: home.clone(),
            fetch: |_: &str| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(MODRINTH.to_vec())
            },
        };
        let request = Request::default();
        let merged = combine_pages(
            first.page_at(&request, 1000),
            second.page_at(&request, 1000),
            1,
        )
        .unwrap();
        assert_eq!(merged.servers.len(), 2);
        assert_eq!(merged.pages, 5);
        first.page_at(&request, 1001).unwrap();
        second.page_at(&request, 1001).unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let exhausted = first
            .page_at(
                &Request {
                    page: 3,
                    ..Request::default()
                },
                1001,
            )
            .unwrap();
        assert!(exhausted.servers.is_empty());
        assert!(!exhausted.partial);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let next = Request {
            page: 2,
            ..Request::default()
        };
        assert!(matches!(
            second.page_at(&next, 1001),
            Err(Error::RateLimited(59))
        ));
        assert!(matches!(
            first.page_at(&next, 1001),
            Err(Error::RateLimited(59))
        ));
        let duplicate = combine_pages(Ok(merged.clone()), Ok(merged.clone()), 1).unwrap();
        assert_eq!(duplicate.servers.len(), 2);
        assert!(
            combine_pages(Ok(merged), Err(Error::Invalid("unavailable".into())), 1)
                .unwrap()
                .partial
        );
        assert!(
            ModrinthDirectory {
                consent: false,
                home: home.clone(),
                fetch: |_: &str| panic!("consent off must not fetch")
            }
            .page_at(&request, 1000)
            .is_err()
        );
        std::fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn modrinth_only_receives_public_tag_filters() {
        let url = modrinth_url(&Request {
            page: 2,
            tag: "survival&private=secret".into(),
            version: "26.3".into(),
        });
        assert!(url.starts_with("https://api.modrinth.com/v3/search?"));
        assert!(url.contains("offset=50"));
        assert!(url.contains("minecraft_java_server"));
        assert!(!url.contains("&private=") && !url.contains("26.3"));
    }
    #[test]
    fn fixtures_local_pages_and_unknown_auth() {
        let page = parse_page(FIXTURE).unwrap();
        assert_eq!(page.servers.len(), 1);
        assert_eq!(page.pages, 2);
        assert_eq!(
            page.servers[0].metadata.auth,
            crate::model::AuthMode::Unknown
        );
        let mock = MockDirectory {
            pages: vec![page.clone()],
        };
        assert_eq!(mock.page(&Request::default()).unwrap().total, 51);
        let local = LocalDirectory {
            servers: (0..101).map(|_| page.servers[0].clone()).collect(),
        };
        assert_eq!(
            local
                .page(&Request {
                    page: 3,
                    ..Request::default()
                })
                .unwrap()
                .servers
                .len(),
            1
        );
        assert!(parse_page(b"garbage").is_err());
        assert!(parse_page(br#"{"servers":null}"#).is_err());
    }
    #[test]
    fn consent_ttl_persistent_rate_limit_and_retry_after() {
        let home = std::env::temp_dir().join(format!("horizon-directory-{}", std::process::id()));
        std::fs::create_dir_all(&home).unwrap();
        let calls = AtomicUsize::new(0);
        let mut directory = PublicDirectory {
            consent: false,
            home: home.clone(),
            fetch: |_: &str| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(FIXTURE.to_vec())
            },
        };
        let request = Request::default();
        assert!(directory.page_at(&request, 1000).is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        directory.consent = true;
        directory.page_at(&request, 1000).unwrap();
        directory.page_at(&request, 1001).unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(matches!(
            directory.page_at(
                &Request {
                    page: 2,
                    ..Request::default()
                },
                1001
            ),
            Err(Error::RateLimited(59))
        ));
        directory.page_at(&request, 1301).unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let failing = PublicDirectory {
            consent: true,
            home: home.clone(),
            fetch: |_: &str| Err(Error::RateLimited(600)),
        };
        assert!(matches!(
            failing.page_at(
                &Request {
                    page: 2,
                    ..Request::default()
                },
                1400
            ),
            Err(Error::RateLimited(600))
        ));
        assert!(matches!(
            directory.page_at(
                &Request {
                    page: 2,
                    ..Request::default()
                },
                1401
            ),
            Err(Error::RateLimited(599))
        ));
        std::fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn requests_only_send_allowlisted_public_parameters() {
        let url = url(&Request {
            page: 2,
            tag: "a&private=secret".into(),
            version: "26.3".into(),
        });
        assert!(url.starts_with(ORIGIN));
        assert!(url.contains("tag=a%26private%3Dsecret"));
        assert!(!url.contains("&private="));
    }
}
