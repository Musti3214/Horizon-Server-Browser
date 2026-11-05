use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    Online,
    Offline,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Metadata {
    pub favorite: bool,
    pub tags: Vec<String>,
    pub notes: String,
    pub group: String,
    pub language: String,
    pub region: String,
    pub auth: AuthMode,
    pub added: u64,
    pub last_played: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Server {
    pub name: String,
    pub address: String,
    /// None asks; true enables; false disables server resource packs.
    pub packs: Option<bool>,
    pub icon: Option<String>,
    pub metadata: Metadata,
    /// The entire original compound, including unknown tags and UTF-16 strings.
    #[serde(skip)]
    pub original: Option<horizon_world::nbt::Tag>,
}
impl Default for Server {
    fn default() -> Self {
        Self {
            name: String::new(),
            address: String::new(),
            packs: None,
            icon: None,
            metadata: Metadata {
                added: crate::unix_seconds(),
                ..Metadata::default()
            },
            original: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Tab {
    #[default]
    Saved,
    Discover,
    Favorites,
    Recent,
    Lan,
}
impl Tab {
    pub const ALL: [Self; 5] = [
        Self::Saved,
        Self::Discover,
        Self::Favorites,
        Self::Recent,
        Self::Lan,
    ];
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    #[default]
    Custom,
    Name,
    Players,
    FreeSlots,
    Ping,
    Version,
    LastPlayed,
    DateAdded,
    Favorites,
}
impl Sort {
    pub const ALL: [Self; 9] = [
        Self::Custom,
        Self::Name,
        Self::Players,
        Self::FreeSlots,
        Self::Ping,
        Self::Version,
        Self::LastPlayed,
        Self::DateAdded,
        Self::Favorites,
    ];
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Settings {
    pub modern: bool,
    pub classic_menu: bool,
    /// Zero disables automatic refresh.
    pub refresh_seconds: u64,
    pub concurrent_pings: usize,
    // Older releases persisted the default-on switch under a different key.
    // Only an explicit confirmation in this release enables the directory.
    #[serde(rename = "online_directory_consent")]
    pub online_directory: bool,
    pub show_offline: bool,
    pub reduced_motion: bool,
    pub default_tab: Tab,
    pub default_sort: Sort,
    pub support_dismissed: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            modern: true,
            classic_menu: false,
            refresh_seconds: 60,
            concurrent_pings: 8,
            online_directory: false,
            show_offline: true,
            reduced_motion: false,
            default_tab: Tab::Saved,
            default_sort: Sort::Custom,
            support_dismissed: false,
        }
    }
}
impl Settings {
    pub fn sanitize(&mut self) {
        self.concurrent_pings = self.concurrent_pings.clamp(1, 16);
        if self.refresh_seconds != 0 {
            self.refresh_seconds = self.refresh_seconds.clamp(15, 3600);
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct View {
    pub search: String,
    pub filters: crate::query::Filters,
    pub sort: Sort,
    pub descending: bool,
    pub tab: Tab,
    pub selected: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub schema: u32,
    pub settings: Settings,
    pub view: View,
    pub metadata: BTreeMap<String, Metadata>,
    pub presets: BTreeMap<String, View>,
    pub recent: Vec<Server>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            schema: 1,
            settings: Settings::default(),
            view: View::default(),
            metadata: BTreeMap::new(),
            presets: BTreeMap::new(),
            recent: Vec::new(),
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Health {
    #[default]
    Pinging,
    Online,
    Offline,
    Timeout,
    DnsError,
    Malformed,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Status {
    pub health: Health,
    pub description: serde_json::Value,
    pub motd: String,
    pub online: Option<u32>,
    pub max: Option<u32>,
    pub ms: Option<u64>,
    pub protocol: Option<i32>,
    pub version: String,
    pub samples: Vec<String>,
    pub favicon: Option<String>,
}
impl Status {
    pub fn compatible(&self) -> bool {
        self.protocol == Some(horizon_protocol::PROTOCOL_VERSION)
    }
}
