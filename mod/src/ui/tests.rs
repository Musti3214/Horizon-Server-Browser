use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
fn browser() -> (Browser, PathBuf) {
    let home = std::env::temp_dir().join(format!(
        "horizon-browser-ui-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(home.join("saves")).unwrap();
    let mut browser = Browser::new(&home.join("saves")).unwrap();
    // These tests use offline/injected replies, never the public directory.
    browser.state.settings.online_directory = false;
    (browser, home)
}

#[test]
fn discovered_server_uses_add_form_without_an_extra_save_button() {
    let (mut b, home) = browser();
    b.state.view.tab = Tab::Discover;
    let server = Server {
        name: "Discovery fixture".into(),
        address: "127.0.0.1:65000".into(),
        ..Server::default()
    };
    b.state.view.selected = Some(server.address.clone());
    b.runtime.get_mut().source = vec![server];
    b.runtime.get_mut().visible = vec![0];
    b.activate(42);
    assert!(
        matches!(&b.modal,Modal::Edit {server,..} if server.name=="Discovery fixture" && server.address=="127.0.0.1:65000")
    );
    b.activate(50);
    assert_eq!(b.catalog.servers[0].name, "Discovery fixture");
    drop(b);
    std::fs::remove_dir_all(home).unwrap();
}

fn populate(b: &mut Browser) {
    b.catalog.servers = (0..3)
        .map(|i| Server {
            name: format!("Local {i}"),
            address: format!("127.0.0.1:{}", 30000 + i),
            ..Server::default()
        })
        .collect();
    b.changed();
    b.sync(&mut b.runtime.borrow_mut());
}
#[test]
fn add_edit_duplicate_remove_and_undo_are_transactional() {
    let (mut b, home) = browser();
    b.activate(42);
    if let Modal::Edit { server, tags, .. } = &mut b.modal {
        server.name = "Home".into();
        server.address = "localhost:65530".into();
        *tags = "Friends, friends, EU".into();
    }
    b.activate(50);
    assert_eq!(b.catalog.servers.len(), 1);
    assert_eq!(b.catalog.servers[0].metadata.tags, ["Friends", "EU"]);
    b.activate(42);
    if let Modal::Edit { server, .. } = &mut b.modal {
        server.name = "Duplicate".into();
        server.address = "LOCALHOST:65530".into();
    }
    b.activate(50);
    assert!(b.error.is_some());
    assert_eq!(b.catalog.servers.len(), 1);
    b.main();
    b.state.view.selected = Some("localhost:65530".into());
    b.selection.insert("localhost:65530".into());
    b.activate(43);
    if let Modal::Edit { server, .. } = &mut b.modal {
        server.metadata.notes = "local note".into();
    }
    b.activate(50);
    assert_eq!(b.catalog.servers[0].metadata.notes, "local note");
    b.activate(44);
    assert_eq!(b.catalog.servers.len(), 1);
    b.activate(51);
    assert_eq!(b.catalog.servers.len(), 1);
    b.activate(44);
    b.activate(50);
    assert!(b.catalog.servers.is_empty());
    b.activate(28);
    assert_eq!(b.catalog.servers.len(), 1);
    drop(b);
    assert_eq!(
        horizon_protocol::servers::load(&home.join("servers.dat")).len(),
        1
    );
    std::fs::remove_dir_all(home).unwrap();
}
#[test]
fn keyboard_multiselect_favorites_presets_and_reorder() {
    let (mut b, home) = browser();
    populate(&mut b);
    b.focus = None;
    b.key(KeyCode::ArrowDown, false, false);
    assert_eq!(b.state.view.selected.as_deref(), Some("127.0.0.1:30000"));
    b.key(KeyCode::ArrowDown, false, true);
    assert_eq!(b.selection.len(), 2);
    b.favorite(None);
    assert!(b.catalog.servers[0].metadata.favorite && b.catalog.servers[1].metadata.favorite);
    b.key(KeyCode::ArrowUp, true, false);
    assert_eq!(b.catalog.servers[0].address, "127.0.0.1:30001");
    b.state.view.search = "Friends".into();
    b.show(Modal::Presets("Home".into()));
    b.activate(50);
    assert!(b.state.presets.contains_key("Home"));
    b.state.view.search.clear();
    b.show(Modal::Presets(String::new()));
    b.activate(120);
    assert_eq!(b.state.view.search, "Friends");
    drop(b);
    std::fs::remove_dir_all(home).unwrap();
}
#[test]
fn layout_stays_inside_minimum_and_4k_gui_sizes() {
    let (mut b, home) = browser();
    b.state.view.filters.max_ping = Some(100);
    b.state.view.filters.favorites = true;
    b.state.view.filters.not_full = true;
    for (w, h) in [
        (320.0, 240.0),
        (427.0, 240.0),
        (854.0, 480.0),
        (640.0, 360.0),
        (1920.0, 1080.0),
        (3840.0, 2160.0),
    ] {
        for modal in [
            Modal::Main,
            Modal::Settings,
            Modal::Filters(0),
            Modal::Filters(1),
            Modal::Sort,
            Modal::Edit {
                index: None,
                server: Box::default(),
                tags: String::new(),
                page: 0,
            },
            Modal::Feedback {
                category: 0,
                text: String::new(),
            },
        ] {
            b.modal = modal;
            let buttons = b.buttons((w, h));
            for (x, y, bw, _, _, _) in &buttons {
                assert!(
                    *x >= 0.0 && *y >= 0.0 && x + bw <= w && y + 20.0 <= h,
                    "{w}x{h}: {x},{y},{bw}"
                );
            }
            for (_, _, [x, y, fw, fh]) in b.form_fields((w, h)) {
                assert!(x >= 0.0 && y >= 0.0 && x + fw <= w && y + fh <= h);
            }
            for (i, a) in buttons.iter().enumerate() {
                for b in &buttons[i + 1..] {
                    assert!(
                        !(a.0 < b.0 + b.2
                            && a.0 + a.2 > b.0
                            && a.1 < b.1 + 20.0
                            && a.1 + 20.0 > b.1),
                        "overlapping buttons {:?} {:?}",
                        a,
                        b
                    );
                }
            }
        }
    }
    drop(b);
    std::fs::remove_dir_all(home).unwrap();
}
#[test]
fn classic_switch_settings_and_load_failure_keep_a_vanilla_route() {
    let (mut b, home) = browser();
    assert!(matches!(b.activate(24), Outcome::Classic));
    drop(b);
    assert!(
        persistence::load_state(&home)
            .unwrap()
            .settings
            .classic_menu
    );
    assert!(native::open(&home.join("saves"), false).is_none());
    std::fs::write(persistence::state_path(&home), b"garbage").unwrap();
    assert!(native::open(&home.join("saves"), false).is_none());
    std::fs::remove_dir_all(home).unwrap();
}

#[test]
fn refresh_during_edit_cannot_overwrite_a_different_external_row() {
    let (mut b, home) = browser();
    populate(&mut b);
    b.catalog.save(&home).unwrap();
    b.select("127.0.0.1:30000".into());
    b.activate(43);
    let mut external = Catalog::load(&home, &Default::default()).unwrap();
    external.servers.remove(0);
    external.save(&home).unwrap();
    b.refresh();
    if let Modal::Edit { server, .. } = &mut b.modal {
        server.name = "Pending edit".into();
    }
    b.activate(50);
    assert!(b.error.is_some());
    let actual = Catalog::load(&home, &Default::default()).unwrap();
    assert_eq!(actual.servers.len(), 2);
    assert_eq!(actual.servers[0].address, "127.0.0.1:30001");
    assert_eq!(actual.servers[1].address, "127.0.0.1:30002");
    drop(b);
    std::fs::remove_dir_all(home).unwrap();
}

#[test]
fn cancel_filters_and_undo_preserve_unrelated_edits() {
    let (mut b, home) = browser();
    populate(&mut b);
    // Open filters via its registered action, then cancel a live toggle.
    b.activate(20);
    if matches!(b.modal, Modal::Filters(_)) {
        b.activate(70);
        assert_eq!(b.state.view.filters.online, Some(true));
        b.key(KeyCode::KeyF, true, false);
        assert_eq!(b.state.view.filters.online, None);
    } else {
        panic!("filter action must open the panel");
    }
    b.select("127.0.0.1:30000".into());
    b.state.view.filters.max_ping = Some(100);
    b.state.view.filters.tags = vec!["friends".into()];
    b.activate(133);
    assert!(b.state.view.filters.max_ping.is_none());
    assert_eq!(b.state.view.filters.tags, ["friends"]);
    b.state.view.filters.tags.clear();
    b.activate(44);
    b.activate(50);
    b.catalog.servers[0].metadata.notes = "edit after removal".into();
    b.activate(28);
    assert_eq!(b.catalog.servers.len(), 3);
    assert_eq!(b.catalog.servers[1].metadata.notes, "edit after removal");
    let saved = std::fs::read(home.join("servers.dat")).unwrap();
    b.show(Modal::Transfer(
        home.join("servers.dat").to_string_lossy().into(),
    ));
    b.activate(55);
    assert!(b.error.is_some());
    assert_eq!(std::fs::read(home.join("servers.dat")).unwrap(), saved);
    drop(b);
    std::fs::remove_dir_all(home).unwrap();
}

#[test]
fn support_prompt_dismissal_persists_and_settings_remains_available() {
    let (mut b, home) = browser();
    assert!(!b.state.settings.support_dismissed);
    let buttons = b.buttons((854.0, 480.0));
    assert!(buttons.iter().any(|(_, _, _, _, _, id)| *id == 62));
    assert!(buttons.iter().any(|(_, _, _, _, _, id)| *id == 63));
    assert!(buttons.iter().any(|(_, _, _, _, _, id)| *id == 64));

    b.activate(63);
    assert!(b.state.settings.support_dismissed);
    let buttons = b.buttons((854.0, 480.0));
    assert!(!buttons.iter().any(|(_, _, _, _, _, id)| *id == 62));
    assert!(!buttons.iter().any(|(_, _, _, _, _, id)| *id == 63));
    assert!(buttons.iter().any(|(_, _, _, _, _, id)| *id == 64));

    drop(b);
    let (mut b2, _) = (Browser::new(&home.join("saves")).unwrap(), &home);
    assert!(b2.state.settings.support_dismissed);

    b2.show(Modal::Settings);
    let settings_buttons = b2.buttons((854.0, 480.0));
    assert!(settings_buttons.iter().any(|(_, _, _, _, _, id)| *id == 90));
    assert!(settings_buttons.iter().any(|(_, _, _, _, _, id)| *id == 91));

    b2.activate(85);
    assert!(!b2.state.settings.support_dismissed);
    drop(b2);
    std::fs::remove_dir_all(home).unwrap();
}

#[test]
fn feedback_modal_category_save_and_clipboard() {
    let (mut b, home) = browser();
    b.activate(64);
    assert!(matches!(b.modal, Modal::Feedback { category: 0, .. }));

    b.activate(161);
    assert!(matches!(b.modal, Modal::Feedback { category: 1, .. }));

    b.activate(162);
    assert!(matches!(b.modal, Modal::Feedback { category: 2, .. }));

    b.activate(96);
    assert!(b.error.is_some());
    assert!(matches!(b.modal, Modal::Feedback { .. }));

    if let Modal::Feedback { text, .. } = &mut b.modal {
        *text = "play.mycraft.de | MyCraft | Survival 1.20".to_string();
    }
    b.activate(96);
    assert!(b.notice.is_some());
    assert!(matches!(b.modal, Modal::Main));

    let log_file = home.join("feedback.log");
    assert!(log_file.exists());
    let content = std::fs::read_to_string(&log_file).unwrap();
    assert!(content.contains("play.mycraft.de"));

    let server_log = home.join("server_submissions.log");
    assert!(server_log.exists());
    let server_content = std::fs::read_to_string(&server_log).unwrap();
    assert!(server_content.contains("play.mycraft.de"));

    drop(b);
    std::fs::remove_dir_all(home).unwrap();
}

#[test]
fn double_click_to_join_and_keyboard_jump() {
    let (mut b, home) = browser();
    populate(&mut b);
    let r = b.runtime.borrow();
    let addr = r.source[0].address.clone();
    drop(r);

    let (left, top, _, _) = Browser::list_area((854.0, 480.0), Tab::Saved);
    let outcome = b.click(None, (left + 50.0, top + 12.0), (854.0, 480.0));
    assert!(matches!(outcome, Outcome::None));
    assert_eq!(b.state.view.selected.as_deref(), Some(addr.as_str()));

    let outcome = b.click(None, (left + 50.0, top + 12.0), (854.0, 480.0));
    assert!(matches!(outcome, Outcome::Connect(a) if a == addr));

    b.key(KeyCode::End, false, false);
    assert_eq!(b.state.view.selected.as_deref(), Some("127.0.0.1:30002"));
    b.key(KeyCode::Home, false, false);
    assert_eq!(b.state.view.selected.as_deref(), Some("127.0.0.1:30000"));

    b.focus = Some(0);
    assert!(!b.escape());
    assert_eq!(b.focus, None);
    assert!(b.escape());

    drop(b);
    std::fs::remove_dir_all(home).unwrap();
}

#[test]
fn discover_infinite_scroll_and_curated_catalog() {
    let (mut b, home) = browser();
    b.state.settings.online_directory = true;
    b.state.view.tab = Tab::Discover;

    let buttons = b.buttons((854.0, 480.0));
    assert!(!buttons.iter().any(|(_, _, _, _, _, id)| *id == 35));
    assert!(!buttons.iter().any(|(_, _, _, _, _, id)| *id == 38));
    assert!(!buttons.iter().any(|(_, _, _, _, _, id)| *id == 36));
    assert!(!buttons.iter().any(|(_, _, _, _, _, id)| *id == 37));

    let mut dir = online::Directory::default();
    let p1 = horizon_server_browser::directory::Page {
        servers: (0..10)
            .map(|i| Server {
                name: format!("S{i}"),
                address: format!("s{i}.test:25565"),
                ..Server::default()
            })
            .collect(),
        page: 1,
        pages: 3,
        total: 30,
        ..Default::default()
    };
    dir.page = Some(p1);
    assert_eq!(dir.page.as_ref().unwrap().servers.len(), 10);
    assert!(dir.can_request_next());

    let p2 = horizon_server_browser::directory::Page {
        servers: (10..20)
            .map(|i| Server {
                name: format!("S{i}"),
                address: format!("s{i}.test:25565"),
                ..Server::default()
            })
            .collect(),
        page: 2,
        pages: 3,
        total: 30,
        ..Default::default()
    };
    *dir.reply.lock().unwrap() = Some(Ok(p2));
    assert!(dir.poll());
    assert_eq!(dir.page.as_ref().unwrap().servers.len(), 20);
    assert_eq!(dir.page.as_ref().unwrap().page, 2);

    drop(b);
    std::fs::remove_dir_all(home).unwrap();
}

#[test]
fn directory_throttling_cannot_skip_remote_pages_or_replay_cancelled_results() {
    let (_, home) = browser();
    let server = Server {
        name: "Public".into(),
        address: "public.test".into(),
        ..Default::default()
    };
    let remote = horizon_server_browser::directory::Page {
        servers: vec![server.clone()],
        page: 1,
        pages: 3,
        total: 101,
        ..Default::default()
    };
    let combined = online::merge_directory(Ok(remote), vec![server.clone()], 1).unwrap();
    assert_eq!(combined.servers.len(), 1);
    assert_eq!(combined.pages, 3);
    assert!(
        online::merge_directory(Err(core::Error::RateLimited(60)), vec![server.clone()], 2)
            .is_err()
    );
    let mut directory = online::Directory::default();
    directory.page = Some(combined);
    directory.requested_page = 2;
    *directory.reply.lock().unwrap() = Some(Err("wait".into()));
    directory.poll();
    assert_eq!(directory.page.as_ref().unwrap().page, 1);
    assert_eq!(directory.requested_page, 1);
    assert!(!directory.can_request_next());
    let stale_reply = directory.reply.clone();
    directory.cancel();
    *stale_reply.lock().unwrap() = Some(Ok(core::directory::Page::default()));
    assert!(!directory.poll());
    assert!(directory.page.is_none());
    std::fs::remove_dir_all(home).unwrap();
}

#[test]
fn completed_directory_reply_is_consumed_before_autoload() {
    let (mut b, home) = browser();
    b.state.settings.online_directory = true;
    b.state.view.tab = Tab::Discover;
    {
        let r = b.runtime.get_mut();
        r.directory.last_filter = Some((String::new(), String::new()));
        *r.directory.reply.lock().unwrap() = Some(Ok(core::directory::Page {
            servers: vec![Server {
                name: "Fixture".into(),
                address: "127.0.0.1:65530".into(),
                ..Default::default()
            }],
            page: 1,
            pages: 1,
            total: 1,
            ..Default::default()
        }));
    }
    b.sync(&mut b.runtime.borrow_mut());
    assert_eq!(b.runtime.borrow().source.len(), 1);
    assert!(!b.runtime.borrow().directory.loading());
    assert!(!home.join("server-directory-cache").exists());
    drop(b);
    std::fs::remove_dir_all(home).unwrap();
}

#[test]
fn more_menu_keeps_presets_and_file_transfers_reachable() {
    let (mut b, home) = browser();
    for (id, field) in [(33, "preset_name"), (34, "file_path")] {
        b.main();
        b.activate(21);
        assert!(
            b.buttons((427.0, 240.0))
                .iter()
                .any(|button| button.5 == id)
        );
        b.activate(id);
        assert_eq!(b.form_fields((427.0, 240.0))[0].1, tr(field));
    }
    drop(b);
    std::fs::remove_dir_all(home).unwrap();
}
