//! Bounded workers, TTL cache and generation cancellation. Dropping a screen
//! never waits for DNS or sockets; old generations cannot publish late results.
use crate::{
    address,
    model::{Health, Status},
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PingError {
    Timeout,
    Dns,
    Offline,
    Malformed,
}
impl PingError {
    pub fn health(self) -> Health {
        match self {
            Self::Timeout => Health::Timeout,
            Self::Dns => Health::DnsError,
            Self::Offline => Health::Offline,
            Self::Malformed => Health::Malformed,
        }
    }
}
type PingResult = std::result::Result<Status, PingError>;
type Probe = dyn Fn(&str, Duration) -> PingResult + Send + Sync;
type Job = (u64, String);

#[derive(Default)]
struct Shared {
    jobs: Mutex<VecDeque<Job>>,
    results: Mutex<VecDeque<JobResult>>,
    cache: Mutex<HashMap<String, (Instant, Status)>>,
    wake: Condvar,
    stop: AtomicBool,
    generation: AtomicU64,
}
type JobResult = (u64, String, Status);
struct Lease {
    shared: Arc<Shared>,
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        self.shared.wake.notify_all();
    }
}
#[derive(Clone)]
pub struct PingService {
    lease: Arc<Lease>,
}
impl std::fmt::Debug for PingService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PingService")
    }
}
impl PingService {
    pub fn new(concurrency: usize) -> std::io::Result<Self> {
        Self::with_probe(concurrency, Arc::new(ping))
    }
    pub(crate) fn with_probe(concurrency: usize, probe: Arc<Probe>) -> std::io::Result<Self> {
        let shared = Arc::new(Shared::default());
        let service = Self {
            lease: Arc::new(Lease {
                shared: shared.clone(),
            }),
        };
        for n in 0..concurrency.clamp(1, 16) {
            let (shared, probe) = (shared.clone(), probe.clone());
            std::thread::Builder::new()
                .name(format!("browser-ping-{n}"))
                .spawn(move || worker(shared, probe))?;
        }
        Ok(service)
    }
    pub fn refresh(&self, addresses: impl IntoIterator<Item = String>, force: bool) {
        let shared = &self.lease.shared;
        let generation = shared.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let mut jobs = shared.jobs.lock().unwrap_or_else(|e| e.into_inner());
        jobs.clear();
        let mut results = shared.results.lock().unwrap_or_else(|e| e.into_inner());
        results.clear();
        let cache = shared.cache.lock().unwrap_or_else(|e| e.into_inner());
        let mut seen = HashSet::new();
        for address in addresses.into_iter().take(100_000) {
            if !seen.insert(address.clone()) {
                continue;
            }
            if !force
                && let Some((when, status)) = cache.get(&address)
                && when.elapsed() < Duration::from_secs(30)
            {
                results.push_back((generation, address, status.clone()));
                continue;
            }
            jobs.push_back((generation, address));
        }
        drop(cache);
        drop(results);
        drop(jobs);
        shared.wake.notify_all();
    }
    pub fn drain(&self) -> Vec<(String, Status)> {
        let shared = &self.lease.shared;
        let generation = shared.generation.load(Ordering::Acquire);
        shared
            .results
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .drain(..)
            .filter(|(g, _, _)| *g == generation)
            .map(|(_, a, s)| (a, s))
            .collect()
    }
    /// Move visible rows ahead of background jobs without restarting probes or
    /// discarding completed results. Long lists must not starve a new server.
    pub fn prioritize(&self, addresses: impl IntoIterator<Item = String>) {
        let wanted = addresses.into_iter().take(256).collect::<HashSet<_>>();
        if wanted.is_empty() {
            return;
        }
        let mut jobs = self
            .lease
            .shared
            .jobs
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut front = VecDeque::new();
        let mut rest = VecDeque::new();
        while let Some(job) = jobs.pop_front() {
            if wanted.contains(&job.1) {
                front.push_back(job);
            } else {
                rest.push_back(job);
            }
        }
        front.append(&mut rest);
        *jobs = front;
    }
}
fn worker(shared: Arc<Shared>, probe: Arc<Probe>) {
    loop {
        let job = {
            let mut jobs = shared.jobs.lock().unwrap_or_else(|e| e.into_inner());
            while jobs.is_empty() && !shared.stop.load(Ordering::Acquire) {
                jobs = shared.wake.wait(jobs).unwrap_or_else(|e| e.into_inner());
            }
            if shared.stop.load(Ordering::Acquire) {
                return;
            }
            jobs.pop_front()
        };
        let Some((generation, address)) = job else {
            continue;
        };
        if generation != shared.generation.load(Ordering::Acquire) {
            continue;
        }
        let status = probe(&address, Duration::from_secs(5)).unwrap_or_else(|e| Status {
            health: e.health(),
            ..Status::default()
        });
        if shared.stop.load(Ordering::Acquire) {
            return;
        }
        if generation == shared.generation.load(Ordering::Acquire) {
            let mut cache = shared.cache.lock().unwrap_or_else(|e| e.into_inner());
            if cache.len() >= 100_000 {
                cache.clear();
            }
            cache.insert(address.clone(), (Instant::now(), status.clone()));
            drop(cache);
            shared
                .results
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push_back((generation, address, status));
        }
    }
}

fn io_error(e: std::io::Error) -> PingError {
    match e.kind() {
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => PingError::Timeout,
        _ => PingError::Offline,
    }
}
fn remaining(deadline: Instant) -> std::result::Result<Duration, PingError> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        Err(PingError::Timeout)
    } else {
        Ok(remaining)
    }
}
fn read_exact(
    stream: &mut TcpStream,
    mut out: &mut [u8],
    deadline: Instant,
) -> std::result::Result<(), PingError> {
    while !out.is_empty() {
        stream
            .set_read_timeout(Some(remaining(deadline)?))
            .map_err(io_error)?;
        let n = stream.read(out).map_err(io_error)?;
        if n == 0 {
            return Err(PingError::Offline);
        }
        out = &mut out[n..];
    }
    Ok(())
}
fn connect(
    addresses: &[SocketAddr],
    deadline: Instant,
) -> std::result::Result<TcpStream, PingError> {
    let mut error = PingError::Offline;
    for address in addresses {
        match TcpStream::connect_timeout(address, remaining(deadline)?.min(Duration::from_secs(1)))
        {
            Ok(s) => {
                s.set_write_timeout(Some(remaining(deadline)?))
                    .map_err(io_error)?;
                return Ok(s);
            }
            Err(e) => error = io_error(e),
        }
    }
    Err(error)
}
fn send(stream: &mut TcpStream, body: &[u8]) -> std::result::Result<(), PingError> {
    let bytes = horizon_protocol::frame::encode(body, None).map_err(|_| PingError::Malformed)?;
    stream.write_all(&bytes).map_err(io_error)
}
fn recv(
    stream: &mut TcpStream,
    deadline: Instant,
) -> std::result::Result<(i32, Vec<u8>), PingError> {
    let mut len = 0usize;
    for i in 0..3 {
        let mut byte = [0];
        read_exact(stream, &mut byte, deadline)?;
        len |= usize::from(byte[0] & 0x7f) << (7 * i);
        if byte[0] & 0x80 == 0 {
            if len == 0 || len > 262_144 {
                return Err(PingError::Malformed);
            }
            let mut body = vec![0; len];
            read_exact(stream, &mut body, deadline)?;
            let mut r = horizon_protocol::buf::Reader::new(&body);
            let id = r.varint().map_err(|_| PingError::Malformed)?;
            return Ok((id, r.rest().to_vec()));
        }
    }
    Err(PingError::Malformed)
}
pub fn ping(address: &str, timeout: Duration) -> PingResult {
    let (host, port) = address::parse(address).map_err(|_| PingError::Malformed)?;
    let deadline = Instant::now() + timeout;
    let addresses =
        horizon_protocol::connection::server_addresses(&host, port).map_err(|_| PingError::Dns)?;
    match modern(&addresses, &host, port, deadline) {
        Ok(status) => Ok(status),
        Err(error) => {
            // Legacy status only; never send a login or any account material.
            if matches!(error, PingError::Timeout | PingError::Dns) {
                return Err(error);
            }
            legacy(&addresses, deadline).or(Err(error))
        }
    }
}
fn modern(addresses: &[SocketAddr], host: &str, port: u16, deadline: Instant) -> PingResult {
    use horizon_protocol::{
        PROTOCOL_VERSION,
        packets::{Empty, handshake, status},
    };
    let mut stream = connect(addresses, deadline)?;
    send(
        &mut stream,
        &handshake::Serverbound::Intention(handshake::Intention {
            protocol: PROTOCOL_VERSION,
            host: host.into(),
            port,
            intent: handshake::STATUS,
        })
        .encode(),
    )?;
    send(
        &mut stream,
        &status::Serverbound::StatusRequest(Empty).encode(),
    )?;
    let (id, body) = recv(&mut stream, deadline)?;
    let status::Clientbound::StatusResponse(reply) =
        status::Clientbound::decode(id, &body).map_err(|_| PingError::Malformed)?
    else {
        return Err(PingError::Malformed);
    };
    let mut parsed = parse_response(reply.json.as_bytes())?;
    let start = Instant::now();
    send(
        &mut stream,
        &status::Serverbound::PingRequest(status::Ping { time: 1 }).encode(),
    )?;
    let (id, body) = recv(&mut stream, deadline)?;
    match status::Clientbound::decode(id, &body).map_err(|_| PingError::Malformed)? {
        status::Clientbound::PongResponse(pong) if pong.time == 1 => {}
        _ => return Err(PingError::Malformed),
    }
    parsed.ms = Some(start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64);
    Ok(parsed)
}
pub fn parse_response(bytes: &[u8]) -> PingResult {
    use horizon_client_core::{component, text::Text};
    if bytes.len() > 262_144 {
        return Err(PingError::Malformed);
    }
    let json: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| PingError::Malformed)?;
    if !json.is_object() {
        return Err(PingError::Malformed);
    }
    let description = json
        .get("description")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let motd = component::to_text(&description, &mut |s| Text::literal(s))
        .resolve_legacy()
        .plain();
    let count = |key: &str| {
        json["players"][key]
            .as_u64()
            .and_then(|v| u32::try_from(v).ok())
    };
    let samples = json["players"]["sample"]
        .as_array()
        .into_iter()
        .flatten()
        .take(256)
        .filter_map(|p| p["name"].as_str())
        .map(|s| s.chars().take(256).collect())
        .collect();
    Ok(Status {
        health: Health::Online,
        description,
        motd,
        online: count("online"),
        max: count("max"),
        protocol: json["version"]["protocol"]
            .as_i64()
            .and_then(|v| i32::try_from(v).ok()),
        version: json["version"]["name"]
            .as_str()
            .unwrap_or("")
            .chars()
            .take(256)
            .collect(),
        samples,
        favicon: json["favicon"]
            .as_str()
            .filter(|s| s.len() <= 65536)
            .map(str::to_owned),
        ms: None,
    })
}
fn legacy(addresses: &[SocketAddr], deadline: Instant) -> PingResult {
    let start = Instant::now();
    let mut stream = connect(addresses, deadline)?;
    stream.write_all(&[0xfe, 0x01]).map_err(io_error)?;
    let mut header = [0; 3];
    read_exact(&mut stream, &mut header, deadline)?;
    if header[0] != 0xff {
        return Err(PingError::Malformed);
    }
    let len = usize::from(u16::from_be_bytes([header[1], header[2]]));
    if len > 32767 {
        return Err(PingError::Malformed);
    }
    let mut bytes = vec![0; len * 2];
    read_exact(&mut stream, &mut bytes, deadline)?;
    let utf16 = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u16::from_be_bytes([p[0], p[1]]))
        .collect::<Vec<_>>();
    let text = String::from_utf16(&utf16).map_err(|_| PingError::Malformed)?;
    let parts = text.split('\0').collect::<Vec<_>>();
    if parts.len() != 6 || parts[0] != "§1" {
        return Err(PingError::Malformed);
    }
    Ok(Status {
        health: Health::Online,
        description: serde_json::Value::String(parts[3].into()),
        motd: crate::query::strip_formatting(parts[3]),
        protocol: Some(parts[1].parse().map_err(|_| PingError::Malformed)?),
        version: parts[2].into(),
        online: Some(parts[4].parse().map_err(|_| PingError::Malformed)?),
        max: Some(parts[5].parse().map_err(|_| PingError::Malformed)?),
        ms: Some(start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64),
        ..Status::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn visible_rows_overtake_background_jobs_without_restarting_workers() {
        let gate = Arc::new(std::sync::Barrier::new(2));
        let (sent, received) = std::sync::mpsc::channel();
        let worker_gate = gate.clone();
        let service = PingService::with_probe(
            1,
            Arc::new(move |address, _| {
                sent.send(address.to_owned()).unwrap();
                if address == "first" {
                    worker_gate.wait();
                }
                Ok(Status::default())
            }),
        )
        .unwrap();
        service.refresh(["first", "middle", "visible"].map(str::to_owned), true);
        assert_eq!(
            received.recv_timeout(Duration::from_secs(5)).unwrap(),
            "first"
        );
        service.prioritize(["visible".to_owned()]);
        gate.wait();
        assert_eq!(
            received.recv_timeout(Duration::from_secs(5)).unwrap(),
            "visible"
        );
        assert_eq!(
            received.recv_timeout(Duration::from_secs(5)).unwrap(),
            "middle"
        );
    }
    use std::sync::atomic::AtomicUsize;
    #[test]
    fn parses_formatted_components_and_rejects_garbage() {
        let s=parse_response(br#"{"description":{"text":"Hello ","extra":[{"text":"world","color":"gold"}]},"players":{"online":4,"max":20,"sample":[{"name":"Alex"}]},"version":{"name":"26.3","protocol":777}}"#).unwrap();
        assert_eq!(s.motd, "Hello world");
        assert_eq!(s.online, Some(4));
        assert_eq!(s.samples, ["Alex"]);
        for bytes in [b"garbage".as_slice(), b"[]", b"null", b"{", b"\xff"] {
            assert_eq!(parse_response(bytes).unwrap_err(), PingError::Malformed);
        }
        let s = parse_response(br#"{"players":{"online":-1,"max":"oops"}}"#).unwrap();
        assert_eq!(s.online, None);
    }
    #[test]
    fn workers_bound_concurrency_cache_and_cancel_stale_generations() {
        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let calls = Arc::new(AtomicUsize::new(0));
        let (a, p, c) = (active.clone(), peak.clone(), calls.clone());
        let service = PingService::with_probe(
            3,
            Arc::new(move |_, _| {
                let n = a.fetch_add(1, Ordering::SeqCst) + 1;
                p.fetch_max(n, Ordering::SeqCst);
                c.fetch_add(1, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(10));
                a.fetch_sub(1, Ordering::SeqCst);
                Ok(Status::default())
            }),
        )
        .unwrap();
        service.refresh((0..30).map(|n| format!("server{n}")), true);
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut replies = Vec::new();
        while replies.len() < 30 && Instant::now() < deadline {
            replies.extend(service.drain());
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(replies.len(), 30);
        assert!(peak.load(Ordering::SeqCst) <= 3);
        service.refresh(["server0".into()], false);
        assert_eq!(service.drain().len(), 1);
        assert_eq!(calls.load(Ordering::SeqCst), 30);
        service.refresh((0..100).map(|n| format!("old{n}")), true);
        service.refresh(["new".into()], true);
        std::thread::sleep(Duration::from_millis(80));
        assert!(service.drain().iter().all(|(s, _)| s == "new"));
        let start = Instant::now();
        drop(service);
        assert!(start.elapsed() < Duration::from_millis(20));
    }
    #[test]
    fn loopback_status_server_checks_pong_and_times_out() {
        use horizon_protocol::packets::status;
        for (garbage, slow) in [(false, false), (true, false), (false, true)] {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            let handle = std::thread::spawn(move || {
                let (mut socket, _) = listener.accept().unwrap();
                let deadline = Instant::now() + Duration::from_secs(2);
                recv(&mut socket, deadline).unwrap();
                recv(&mut socket, deadline).unwrap();
                if slow {
                    std::thread::sleep(Duration::from_millis(150));
                    return;
                }
                if garbage {
                    socket.write_all(&[0xff, 0xff, 0xff]).unwrap();
                    return;
                }
                send(
                    &mut socket,
                    &status::Clientbound::StatusResponse(status::StatusResponse {
                        json: r#"{"description":"§aLocal","version":{"protocol":777}}"#.into(),
                    })
                    .encode(),
                )
                .unwrap();
                recv(&mut socket, deadline).unwrap();
                send(
                    &mut socket,
                    &status::Clientbound::PongResponse(status::Ping { time: 1 }).encode(),
                )
                .unwrap();
            });
            let reply = modern(
                &[addr],
                "localhost",
                addr.port(),
                Instant::now() + Duration::from_millis(100),
            );
            if slow {
                assert_eq!(reply.unwrap_err(), PingError::Timeout);
            } else if garbage {
                assert_eq!(reply.unwrap_err(), PingError::Malformed);
            } else {
                assert_eq!(reply.unwrap().motd, "Local");
            }
            handle.join().unwrap();
        }
    }
    #[test]
    fn legacy_fallback_decodes_a_bounded_utf16_reply() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 2];
            stream.read_exact(&mut request).unwrap();
            assert_eq!(request, [0xfe, 0x01]);
            let text = "§1\0".to_owned() + "47\0" + "1.8\0" + "§aLegacy\0" + "2\0" + "20";
            let chars = text.encode_utf16().collect::<Vec<_>>();
            let mut reply = vec![0xff];
            reply.extend_from_slice(&(chars.len() as u16).to_be_bytes());
            for c in chars {
                reply.extend_from_slice(&c.to_be_bytes());
            }
            stream.write_all(&reply).unwrap();
        });
        let status = legacy(&[address], Instant::now() + Duration::from_secs(1)).unwrap();
        assert_eq!(status.motd, "Legacy");
        assert_eq!(status.online, Some(2));
        assert_eq!(status.protocol, Some(47));
        handle.join().unwrap();
    }
}
