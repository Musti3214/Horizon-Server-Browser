use super::*;

fn csv(text: &str) -> Vec<String> {
    let mut values = Vec::new();
    for value in text
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .take(32)
    {
        if !values
            .iter()
            .any(|s: &String| s.eq_ignore_ascii_case(value))
        {
            values.push(value.chars().take(64).collect());
        }
    }
    values
}
impl Browser {
    pub(super) fn prepare_filters(&mut self) {
        self.filters_before = Some(self.state.view.filters.clone());
        let f = &self.state.view.filters;
        self.fields = vec![
            f.min_players.map(|n| n.to_string()).unwrap_or_default(),
            f.max_players.map(|n| n.to_string()).unwrap_or_default(),
            f.max_ping.map(|n| n.to_string()).unwrap_or_default(),
            f.specific_version.clone(),
            f.tags.join(", "),
            f.language.clone(),
            f.region.clone(),
            f.group.clone(),
        ];
    }
    pub(super) fn field_value(&self, index: usize) -> Option<&str> {
        match &self.modal {
            Modal::Main if index == 0 => Some(&self.state.view.search),
            Modal::Edit { server, tags, .. } => match index {
                0 => Some(&server.name),
                1 => Some(&server.address),
                2 => Some(tags),
                3 => Some(&server.metadata.notes),
                4 => Some(&server.metadata.group),
                5 => Some(&server.metadata.language),
                6 => Some(&server.metadata.region),
                _ => None,
            },
            Modal::Direct(value)
            | Modal::Presets(value)
            | Modal::Transfer(value)
            | Modal::BulkTag(value)
            | Modal::Feedback { text: value, .. }
                if index == 0 =>
            {
                Some(value)
            }
            Modal::Filters(_) => self.fields.get(index).map(String::as_str),
            _ => None,
        }
    }
    pub fn focused_text(&mut self) -> Option<(crate::clipboard::FieldId, &mut String, usize)> {
        let index = self.focus?;
        let feedback = matches!(self.modal,Modal::Feedback {..});
        let value = match &mut self.modal {
            Modal::Main if index == 0 => &mut self.state.view.search,
            Modal::Edit { server, tags, .. } => match index {
                0 => &mut server.name,
                1 => &mut server.address,
                2 => tags,
                3 => &mut server.metadata.notes,
                4 => &mut server.metadata.group,
                5 => &mut server.metadata.language,
                6 => &mut server.metadata.region,
                _ => return None,
            },
            Modal::Direct(value)
            | Modal::Presets(value)
            | Modal::Transfer(value)
            | Modal::BulkTag(value)
            | Modal::Feedback { text: value, .. }
                if index == 0 =>
            {
                value
            }
            Modal::Filters(_) => self.fields.get_mut(index)?,
            _ => return None,
        };
        Some((
            ("browser.field", index),
            value,
            if feedback {500} else if index == 3 { 4096 } else { 512 },
        ))
    }
    pub fn focused_box(&self, size: (f32, f32)) -> Option<[f32; 4]> {
        let index = self.focus?;
        self.form_fields(size)
            .into_iter()
            .find(|(i, _, _)| *i == index)
            .map(|(_, _, r)| r)
    }
    pub(super) fn form_fields(&self, (w, h): (f32, f32)) -> Vec<(usize, String, [f32; 4])> {
        let x = ((w - 360.0).max(24.0) / 2.0).floor();
        let width = (w - 24.0).min(360.0);
        let rows: &[(usize, &str)] = match &self.modal {
            Modal::Main => {
                let (offset, width) = draw::viewport(w);
                return vec![(0, tr("search"), [offset + 12.0, 27.0, width - 84.0, 20.0])];
            }
            Modal::Edit { page: 0, .. } => {
                &[(0, "name"), (1, "address"), (2, "tags"), (3, "notes")]
            }
            Modal::Edit { .. } => &[(4, "group"), (5, "language"), (6, "region")],
            Modal::Direct(_) => &[(0, "address")],
            Modal::Filters(1) => &[
                (0, "min_players"),
                (1, "max_players"),
                (2, "max_ping"),
                (3, "specific_version"),
            ],
            Modal::Filters(2) => &[(4, "tags"), (5, "language"), (6, "region"), (7, "group")],
            Modal::Presets(_) => &[(0, "preset_name")],
            Modal::Transfer(_) => &[(0, "file_path")],
            Modal::BulkTag(_) => &[(0, "tags")],
            Modal::Feedback { category, .. } => match category {
                2 => &[(0, "feedback_server_prompt")],
                _ => &[(0, "feedback_prompt")],
            },
            _ => &[],
        };
        rows.iter()
            .enumerate()
            .map(|(row, (i, label))| {
                (
                    *i,
                    tr(label),
                    [
                        x,
                        (if matches!(self.modal, Modal::Filters(_)) {
                            57.0
                        } else if matches!(self.modal, Modal::Feedback { .. }) {
                            75.0
                        } else {
                            47.0
                        }) + row as f32 * ((h - 100.0) / 4.0).clamp(27.0, 34.0),
                        width,
                        20.0,
                    ],
                )
            })
            .collect()
    }
    pub(super) fn activate_form(&mut self, id: u8) -> Outcome {
        match self.modal.clone() {
            Modal::Edit {
                index,
                mut server,
                tags,
                page,
            } => match id {
                50 => {
                    server.name = server.name.trim().to_owned();
                    server.address = server.address.trim().to_owned();
                    server.metadata.tags = csv(&tags);
                    if let Err(e) = self.catalog.validate(&server, index) {
                        self.error = Some(match e {
                            core::Error::Duplicate => tr("duplicate"),
                            _ => tr("invalid_address"),
                        });
                        return Outcome::None;
                    }
                    let before = self.catalog.servers.clone();
                    match index {
                        Some(i) if i < self.catalog.servers.len() => {
                            self.catalog.servers[i] = *server.clone()
                        }
                        _ => self.catalog.servers.push(*server.clone()),
                    }
                    if self.save_catalog(before) {
                        self.state.view.selected = Some(server.address.clone());
                        self.selection = BTreeSet::from([server.address.clone()]);
                        self.main();
                        self.refresh();
                    }
                }
                60 => {
                    server.packs = match server.packs {
                        None => Some(true),
                        Some(true) => Some(false),
                        Some(false) => None,
                    };
                    self.modal = Modal::Edit {
                        index,
                        server,
                        tags,
                        page,
                    };
                }
                61 => {
                    server.metadata.auth = match server.metadata.auth {
                        AuthMode::Unknown => AuthMode::Offline,
                        AuthMode::Offline => AuthMode::Online,
                        AuthMode::Online => AuthMode::Unknown,
                    };
                    self.modal = Modal::Edit {
                        index,
                        server,
                        tags,
                        page,
                    };
                }
                62 => {
                    self.modal = Modal::Edit {
                        index,
                        server,
                        tags,
                        page: 1 - page,
                    };
                    self.focus = None;
                }
                _ => {}
            },
            Modal::Direct(address) if id == 50 => {
                if address::parse(&address).is_err() {
                    self.error = Some(tr("invalid_address"));
                } else {
                    self.record_recent(Server {
                        name: address.clone(),
                        address: address.clone(),
                        ..Server::default()
                    });
                    return Outcome::Connect(address);
                }
            }
            Modal::Remove if id == 50 => {
                let before = self.catalog.servers.clone();
                let removed = before
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| self.selection.contains(&s.address))
                    .map(|(i, s)| (i, s.clone()))
                    .collect::<Vec<_>>();
                self.catalog
                    .servers
                    .retain(|s| !self.selection.contains(&s.address));
                if before.len() != self.catalog.servers.len() && self.save_catalog(before.clone()) {
                    self.undo = Some((removed, Instant::now()));
                    self.selection.clear();
                    self.state.view.selected = None;
                    self.main();
                    self.notice = Some((tr("removed"), Instant::now()));
                }
            }
            Modal::Filters(_) => {
                let f = &mut self.state.view.filters;
                match id {
                    52..=54 => {
                        self.modal = Modal::Filters(usize::from(id - 52));
                        self.focus = None;
                    }
                    70 => {
                        f.online = match f.online {
                            None => Some(true),
                            Some(true) => Some(false),
                            Some(false) => None,
                        }
                    }
                    71 => {
                        f.version = match f.version {
                            query::Version::Any => query::Version::Compatible,
                            query::Version::Compatible => query::Version::Specific,
                            query::Version::Specific => query::Version::Any,
                        }
                    }
                    72 => f.free_slots = !f.free_slots,
                    73 => f.not_empty = !f.not_empty,
                    74 => f.not_full = !f.not_full,
                    75 => f.favorites = !f.favorites,
                    76 => f.joinable = !f.joinable,
                    77 => {
                        f.auth = match f.auth {
                            None => Some(AuthMode::Offline),
                            Some(AuthMode::Offline) => Some(AuthMode::Online),
                            _ => None,
                        }
                    }
                    50 => {
                        let number = |i: usize| {
                            if self.fields[i].trim().is_empty() {
                                Ok(None)
                            } else {
                                self.fields[i].trim().parse::<u32>().map(Some)
                            }
                        };
                        match (number(0), number(1), number(2)) {
                            (Ok(min), Ok(max), Ok(ping))
                                if min.zip(max).is_none_or(|(a, b)| a <= b) =>
                            {
                                f.min_players = min;
                                f.max_players = max;
                                f.max_ping = ping.map(u64::from);
                                f.specific_version = self.fields[3].trim().into();
                                f.tags = csv(&self.fields[4]);
                                f.language = self.fields[5].trim().into();
                                f.region = self.fields[6].trim().into();
                                f.group = self.fields[7].trim().into();
                                self.filters_before = None;
                                self.main();
                            }
                            _ => self.error = Some(tr("invalid_filter")),
                        }
                    }
                    _ => {}
                }
                self.changed();
            }
            Modal::Sort if (100..109).contains(&id) => {
                self.state.view.sort = Sort::ALL[usize::from(id - 100)];
                self.main();
                self.changed();
            }
            Modal::Sort if id == 33 || id == 34 => {
                self.main();
                return self.activate(id);
            }
            Modal::Settings => {
                let s = &mut self.state.settings;
                match id {
                    80 => s.modern = !s.modern,
                    81 => s.classic_menu = !s.classic_menu,
                    82 => {
                        s.concurrent_pings = match s.concurrent_pings {
                            1..=3 => 4,
                            4..=7 => 8,
                            8..=15 => 16,
                            _ => 1,
                        }
                    }
                    83 => {
                        s.refresh_seconds = match s.refresh_seconds {
                            0 => 30,
                            1..=30 => 60,
                            31..=60 => 120,
                            _ => 0,
                        }
                    }
                    84 => s.online_directory = !s.online_directory,
                    85 => {
                        *s = core::model::Settings::default();
                        self.state.view = View::default();
                    }
                    86 => s.reduced_motion = !s.reduced_motion,
                    87 => {
                        let at = Tab::ALL
                            .iter()
                            .position(|t| *t == s.default_tab)
                            .unwrap_or(0);
                        s.default_tab = Tab::ALL[(at + 1) % 5];
                        self.state.view.tab = s.default_tab;
                    }
                    88 => {
                        let at = Sort::ALL
                            .iter()
                            .position(|t| *t == s.default_sort)
                            .unwrap_or(0);
                        s.default_sort = Sort::ALL[(at + 1) % 9];
                        self.state.view.sort = s.default_sort;
                    }
                    89 => s.show_offline = !s.show_offline,
                    90 => {
                        super::open_support_url();
                        return Outcome::None;
                    }
                    91 => {
                        self.focus = Some(0);
                        self.button_focus = None;
                        self.show(Modal::Feedback {
                            category: 0,
                            text: String::new(),
                        });
                        return Outcome::None;
                    }
                    50 => {
                        self.save_metadata();
                        self.main();
                        return Outcome::None;
                    }
                    _ => {}
                }
                if let Err(e) = self.persist() {
                    self.error = Some(label_value("save_error", e));
                }
                self.changed();
                if id == 82 {
                    match PingService::new(self.state.settings.concurrent_pings) {
                        Ok(service) => self.runtime.get_mut().service = service,
                        Err(e) => self.error = Some(label_value("load_error", e)),
                    };
                    self.refresh();
                }
            }
            Modal::Consent if id == 50 => {
                self.state.settings.online_directory = true;
                self.save_metadata();
                self.show(Modal::Settings);
            }
            Modal::Presets(name) => {
                if id == 50 {
                    let name = name.trim();
                    if name.is_empty() {
                        self.error = Some(tr("preset_name"));
                    } else {
                        self.state
                            .presets
                            .insert(name.to_owned(), self.state.view.clone());
                        self.save_metadata();
                        self.main();
                    }
                } else if (120..140).contains(&id)
                    && let Some(view) = self
                        .state
                        .presets
                        .values()
                        .nth(self.preset_page * 6 + usize::from(id - 120))
                        .cloned()
                {
                    self.state.view = view;
                    self.main();
                    self.changed();
                } else if id == 55 {
                    self.state.presets.remove(name.trim());
                    self.save_metadata();
                } else if id == 56 {
                    self.preset_page = self.preset_page.saturating_sub(1);
                } else if id == 57 && (self.preset_page + 1) * 6 < self.state.presets.len() {
                    self.preset_page += 1;
                }
            }
            Modal::BulkTag(tags) if id == 50 => {
                let tags = csv(&tags);
                for s in &mut self.catalog.servers {
                    if self.selection.contains(&s.address) {
                        for tag in &tags {
                            if !s.metadata.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
                                s.metadata.tags.push(tag.clone());
                            }
                        }
                    }
                }
                self.save_metadata();
                self.main();
            }
            Modal::Transfer(path) => {
                let path = PathBuf::from(path);
                if matches!(id, 55 | 56) && self.reserved_export(&path) {
                    self.error = Some(tr("export_reserved"));
                    return Outcome::None;
                }
                let before = self.catalog.servers.clone();
                let result = match id {
                    50 => {
                        if path.extension().is_some_and(|e| e == "dat") {
                            self.catalog.import_dat(&path)
                        } else {
                            self.catalog.import_json(&path)
                        }
                    }
                    55 => self.catalog.export_json(&path).map(|_| 0),
                    56 => self.export_dat(&path).map(|_| 0),
                    _ => return Outcome::None,
                };
                match result {
                    Ok(n) => {
                        if id == 50 && !self.save_catalog(before) {
                            return Outcome::None;
                        }
                        self.notice = Some((
                            if id == 50 {
                                label_value("imported", n)
                            } else {
                                tr("exported")
                            },
                            Instant::now(),
                        ));
                        self.main();
                    }
                    Err(e) => self.error = Some(label_value("file_error", e)),
                }
            }
            Modal::Feedback { category, text } => match id {
                160..=163 => {
                    self.modal = Modal::Feedback {
                        category: (id - 160) as usize,
                        text,
                    };
                    self.changed();
                }
                50 | 96 | 97 => {
                    let trimmed = text.trim();
                    if trimmed.is_empty() {
                        self.error = Some(tr("feedback_empty"));
                        return Outcome::None;
                    }
                    let cat_name = match category {
                        0 => tr("feedback_idea"),
                        1 => tr("feedback_bug"),
                        2 => tr("feedback_server"),
                        _ => tr("feedback_general"),
                    };
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    let line = format!("[{timestamp}] [{cat_name}] {trimmed}\n");
                    let log_path = self.home.join("feedback.log");
                    let saved = (|| -> std::io::Result<()> {
                    let mut file = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&log_path)?;
                        use std::io::Write;
                        file.write_all(line.as_bytes())?;
                    if category == 2 {
                        let server_log = self.home.join("server_submissions.log");
                        let mut file = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&server_log)?;
                            file.write_all(line.as_bytes())?;
                    }
                    Ok(()) })();
                    if saved.is_err() { self.error=Some(tr("feedback_draft_failed")); return Outcome::None; }
                    if id != 96 {
                        self.runtime.get_mut().feedback.send(&self.home,category,trimmed);
                        self.error=None;
                        return Outcome::None;
                    }
                    self.notice = Some((
                        if category == 2 {
                            tr("feedback_server_saved")
                        } else {
                            tr("feedback_saved")
                        },
                        Instant::now(),
                    ));
                    self.main();
                }
                _ => {}
            },
            _ => {}
        }
        Outcome::None
    }
    fn export_dat(&self, path: &Path) -> core::Result<()> {
        // A vanilla export is a byte-for-byte copy of the current saved NBT,
        // including opaque fields. Never synthesize a lossy new servers.dat.
        let bytes = persistence::read_bounded(&self.home.join("servers.dat"))?;
        persistence::atomic_write(path, &bytes)
    }
    fn reserved_export(&self, path: &Path) -> bool {
        let resolve = |path: &Path| {
            std::fs::canonicalize(path).ok().or_else(|| {
                Some(
                    std::fs::canonicalize(path.parent()?)
                        .ok()?
                        .join(path.file_name()?),
                )
            })
        };
        let Some(target) = resolve(path) else {
            return false;
        };
        [
            "servers.dat",
            "servers.dat.before-modern-browser",
            "horizon-server-browser.json",
        ]
        .iter()
        .any(|name| resolve(&self.home.join(name)).is_some_and(|reserved| reserved == target))
    }
}
