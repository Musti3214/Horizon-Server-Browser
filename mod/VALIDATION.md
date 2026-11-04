# Server Atlas 2.2.0 — native Horizon validation

Validated on Windows x86-64 on 4 November 2026. This release targets the latest published Nightly checked at the start of the migration: **Nightly 2026-11-04 · 6475f62**, published at 16:29:39 UTC. Tags with several builds on one date must not be ordered lexicographically by hash.

## Inputs and package

- Horizon: `6475f62f3c834b7489e1816bf47f47ea5eb2b879`.
- horizon-mixins: `a2af5c0dd4117d51ecd70066ba7a00b4a2c10dab`.
- Rust 1.99.1, Windows GNU, locked Cargo dependencies, fast profile, default client features. The player's prepared Minecraft 26.3 resources were read only and are never distributed.
- `server-browser-2.2.0.hnmod`: **163,518 bytes**; SHA-256 **`e65411ec3ed57c2ef288b627d0337cb1592762c24a869024a301b4c57bdfa94e`**.
- 30 package entries, including the full MIT license. Every bundled source matches the release export. Official `cargo horizon-mod check` passes; the package is unsigned.
- 25 injections, two modules, no additional Cargo dependencies or host patch. All 50 upstream Cargo manifests, lockfiles and WIT contracts match the pristine source byte for byte.

## Migration and build

The reusable title and multiplayer UI now belong to `horizon-client-lib`. The runtime is inserted into client_lib.rs and the corresponding mixin targets use horizon_client_lib. Rendering hooks draw_with_items, the scroll hook returns bool, early keyboard handling returns Action, connection actions preserve the new LAN flag, and DNS results expose their addresses field. Keyboard events hook the specific Game window after splitscreen dispatch. Native game, horizon-ctl and the official package checker compiled successfully.

All **44 browser Rust tests passed, zero failed**: persistence/unknown NBT fields, backups/conflicts, 10,000 entries, filters/sorting/undo/profiles, input/layout, bounded status requests, directory consent/cache/rate limits/partial failure, Modrinth parsing/deduplication, 2,835 bundled addresses, local feedback fixtures and native lifecycle. The existing cancellation test also checks that queued connections preserve LAN classification. The newer upstream build emits Windows/C compiler and incremental-cache warnings; this release does not claim a clean Clippy run.

## Actual game checks

Each client and server used a fresh scratch home, no account, disabled telemetry/Discord/audio, and separate settings, worlds, clipboard and launcher paths.

- `verify_native.py --accept-eula`: real rendered workflows with 10,000 saved entries; search/Ctrl+F, add/edit, favorites, filters, remove/undo, ordering, saved profiles, controller tabs, Classic/Minecraft switching and Mods on/off. GUI matrix at 854×480, 1920×1080 and 3840×2160, scales 1/2/3/4/auto and German. A real isolated Horizon dedicated server verified online/compatible filters, Join, Direct Connect and return to the browser. Original NBT backups and unchanged server files remain byte-identical.
- `verify_minecraft_style.py`: German Minecraft buttons and dialogs, GUI scales, value filters and explicit discovery Off persistence.
- `verify_controls.py`: actual controller navigation to search, Horizon's on-screen keyboard, typing, backspace and close; saved server bytes unchanged. The test accounts for the current first controller focus independently of mouse focus.
- `verify_discovery.py --allow-public-directory`: live Modrinth and minecraftserve.rs listing pages parsed, cached and displayed; default-on discovery, persistent explicit Off, no separate Load directory/Save selected buttons, unchanged saved servers. Third-party metadata is not labeled as a fresh direct status ping.
- Twelve current-build function frames plus one real public discovery frame form the website gallery. Function captures use local example servers and notes, never private player data.

Local evidence is in the ignored `publish/native-update/` work folder. The public registry review kit contains a sanitized result summary. Historical 2.1.0 and 2.0 evidence is retained separately; their release files are unchanged.

## Installation and limits

Native mods still require the official launcher's normal build and the player's trust decision. Copying the .hnmod alone cannot update an already compiled client. The isolated developer test did not accept that decision on the player's behalf. Future Nightlies and combinations with other native mods need separate compatibility checks.

Saved data keeps the persistent server_browser ID and vanilla servers.dat format. No game assets, accounts, mail credentials or companion executable are included. Feedback saves local drafts; automatic email remains unconfigured. Separate Fabric and Forge 2.2.0 beta JARs retain their own validation in java/VALIDATION.md. Registry, Modrinth, CurseForge and Reddit publication are separate from a successful local build or GitHub release and must be reported from actual submission results.
