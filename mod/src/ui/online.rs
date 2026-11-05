use super::*;
use horizon_server_browser as core;
use horizon_server_browser::directory::{
    ModrinthDirectory, Page, PublicDirectory, Request, ServerDirectory, combine_pages,
};
use std::{
    io::Read,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
static DIRECTORY_BUSY: AtomicBool = AtomicBool::new(false);
struct DirectoryJob;
impl Drop for DirectoryJob {
    fn drop(&mut self) {
        DIRECTORY_BUSY.store(false, Ordering::Release);
    }
}

fn matches_request(s: &Server, request: &Request) -> bool {
    let tag = request.tag.trim().to_lowercase();
    if !tag.is_empty() {
        let tag_match = s
            .metadata
            .tags
            .iter()
            .any(|t| t.to_lowercase().contains(&tag));
        let name_match = s.name.to_lowercase().contains(&tag);
        if !tag_match && !name_match {
            return false;
        }
    }
    true
}

#[derive(Clone, Debug, Default)]
pub(super) struct Directory {
    pub page: Option<Page>,
    pub message: Option<String>,
    pub last_filter: Option<(String, String)>,
    pub requested_page: u32,
    busy: Option<Arc<AtomicBool>>,
    retry_at: Option<Instant>,
    refresh_at: Option<Instant>,
    pub(super) reply: Arc<Mutex<Option<std::result::Result<Page, String>>>>,
}
impl Directory {
    pub fn refresh(&mut self) {
        self.refresh_at = Some(Instant::now());
    }
    pub fn needs_first_page(&self, now: Instant) -> bool {
        !self.loading()
            && self.retry_at.is_none_or(|at| now >= at)
            && (self.page.is_none() || self.refresh_at.is_some_and(|at| now >= at))
    }
    pub fn loading(&self) -> bool {
        self.busy
            .as_ref()
            .is_some_and(|b| b.load(Ordering::Acquire))
    }
    pub fn can_request_next(&self) -> bool {
        !self.loading()
            && self.retry_at.is_none_or(|at| Instant::now() >= at)
            && self
                .page
                .as_ref()
                .is_some_and(|p| p.page < p.pages && self.requested_page <= p.page)
    }
    pub fn cancel(&mut self) {
        if let Some(flag) = self.busy.take() {
            flag.store(false, Ordering::Release);
        }
        self.page = None;
        self.message = None;
        self.retry_at = None;
        self.refresh_at = None;
        self.reply = Arc::default();
        self.requested_page = 0;
    }
    pub fn poll(&mut self) -> bool {
        let reply = self.reply.lock().unwrap_or_else(|e| e.into_inner()).take();
        match reply {
            Some(Ok(mut new_page)) => {
                if new_page.partial
                    && self.requested_page == 1
                    && let Some(existing) = &self.page
                {
                    let mut seen: std::collections::HashSet<_> = new_page
                        .servers
                        .iter()
                        .filter_map(|s| core::address::key(&s.address).ok())
                        .collect();
                    for server in &existing.servers {
                        if let Ok(key) = core::address::key(&server.address)
                            && seen.insert(key)
                        {
                            new_page.servers.push(server.clone());
                        }
                    }
                    for (key, status) in &existing.statuses {
                        new_page
                            .statuses
                            .entry(key.clone())
                            .or_insert_with(|| status.clone());
                    }
                    new_page.pages = new_page.pages.max(existing.pages);
                    new_page.total = new_page.total.max(existing.total);
                }
                if new_page.page > 1
                    && let Some(existing) = &mut self.page
                {
                    let mut seen: std::collections::HashSet<_> = existing
                        .servers
                        .iter()
                        .filter_map(|s| core::address::key(&s.address).ok())
                        .collect();
                    for s in new_page.servers {
                        if let Ok(key) = core::address::key(&s.address)
                            && seen.insert(key)
                        {
                            existing.servers.push(s);
                        }
                    }
                    for (k, v) in new_page.statuses {
                        existing.statuses.entry(k).or_insert(v);
                    }
                    existing.page = new_page.page;
                    existing.partial |= new_page.partial;
                    existing.pages = new_page.pages;
                    existing.total = existing
                        .total
                        .max(new_page.total)
                        .max(existing.servers.len() as u64);
                } else {
                    self.page = Some(new_page);
                }
                self.message = if self.page.as_ref().is_some_and(|page| page.partial) {
                    Some(tr("directory_partial"))
                } else {
                    None
                };
                self.retry_at = None;
                self.refresh_at = Some(Instant::now() + Duration::from_secs(300));
                true
            }
            Some(Err(error)) => {
                self.message = Some(error);
                self.requested_page = self.page.as_ref().map_or(0, |p| p.page);
                self.retry_at = Some(Instant::now() + Duration::from_secs(60));
                false
            }
            None => false,
        }
    }
    pub fn request(&mut self, home: PathBuf, consent: bool, request: Request) {
        if !consent || self.loading() {
            return;
        }
        // Cancellation never waits for OS DNS/HTTP. Keep at most one worker
        // alive even while the user repeatedly changes filters or reopens.
        if DIRECTORY_BUSY
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }
        let job = DirectoryJob;
        self.requested_page = request.page;
        let busy = Arc::new(AtomicBool::new(true));
        self.busy = Some(busy.clone());
        let reply = self.reply.clone();
        let all_curated = super::provider_servers();
        let had_page = self.page.is_some();
        self.message = None;
        let spawn = std::thread::Builder::new()
            .name("browser-directory".into())
            .spawn(move || {
                let _job = job;
                let provider = PublicDirectory {
                    consent,
                    home: home.clone(),
                    fetch: fetch as fn(&str) -> core::Result<Vec<u8>>,
                };
                let modrinth = ModrinthDirectory {
                    consent,
                    home,
                    fetch: fetch as fn(&str) -> core::Result<Vec<u8>>,
                };
                let remote = combine_pages(
                    provider.page(&request),
                    modrinth.page(&request),
                    request.page,
                );
                let mut curated_matches: Vec<Server> = all_curated
                    .into_iter()
                    .filter(|s| matches_request(s, &request))
                    .collect();
                // A failed refresh keeps the last good page. Only a cold start
                // falls back to the shipped address snapshot; it has no cached
                // player counts or claims about current availability.
                if remote.is_err() && request.page == 1 && !had_page {
                    curated_matches.extend(
                        super::catalog::DIRECTORY_SNAPSHOT
                            .iter()
                            .map(|(name, address, tags)| {
                                let mut server = Server {
                                    name: (*name).into(),
                                    address: (*address).into(),
                                    ..Server::default()
                                };
                                server.metadata.tags =
                                    tags.iter().map(|tag| (*tag).into()).collect();
                                server
                            })
                            .filter(|s| matches_request(s, &request)),
                    );
                }
                let result = if remote.is_err() && request.page == 1 && had_page {
                    Err(tr("directory_error"))
                } else {
                    merge_directory(remote, curated_matches, request.page)
                };
                if busy.load(Ordering::Acquire) {
                    *reply.lock().unwrap_or_else(|e| e.into_inner()) = Some(result);
                    busy.store(false, Ordering::Release);
                }
            });
        if spawn.is_err() {
            if let Some(flag) = &self.busy {
                flag.store(false, Ordering::Release);
            }
            self.message = Some(tr("directory_error"));
        }
    }
}
// Curated entries are one prefix, never a second pagination scheme. A failed
// remote page must not advance the remote cursor or skip servers during throttling.
pub(super) fn merge_directory(
    remote: core::Result<Page>,
    curated: Vec<Server>,
    number: u32,
) -> Result<Page, String> {
    let mut page = match remote {
        Ok(page) => page,
        Err(_) if number == 1 && !curated.is_empty() => Page {
            page: 1,
            pages: 1,
            total: 0,
            partial: true,
            ..Page::default()
        },
        Err(error) => {
            return Err(match error {
                core::Error::RateLimited(_) => tr("directory_rate"),
                _ => tr("directory_error"),
            });
        }
    };
    if number == 1 {
        let mut seen = std::collections::HashSet::new();
        let mut servers = Vec::new();
        let mut added = 0;
        let remote_count = page.servers.len();
        for (index, server) in page.servers.drain(..).chain(curated).enumerate() {
            if !server.metadata.tags.iter().any(|tag| query::offline_access_tag(tag))
                && let Ok(key) = core::address::key(&server.address)
                && seen.insert(key)
            {
                if index >= remote_count {
                    added += 1;
                }
                servers.push(server);
            }
        }
        page.servers = servers;
        page.total = (page.total + added).max(page.servers.len() as u64);
    }
    page.servers.retain(|server| {
        !server.metadata.tags.iter().any(|tag| query::offline_access_tag(tag))
    });
    Ok(page)
}

fn fetch(url: &str) -> core::Result<Vec<u8>> {
    if !url.starts_with("https://minecraftserve.rs/api/servers?")
        && !url.starts_with("https://api.modrinth.com/v3/search?")
    {
        return Err(core::Error::Invalid("directory origin refused".into()));
    }
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .timeout(Duration::from_secs(8))
        .build();
    let response = match agent
        .get(url)
        .set(
            "User-Agent",
            "ServerAtlas/2.2.0 (+https://musti3214.github.io/Horizon-Server-Browser/)",
        )
        .call()
    {
        Ok(response) => response,
        Err(ureq::Error::Status(429, response)) => {
            return Err(core::Error::RateLimited(
                response
                    .header("Retry-After")
                    .and_then(|s| {
                        s.parse().ok().or_else(|| {
                            chrono::DateTime::parse_from_rfc2822(s).ok().map(|date| {
                                (date.timestamp() - chrono::Utc::now().timestamp()).max(60) as u64
                            })
                        })
                    })
                    .unwrap_or(60),
            ));
        }
        Err(_) => return Err(core::Error::Invalid("directory request failed".into())),
    };
    if response.status() != 200 {
        return Err(core::Error::Invalid("directory response refused".into()));
    }
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(8_388_609)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 8_388_608 {
        return Err(core::Error::Invalid("directory response too large".into()));
    }
    Ok(bytes)
}

#[cfg(test)]
mod freshness_tests {
    use super::*;
    #[test]
    fn a_partial_refresh_keeps_the_previous_good_servers() {
        let server = |name: &str, address: &str| Server {
            name: name.into(),
            address: address.into(),
            ..Server::default()
        };
        let mut directory = Directory::default();
        directory.requested_page = 1;
        directory.page = Some(Page {
            servers: vec![server("Previous source", "old.example.org")],
            page: 1,
            pages: 5,
            total: 201,
            ..Page::default()
        });
        *directory.reply.lock().unwrap() = Some(Ok(Page {
            servers: vec![server("Available source", "new.example.org")],
            page: 1,
            pages: 2,
            partial: true,
            ..Page::default()
        }));
        assert!(directory.poll());
        let page = directory.page.as_ref().unwrap();
        assert_eq!(page.servers.len(), 2);
        assert_eq!(page.pages, 5);
        assert!(directory.message.is_some());
    }
    #[test]
    fn refresh_retains_good_results_and_respects_the_retry_window() {
        let now = Instant::now();
        let mut directory = Directory::default();
        assert!(directory.needs_first_page(now));
        *directory.reply.lock().unwrap() = Some(Ok(Page {
            page: 1,
            pages: 1,
            ..Page::default()
        }));
        assert!(directory.poll());
        assert!(!directory.needs_first_page(now + Duration::from_secs(299)));
        assert!(directory.needs_first_page(now + Duration::from_secs(301)));
        *directory.reply.lock().unwrap() = Some(Err("temporary failure".into()));
        assert!(!directory.poll());
        assert!(directory.page.is_some());
        assert!(!directory.needs_first_page(now + Duration::from_secs(59)));
        directory.refresh_at = Some(now);
        assert!(directory.needs_first_page(now + Duration::from_secs(61)));
        directory.cancel();
        assert!(directory.page.is_none());
    }
}
