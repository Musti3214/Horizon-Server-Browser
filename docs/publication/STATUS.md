# Server Atlas publication and review status

Updated 5 November 2026 (Europe/Berlin).

## Horizon native edition 2.2.0

The [native release](https://github.com/Musti3214/Horizon-Server-Browser/releases/tag/v2.2.0) is public. Its exact file is `server-browser-2.2.0.hnmod`, 163518 bytes, SHA-256 `e65411ec3ed57c2ef288b627d0337cb1592762c24a869024a301b4c57bdfa94e`.

It is pinned to Horizon Nightly 2026-11-04 · 6475f62, full revision `6475f62f3c834b7489e1816bf47f47ea5eb2b879`. The official package check, native build, 44 Rust tests and isolated game checks pass. The earlier library migration conflict is resolved. [Validation and limits](../../mod/VALIDATION.md).

Install through the normal native mod workflow and rebuild the selected Horizon instance after changing the bundle. The game does not hot-load native source bundles. No separate host patch or companion executable is required.

## Websites and other editions

- [Primary website](https://musti3214.github.io/Horizon-Server-Browser/) — generated and validated by this repository's GitHub Pages workflow.
- [Published mirror](https://server-browser-mod.indiebini.chatgpt.site/) — version 6 includes 13 current real game captures and verified native/Fabric/Forge artifact checksums, in English, German, French and Spanish.
- [Fabric and Forge public betas](https://github.com/Musti3214/Horizon-Server-Browser/releases/tag/v2.2.0-beta.1) — separate Java files for Minecraft 26.3 / Java 25; each loader passed 96 integration checks. See [Java runtime validation](../../java/VALIDATION.md) for coverage and remaining limits.

## Official catalogs and community posts

**No official Horizon registry PR, approval or listing is confirmed.** The [public review request](https://github.com/Musti3214/Horizon-Server-Browser/issues/1) was updated to 2.2.0 and the current [review kit](../horizon-registry/) was sent in the existing conversation with Horizon's published maintainer contact. The official form remains the intended submission route; it could not be completed because Chrome automation is unavailable. Every native version requires human source review.

Modrinth and CurseForge projects have not been created or submitted by this session. No Reddit posts have been published. Prepared descriptions and actual game captures are available for those submissions. A GitHub release or a local launcher build does not create a marketplace listing.

The project was developed and publication materials were prepared with Codex assistance at the owner's request. Feedback currently saves local drafts; automatic email delivery is unconfigured.
