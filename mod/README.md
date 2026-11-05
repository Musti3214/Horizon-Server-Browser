# Server Atlas 2.2.1 — native Horizon edition

A native Rust multiplayer browser for Minecraft Java 26.3, using Horizon's official `.hnmod` mechanism. This update targets Horizon Nightly 2026-11-04 · 6475f62 and addresses the registry review of 2.2.0.

## Install

1. Download `server-browser-2.2.1.hnmod` and choose a dedicated Horizon instance.
2. Put this single file in the instance's `horizon-native-mods` folder. Keep a backup of an older version outside that folder and leave unrelated mods alone.
3. Enable server_browser on the launcher's Mods page. Choose the ordinary native Build action, review the source/mod list and make the native-code trust decision yourself.
4. Launch the instance after its build finishes and open Multiplayer.

Native mods are compiled into the game and run with your user permissions. Updating the file requires rebuilding; a previously built client cannot hot-load new Rust code. Other native mods and future Nightlies may need separate compatibility checks. No separately installed host patch, custom launcher or companion executable is required.

## Features and local data

Minecraft-style buttons, compact translucent server rows, search, My Servers/Discover/Favorites/Recent/LAN views, status/version/ping/player filters, local tags/notes/groups, saved filter profiles, custom ordering, JSON/NBT import/export and Classic view. The persistent mod ID remains server_browser and unknown vanilla servers.dat fields are preserved.

Online discovery uses Modrinth and minecraftserve.rs after you confirm the privacy screen. Users upgrading from 2.2.0 also confirm once. Requests are throttled per provider and cached. Failed refreshes preserve usable results. Directory requests include public tag and version filters where supported. Personal saved addresses, searches and notes stay local.

Discover sends direct status pings to visible rows and a small margin. Scrolling replaces queued probes with the new window. Automatic refresh checks that same window or the saved list, depending on the active tab. Discover excludes listings marked for offline access. Account mode remains a private note on saved servers. Connections and login use Horizon's implementation.

Feedback and public server suggestions save local drafts. The **☕ Ko-fi** toolbar button opens `https://ko-fi.com/foryou41` in your default browser.

## Build and review

With Python and Rust 1.99.1's rustfmt on PATH, run `python build_bundle.py`. The deterministic package contains horizon-native.json, the actual mixin manifest, complete Rust runtime/hooks and MIT license. It contains no game assets, player accounts, executable attachments or Java loader JARs.

`src/core/` handles persistence, queries and bounded networking; `src/ui/` contains the interface and dialogs; `native.rs` adapts Horizon menus and input. The generator builds the official mixin bundle in `mixins/server_browser/`. stage.ps1 applies it to a supplied pristine Horizon source tree in a separate managed work folder. All upstream Cargo manifests, locks and WIT files remain unchanged.

Meaningful Rust tests and isolated game fixtures are documented in VALIDATION.md. The verify_*.py scripts need a separately built pinned client, horizon-ctl and your own prepared Minecraft 26.3 resources. The dedicated-server fixture additionally requires explicit Minecraft EULA acceptance.

Separate Fabric and Forge public beta editions are in the repository's java/ directory; their JARs and validation are independent of this native bundle. A successful local build or GitHub release does not automatically publish a Horizon registry listing. Every native registry version requires human review.

License: MIT. Independent community project by shuab, developed with Codex assistance. Not an official Minecraft product or associated with Mojang or Microsoft.
