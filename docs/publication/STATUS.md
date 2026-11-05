# Server Atlas publication and review status

Updated 5 November 2026 (Europe/Berlin).

## Horizon review revision 2.2.1

The native package and public source address the [2.2.0 registry review](https://github.com/horizon-rust/horizon-mods/pull/248#issuecomment-6656913316). Discover probes the visible window plus a small margin. Offline-access search controls and flagged discovery listings were removed. Online discovery requires explicit consent, feedback stays local, and the MIT copyright holder is shuab.

The [2.2.1 native file](../../downloads/server-browser-2.2.1.hnmod) passed the official Horizon package check. See [validation](../../mod/VALIDATION-2.2.1.md) for the tested revision and checksum. Horizon's registry review is a separate step.

## Horizon native edition 2.2.0

The [native release](https://github.com/Musti3214/Horizon-Server-Browser/releases/tag/v2.2.0) is public. Its exact file is `server-browser-2.2.0.hnmod`, 163518 bytes, SHA-256 `e65411ec3ed57c2ef288b627d0337cb1592762c24a869024a301b4c57bdfa94e`.

It is pinned to Horizon Nightly 2026-11-04 · 6475f62, full revision `6475f62f3c834b7489e1816bf47f47ea5eb2b879`. The official package check, native build, 44 Rust tests and isolated game checks pass. The earlier library migration conflict is resolved. [Validation and limits](../../mod/VALIDATION.md).

Install through the normal native mod workflow and rebuild the selected Horizon instance after changing the bundle. The game does not hot-load native source bundles. No separate host patch or companion executable is required.

## Websites and other editions

- [Primary website](https://musti3214.github.io/Horizon-Server-Browser/) — generated and validated by this repository's GitHub Pages workflow.
- [Published mirror](https://server-browser-mod.indiebini.chatgpt.site/) — version 6 includes 13 current real game captures and verified native/Fabric/Forge artifact checksums, in English, German, French and Spanish.
- [Fabric and Forge public betas](https://github.com/Musti3214/Horizon-Server-Browser/releases/tag/v2.2.0-beta.1) — separate Java files for Minecraft 26.3 / Java 25; each loader passed 96 integration checks. See [Java runtime validation](../../java/VALIDATION.md) for coverage and remaining limits.

## Official catalogs and community posts

The first Horizon submission, [registry PR #248](https://github.com/horizon-rust/horizon-mods/pull/248), covers `server_browser` 2.2.0. Its [automated registry check](https://github.com/horizon-rust/horizon-mods/actions/runs/41939774642) passed. The [source review](https://github.com/horizon-rust/horizon-mods/pull/248#issuecomment-6656913316) requested a revised version for a new submission. A reviewer must approve and merge the registry entry before it appears in the catalog.

Community introductions were published and their individual pages verified on 5 November 2026:

- [r/ModdedMinecraft — with two real game screenshots](https://www.reddit.com/r/ModdedMinecraft/comments/1xo4itr/server_atlas_a_minecraftstyle_multiplayer_browser/)
- [r/feedthebeast — multiplayer workflow showcase](https://www.reddit.com/r/feedthebeast/comments/1xo4m7b/server_atlas_local_notes_favorites_and_reusable/)
- [r/fabricmc — Fabric beta introduction](https://www.reddit.com/r/fabricmc/comments/1xo4n5a/server_atlas_for_fabric_263_a_familiar/)
- [r/MinecraftForge — Forge beta showcase and test limits](https://www.reddit.com/r/MinecraftForge/comments/1xo4o5k/server_atlas_forge_263_beta_saved_server_groups/)

- [Modrinth](https://modrinth.com/mod/server-atlas) — the project is under moderation. Separate Fabric and Forge beta versions are uploaded for Minecraft Java 26.3.
- [CurseForge author dashboard](https://authors.curseforge.com/#/projects/1784709/files) — both beta files have been uploaded and are processing. The project and files await platform approval.

Reddit posts remain subject to community moderation.

The project was developed and publication materials were prepared with Codex assistance at the owner's request. Feedback currently saves local drafts; automatic email delivery is unconfigured.
