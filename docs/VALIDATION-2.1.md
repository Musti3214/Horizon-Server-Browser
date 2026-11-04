# Native Server Browser 2.1.0 validation

Validated locally on Windows x86-64, 3 November 2026. This release uses the
official native source-mod mechanism; it needs no separately installed host patch.
Historical 2.0 results are retained in `VALIDATION-2.0.md`.

## Reproducible inputs

- Official Horizon: `142388e1c136e622652ec7505af15d8ad195e041`.
- Official horizon-mixins: `a2af5c0dd4117d51ecd70066ba7a00b4a2c10dab`.
- Rust 1.99.1, Windows GNU; Cargo locked/offline, fast profile, default features
  plus mods. Minecraft resources: the player's prepared 26.3, never distributed.
- Package `server-browser-2.1.0.hnmod`: 162,503 bytes; SHA-256
  `bb6bb8e5e89299a0d853ce85db520205564da8b77576b61fbf6054eb200bfd58`.
- All 29 source files in the package match the release source exactly. All 51
  Cargo manifests/locks and WIT contracts remain identical to official Horizon.

## Completed checks

- Official mixin application: 25 injections, 2 modules, no added dependencies,
  crates, bridge, data overlay or post-build command.
- All 44 browser Rust tests passed (0 failed): persistence/unknown fields,
  conflicts/backups, 10,000 entries, filtering/sorting/undo/presets, input/layout,
  bounded direct pings, directory consent/cache/rates and partial failure,
  Modrinth Java parsing/bounds/normalization, deduplication, exhausted sources,
  2,835 unique catalog addresses and local mail transport fixtures.
- Library Clippy with `-D warnings` passed.
- Native client and horizon-ctl built successfully in 9m03s. This was an isolated
  developer build; the player's native-code trust/build decision was not accepted.

## Actual rendered game checks

Every run used a fresh disposable home, no account, disabled telemetry/Discord
and no access to player worlds/settings/server files.

- `verify_native.py --quick` passed: 10,000 saved servers with virtualized rows,
  search/Ctrl+F, add/edit, favorites, filters, remove/undo, reorder, presets,
  controller tabs, Classic/Minecraft switching and native Mods on/off. Original
  NBT backup and unedited server files remain byte-identical.
- Local server proposal draft saved without a mail request; recipient absent
  from game UI. Automatic mail delivery remains unconfigured.
- `verify_minecraft_style.py` passed: real textured Minecraft buttons, German
  UI at 1920x1080, GUI scales 1/2/3/4, settings and value-filter dialogs. Explicit
  discovery Off remains Off and servers.dat remains unchanged. Screenshots inspected.
- `verify_discovery.py --allow-public-directory` passed: actual HTTPS responses
  from both minecraftserve.rs (50 parsed entries) and Modrinth (49 valid Java
  entries on page 1), cached/displayed in the game. No remote ping metadata is
  presented as a direct ping. Default-on discovery, persistent explicit Off,
  absent Load directory/Save selected and unchanged servers.dat verified.
- Twelve new German function screenshots were captured by
  `sites/server-browser-web/tools/capture_gallery.py` with local example servers;
  a thirteenth is the real public-directory test. These are unedited game frames.

Logs, reports and screenshots: ignored local folder
`horizon-server-browser/.local/native-browser/minecraft-20261103/`.
The broader 2.0 dedicated-server/4K matrix was not rerun for this presentation
and directory-source update; its historical results remain in VALIDATION-2.0.md.

## Delivery

The identical checksummed package is on the Desktop, in the prepared independent
`Server Browser 2.1` instance and in the website download directory. Existing
profiles/mods were preserved and the selected default profile was not changed.
To play 2.1, the player must build that instance's mod set in the official launcher
and then launch it. An already built 2.0 client cannot hot-load the new sources.

Fabric/Forge ports are planned and are not included. Mail-service activation
remains pending. This package contains only original Rust integration sources,
configuration and translations: no game resources, player data, external mod
implementation, binaries or private mail credentials.
