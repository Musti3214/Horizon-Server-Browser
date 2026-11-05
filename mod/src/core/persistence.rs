use crate::{
    Error, Result, address,
    model::{Metadata, Server, State},
};
use horizon_world::nbt::{self, JavaString, NamedTag, Tag};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const MAX_FILE: u64 = 32 * 1024 * 1024;
static NEXT: AtomicU64 = AtomicU64::new(0);

/// Read bounded files; malformed or future files are errors, never an empty list
/// that can subsequently overwrite the original catalogue.
pub fn read_bounded(path: &Path) -> Result<Vec<u8>> {
    let file = fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.take(MAX_FILE + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE {
        return Err(Error::Invalid("file is too large".into()));
    }
    Ok(bytes)
}

/// Unique same-directory temporary file, flushed before an atomic replacement.
/// No remove-then-rename gap, including on Windows. Failed writes clean up only
/// the temporary file this invocation created.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .ok_or_else(|| Error::Invalid("missing file name".into()))?
        .to_string_lossy();
    let tmp = parent.join(format!(
        ".{file_name}.{}.{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&tmp, path)?;
        #[cfg(unix)]
        fs::File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

pub fn state_path(home: &Path) -> PathBuf {
    home.join("horizon-server-browser.json")
}
pub fn load_state(home: &Path) -> Result<State> {
    let path = state_path(home);
    let mut state: State = match read_bounded(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes)?,
        Err(Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => State::default(),
        Err(e) => return Err(e),
    };
    if state.schema != 1 {
        return Err(Error::NewerSchema(state.schema));
    }
    state.settings.sanitize();
    Ok(state)
}
pub fn save_state(home: &Path, state: &State) -> Result<()> {
    if state.schema != 1 {
        return Err(Error::NewerSchema(state.schema));
    }
    atomic_write(&state_path(home), &serde_json::to_vec_pretty(state)?)
}

#[derive(Clone, Debug)]
pub struct Catalog {
    pub servers: Vec<Server>,
    root: NamedTag,
    opaque: Vec<Tag>,
    /// Detect edits by another screen/launcher rather than overwriting them.
    disk: Option<Vec<u8>>,
}

impl Catalog {
    pub fn load(home: &Path, metadata: &BTreeMap<String, Metadata>) -> Result<Self> {
        let disk = match read_bounded(&home.join("servers.dat")) {
            Ok(bytes) => Some(bytes),
            Err(Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e),
        };
        let root = match &disk {
            Some(bytes) => {
                let (root, used) = nbt::read_named(bytes, nbt::Limits::default())
                    .map_err(|e| Error::Invalid(e.to_string()))?;
                if used != bytes.len() || !matches!(root.tag, Tag::Compound(_)) {
                    return Err(Error::Invalid("servers.dat is not an NBT compound".into()));
                }
                root
            }
            None => NamedTag {
                name: JavaString::from(""),
                tag: Tag::compound([("servers", Tag::List(Vec::new()))]),
            },
        };
        let mut servers = Vec::new();
        let mut opaque = Vec::new();
        match root.tag.get("servers") {
            Some(Tag::List(entries)) => {
                for tag in entries {
                    if let Some(mut server) = from_tag(tag) {
                        if let Ok(key) = address::key(&server.address)
                            && let Some(m) = metadata.get(&key)
                        {
                            server.metadata = m.clone();
                        }
                        servers.push(server);
                    } else {
                        opaque.push(tag.clone());
                    }
                }
            }
            Some(_) => {
                return Err(Error::Invalid(
                    "servers.dat servers field is not a list".into(),
                ));
            }
            None => {}
        }
        Ok(Self {
            servers,
            root,
            opaque,
            disk,
        })
    }

    pub fn validate(&self, server: &Server, editing: Option<usize>) -> Result<()> {
        if server.name.trim().is_empty() {
            return Err(Error::Invalid("empty server name".into()));
        }
        let key = address::key(&server.address)?;
        if self.servers.iter().enumerate().any(|(i, s)| {
            Some(i) != editing && address::key(&s.address).is_ok_and(|other| key == other)
        }) {
            return Err(Error::Duplicate);
        }
        Ok(())
    }

    pub fn save(&mut self, home: &Path) -> Result<()> {
        let path = home.join("servers.dat");
        let current = match read_bounded(&path) {
            Ok(bytes) => Some(bytes),
            Err(Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e),
        };
        if current != self.disk {
            return Err(Error::Invalid(
                "servers.dat changed externally; refresh before saving".into(),
            ));
        }
        // Keep exactly the original bytes, including unknown fields, before the
        // first modern write. create_new prevents replacing an existing backup.
        if let Some(bytes) = &current {
            let backup = path.with_file_name("servers.dat.before-modern-browser");
            match OpenOptions::new().write(true).create_new(true).open(backup) {
                Ok(mut file) => {
                    file.write_all(bytes)?;
                    file.sync_all()?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e.into()),
            }
        }
        let mut root = self.root.clone();
        let Tag::Compound(map) = &mut root.tag else {
            return Err(Error::Invalid("invalid root".into()));
        };
        let tags = self
            .servers
            .iter()
            .map(to_tag)
            .chain(self.opaque.iter().cloned())
            .collect();
        map.insert(JavaString::from("servers"), Tag::List(tags));
        let bytes = nbt::write_named(&root, nbt::Limits::default())
            .map_err(|e| Error::Invalid(e.to_string()))?;
        atomic_write(&path, &bytes)?;
        self.root = root;
        self.disk = Some(bytes);
        Ok(())
    }

    pub fn collect_metadata(&self, state: &mut State) {
        for server in &self.servers {
            if let Ok(key) = address::key(&server.address) {
                state.metadata.insert(key, server.metadata.clone());
            }
        }
    }

    pub fn export_json(&self, path: &Path) -> Result<()> {
        atomic_write(path, &serde_json::to_vec_pretty(&self.servers)?)
    }
    /// Validate the entire import before changing anything; ignore duplicates.
    pub fn import_json(&mut self, path: &Path) -> Result<usize> {
        let incoming: Vec<Server> = serde_json::from_slice(&read_bounded(path)?)?;
        let mut staged = self.clone();
        let mut added = 0;
        for mut server in incoming {
            match staged.validate(&server, None) {
                Ok(()) => {
                    server.original = None;
                    staged.servers.push(server);
                    added += 1;
                }
                Err(Error::Duplicate) => {}
                Err(e) => return Err(e),
            }
        }
        self.servers = staged.servers;
        Ok(added)
    }
    pub fn import_dat(&mut self, path: &Path) -> Result<usize> {
        // Decode the file in place without assuming its name is servers.dat.
        let bytes = read_bounded(path)?;
        let (root, _) = nbt::read_named(&bytes, nbt::Limits::default())
            .map_err(|e| Error::Invalid(e.to_string()))?;
        let Some(Tag::List(tags)) = root.tag.get("servers") else {
            return Err(Error::Invalid("missing servers list".into()));
        };
        let mut staged = self.clone();
        let mut added = 0;
        for server in tags.iter().filter_map(from_tag) {
            match staged.validate(&server, None) {
                Ok(()) => {
                    staged.servers.push(server);
                    added += 1;
                }
                Err(Error::Duplicate) => {}
                Err(e) => return Err(e),
            }
        }
        self.servers = staged.servers;
        Ok(added)
    }
}

fn from_tag(tag: &Tag) -> Option<Server> {
    if matches!(tag.get("hidden"), Some(Tag::Byte(1))) {
        return None;
    }
    let string = |key| {
        if let Some(Tag::String(s)) = tag.get(key) {
            Some(s.to_string_lossy())
        } else {
            None
        }
    };
    Some(Server {
        name: string("name").unwrap_or_default(),
        address: string("ip")?,
        icon: string("icon"),
        packs: match tag.get("acceptTextures") {
            Some(Tag::Byte(b)) => Some(*b != 0),
            _ => None,
        },
        original: Some(tag.clone()),
        ..Server::default()
    })
}
fn to_tag(server: &Server) -> Tag {
    let mut tag = server
        .original
        .clone()
        .unwrap_or_else(|| Tag::Compound(BTreeMap::new()));
    if let Tag::Compound(map) = &mut tag {
        for (key, value) in [
            (
                "name",
                Some(Tag::String(JavaString::from(server.name.as_str()))),
            ),
            (
                "ip",
                Some(Tag::String(JavaString::from(server.address.as_str()))),
            ),
            (
                "icon",
                server
                    .icon
                    .as_deref()
                    .map(|v| Tag::String(JavaString::from(v))),
            ),
            (
                "acceptTextures",
                server.packs.map(|v| Tag::Byte(i8::from(v))),
            ),
        ] {
            // Keep the original UTF-16 bytes when the user didn't edit the value.
            if let Some(Tag::String(old)) = map.get(&JavaString::from(key))
                && value.as_ref().is_some_and(|new| matches!(new,Tag::String(s) if s.to_string_lossy()==old.to_string_lossy())) { continue; }
            match value {
                Some(v) => {
                    map.insert(JavaString::from(key), v);
                }
                None => {
                    map.remove(&JavaString::from(key));
                }
            }
        }
    }
    tag
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scratch() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "horizon-browser-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
    #[test]
    fn unknown_fields_hidden_entries_and_original_backup_survive_edit() {
        let dir = scratch();
        let path = dir.join("servers.dat");
        let raw = Tag::compound([
            ("name", Tag::String(JavaString(vec![0xd800]))),
            ("ip", Tag::String(JavaString::from("localhost"))),
            ("foreign", Tag::Long(42)),
            ("acceptedCodeOfConduct", Tag::Int(123)),
        ]);
        let hidden = Tag::compound([
            ("hidden", Tag::Byte(1)),
            ("ip", Tag::String(JavaString::from("secret"))),
        ]);
        let root = NamedTag {
            name: JavaString::from("custom"),
            tag: Tag::compound([
                ("root_extra", Tag::Int(9)),
                ("servers", Tag::List(vec![raw, hidden.clone()])),
            ]),
        };
        let original = nbt::write_named(&root, nbt::Limits::default()).unwrap();
        fs::write(&path, &original).unwrap();
        let mut c = Catalog::load(&dir, &BTreeMap::new()).unwrap();
        assert_eq!(c.servers.len(), 1);
        c.servers[0].packs = Some(true);
        c.save(&dir).unwrap();
        let (new, _) = nbt::read_named(&fs::read(&path).unwrap(), nbt::Limits::default()).unwrap();
        assert_eq!(new.name, root.name);
        assert_eq!(new.tag.get("root_extra"), Some(&Tag::Int(9)));
        let Some(Tag::List(list)) = new.tag.get("servers") else {
            panic!()
        };
        assert_eq!(
            list[0].get("name"),
            Some(&Tag::String(JavaString(vec![0xd800])))
        );
        assert_eq!(list[0].get("foreign"), Some(&Tag::Long(42)));
        assert_eq!(list[1], hidden);
        assert_eq!(
            fs::read(dir.join("servers.dat.before-modern-browser")).unwrap(),
            original
        );
        c.save(&dir).unwrap();
        assert_eq!(horizon_protocol::servers::load(&path).len(), 1);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn corrupt_and_externally_changed_files_are_never_overwritten() {
        let dir = scratch();
        let mut c = Catalog::load(&dir, &BTreeMap::new()).unwrap();
        fs::write(dir.join("servers.dat"), b"garbage").unwrap();
        assert!(c.save(&dir).is_err());
        assert!(Catalog::load(&dir, &BTreeMap::new()).is_err());
        assert_eq!(fs::read(dir.join("servers.dat")).unwrap(), b"garbage");
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn metadata_presets_json_and_duplicates_round_trip() {
        let dir = scratch();
        let mut c = Catalog::load(&dir, &BTreeMap::new()).unwrap();
        let mut server = Server {
            name: "Home".into(),
            address: "localhost".into(),
            ..Server::default()
        };
        server.metadata.notes = "kept local".into();
        server.metadata.favorite = true;
        c.servers.push(server.clone());
        assert!(matches!(c.validate(&server, None), Err(Error::Duplicate)));
        c.save(&dir).unwrap();
        let mut state = State::default();
        c.collect_metadata(&mut state);
        state.presets.insert("Friends".into(), state.view.clone());
        save_state(&dir, &state).unwrap();
        let state = load_state(&dir).unwrap();
        assert!(state.presets.contains_key("Friends"));
        let c2 = Catalog::load(&dir, &state.metadata).unwrap();
        assert_eq!(c2.servers[0].metadata.notes, "kept local");
        c.export_json(&dir.join("list.json")).unwrap();
        assert_eq!(c.import_json(&dir.join("list.json")).unwrap(), 0);
        fs::write(dir.join("bad.json"),br#"[{"name":"Good","address":"new.example.org"},{"name":"Bad","address":"https://bad"}]"#).unwrap();
        assert!(c.import_json(&dir.join("bad.json")).is_err());
        assert_eq!(c.servers.len(), 1);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn versioned_fixture_defaults_unknown_fields_and_future_schema() {
        let dir = scratch();
        assert!(!load_state(&dir).unwrap().settings.online_directory);
        let path = state_path(&dir);
        fs::write(&path, include_bytes!("../tests/fixtures/state-v1.json")).unwrap();
        let mut state = load_state(&dir).unwrap();
        assert!(state.settings.modern);
        assert!(!state.settings.online_directory);
        assert!(!state.settings.support_dismissed);
        assert_eq!(state.settings.concurrent_pings, 8);
        assert!(state.metadata["localhost:25565"].favorite);
        save_state(&dir, &state).unwrap();
        assert_eq!(
            load_state(&dir).unwrap().extra["future_top_level"]["retained"],
            true
        );
        state.schema = 2;
        assert!(matches!(
            save_state(&dir, &state),
            Err(Error::NewerSchema(2))
        ));
        fs::write(&path, br#"{"schema":2}"#).unwrap();
        assert!(matches!(load_state(&dir), Err(Error::NewerSchema(2))));
        fs::remove_dir_all(dir).unwrap();
    }
}
