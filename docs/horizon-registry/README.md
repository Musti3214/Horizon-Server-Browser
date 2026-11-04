# Server Atlas 2.2.0 — Horizon registry review proposal

**Prepared for review; no registry submission or acceptance is claimed.**

The package was migrated and tested against the latest published Nightly checked on 4 November 2026: `6475f62f3c834b7489e1816bf47f47ea5eb2b879`. The previous 2.1.0 proposal's library-glob planning failure is resolved in this new version. Its published bytes remain unchanged.

Upload `server_browser-2.2.0.hnmod` and `icons/server_browser.png` at https://playhorizon.dev/mods/submit after GitHub sign-in. Use the project fields in `mods/server_browser.toml`. Keep owner **Musti3214**, ID **server_browser**, MIT, Minecraft **26.3**, and native-mod. The filename is a registry-friendly rename of the exact GitHub release file; SHA-256: `e65411ec3ed57c2ef288b627d0337cb1592762c24a869024a301b4c57bdfa94e`.

`NIGHTLY-VALIDATION.json` records the current source audit, official package check, 44 Rust tests and real isolated UI/discovery tests. No game assets, personal profiles, account files, mail credentials or executable attachments are included. The mod is unsigned; signing is optional and no signing identity is claimed.

For manual registry review, the exact bundle must first be uploaded to the registry's `staging` prerelease as `e65411ec3ed57c2ef288b627d0337cb1592762c24a869024a301b4c57bdfa94e.hnmod` by the submission service or an authorized maintainer. A TOML in this project repository alone does not create a registry PR or listing. Each native version requires human source review before the site and launcher list it.

GitHub release: https://github.com/Musti3214/Horizon-Server-Browser/releases/tag/v2.2.0
Source: https://github.com/Musti3214/Horizon-Server-Browser/tree/v2.2.0/mod
Actual screenshots: https://musti3214.github.io/Horizon-Server-Browser/screenshots/

The browser uses internal menu targets in addition to native integration hooks; the bundle remains pinned. Future Nightlies and combined native mod sets need separate validation. Native installation still uses the official launcher's ordinary compile/trust workflow. Feedback remains a local draft; automatic email is unconfigured.

An earlier `server-atlas-horizon-review-2.1.0.zip` is retained as a historical proposal, not the current download.
