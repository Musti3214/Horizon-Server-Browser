# Server Atlas 2.2.0 — native Horizon edition

A native Rust multiplayer browser for Minecraft Java 26.3, using Horizon's official `.hnmod` mechanism. Version 2.2.0 is updated and tested on Horizon Nightly 2026-11-04 · 6475f62. Exact pins, checksums and test coverage are in VALIDATION.md.

## Install

1. Download `server-browser-2.2.0.hnmod` and choose a dedicated Horizon instance.
2. Put this single file in the instance's `horizon-native-mods` folder. Keep a backup of an older version outside that folder and leave unrelated mods alone.
3. Enable server_browser on the launcher's Mods page. Choose the ordinary native Build action, review the source/mod list and make the native-code trust decision yourself.
4. Launch the instance after its build finishes and open Multiplayer.

Native mods are compiled into the game and run with your user permissions. Updating the file requires rebuilding; a previously built client cannot hot-load new Rust code. Other native mods and future Nightlies may need separate compatibility checks. No separately installed host patch, custom launcher or companion executable is required.

## Features and local data

Minecraft-style buttons, compact translucent server rows, search, My Servers/Discover/Favorites/Recent/LAN views, status/version/ping/player filters, local tags/notes/groups, saved filter profiles, custom ordering, JSON/NBT import/export and Classic view. The persistent mod ID remains server_browser and unknown vanilla servers.dat fields are preserved.

Public discovery uses Modrinth and minecraftserve.rs and includes a dated snapshot of 2,835 unique addresses. New settings enable it; an explicitly saved Off stays Off. Listing requests are independently throttled and cached; failed refreshes preserve usable prior data. Personal saved addresses, searches and notes are not sent to directory providers. Direct status requests contact the servers being checked. Connections and login use Horizon's implementation.

Feedback and public suggestions save local drafts. Automatic email delivery is not configured. Never embed a mail password or private management token. feedback-public.json supports a service's public submission key only.

## Build and review

With Python and Rust 1.99.1's rustfmt on PATH, run `python build_bundle.py`. The deterministic package contains horizon-native.json, the actual mixin manifest, complete Rust runtime/hooks and MIT license. It contains no game assets, player accounts, executable attachments or Java loader JARs.

`src/core/` handles persistence, queries and bounded networking; `src/ui/` contains the interface and dialogs; `native.rs` adapts Horizon menus and input. The generator builds the official mixin bundle in `mixins/server_browser/`. stage.ps1 applies it to a supplied pristine Horizon source tree in a separate managed work folder. All upstream Cargo manifests, locks and WIT files remain unchanged.

Meaningful Rust tests and isolated game fixtures are documented in VALIDATION.md. The verify_*.py scripts need a separately built pinned client, horizon-ctl and your own prepared Minecraft 26.3 resources. The dedicated-server fixture additionally requires explicit Minecraft EULA acceptance.

Separate Fabric and Forge public beta editions are in the repository's java/ directory; their JARs and validation are independent of this native bundle. A successful local build or GitHub release does not automatically publish a Horizon registry listing. Every native registry version requires human review.

License: MIT. Independent community project by shuab, developed with Codex assistance. Not an official Minecraft product or associated with Mojang or Microsoft.
