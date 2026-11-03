use super::*;
use crate::{
    hud::rgba,
    text::Text,
    widgets::{self, Kind},
};
pub(super) const ROW: f32 = 60.0;
const INK: u32 = 0xffffffff;
const MUTED: u32 = 0xffa0a0a0;
// HudBatch packs colors as ABGR; these are Minecraft's gold, red and lime.
const GOLD: u32 = 0xff00aaff;
const EMERALD: u32 = 0xff55ff55;
const RED: u32 = 0xff5555ff;
const LIME: u32 = 0xff55ffaa;
const SLATE_MUTED: u32 = 0xffa0a0a0;
const DARK_SLATE: u32 = 0xff707070;

fn safe_icon(bytes: &[u8]) -> Option<std::sync::Arc<Vec<u8>>> {
    let reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .ok()?;
    if reader.info().width != 64 || reader.info().height != 64 {
        return None;
    }
    crate::multiplayer::icon_pixels(bytes)
}
pub(super) fn inside((x, y): (f32, f32), [rx, ry, w, h]: [f32; 4]) -> bool {
    x >= rx && y >= ry && x < rx + w && y < ry + h
}
fn text(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    value: &str,
    x: f32,
    y: f32,
    width: f32,
    color: u32,
) {
    let start = batch.vertices.len();
    batch.text(atlas, value, x, y, color, true);
    batch.clip_since(start, x, y, x + width, y + 10.0);
}

// Keep the title panorama visible, like Minecraft's own server list.
pub(super) fn viewport(width: f32) -> (f32, f32) {
    let content = width.min(740.0);
    (((width - content) / 2.0).floor(), content)
}
fn frame(batch: &mut HudBatch, atlas: &HudAtlas, [x, y, w, h]: [f32; 4], color: u32) {
    batch.fill(atlas, x, y, w, 1.0, color);
    batch.fill(atlas, x, y + h - 1.0, w, 1.0, color);
    batch.fill(atlas, x, y, 1.0, h, color);
    batch.fill(atlas, x + w - 1.0, y, 1.0, h, color);
}
fn card_panel(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    rect: [f32; 4],
    selected: bool,
    hovered: bool,
) {
    let [x, y, w, h] = rect;
    batch.fill(
        atlas,
        x,
        y,
        w,
        h,
        rgba(
            0,
            0,
            0,
            if selected {
                112
            } else if hovered {
                80
            } else {
                38
            },
        ),
    );
    if selected {
        frame(batch, atlas, rect, rgba(225, 225, 225, 255));
        frame(
            batch,
            atlas,
            [x + 1.0, y + 1.0, w - 2.0, h - 2.0],
            rgba(0, 0, 0, 180),
        );
    } else if hovered {
        frame(batch, atlas, rect, rgba(180, 180, 180, 150));
    }
}
fn details_panel(batch: &mut HudBatch, atlas: &HudAtlas, rect: [f32; 4]) {
    let [x, y, w, h] = rect;
    batch.fill(atlas, x, y, w, h, rgba(0, 0, 0, 92));
    batch.fill(atlas, x, y, 1.0, h, rgba(210, 210, 210, 60));
}

fn sort_name(sort: Sort) -> String {
    tr(match sort {
        Sort::Custom => "custom",
        Sort::Name => "name",
        Sort::Players => "players",
        Sort::FreeSlots => "free_slots",
        Sort::Ping => "ping",
        Sort::Version => "version",
        Sort::LastPlayed => "last_played",
        Sort::DateAdded => "date_added",
        Sort::Favorites => "favorites_first",
    })
}
fn tab_name(tab: Tab) -> String {
    tr(match tab {
        Tab::Saved => "my_servers",
        Tab::Discover => "discover",
        Tab::Favorites => "favorites",
        Tab::Recent => "recent",
        Tab::Lan => "lan",
    })
}
fn auth_name(auth: AuthMode) -> String {
    tr(match auth {
        AuthMode::Unknown => "auth_unknown",
        AuthMode::Offline => "auth_offline",
        AuthMode::Online => "auth_online",
    })
}
fn health(status: Option<&Status>) -> String {
    tr(match status.map(|s| s.health).unwrap_or(Health::Pinging) {
        Health::Pinging => "pinging",
        Health::Online => "online",
        Health::Offline => "offline",
        Health::Timeout => "timeout",
        Health::DnsError => "dns_error",
        Health::Malformed => "malformed",
    })
}
fn on(value: bool) -> String {
    tr(if value { "on" } else { "off" })
}

impl Browser {
    pub(super) fn draw_buttons(&self, atlas: &HudAtlas, batch: &mut HudBatch, cursor: (f32, f32)) {
        for (x, y, w, label, enabled, id) in self.buttons((batch.width.ceil(), batch.height.ceil()))
        {
            let hovered = enabled && crate::ui::hovered(cursor, x, y, w, 20.0);
            // Use the game's real resource-pack-aware Minecraft button widget.
            crate::ui::button(
                batch,
                atlas,
                x,
                y,
                w,
                20.0,
                &Text::literal(label),
                enabled,
                hovered,
            );
            let tab = matches!(self.modal, Modal::Main) && (10..15).contains(&id);
            let selected = tab && self.state.view.tab == Tab::ALL[(id - 10) as usize];
            let sort = matches!(self.modal, Modal::Main)
                && matches!(
                    (id, self.state.view.sort),
                    (52, Sort::Players) | (53, Sort::Ping) | (54, Sort::Name)
                );
            if selected || sort {
                frame(batch, atlas, [x, y, w, 20.0], rgba(255, 255, 255, 220));
            }
        }
    }
    fn settings_width() -> f32 {
        (tr("settings").chars().count() as f32 * 6.0 + 12.0).clamp(55.0, 100.0)
    }
    fn active_chips(&self) -> Vec<(u8, String)> {
        let f = &self.state.view.filters;
        [
            (130, f.online.is_some(), "status"),
            (131, f.version != query::Version::Any, "version"),
            (
                132,
                f.min_players.is_some()
                    || f.max_players.is_some()
                    || f.free_slots
                    || f.not_empty
                    || f.not_full,
                "players",
            ),
            (133, f.max_ping.is_some(), "ping"),
            (134, !f.tags.is_empty(), "tag"),
            (135, !f.language.is_empty(), "language"),
            (136, !f.region.is_empty(), "region"),
            (137, f.favorites, "favorites"),
            (138, f.auth.is_some(), "auth"),
            (139, f.joinable, "joinable"),
            (140, !f.group.is_empty(), "group"),
        ]
        .into_iter()
        .filter(|(_, active, _)| *active)
        .map(|(id, _, key)| (id, format!("{} ×", tr(key))))
        .collect()
    }
    pub(super) fn list_area((w, h): (f32, f32), tab: Tab) -> (f32, f32, f32, f32) {
        let (offset, w) = viewport(w);
        let width = if w >= 720.0 { w - 256.0 } else { w - 24.0 };
        let footer = if tab == Tab::Discover { 57.0 } else { 81.0 };
        let top = if tab == Tab::Discover { 117.0 } else { 103.0 };
        (offset + 12.0, top, (h - footer).max(top + 1.0), width)
    }
    pub fn buttons(&self, (w, h): (f32, f32)) -> Vec<ButtonSpec> {
        let mut result = Vec::new();
        let mut b = |id: u8, label: String, x: f32, y: f32, width: f32, enabled: bool| {
            result.push((x.floor(), y.floor(), width.floor(), label, enabled, id))
        };
        if matches!(self.modal, Modal::Main) {
            let (offset, w) = viewport(w);
            let settings_width = Self::settings_width();
            b(
                23,
                tr("settings"),
                w - settings_width - 12.0,
                3.0,
                settings_width,
                true,
            );
            b(
                24,
                tr("classic"),
                w - settings_width - 71.0,
                3.0,
                55.0,
                true,
            );
            let kofi_right = w - settings_width - 75.0;
            if !self.state.settings.support_dismissed && w >= 360.0 {
                let is_compact = w < 440.0;
                let support_w = if is_compact { 28.0 } else { 62.0 };
                let feedback_w = 18.0;
                let dismiss_w = 18.0;
                let dismiss_x = kofi_right - dismiss_w;
                let feedback_x = dismiss_x - feedback_w - 2.0;
                let btn_x = feedback_x - support_w - 2.0;
                b(
                    62,
                    if is_compact {
                        "☕".to_string()
                    } else {
                        tr("support")
                    },
                    btn_x,
                    3.0,
                    support_w,
                    true,
                );
                b(64, "💬".to_string(), feedback_x, 3.0, feedback_w, true);
                b(63, "×".to_string(), dismiss_x, 3.0, dismiss_w, true);
            } else if w >= 320.0 {
                let feedback_w = 18.0;
                let feedback_x = kofi_right - feedback_w;
                b(64, "💬".to_string(), feedback_x, 3.0, feedback_w, true);
            }
            b(26, tr("clear"), w - 66.0, 27.0, 54.0, true);
            let tab_w = (w - 24.0 - 16.0) / 5.0;
            for (i, tab) in Tab::ALL.into_iter().enumerate() {
                let label = format!(
                    "{}{}",
                    if self.state.view.tab == tab {
                        "• "
                    } else {
                        ""
                    },
                    tab_name(tab)
                );
                b(
                    10 + i as u8,
                    label,
                    12.0 + i as f32 * (tab_w + 4.0),
                    51.0,
                    tab_w,
                    true,
                );
            }
            let chips = self.active_chips();
            let filter_label = if chips.is_empty() {
                tr("filters")
            } else {
                format!("{} ({})", tr("filters"), chips.len())
            };
            let players_label = if self.state.view.sort == Sort::Players {
                format!(
                    "• {} {}",
                    tr("players"),
                    if self.state.view.descending {
                        "▼"
                    } else {
                        "▲"
                    }
                )
            } else {
                tr("players")
            };
            let ping_label = if self.state.view.sort == Sort::Ping {
                format!(
                    "• {} {}",
                    tr("ping"),
                    if self.state.view.descending {
                        "▲"
                    } else {
                        "▼"
                    }
                )
            } else {
                tr("ping")
            };
            let name_label = if self.state.view.sort == Sort::Name {
                format!(
                    "• {} {}",
                    tr("name"),
                    if self.state.view.descending {
                        "▼"
                    } else {
                        "▲"
                    }
                )
            } else {
                tr("name")
            };

            let is_narrow = w < 420.0;
            let (f_w, p_w, pi_w, n_w, more_w, reset_w) = if is_narrow {
                (52.0, 64.0, 44.0, 44.0, 26.0, 26.0)
            } else {
                (64.0, 78.0, 56.0, 56.0, 30.0, 30.0)
            };

            let mut x = 12.0;
            b(20, filter_label, x, 77.0, f_w, true);
            x += f_w + 4.0;
            b(52, players_label, x, 77.0, p_w, true);
            x += p_w + 4.0;
            b(53, ping_label, x, 77.0, pi_w, true);
            x += pi_w + 4.0;
            b(54, name_label, x, 77.0, n_w, true);
            x += n_w + 4.0;

            let right_start = w - 12.0 - more_w - 4.0 - reset_w;
            let available_for_chips = (right_start - x - 4.0).max(0.0);
            let chip_w = 72.0;
            let chips_fit = (available_for_chips / (chip_w + 4.0)).floor() as usize;
            for (id, label) in chips.iter().take(chips_fit) {
                b(*id, label.clone(), x, 77.0, chip_w, true);
                x += chip_w + 4.0;
            }

            b(21, "…".into(), right_start, 77.0, more_w, true);
            b(
                27,
                "↺".into(),
                right_start + more_w + 4.0,
                77.0,
                reset_w,
                true,
            );

            let selected = self.selected_server().is_some();
            let saved = selected && self.selected_index().is_some();
            if self.state.view.tab != Tab::Discover {
                let minimum = [40.0_f32, 40.0, 20.0, 20.0, 40.0, 48.0, 48.0];
                let keys = ["favorite", "tag", "up", "down", "details", "reset", "undo"];
                let widths = std::array::from_fn::<_, 7, _>(|i| {
                    minimum[i].max(tr(keys[i]).chars().count() as f32 * 6.0 + 10.0)
                });
                let factor = ((w - 42.0) / widths.iter().sum::<f32>()).min(1.0);
                let mut x = 12.0;
                for (i, (id, key, enabled)) in [
                    (29, "favorite", saved),
                    (30, "tag", saved),
                    (31, "up", saved),
                    (32, "down", saved),
                    (25, "details", selected),
                    (27, "reset", true),
                    (
                        28,
                        "undo",
                        self.undo
                            .as_ref()
                            .is_some_and(|(_, t)| t.elapsed() < Duration::from_secs(30)),
                    ),
                ]
                .into_iter()
                .enumerate()
                {
                    b(id, tr(key), x, h - 77.0, widths[i] * factor, enabled);
                    x += widths[i] * factor + 3.0;
                }
            }
            let fw = ((w - 24.0 - 8.0) / 3.0).floor();
            for (i, (id, key, enabled)) in [
                (40, "join", selected),
                (41, "direct", true),
                (42, "add", true),
            ]
            .into_iter()
            .enumerate()
            {
                b(
                    id,
                    tr(key),
                    12.0 + i as f32 * (fw + 4.0),
                    h - 52.0,
                    fw,
                    enabled,
                );
            }
            let fw = ((w - 24.0 - 12.0) / 4.0).floor();
            for (i, (id, key, enabled)) in [
                (
                    if self.state.view.tab == Tab::Discover {
                        25
                    } else {
                        43
                    },
                    if self.state.view.tab == Tab::Discover {
                        "details"
                    } else {
                        "edit"
                    },
                    if self.state.view.tab == Tab::Discover {
                        selected
                    } else {
                        saved
                    },
                ),
                (44, "remove", saved),
                (45, "refresh", true),
                (46, "back", true),
            ]
            .into_iter()
            .enumerate()
            {
                b(
                    id,
                    tr(key),
                    12.0 + i as f32 * (fw + 4.0),
                    h - 28.0,
                    fw,
                    enabled,
                );
            }
            for button in &mut result {
                button.0 += offset;
            }
            return result;
        }
        let x = ((w - 360.0).max(24.0) / 2.0).floor();
        let fw = (w - 24.0).min(360.0);
        if matches!(self.modal, Modal::Feedback { .. }) {
            b(51, tr("cancel"), x, h - 28.0, fw, true);
        } else {
            b(
                51,
                tr("back"),
                x + fw / 2.0 + 2.0,
                h - 28.0,
                fw / 2.0 - 2.0,
                true,
            );
            if !matches!(self.modal, Modal::Sort | Modal::Details) {
                b(
                    50,
                    tr(if matches!(self.modal, Modal::Consent) {
                        "consent"
                    } else if matches!(self.modal, Modal::Remove) {
                        "confirm_remove"
                    } else if matches!(self.modal, Modal::Transfer(_)) {
                        "import"
                    } else {
                        "apply"
                    }),
                    x,
                    h - 28.0,
                    fw / 2.0 - 2.0,
                    match &self.modal {
                        Modal::Edit { index, server, .. } => {
                            self.catalog.validate(server, *index).is_ok()
                        }
                        Modal::Direct(address) => address::parse(address).is_ok(),
                        _ => true,
                    },
                );
            }
        }
        match &self.modal {
            Modal::Edit { server, page, .. } => {
                let y = h - 55.0;
                let packs = tr(match server.packs {
                    None => "packs_ask",
                    Some(true) => "packs_on",
                    Some(false) => "packs_off",
                });
                b(60, packs, x, y, fw * 0.34 - 3.0, true);
                b(
                    61,
                    auth_name(server.metadata.auth),
                    x + fw * 0.34,
                    y,
                    fw * 0.39 - 3.0,
                    true,
                );
                b(
                    62,
                    tr(if *page == 0 { "more" } else { "previous" }),
                    x + fw * 0.73,
                    y,
                    fw * 0.27,
                    true,
                );
            }
            Modal::Filters(page) => {
                for (i, key) in ["status_filters", "value_filters", "tag_filters"]
                    .iter()
                    .enumerate()
                {
                    b(
                        52 + i as u8,
                        tr(key),
                        x + i as f32 * (fw + 4.0) / 3.0,
                        24.0,
                        (fw - 8.0) / 3.0,
                        true,
                    );
                }
                if *page == 0 {
                    let f = &self.state.view.filters;
                    let values = [
                        label_value(
                            "status",
                            f.online
                                .map_or(tr("any"), |v| tr(if v { "online" } else { "offline" })),
                        ),
                        label_value(
                            "version",
                            tr(match f.version {
                                query::Version::Any => "any",
                                query::Version::Compatible => "compatible",
                                query::Version::Specific => "specific_version",
                            }),
                        ),
                        label_value("free_slots", on(f.free_slots)),
                        label_value("not_empty", on(f.not_empty)),
                        label_value("not_full", on(f.not_full)),
                        label_value("favorites", on(f.favorites)),
                        label_value("joinable", on(f.joinable)),
                        label_value("auth", f.auth.map(auth_name).unwrap_or_else(|| tr("any"))),
                    ];
                    for (i, value) in values.into_iter().enumerate() {
                        b(
                            70 + i as u8,
                            value,
                            x + (i % 2) as f32 * (fw + 4.0) / 2.0,
                            51.0 + (i / 2) as f32 * 27.0,
                            (fw - 4.0) / 2.0,
                            true,
                        );
                    }
                }
            }
            Modal::Sort => {
                for (i, sort) in Sort::ALL.into_iter().enumerate() {
                    b(
                        100 + i as u8,
                        sort_name(sort),
                        x + (i % 2) as f32 * (fw + 4.0) / 2.0,
                        37.0 + (i / 2) as f32 * 25.0,
                        (fw - 4.0) / 2.0,
                        true,
                    );
                }
                b(33, tr("presets"), x, h - 55.0, (fw - 4.0) / 2.0, true);
                b(
                    34,
                    tr("files"),
                    x + (fw + 4.0) / 2.0,
                    h - 55.0,
                    (fw - 4.0) / 2.0,
                    true,
                );
            }
            Modal::Settings => {
                let s = &self.state.settings;
                let values = [
                    label_value("modern", on(s.modern)),
                    label_value(
                        "menu_style",
                        tr(if s.classic_menu {
                            "classic"
                        } else {
                            "modern_style"
                        }),
                    ),
                    label_value("concurrency", s.concurrent_pings),
                    label_value(
                        "auto_refresh",
                        if s.refresh_seconds == 0 {
                            tr("off")
                        } else {
                            format!("{} s", s.refresh_seconds)
                        },
                    ),
                    label_value("online_directory", on(s.online_directory)),
                    tr("defaults"),
                    label_value("reduced_motion", on(s.reduced_motion)),
                    label_value("default_tab", tab_name(s.default_tab)),
                    label_value("default_sort", sort_name(s.default_sort)),
                    label_value("show_offline", on(s.show_offline)),
                ];
                for (i, value) in values.into_iter().enumerate() {
                    b(
                        80 + i as u8,
                        value,
                        x + (i % 2) as f32 * (fw + 4.0) / 2.0,
                        37.0 + (i / 2) as f32 * 27.0,
                        (fw - 4.0) / 2.0,
                        true,
                    );
                }
                let half_w = (fw - 4.0) / 2.0;
                b(90, tr("support_project"), x, h - 55.0, half_w, true);
                b(91, tr("feedback"), x + half_w + 4.0, h - 55.0, half_w, true);
            }
            Modal::Presets(_) => {
                for (i, name) in self
                    .state
                    .presets
                    .keys()
                    .skip(self.preset_page * 6)
                    .take(6)
                    .enumerate()
                {
                    b(
                        120 + i as u8,
                        name.clone(),
                        x + (i % 2) as f32 * (fw + 4.0) / 2.0,
                        80.0 + (i / 2) as f32 * 24.0,
                        (fw - 4.0) / 2.0,
                        true,
                    );
                }
                b(
                    56,
                    tr("previous_page"),
                    x,
                    157.0,
                    (fw - 4.0) / 2.0,
                    self.preset_page > 0,
                );
                b(
                    57,
                    tr("next_page"),
                    x + (fw + 4.0) / 2.0,
                    157.0,
                    (fw - 4.0) / 2.0,
                    (self.preset_page + 1) * 6 < self.state.presets.len(),
                );
                b(55, tr("remove_preset"), x, h - 55.0, fw, true);
            }
            Modal::Transfer(_) => {
                b(55, tr("export_json"), x, 90.0, fw, true);
                b(56, tr("export_nbt"), x, 115.0, fw, true);
            }
            Modal::Feedback { category, .. } => {
                let cat_w = (fw - 12.0) / 4.0;
                let cats = [
                    (160, tr("feedback_idea")),
                    (161, tr("feedback_bug")),
                    (162, tr("feedback_server")),
                    (163, tr("feedback_general")),
                ];
                for (i, (id, label)) in cats.into_iter().enumerate() {
                    let label = if *category == i {
                        format!("• {label}")
                    } else {
                        label
                    };
                    b(id, label, x + i as f32 * (cat_w + 4.0), 45.0, cat_w, true);
                }
                let half_w = (fw - 4.0) / 2.0;
                b(96, tr("feedback_save"), x, h - 55.0, half_w, true);
                b(
                    97,
                    tr("feedback_send"),
                    x + half_w + 4.0,
                    h - 55.0,
                    half_w,
                    self.runtime.borrow().feedback.status != feedback::Status::Sending,
                );
            }
            _ => {}
        }
        result
    }

    pub fn draw(
        &self,
        atlas: &HudAtlas,
        batch: &mut HudBatch,
        caret: &mut Caret,
        focused_at: Instant,
        cursor: (f32, f32),
    ) {
        let size = (batch.width.ceil(), batch.height.ceil());
        let (w, h) = size;
        batch.fill(
            atlas,
            0.0,
            0.0,
            w,
            h,
            rgba(
                0,
                0,
                0,
                if matches!(self.modal, Modal::Main) {
                    68
                } else {
                    158
                },
            ),
        );
        let mut r = self.runtime.borrow_mut();
        r.size = size;
        self.sync(&mut r);
        if matches!(self.modal, Modal::Main) {
            let (left, top, bottom, width) = Self::list_area(size, self.state.view.tab);

            let (offset, w) = viewport(w);
            batch.fill(atlas, offset + 8.0, 0.0, w - 16.0, 103.0, rgba(0, 0, 0, 94));
            batch.fill(
                atlas,
                offset + 8.0,
                102.0,
                w - 16.0,
                1.0,
                rgba(255, 255, 255, 38),
            );
            batch.fill(
                atlas,
                offset + 8.0,
                bottom,
                w - 16.0,
                h - bottom,
                rgba(0, 0, 0, 106),
            );
            batch.fill(
                atlas,
                offset + 8.0,
                bottom,
                w - 16.0,
                1.0,
                rgba(255, 255, 255, 38),
            );

            let title_max_w = if !self.state.settings.support_dismissed && w >= 360.0 {
                let support_w = if w < 440.0 { 28.0 } else { 62.0 };
                let kofi_right = w - Self::settings_width() - 75.0;
                let btn_x = kofi_right - 18.0 - 18.0 - 2.0 - support_w - 2.0;
                (btn_x - 16.0).max(80.0)
            } else if w >= 320.0 {
                let kofi_right = w - Self::settings_width() - 75.0;
                (kofi_right - 18.0 - 16.0).max(80.0)
            } else {
                (w - 150.0).max(80.0)
            };
            text(
                batch,
                atlas,
                &tr("title"),
                offset + 13.0,
                10.0,
                title_max_w,
                rgba(16, 12, 4, 180),
            );
            text(
                batch,
                atlas,
                &tr("title"),
                offset + 12.0,
                9.0,
                title_max_w,
                INK,
            );
            let count = crate::text::format_translation(
                &tr("result_count"),
                &[
                    Text::literal(r.visible.len().to_string()),
                    Text::literal(r.source.len().to_string()),
                ],
            )
            .plain();
            let show_count = if !self.state.settings.support_dismissed {
                w > 620.0
            } else {
                w > 500.0
            };
            if show_count {
                let cw = atlas.font.width_text(&Text::literal(&count)).ceil() + 14.0;
                let cx = offset + (w / 2.0 - cw / 2.0).floor();
                text(batch, atlas, &count, cx + 7.0, 8.0, cw - 14.0, SLATE_MUTED);
            }
            let clip = batch.vertices.len();
            let first = (r.smooth / ROW).floor().max(0.0) as usize;
            let end = (first + ((bottom - top) / ROW).ceil() as usize + 1).min(r.visible.len());
            for row in first..end {
                let s = &r.source[r.visible[row]];
                let status = r.statuses.get(&s.address);
                let y = (top + row as f32 * ROW - r.smooth).round();
                let selected = self.selection.contains(&s.address)
                    || self.state.view.selected.as_ref() == Some(&s.address);
                let hovered = inside(cursor, [left, y, width - 9.0, ROW - 4.0]);
                card_panel(
                    batch,
                    atlas,
                    [left, y, width - 9.0, ROW - 4.0],
                    selected,
                    hovered,
                );
                if y >= top && y + ROW - 4.0 <= bottom {
                    widgets::record_as(
                        &s.name,
                        [left, y, width - 24.0, ROW - 4.0],
                        true,
                        Kind::Row,
                    );
                    widgets::target_as([left, y, width - 24.0, ROW - 4.0], Kind::Row);
                }
                let png = status
                    .and_then(|s| s.favicon.as_deref())
                    .and_then(|url| {
                        url.split_once("base64,")
                            .map(|(_, b)| b)
                            .or_else(|| url.strip_prefix("data:image/png;base64,"))
                    })
                    .or(s.icon.as_deref());
                let pixels = png.filter(|s| s.len() <= 65536).and_then(|data| {
                    use std::hash::{Hash, Hasher};
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    data.hash(&mut hasher);
                    let hash = hasher.finish();
                    let mut icons = r.icons.borrow_mut();
                    if icons.len() > 1024 {
                        icons.clear();
                    }
                    if icons.get(&s.address).is_none_or(|(old, _)| *old != hash) {
                        icons.insert(
                            s.address.clone(),
                            (
                                hash,
                                crate::skins::base64_decode(data).and_then(|b| safe_icon(&b)),
                            ),
                        );
                    }
                    icons.get(&s.address).and_then(|(_, pixels)| pixels.clone())
                });

                // Vanilla server icons are plain 32px squares, not inventory slots.
                let slot_x = left + 8.0;
                let slot_y = y + 10.0;
                crate::multiplayer::draw_server_icon(
                    atlas,
                    batch,
                    pixels.as_deref().map(Vec::as_slice),
                    row - first,
                    slot_x + 2.0,
                    slot_y + 2.0,
                );

                // Authentic Mojang Join Play-Arrow Overlay on Hover
                if hovered {
                    let rx = cursor.0 - (slot_x + 2.0);
                    let ry = cursor.1 - (slot_y + 2.0);
                    let over_icon = (0.0..=32.0).contains(&rx) && (0.0..=32.0).contains(&ry);
                    batch.fill(
                        atlas,
                        slot_x + 2.0,
                        slot_y + 2.0,
                        32.0,
                        32.0,
                        rgba(0, 0, 0, 110),
                    );
                    let join_sprite = if over_icon {
                        "gui/sprites/server_list/join_highlighted"
                    } else {
                        "gui/sprites/server_list/join"
                    };
                    batch.sprite_scaled(
                        atlas,
                        join_sprite,
                        slot_x + 2.0,
                        slot_y + 2.0,
                        32.0,
                        32.0,
                        u32::MAX,
                    );
                }

                // Row Line 1 (y + 6.0): Name, Player count, Favorite Star
                text(
                    batch,
                    atlas,
                    &s.name,
                    left + 50.0,
                    y + 6.0,
                    (width - 170.0).max(60.0),
                    INK,
                );

                let is_online = status.is_some_and(|s| s.health == Health::Online);
                let count_str = status
                    .and_then(|s| s.online.zip(s.max))
                    .map_or_else(|| tr("unknown_count"), |(n, m)| format!("{n}/{m}"));

                let pill_w = 78.0;
                let pill_x = left + width - 110.0;
                let pill_y = y + 5.0;
                text(
                    batch,
                    atlas,
                    &count_str,
                    pill_x + 12.0,
                    pill_y + 2.5,
                    pill_w - 14.0,
                    if is_online { MUTED } else { DARK_SLATE },
                );

                text(
                    batch,
                    atlas,
                    if s.metadata.favorite { "★" } else { "☆" },
                    left + width - 26.0,
                    y + 6.0,
                    12.0,
                    if s.metadata.favorite {
                        GOLD
                    } else if hovered {
                        SLATE_MUTED
                    } else {
                        DARK_SLATE
                    },
                );

                // Row Line 2 (y + 19.0): Colored MOTD (Line 1)
                let motd = status
                    .map(|s| {
                        crate::component::to_text(&s.description, &mut |s| Text::literal(s))
                            .resolve_legacy()
                    })
                    .unwrap_or_else(|| Text::literal(health(status)));
                let motd_lines = atlas.font.split(&motd, (width - 165.0).max(40.0));
                let has_line2 = motd_lines.get(1).is_some_and(|l| !l.is_empty());
                if let Some(line) = motd_lines.first() {
                    let start = batch.vertices.len();
                    batch.text_styled(atlas, line, left + 50.0, y + 19.0, INK, true);
                    batch.clip_since(start, left + 50.0, y + 18.0, left + width - 115.0, y + 29.0);
                }

                // Row Line 3 (y + 31.0): MOTD Line 2 or Server Address (Host:Port)
                if let Some(line2) = motd_lines.get(1).filter(|l| !l.is_empty()) {
                    let start = batch.vertices.len();
                    batch.text_styled(atlas, line2, left + 50.0, y + 31.0, 0xffa0a0a0, true);
                    batch.clip_since(start, left + 50.0, y + 30.0, left + width - 20.0, y + 41.0);
                } else {
                    text(
                        batch,
                        atlas,
                        &s.address,
                        left + 50.0,
                        y + 31.0,
                        (width - 170.0).max(40.0),
                        0xffa0a0a0,
                    );
                }

                let ms = status.and_then(|s| s.ms);
                let sprite = match status.map(|s| s.health) {
                    Some(Health::Online) => format!(
                        "gui/sprites/server_list/ping_{}",
                        match ms.unwrap_or(9999) {
                            0..150 => 5,
                            150..300 => 4,
                            300..600 => 3,
                            600..1000 => 2,
                            _ => 1,
                        }
                    ),
                    Some(Health::Pinging) | None => format!(
                        "gui/sprites/server_list/pinging_{}",
                        if self.state.settings.reduced_motion {
                            3
                        } else {
                            (r.opened.elapsed().as_millis() / 160 + row as u128) % 5 + 1
                        }
                    ),
                    _ => "gui/sprites/server_list/unreachable".into(),
                };
                let ping_pill_w = 78.0;
                let ping_pill_x = left + width - 110.0;
                let ping_pill_y = y + 20.0;
                batch.sprite_scaled(
                    atlas,
                    &sprite,
                    ping_pill_x + 4.0,
                    ping_pill_y + 2.5,
                    10.0,
                    8.0,
                    INK,
                );

                let ping_color = match ms {
                    Some(0..=80) => EMERALD,
                    Some(81..=160) => LIME,
                    Some(161..=280) => GOLD,
                    Some(_) => RED,
                    None => SLATE_MUTED,
                };
                text(
                    batch,
                    atlas,
                    &ms.map_or_else(|| health(status), |ms| format!("{ms} ms")),
                    ping_pill_x + 18.0,
                    ping_pill_y + 2.5,
                    ping_pill_w - 20.0,
                    ping_color,
                );

                // Row Line 4 (y + 44.0): Version badge chip & tags
                let version = status.map_or("", |s| s.version.as_str());
                let compatible = status.is_some_and(Status::compatible);
                let mut tag_x = left + 50.0;

                if !version.is_empty() {
                    let status_tag = if status.is_none_or(|s| s.protocol.is_none()) {
                        "unknown"
                    } else if compatible {
                        "compatible"
                    } else {
                        "incompatible"
                    };
                    let chip_text = format!("{version} · {}", tr(status_tag));
                    let tw = atlas.font.width_text(&Text::literal(&chip_text)).ceil();
                    let chip_w = tw + 6.0;

                    let text_col = if compatible {
                        EMERALD
                    } else if status.is_none_or(|s| s.protocol.is_none()) {
                        MUTED
                    } else {
                        RED
                    };
                    text(
                        batch,
                        atlas,
                        &chip_text,
                        tag_x + 3.0,
                        y + 44.0,
                        tw,
                        text_col,
                    );
                    tag_x += chip_w + 6.0;
                }

                if has_line2 && tag_x < left + width - 50.0 {
                    let addr_text = format!("· {}", s.address);
                    let aw = atlas.font.width_text(&Text::literal(&addr_text)).ceil();
                    text(
                        batch,
                        atlas,
                        &addr_text,
                        tag_x,
                        y + 44.0,
                        (left + width - 15.0 - tag_x).min(aw),
                        0xffa0a0a0,
                    );
                    tag_x += aw + 6.0;
                }

                let group_and_tags = if s.metadata.group.is_empty() {
                    s.metadata.tags.join(" · ")
                } else if s.metadata.tags.is_empty() {
                    s.metadata.group.clone()
                } else {
                    format!("{} · {}", s.metadata.group, s.metadata.tags.join(" · "))
                };
                if !group_and_tags.is_empty() && tag_x < left + width - 20.0 {
                    text(
                        batch,
                        atlas,
                        &format!("◆ {group_and_tags}"),
                        tag_x,
                        y + 44.0,
                        (left + width - 15.0 - tag_x).max(20.0),
                        0xffa0a0a0,
                    );
                }
            }
            batch.clip_since(clip, left, top, left + width, bottom);
            crate::ui::scrollbar(
                batch,
                atlas,
                left + width - 6.0,
                top,
                bottom,
                r.visible.len() as f32 * ROW,
                r.scroll.amount,
            );

            // Authentic Minecraft Java Edition 4-step vignette shadows below header & above footer
            for (i, col) in [
                rgba(0, 0, 0, 150),
                rgba(0, 0, 0, 100),
                rgba(0, 0, 0, 55),
                rgba(0, 0, 0, 20),
            ]
            .iter()
            .enumerate()
            {
                batch.fill(atlas, left, top + i as f32, width, 1.0, *col);
            }

            for (i, col) in [
                rgba(0, 0, 0, 20),
                rgba(0, 0, 0, 55),
                rgba(0, 0, 0, 100),
                rgba(0, 0, 0, 150),
            ]
            .iter()
            .enumerate()
            {
                batch.fill(atlas, left, bottom - 4.0 + i as f32, width, 1.0, *col);
            }
            if r.visible.is_empty() {
                let key = match self.state.view.tab {
                    Tab::Discover => {
                        if self.state.settings.online_directory {
                            "directory_ready"
                        } else {
                            "directory_disabled"
                        }
                    }
                    Tab::Lan => "lan_empty",
                    _ => {
                        if r.source.is_empty() {
                            "empty"
                        } else {
                            "no_results"
                        }
                    }
                };
                self.paragraph(
                    batch,
                    atlas,
                    &tr(key),
                    [left + 12.0, top + 12.0, width - 24.0, bottom - top - 12.0],
                    MUTED,
                );
            }
            if self.state.view.tab == Tab::Discover {
                let message = if r.directory.loading() {
                    tr("directory_loading")
                } else if let Some(message) = &r.directory.message {
                    message.clone()
                } else if self.state.settings.online_directory {
                    tr("directory_source")
                } else {
                    tr("directory_disabled")
                };
                let color = if r.directory.message.is_some() {
                    RED
                } else if self.state.settings.online_directory {
                    0xffbfd8ba
                } else {
                    MUTED
                };
                batch.fill(atlas, left, top - 14.0, width, 14.0, rgba(0, 0, 0, 70));
                batch.fill(atlas, left + 4.0, top - 8.0, 3.0, 3.0, color);
                text(
                    batch,
                    atlas,
                    &message,
                    left + 12.0,
                    top - 11.0,
                    width - 20.0,
                    color,
                );
            }
            if w >= 720.0 {
                self.draw_details(
                    atlas,
                    batch,
                    &mut r,
                    [offset + w - 232.0, top, 220.0, bottom - top],
                );
            }
            if w <= 500.0 && self.state.settings.support_dismissed {
                text(
                    batch,
                    atlas,
                    &count,
                    offset + 122.0,
                    9.0,
                    (w - Self::settings_width() - 197.0).max(0.0),
                    MUTED,
                );
            }
        } else {
            let title = match self.modal {
                Modal::Edit { index: None, .. } => "add",
                Modal::Edit { .. } => "edit",
                Modal::Direct(_) => "direct",
                Modal::Remove => "confirm_remove",
                Modal::Filters(_) => "filters",
                Modal::Sort => "sort",
                Modal::Settings => "settings",
                Modal::Presets(_) => "presets",
                Modal::Transfer(_) => "files",
                Modal::BulkTag(_) => "tag",
                Modal::Details => "details",
                Modal::Consent => "privacy",
                Modal::Feedback { .. } => "feedback_title",
                Modal::Main => "title",
            };
            text(batch, atlas, &tr(title), 12.0, 10.0, w - 24.0, INK);
            if matches!(self.modal, Modal::Details) {
                self.draw_details(atlas, batch, &mut r, [12.0, 32.0, w - 24.0, h - 68.0]);
            }
            if matches!(self.modal, Modal::Consent) {
                self.paragraph(
                    batch,
                    atlas,
                    &tr("privacy_note"),
                    [20.0, 44.0, w - 40.0, h - 100.0],
                    INK,
                );
            }
            if matches!(self.modal, Modal::Remove) {
                self.paragraph(
                    batch,
                    atlas,
                    &format!("{} ({})", tr("remove_note"), self.selection.len()),
                    [20.0, 48.0, w - 40.0, h - 100.0],
                    INK,
                );
            }
            if let Modal::Feedback { text: content, .. } = &self.modal {
                let counter = format!("{}/500", content.chars().count());
                text(batch, atlas, &counter, w - 80.0, 10.0, 68.0, MUTED);
                let status = if r.feedback.status == feedback::Status::Idle
                    && !feedback::Sender::configured()
                {
                    feedback::Status::NotConfigured
                } else {
                    r.feedback.status
                };
                self.paragraph(
                    batch,
                    atlas,
                    &tr(status.key()),
                    [20.0, 108.0, w - 40.0, (h - 180.0).max(24.0)],
                    MUTED,
                );
            }
        }
        drop(r);
        if matches!(self.modal, Modal::Main) {
            let (offset, w) = viewport(w);
            let tab_w = (w - 40.0) / 5.0;
            for i in 0..5 {
                widgets::target_as(
                    [offset + 12.0 + i as f32 * (tab_w + 4.0), 51.0, tab_w, 20.0],
                    Kind::Tab(self.state.view.tab == Tab::ALL[i]),
                );
            }
        }
        for (index, label, rect) in self.form_fields(size) {
            if let Some(value) = self.field_value(index) {
                if !matches!(self.modal, Modal::Main) {
                    text(
                        batch,
                        atlas,
                        &label,
                        rect[0],
                        rect[1] - 10.0,
                        rect[2],
                        MUTED,
                    );
                }
                caret.draw(
                    batch,
                    atlas,
                    (
                        ("browser.field", index),
                        value,
                        if matches!(self.modal, Modal::Feedback { .. }) {
                            500
                        } else if index == 3 {
                            4096
                        } else {
                            512
                        },
                    ),
                    rect,
                    self.focus == Some(index),
                    focused_at,
                );
                widgets::record_as(&label, rect, true, Kind::Field);
                widgets::target_as(rect, Kind::Field);
                if value.is_empty() {
                    text(
                        batch,
                        atlas,
                        &label,
                        rect[0] + 5.0,
                        rect[1] + 6.0,
                        rect[2] - 12.0,
                        MUTED,
                    );
                }
            }
        }
        let validation = match &self.modal {
            Modal::Edit { index, server, .. }
                if !server.name.is_empty() || !server.address.is_empty() =>
            {
                self.catalog.validate(server, *index).err().map(|e| {
                    tr(if server.name.trim().is_empty() {
                        "name_required"
                    } else if matches!(e, core::Error::Duplicate) {
                        "duplicate"
                    } else {
                        "invalid_address"
                    })
                })
            }
            Modal::Direct(address) if !address.is_empty() && address::parse(address).is_err() => {
                Some(tr("invalid_address"))
            }
            _ => None,
        };
        if let Some(error) = self.error.as_ref().or(validation.as_ref()) {
            text(
                batch,
                atlas,
                error,
                12.0,
                24.0,
                w - 24.0,
                rgba(255, 145, 128, 255),
            );
        } else if let Some((notice, when)) = &self.notice
            && when.elapsed() < Duration::from_secs(5)
        {
            text(batch, atlas, notice, 12.0, h - 88.0, w - 24.0, INK);
        }
    }
    fn paragraph(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        value: &str,
        [x, y, w, h]: [f32; 4],
        color: u32,
    ) {
        let clip = batch.vertices.len();
        for (i, line) in atlas
            .font
            .split(&Text::literal(value), w)
            .iter()
            .enumerate()
        {
            batch.text_styled(
                atlas,
                &line.clone().with_color(color & 0xffffff),
                x,
                y + i as f32 * 11.0,
                color,
                true,
            );
        }
        batch.clip_since(clip, x, y, x + w, y + h);
    }
    fn draw_details(
        &self,
        atlas: &HudAtlas,
        batch: &mut HudBatch,
        r: &mut Runtime,
        rect: [f32; 4],
    ) {
        let [x, y, w, h] = rect;
        details_panel(batch, atlas, rect);
        let Some(s) = self
            .state
            .view
            .selected
            .as_ref()
            .and_then(|a| r.source.iter().find(|s| &s.address == a))
        else {
            self.paragraph(
                batch,
                atlas,
                &tr("select_details"),
                [x + 9.0, y + 9.0, w - 18.0, h - 18.0],
                SLATE_MUTED,
            );
            return;
        };
        let s = s.clone();
        let status = r.statuses.get(&s.address);
        let mut lines = vec![
            Text::literal(&s.name),
            Text::literal(&s.address),
            Text::literal(health(status)),
            Text::literal(auth_name(s.metadata.auth)),
        ];
        if let Some(p) = status {
            lines.push(Text::literal(format!(
                "{} / {}",
                p.version,
                p.protocol
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| tr("unknown"))
            )));
            lines.push(
                crate::component::to_text(&p.description, &mut |s| Text::literal(s))
                    .resolve_legacy(),
            );
            lines.push(Text::literal(label_value(
                "players",
                p.online
                    .zip(p.max)
                    .map_or_else(|| tr("unknown_count"), |(n, m)| format!("{n}/{m}")),
            )));
            if !p.samples.is_empty() {
                lines.push(Text::literal(p.samples.join(", ")));
            }
        }
        lines.push(Text::literal(label_value(
            "tags",
            s.metadata.tags.join(", "),
        )));
        lines.push(Text::literal(label_value("notes", &s.metadata.notes)));
        lines.push(Text::literal(tr("auth_note")));
        let wrapped = lines
            .iter()
            .flat_map(|line| atlas.font.split(line, w - 27.0))
            .collect::<Vec<_>>();
        let (top, bottom) = (y + 9.0, y + h - 44.0);
        r.detail_height = wrapped.len() as f32 * 12.0;
        r.detail_scroll.clamp(top, bottom, r.detail_height);
        let clip = batch.vertices.len();
        for (i, line) in wrapped.iter().enumerate() {
            let ly = top + i as f32 * 12.0 - r.detail_scroll.amount;
            if ly + 10.0 >= top && ly <= bottom {
                batch.text_styled(atlas, line, x + 9.0, ly, INK, true);
            }
        }
        batch.clip_since(clip, x + 8.0, top, x + w - 12.0, bottom);
        crate::ui::scrollbar(
            batch,
            atlas,
            x + w - 8.0,
            top,
            bottom,
            r.detail_height,
            r.detail_scroll.amount,
        );
        widgets::record_as(&tr("details"), rect, true, Kind::Row);
        widgets::target_as(rect, Kind::Row);
        if let Some(history) = r.history.get(&s.address) {
            let max = history.iter().copied().max().unwrap_or(1).max(1) as f32;
            let bw = (w - 20.0) / 32.0;
            for (i, &ms) in history.iter().enumerate() {
                let bar = 20.0 * (ms as f32 / max);
                let bar_color = match ms {
                    0..=80 => rgba(85, 255, 85, 230),
                    81..=160 => rgba(170, 255, 85, 230),
                    161..=280 => rgba(255, 170, 0, 230),
                    _ => rgba(255, 85, 85, 230),
                };
                batch.fill(
                    atlas,
                    x + 10.0 + i as f32 * bw,
                    y + h - 10.0 - bar,
                    bw.max(1.0),
                    bar,
                    bar_color,
                );
            }
            text(
                batch,
                atlas,
                &tr("latency_history"),
                x + 9.0,
                y + h - 39.0,
                w - 18.0,
                SLATE_MUTED,
            );
        }
    }
    pub fn tooltip(&self, atlas: &HudAtlas, batch: &mut HudBatch, cursor: (f32, f32)) {
        if let Some(id) = self.button_focus
            && let Some((x, y, w, _, _, _)) = self
                .buttons((batch.width.ceil(), batch.height.ceil()))
                .into_iter()
                .find(|b| b.5 == id)
        {
            batch.fill(atlas, x, y, w, 1.0, INK);
            batch.fill(atlas, x, y + 19.0, w, 1.0, INK);
        }
        if !matches!(self.modal, Modal::Main) {
            return;
        }
        let r = self.runtime.borrow();
        let (left, top, bottom, width) = Self::list_area(r.size, self.state.view.tab);
        if !inside(cursor, [left, top, width, bottom - top]) {
            return;
        }
        let row = ((cursor.1 - top + r.smooth) / ROW).floor().max(0.0) as usize;
        if let Some(&i) = r.visible.get(row) {
            let s = &r.source[i];
            let status = r.statuses.get(&s.address);
            let value = if cursor.0 > left + width - 120.0 && cursor.0 < left + width - 32.0 {
                status.map(|s| s.samples.join("\n")).unwrap_or_default()
            } else {
                format!(
                    "{}\n{}\n{}",
                    health(status),
                    auth_name(s.metadata.auth),
                    tr("auth_note")
                )
            };
            if !value.is_empty() {
                let max_lines = ((batch.height - 20.0) / 10.0).floor().max(1.0) as usize;
                let lines = value
                    .lines()
                    .flat_map(|s| {
                        atlas
                            .font
                            .split(&Text::literal(s), (batch.width - 30.0).min(300.0))
                    })
                    .take(max_lines)
                    .collect::<Vec<_>>();
                crate::ui::tooltip(batch, atlas, &lines, cursor);
            }
        }
    }
}
