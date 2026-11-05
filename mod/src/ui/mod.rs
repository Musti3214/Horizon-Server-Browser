//! Modern Multiplayer. Classic lives in `multiplayer.rs` and is never rendered
//! through this module. Network results and visible rows are retained, not rebuilt
//! for every frame. All test runs must use a scratch game home.
mod draw;
mod forms;
mod online;
#[cfg(test)]
mod tests;

use crate::{
    hud::{HudAtlas, HudBatch},
    multiplayer::{ButtonSpec, Outcome},
    ui::{Caret, Scroll},
};
use horizon_server_browser::{
    self as core, address,
    model::{AuthMode, Health, Server, Sort, State, Status, Tab, View},
    persistence::{self, Catalog},
    query,
    service::PingService,
};
use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap, VecDeque},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use winit::keyboard::KeyCode;

fn tr(key: &str) -> String {
    crate::lang::get_or(&format!("horizon.browser.{key}"), key)
}
fn label_value(key: &str, value: impl std::fmt::Display) -> String {
    format!("{}: {value}", tr(key))
}

#[derive(Clone, Default)]
pub struct Provider {
    pub servers: Vec<Server>,
    pub support_url: Option<String>,
}
thread_local! {
    // Only the client UI thread publishes providers. An integrated server must
    // never change the window's menu registration.
    static PROVIDER: RefCell<Option<Provider>> = const { RefCell::new(None) };
}
pub fn set_provider(provider: Option<Provider>) {
    PROVIDER.with(|p| *p.borrow_mut() = provider);
}
pub fn available() -> bool {
    PROVIDER.with(|p| p.borrow().is_some())
}
fn provider_servers() -> Vec<Server> {
    PROVIDER.with(|p| {
        p.borrow()
            .as_ref()
            .map(|p| p.servers.clone())
            .unwrap_or_default()
    })
}

pub fn open_support_url() {
    let Some(url) = PROVIDER.with(|p| p.borrow().as_ref().and_then(|p| p.support_url.clone()))
    else {
        return;
    };
    #[cfg(windows)]
    if let Err(error) = open::that_detached(&url) {
        log::warn!("Horizon: could not open URI {url}: {error}");
    }
    #[cfg(not(windows))]
    {
        let program = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        if let Err(error) = std::process::Command::new(program).arg(&url).spawn() {
            log::warn!("Horizon: could not open URI {url}: {error}");
        }
    }
}

#[derive(Clone, Debug)]
enum Modal {
    Main,
    Edit {
        index: Option<usize>,
        server: Box<Server>,
        tags: String,
        page: usize,
    },
    Direct(String),
    Remove,
    Filters(usize),
    Sort,
    Settings,
    Presets(String),
    Transfer(String),
    BulkTag(String),
    Details,
    Consent,
    Feedback {
        category: usize,
        text: String,
    },
}

#[derive(Clone, Debug)]
struct Runtime {
    service: PingService,
    directory: online::Directory,
    statuses: HashMap<String, Status>,
    history: HashMap<String, VecDeque<u64>>,
    icons: RefCell<HashMap<String, (u64, Option<std::sync::Arc<Vec<u8>>>)>>,
    source: Vec<Server>,
    visible: Vec<usize>,
    view: View,
    revision: u64,
    scroll: Scroll,
    detail_scroll: Scroll,
    detail_height: f32,
    smooth: f32,
    ping_window: Option<(usize, usize)>,
    refresh_requested: bool,
    frame: Instant,
    opened: Instant,
    refreshed: Instant,
    search_pending: (String, Instant),
    size: (f32, f32),
    lan_games: Vec<crate::lan::discovery::LanGame>,
}

fn viewport_addresses(r: &Runtime, first: usize, count: usize) -> Vec<String> {
    const MARGIN: usize = 2;
    let start = first.saturating_sub(MARGIN).min(r.visible.len());
    let end = first
        .saturating_add(count)
        .saturating_add(MARGIN)
        .min(r.visible.len());
    r.visible[start..end]
        .iter()
        .map(|&index| r.source[index].address.clone())
        .collect()
}

#[derive(Clone, Debug)]
pub struct Browser {
    home: PathBuf,
    catalog: Catalog,
    state: State,
    modal: Modal,
    focus: Option<usize>,
    button_focus: Option<u8>,
    selection: BTreeSet<String>,
    anchor: Option<String>,
    drag_from: Option<String>,
    drag_to: Option<String>,
    modifiers: (bool, bool),
    revision: u64,
    runtime: RefCell<Runtime>,
    lan: crate::lan::discovery::Scanner,
    undo: Option<(Vec<(usize, Server)>, Instant)>,
    notice: Option<(String, Instant)>,
    fields: Vec<String>,
    filters_before: Option<query::Filters>,
    preset_page: usize,
    error: Option<String>,
    last_row_click: Option<(Instant, String)>,
}

impl Browser {
    pub fn new(saves: &Path) -> core::Result<Self> {
        let home = saves.parent().unwrap_or(Path::new(".")).to_path_buf();
        let state = persistence::load_state(&home)?;
        let catalog = Catalog::load(&home, &state.metadata)?;
        let service = PingService::new(state.settings.concurrent_pings)?;
        let now = Instant::now();
        let selection = state.view.selected.clone().into_iter().collect();
        let runtime = Runtime {
            service,
            directory: online::Directory::default(),
            statuses: HashMap::new(),
            history: HashMap::new(),
            icons: RefCell::default(),
            source: Vec::new(),
            visible: Vec::new(),
            view: state.view.clone(),
            revision: u64::MAX,
            scroll: Scroll::default(),
            detail_scroll: Scroll::default(),
            detail_height: 0.0,
            smooth: 0.0,
            ping_window: None,
            refresh_requested: false,
            frame: now,
            opened: now,
            refreshed: now,
            search_pending: (state.view.search.clone(), now),
            size: (427.0, 240.0),
            lan_games: Vec::new(),
        };
        Ok(Self {
            home,
            catalog,
            state,
            modal: Modal::Main,
            focus: Some(0),
            button_focus: None,
            selection,
            anchor: None,
            drag_from: None,
            drag_to: None,
            modifiers: (false, false),
            revision: 0,
            runtime: RefCell::new(runtime),
            lan: crate::lan::discovery::Scanner::start(),
            undo: None,
            notice: None,
            fields: vec![String::new(); 8],
            filters_before: None,
            preset_page: 0,
            error: None,
            last_row_click: None,
        })
    }
    pub fn settings(saves: &Path) -> core::Result<Self> {
        let mut b = Self::new(saves)?;
        b.modal = Modal::Settings;
        b.focus = None;
        Ok(b)
    }
    fn persist(&self) -> core::Result<()> {
        let mut state = self.state.clone();
        self.catalog.collect_metadata(&mut state);
        persistence::save_state(&self.home, &state)
    }
    fn changed(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
    fn save_catalog(&mut self, before: Vec<Server>) -> bool {
        match self.catalog.save(&self.home) {
            Ok(()) => {
                self.changed();
                if let Err(e) = self.persist() {
                    self.error = Some(label_value("save_error", e));
                }
                true
            }
            Err(e) => {
                self.catalog.servers = before;
                self.error = Some(label_value("save_error", e));
                false
            }
        }
    }
    fn save_metadata(&mut self) {
        self.changed();
        if let Err(e) = self.persist() {
            self.error = Some(label_value("save_error", e));
        }
    }
    fn show(&mut self, modal: Modal) {
        self.modal = modal;
        self.focus = None;
        self.button_focus = None;
        self.error = None;
    }
    fn main(&mut self) {
        self.show(Modal::Main);
    }
    pub fn set_modifiers(&mut self, ctrl: bool, shift: bool) {
        self.modifiers = (ctrl, shift);
    }
    fn selected_server(&self) -> Option<Server> {
        let r = self.runtime.borrow();
        self.state
            .view
            .selected
            .as_ref()
            .and_then(|a| {
                r.visible
                    .iter()
                    .map(|&i| &r.source[i])
                    .find(|s| &s.address == a)
            })
            .cloned()
    }
    fn selected_index(&self) -> Option<usize> {
        self.state
            .view
            .selected
            .as_ref()
            .and_then(|a| self.catalog.servers.iter().position(|s| &s.address == a))
    }
    fn connect(&mut self) -> Outcome {
        let Some(server) = self.selected_server() else {
            return Outcome::None;
        };
        let address = server.address.clone();
        self.record_recent(server);
        Outcome::Connect { address, lan: self.state.view.tab == Tab::Lan }
    }
    fn record_recent(&mut self, mut server: Server) {
        let now = core::unix_seconds();
        server.metadata.last_played = now;
        if let Some(s) = self
            .catalog
            .servers
            .iter_mut()
            .find(|s| s.address == server.address)
        {
            s.metadata.last_played = now;
        }
        self.state.recent.retain(|s| s.address != server.address);
        self.state.recent.insert(0, server);
        self.state.recent.truncate(100);
        self.save_metadata();
    }
    pub fn refresh(&mut self) {
        // An edit owns an index in this snapshot. Reloading it underneath the
        // form could retarget the edit after another process removes a row.
        // Saving still checks the on-disk fingerprint; cancel/reopen to reload.
        if matches!(self.modal, Modal::Edit { .. }) {
            return;
        }
        if matches!(self.state.view.tab, Tab::Lan | Tab::Discover | Tab::Recent) {
            let r = self.runtime.get_mut();
            if self.state.view.tab == Tab::Discover {
                r.directory.refresh();
            }
            r.refresh_requested = true;
            return;
        }
        // Reload before a mutation if another launcher changed the file. Modern
        // pings never write servers.dat, so read/write races cannot destroy tags.
        self.catalog.collect_metadata(&mut self.state);
        match Catalog::load(&self.home, &self.state.metadata) {
            Ok(c) => self.catalog = c,
            Err(e) => {
                self.error = Some(label_value("load_error", e));
                return;
            }
        }
        let r = self.runtime.get_mut();
        r.statuses.clear();
        r.refresh_requested = true;
        self.changed();
    }
    fn sync(&self, r: &mut Runtime) {
        let current_filter = (
            self.state
                .view
                .filters
                .tags
                .first()
                .filter(|tag| !query::offline_access_tag(tag))
                .cloned()
                .unwrap_or_default(),
            if self.state.view.filters.version == query::Version::Specific {
                self.state.view.filters.specific_version.clone()
            } else {
                String::new()
            },
        );
        if self.state.view.tab == Tab::Discover
            && self.state.settings.online_directory
            && r.directory.last_filter.as_ref() != Some(&current_filter)
        {
            r.directory.cancel();
            r.directory.last_filter = Some(current_filter.clone());
            r.scroll = Scroll::default();
        }
        let directory_changed = if self.state.settings.online_directory {
            r.directory.poll()
        } else {
            r.directory.cancel();
            false
        };
        if self.state.view.tab == Tab::Discover
            && self.state.settings.online_directory
            && r.directory.needs_first_page(Instant::now())
        {
            r.directory.request(
                self.home.clone(),
                true,
                core::directory::Request {
                    page: 1,
                    tag: current_filter.0.clone(),
                    version: current_filter.1.clone(),
                },
            );
        }
        let updates = r.service.drain();
        let mut dirty = !updates.is_empty();
        for (address, status) in updates {
            if let Some(ms) = status.ms {
                let h = r.history.entry(address.clone()).or_default();
                h.push_back(ms);
                if h.len() > 32 {
                    h.pop_front();
                }
            }
            r.statuses.insert(address, status);
        }
        let force_refresh = r.refresh_requested
            || self.state.settings.refresh_seconds > 0
                && r.refreshed.elapsed() >= Duration::from_secs(self.state.settings.refresh_seconds);
        if force_refresh {
            r.refreshed = Instant::now();
            r.refresh_requested = false;
        }
        let games = if self.state.view.tab == Tab::Lan {
            self.lan.games()
        } else {
            Vec::new()
        };
        let source_changed = directory_changed
            || r.revision != self.revision
            || r.view.tab != self.state.view.tab
            || games != r.lan_games;
        if source_changed {
            r.source = match self.state.view.tab {
                Tab::Lan => games
                    .iter()
                    .map(|g| Server {
                        name: g.motd.clone(),
                        address: g.address.clone(),
                        ..Server::default()
                    })
                    .collect(),
                Tab::Recent => self
                    .state
                    .recent
                    .iter()
                    .map(|s| {
                        self.catalog
                            .servers
                            .iter()
                            .find(|saved| saved.address == s.address)
                            .unwrap_or(s)
                            .clone()
                    })
                    .collect(),
                Tab::Discover if self.state.settings.online_directory => r
                    .directory
                    .page
                    .as_ref()
                    .map(|p| p.servers.clone())
                    .unwrap_or_default(),
                Tab::Discover => Vec::new(),
                _ => self.catalog.servers.clone(),
            };
            let tab_changed = r.view.tab != self.state.view.tab;
            if (directory_changed || (tab_changed && self.state.view.tab == Tab::Discover))
                && let Some(page) = &r.directory.page
            {
                for (address, status) in &page.statuses {
                    r.statuses
                        .entry(address.clone())
                        .or_insert_with(|| status.clone());
                }
            }
            r.lan_games = games;
            r.revision = self.revision;
            dirty = true;
        }
        if r.search_pending.0 != self.state.view.search {
            r.search_pending = (self.state.view.search.clone(), Instant::now());
        }
        let mut view = self.state.view.clone();
        if r.search_pending.1.elapsed() < Duration::from_millis(60) {
            view.search.clone_from(&r.view.search);
        }
        let view_changed = view != r.view;
        if view_changed {
            dirty = true;
        }
        if dirty {
            r.visible = query::indices(&r.source, &r.statuses, &view);
            if !self.state.settings.show_offline {
                r.visible.retain(|&i| {
                    r.statuses
                        .get(&r.source[i].address)
                        .is_none_or(|s| s.health == Health::Online || s.health == Health::Pinging)
                });
            }
            r.view = view;
        }
        let dt = r.frame.elapsed().as_secs_f32().min(0.1);
        r.frame = Instant::now();
        let (_, top, bottom, _) = Self::list_area(r.size,self.state.view.tab);
        r.scroll
            .clamp(top, bottom, r.visible.len() as f32 * draw::ROW);
        r.smooth = if self.state.settings.reduced_motion {
            r.scroll.amount
        } else {
            r.smooth + (r.scroll.amount - r.smooth) * (dt * 18.0).min(1.0)
        };
        let first = (r.smooth / draw::ROW).floor().max(0.0) as usize;
        let count = ((bottom - top) / draw::ROW).ceil().max(0.0) as usize + 1;
        let window = (first, count);
        if matches!(self.state.view.tab, Tab::Saved | Tab::Favorites) {
            if source_changed || force_refresh || r.ping_window.is_none() {
                r.service.refresh(self.catalog.servers.iter().map(|s| s.address.clone()), force_refresh);
            }
            if source_changed || force_refresh || view_changed || r.ping_window != Some(window) {
                r.service.prioritize(r.visible.iter().skip(first).take(count).map(|&i| r.source[i].address.clone()));
            }
            r.ping_window = Some(window);
        } else if !matches!(self.modal, Modal::Main) {
            if r.ping_window.take().is_some() || source_changed {
                r.service.refresh([], false);
            }
        } else if source_changed || view_changed || force_refresh || r.ping_window != Some(window) {
            // Results can change sorting and filtering. A result alone must
            // never schedule another window and walk the entire directory.
            let addresses = viewport_addresses(r, first, count);
            r.service.refresh(addresses, force_refresh);
            r.ping_window = Some(window);
        }
        if self.state.view.tab == Tab::Discover
            && self.state.settings.online_directory
            && r.directory.can_request_next()
            && (first + count + 8 >= r.visible.len() || r.visible.len() <= count)
        {
            let next_page = r.directory.page.as_ref().map_or(1, |p| p.page + 1);
            r.directory.request(
                self.home.clone(),
                true,
                core::directory::Request {
                    page: next_page,
                    tag: current_filter.0,
                    version: current_filter.1,
                },
            );
        }
    }
    pub fn wheel(&mut self, delta: f32, cursor: (f32, f32), size: (f32, f32)) {
        let r = self.runtime.get_mut();
        if matches!(self.modal, Modal::Details)
            || matches!(self.modal, Modal::Main) && size.0 >= 720.0 && cursor.0 >= draw::viewport(size.0).0 + draw::viewport(size.0).1 - 232.0
        {
            let top = if matches!(self.modal, Modal::Details) {
                41.0
            } else {
                112.0
            };
            r.detail_scroll
                .wheel(delta, 27.0, top, size.1 - 123.0, r.detail_height);
            return;
        }
        let (_, top, bottom, _) = Self::list_area(size,self.state.view.tab);
        r.scroll.wheel(
            delta,
            draw::ROW,
            top,
            bottom,
            r.visible.len() as f32 * draw::ROW,
        );
    }
    pub fn drag(&mut self, y: f32, size: (f32, f32)) {
        let r = self.runtime.get_mut();
        let (_, top, bottom, _) = Self::list_area(size,self.state.view.tab);
        if self.drag_from.is_some() {
            let row = ((y - top + r.smooth) / draw::ROW).floor().max(0.0) as usize;
            self.drag_to = r.visible.get(row).map(|&i| r.source[i].address.clone());
        } else {
            r.scroll
                .drag(y, top, bottom, r.visible.len() as f32 * draw::ROW);
        }
    }
    pub fn release(&mut self) {
        self.runtime.get_mut().scroll.release();
        if let (Some(from), Some(to)) = (self.drag_from.take(), self.drag_to.take())
            && from != to
            && self.state.view.sort == Sort::Custom
        {
            let (from, to) = (
                self.catalog.servers.iter().position(|s| s.address == from),
                self.catalog.servers.iter().position(|s| s.address == to),
            );
            if let (Some(from), Some(to)) = (from, to) {
                let before = self.catalog.servers.clone();
                let s = self.catalog.servers.remove(from);
                self.catalog.servers.insert(to, s);
                self.save_catalog(before);
            }
        }
    }
    fn select(&mut self, address: String) {
        self.runtime.get_mut().detail_scroll = Scroll::default();
        if self.modifiers.1
            && let Some(anchor) = &self.anchor
        {
            let r = self.runtime.borrow();
            let a = r
                .visible
                .iter()
                .position(|&i| &r.source[i].address == anchor);
            let b = r
                .visible
                .iter()
                .position(|&i| r.source[i].address == address);
            if let (Some(a), Some(b)) = (a, b) {
                for &i in &r.visible[a.min(b)..=a.max(b)] {
                    self.selection.insert(r.source[i].address.clone());
                }
            }
        } else if self.modifiers.0 {
            if !self.selection.insert(address.clone()) {
                self.selection.remove(&address);
            }
            self.anchor = Some(address.clone());
        } else {
            self.selection.clear();
            self.selection.insert(address.clone());
            self.anchor = Some(address.clone());
        }
        self.state.view.selected = Some(address);
        self.focus = None;
        self.button_focus = None;
    }
    fn favorite(&mut self, one: Option<&str>) {
        let value = self
            .catalog
            .servers
            .iter()
            .find(|s| one.map_or_else(|| self.selection.contains(&s.address), |a| a == s.address))
            .is_none_or(|s| !s.metadata.favorite);
        for s in &mut self.catalog.servers {
            if one.map_or_else(|| self.selection.contains(&s.address), |a| a == s.address) {
                s.metadata.favorite = value;
            }
        }
        self.save_metadata();
    }
    fn reorder(&mut self, delta: isize) {
        if let Some(i) = self.selected_index() {
            let to = i
                .saturating_add_signed(delta)
                .min(self.catalog.servers.len().saturating_sub(1));
            let before = self.catalog.servers.clone();
            self.catalog.servers.swap(i, to);
            self.save_catalog(before);
        }
    }
    pub fn click(&mut self, id: Option<u8>, cursor: (f32, f32), size: (f32, f32)) -> Outcome {
        if let Some(id) = id {
            return self.activate(id);
        }
        for (index, _, rect) in self.form_fields(size) {
            if draw::inside(cursor, rect) {
                self.focus = Some(index);
                self.button_focus = None;
                return Outcome::None;
            }
        }
        if !matches!(self.modal, Modal::Main) {
            return Outcome::None;
        }
        let (left, top, bottom, width) = Self::list_area(size,self.state.view.tab);
        let mut r = self.runtime.borrow_mut();
        self.sync(&mut r);
        let content = r.visible.len() as f32 * draw::ROW;
        if r.scroll
            .press(cursor, left + width - 6.0, top, bottom, content)
        {
            return Outcome::None;
        }
        if !draw::inside(cursor, [left, top, width, bottom - top]) {
            self.focus = None;
            return Outcome::None;
        }
        let row = ((cursor.1 - top + r.smooth) / draw::ROW).floor().max(0.0) as usize;
        let address = r.visible.get(row).map(|&i| r.source[i].address.clone());
        drop(r);
        if let Some(address) = address {
            if cursor.0 > left + width - 24.0 {
                self.favorite(Some(&address));
            } else {
                let was_selected = self.state.view.selected.as_ref() == Some(&address);
                let on_icon = cursor.0 >= left + 8.0 && cursor.0 <= left + 44.0;
                let now = Instant::now();
                let is_double_click = self.last_row_click.as_ref().is_some_and(|(t, addr)| {
                    addr == &address
                        && now.duration_since(*t) < std::time::Duration::from_millis(400)
                });
                self.last_row_click = Some((now, address.clone()));
                self.select(address.clone());
                if is_double_click || (was_selected && on_icon) {
                    return Outcome::Connect { address, lan: self.state.view.tab == Tab::Lan };
                }
                if cursor.0 < left + 10.0 {
                    self.drag_from = Some(address);
                }
            }
        }
        Outcome::None
    }
    fn activate(&mut self, id: u8) -> Outcome {
        self.error = None;
        if matches!(self.modal, Modal::Main) {
            match id {
                130..=140 => {
                    let f = &mut self.state.view.filters;
                    match id {
                        130 => f.online = None,
                        131 => {
                            f.version = query::Version::Any;
                            f.specific_version.clear();
                        }
                        132 => {
                            f.min_players = None;
                            f.max_players = None;
                            f.free_slots = false;
                            f.not_empty = false;
                            f.not_full = false;
                        }
                        133 => f.max_ping = None,
                        134 => f.tags.clear(),
                        135 => f.language.clear(),
                        136 => f.region.clear(),
                        137 => f.favorites = false,
                        140 => f.group.clear(),
                        _ => {}
                    }
                    self.changed();
                }
                10..=14 => {
                    let new_tab = Tab::ALL[usize::from(id - 10)];
                    self.state.view.tab = new_tab;
                    self.runtime.get_mut().scroll = Scroll::default();
                    self.changed();
                }
                20 => {
                    self.prepare_filters();
                    self.show(Modal::Filters(0));
                }
                21 => self.show(Modal::Sort),
                22 => {
                    self.state.view.descending = !self.state.view.descending;
                    self.changed();
                }
                52 => {
                    if self.state.view.sort == Sort::Players {
                        self.state.view.descending = !self.state.view.descending;
                    } else {
                        self.state.view.sort = Sort::Players;
                        self.state.view.descending = true;
                    }
                    self.changed();
                }
                53 => {
                    if self.state.view.sort == Sort::Ping {
                        self.state.view.descending = !self.state.view.descending;
                    } else {
                        self.state.view.sort = Sort::Ping;
                        self.state.view.descending = false;
                    }
                    self.changed();
                }
                54 => {
                    if self.state.view.sort == Sort::Name {
                        self.state.view.descending = !self.state.view.descending;
                    } else {
                        self.state.view.sort = Sort::Name;
                        self.state.view.descending = false;
                    }
                    self.changed();
                }
                23 => self.show(Modal::Settings),
                24 => {
                    self.state.settings.classic_menu = true;
                    let _ = self.persist();
                    return Outcome::Classic;
                }
                62 => {
                    open_support_url();
                }
                63 => {
                    self.state.settings.support_dismissed = true;
                    let _ = self.persist();
                    self.changed();
                }
                64 => {
                    self.focus = Some(0);
                    self.button_focus = None;
                    self.show(Modal::Feedback {
                        category: 0,
                        text: String::new(),
                    });
                }
                25 => self.show(Modal::Details),
                26 => {
                    self.state.view.search.clear();
                    self.focus = Some(0);
                    self.changed();
                }
                27 => {
                    self.state.view.search.clear();
                    self.state.view.filters = query::Filters::default();
                    self.state.view.sort = self.state.settings.default_sort;
                    self.changed();
                }
                28 => {
                    if let Some((removed, when)) = self.undo.take()
                        && when.elapsed() < Duration::from_secs(30)
                    {
                        let before = self.catalog.servers.clone();
                        for (index, server) in &removed {
                            if self.catalog.validate(server, None).is_ok() {
                                self.catalog.servers.insert(
                                    (*index).min(self.catalog.servers.len()),
                                    server.clone(),
                                );
                            }
                        }
                        if !self.save_catalog(before) {
                            self.undo = Some((removed, when));
                        }
                    }
                }
                29 => self.favorite(None),
                30 => {
                    self.show(Modal::BulkTag(String::new()));
                    self.focus = Some(0);
                }
                31 => self.reorder(-1),
                32 => self.reorder(1),
                33 => {
                    self.show(Modal::Presets(String::new()));
                    self.focus = Some(0);
                }
                34 => {
                    self.show(Modal::Transfer(
                        self.home
                            .join("servers.json")
                            .to_string_lossy()
                            .into_owned(),
                    ));
                    self.focus = Some(0);
                }
                35 => {
                    if self.state.settings.online_directory {
                        let r = self.runtime.get_mut();
                        r.directory.refresh();
                    } else {
                        self.show(Modal::Consent);
                    }
                }
                38 => {
                    if let Some(mut server) = self.selected_server() {
                        server.original = None;
                        if self.catalog.validate(&server, None).is_ok() {
                            let before = self.catalog.servers.clone();
                            self.catalog.servers.push(server);
                            self.save_catalog(before);
                        }
                    }
                }
                40 => return self.connect(),
                41 => {
                    self.show(Modal::Direct(String::new()));
                    self.focus = Some(0);
                }
                42 => {
                    let mut server=if self.state.view.tab==Tab::Discover {self.selected_server().unwrap_or_default()} else {Server::default()};
                    server.original=None;
                    let tags=server.metadata.tags.join(", ");
                    self.show(Modal::Edit {
                        index: None,
                        server: Box::new(server),
                        tags,
                        page: 0,
                    });
                    self.focus = Some(0);
                }
                43 => {
                    if let Some(index) = self.selected_index() {
                        let s = self.catalog.servers[index].clone();
                        let tags = s.metadata.tags.join(", ");
                        self.show(Modal::Edit {
                            index: Some(index),
                            server: Box::new(s),
                            tags,
                            page: 0,
                        });
                        self.focus = Some(0);
                    }
                }
                44 => self.show(Modal::Remove),
                45 => self.refresh(),
                46 => return Outcome::Back,
                _ => {}
            }
            return Outcome::None;
        }
        if id == 51 {
            self.cancel_filters();
            self.main();
            return Outcome::None;
        }
        self.activate_form(id)
    }
    pub fn escape(&mut self) -> bool {
        if matches!(self.modal, Modal::Main) {
            if self.focus.is_some() {
                self.focus = None;
                return false;
            }
            true
        } else {
            self.cancel_filters();
            self.main();
            false
        }
    }
    fn cancel_filters(&mut self) {
        if matches!(self.modal, Modal::Filters(_))
            && let Some(filters) = self.filters_before.take()
        {
            self.state.view.filters = filters;
            self.changed();
        }
    }
    pub fn key(&mut self, key: KeyCode, ctrl: bool, shift: bool) -> Option<Outcome> {
        self.modifiers = (ctrl, shift);
        if ctrl && key == KeyCode::KeyF {
            self.cancel_filters();
            self.main();
            self.focus = Some(0);
            return Some(Outcome::None);
        }
        if ctrl && key == KeyCode::KeyN {
            self.cancel_filters();
            self.main();
            return Some(self.activate(42));
        }
        if key == KeyCode::F5 {
            self.refresh();
            return Some(Outcome::None);
        }
        if key == KeyCode::Tab {
            let size = self.runtime.borrow().size;
            let mut targets = self
                .form_fields(size)
                .iter()
                .map(|(i, _, _)| (Some(*i), None))
                .collect::<Vec<_>>();
            targets.extend(
                self.buttons(size)
                    .iter()
                    .filter(|b| b.4)
                    .map(|b| (None, Some(b.5))),
            );
            if !targets.is_empty() {
                let current = targets
                    .iter()
                    .position(|t| *t == (self.focus, self.button_focus));
                let next = match current {
                    Some(i) => {
                        if shift {
                            (i + targets.len() - 1) % targets.len()
                        } else {
                            (i + 1) % targets.len()
                        }
                    }
                    None => 0,
                };
                (self.focus, self.button_focus) = targets[next];
            }
            return Some(Outcome::None);
        }
        if matches!(self.modal, Modal::Details)
            && matches!(
                key,
                KeyCode::PageDown | KeyCode::PageUp | KeyCode::ArrowDown | KeyCode::ArrowUp
            )
        {
            let scroll = &mut self.runtime.get_mut().detail_scroll;
            scroll.amount = (scroll.amount
                + if matches!(key, KeyCode::PageUp | KeyCode::ArrowUp) {
                    -48.0
                } else {
                    48.0
                })
            .max(0.0);
            return Some(Outcome::None);
        }
        if key == KeyCode::Enter || key == KeyCode::NumpadEnter {
            let id = self
                .button_focus
                .unwrap_or(if matches!(self.modal, Modal::Main) {
                    40
                } else if matches!(self.modal, Modal::Feedback { .. }) {
                    96
                } else {
                    50
                });
            return Some(self.activate(id));
        }
        if !matches!(self.modal, Modal::Main) {
            return None;
        }
        if ctrl && key == KeyCode::KeyA && self.focus.is_none() {
            let r = self.runtime.borrow();
            self.selection = r
                .visible
                .iter()
                .map(|&i| r.source[i].address.clone())
                .collect();
            return Some(Outcome::None);
        }
        if key == KeyCode::Delete && self.focus.is_none() {
            return Some(self.activate(44));
        }
        if matches!(
            key,
            KeyCode::ArrowDown
                | KeyCode::ArrowUp
                | KeyCode::PageDown
                | KeyCode::PageUp
                | KeyCode::Home
                | KeyCode::End
        ) {
            let r = self.runtime.borrow();
            let len = r.visible.len();
            if len == 0 {
                return Some(Outcome::None);
            }
            let current = self
                .state
                .view
                .selected
                .as_ref()
                .and_then(|a| r.visible.iter().position(|&i| &r.source[i].address == a));
            let at = match key {
                KeyCode::Home => 0,
                KeyCode::End => len.saturating_sub(1),
                KeyCode::PageDown => current.map_or(0, |i| (i + 5).min(len.saturating_sub(1))),
                KeyCode::PageUp => current.unwrap_or(0).saturating_sub(5),
                KeyCode::ArrowDown => current.map_or(0, |i| (i + 1).min(len.saturating_sub(1))),
                _ => current.unwrap_or(0).saturating_sub(1),
            };
            let address = r.visible.get(at).map(|&i| r.source[i].address.clone());
            drop(r);
            if ctrl && matches!(key, KeyCode::ArrowUp | KeyCode::ArrowDown) {
                self.reorder(if key == KeyCode::ArrowUp { -1 } else { 1 });
            } else if let Some(address) = address {
                self.select(address);
                let r = self.runtime.get_mut();
                let (_, top, bottom, _) = Self::list_area(r.size,self.state.view.tab);
                let y = at as f32 * draw::ROW;
                if y < r.scroll.amount {
                    r.scroll.amount = y;
                } else if y + draw::ROW > r.scroll.amount + bottom - top {
                    r.scroll.amount = y + draw::ROW - (bottom - top);
                }
            }
            return Some(Outcome::None);
        }
        None
    }
}
impl Drop for Browser {
    fn drop(&mut self) {
        self.runtime.get_mut().directory.cancel();
        if let Err(e) = self.persist() {
            log::warn!("server browser settings: {e}");
        }
    }
}
