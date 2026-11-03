# Native Server Browser 2.0.0 validation

Validated locally on Windows x86-64, 2 November 2026. This is a source native mod
for the official launcher, not a WASM file and not a separately patched host.

## Reproducible inputs

- Minecraft resources: the player's prepared 26.3 resources (never packaged).
- Official Horizon: `142388e1c136e622652ec7505af15d8ad195e041`.
- Official horizon-mixins: `a2af5c0dd4117d51ecd70066ba7a00b4a2c10dab`.
- Rust 1.99.1, Windows GNU toolchain; Cargo `--locked --profile fast`.
- Package SHA-256: `d83bf70dbd650bfa25156dcf93f1a279c17faa549a5dddcb074ae31b51b11faa`.
- Package size: 85,202 bytes. The final package differs from the compiled/tested
  candidate only by the added English/German Cancel translation. An automated
  comparison proves all code outside those two language constants is identical;
  both JSON dictionaries and all literal UI translation keys were checked.

## Completed checks

- Official `cargo-horizon-mod check server-browser-2.0.0.hnmod --json`: `ok:true`,
  `kind:native-mod`, id `server_browser`, version `2.0.0`, game `26.3`.
  `code:false` is this checker's normal report for source native bundles; the
  complete Rust implementation is in `mixins/server_browser/runtime.rs`.
- Official mixin application: 25 injections, 2 modules; no dependencies, crates,
  data overlays, bridge, post-build commands or external source references.
- Source contract audit: `Cargo.toml`, `Cargo.lock`, the WIT API, WASM manifest
  parser and WASM runtime are byte-identical to official Horizon. Integration is
  entirely described by the native bundle's official mixins.
- `cargo check --locked --profile fast -p horizon-client --features mods --bins`.
- `cargo clippy --locked --profile fast -p horizon-client --features mods --lib
  -- -D warnings`.
- Bin Clippy passed with `-D warnings -A clippy::collapsible_if`. Official pinned
  Horizon has two unrelated `collapsible_if` findings in `game.rs` (carried-item
  drop and shutdown). They have not been changed in the mod.
- Native client and `horizon-ctl` built with default audio/controller support
  and the `mods` feature.
- All 40 browser Rust tests passed, including the cancellation regression:
  persistence/unknown NBT and JSON fields, backups and external-edit conflicts,
  10,000 entries, all filters/sorts, bounded status workers/timeouts/cache,
  directory defaults/throttling/refresh/backoff, forms, undo, presets, keyboard,
  layout, the 488 unique catalog addresses and offline mail transport fixtures.

## Real GUI and network checks

`verify_native.py` launches the compiled native game with independent scratch
homes and drives real rendered controls through `horizon-ctl`. All passed:

- 10,000 saved servers, virtualized rows, search/Ctrl+F, add/edit fields,
  favorites, filters, delete/undo, reorder, presets, controller tabs,
  Classic/modern switching.
- Options > Mods on/off; disabled mod uses Classic; unedited `servers.dat` is
  byte-identical. The original NBT backup also remains identical after edits.
- 18 screenshots: 854x480, 1920x1080 and 3840x2160, scales 1/2/3/4/auto and
  German. Representative screenshots inspected visually.
- Centered translucent rows and bounded header/footer over the existing
  panorama; custom tab/sort buttons; Load directory and Save selected absent.
- The public API was exercised from a fresh real game home: default-on Discover
  loaded, parsed, cached and displayed 50 real HTTPS results. Explicit Off
  persisted; servers.dat remained byte-identical.
- A public server proposal draft was saved in the real UI without starting a
  mail request. No recipient address appeared in the game UI.
- An actual isolated `horizon-client --dedicated`: status/compatible-online
  filters, Join and Direct Connect. The player explicitly accepted the EULA
  for that scratch server. No real account or public server was used.

Detailed results: `ui-design-mail/report.json`,
`discovery-design-mail/report.json`, `design-mail-unit-tests.log`,
`design-mail-clippy.log`, `design-mail-build.log`, `translation-final-audit.json`
and `final-translation-apply.log`. Logs and screenshots remain under
`horizon-server-browser/.local/native-browser/`; these contain no distribution
assets and are not inside the `.hnmod`.

## Scope and installation

The standard launcher's native build remains a player action. Installing a native
source mod does not hot-load it into an already running stock client. The player
reviews and confirms the ordinary **I trust these mods: build** action once for
the chosen set of mods. No custom launcher, companion executable or host patch
needs to be delivered. The package is pinned; future incompatible Horizon
changes require a new tested mod release. macOS/Linux and the controller's
on-screen keyboard have not been verified in this run. This is not a claim
that the entire upstream Horizon test/parity suite was run.

## Mail-service blocker

The user authorized free Static Forms signup for their recipient with the EU
storage option, marketing off, and acceptance of its terms. The registration
attempt stopped at Cloudflare. The user also could not complete that check.
There is no verified account or public submission key, and no successful email
delivery test. The bundled key is empty: automatic sending remains explicitly
unconfigured. The implemented sender and its failure/cooldown handling were
tested with offline transports only. No mail password or management token is
included. The service's free shared quota is 250 submissions per 30 days;
acceptance by its API alone does not prove Outlook delivery.
