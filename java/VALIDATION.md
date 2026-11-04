# Server Atlas validation — 2.2.0 / Minecraft 26.3

Test date: 4 November 2026. Windows 11 x64, Temurin JDK 25.0.5+8, NVIDIA GTX 1650. Test profiles and files are isolated from the player's profiles.

## Automated checks

Both loader builds run the same integration suite: **96 checks passed for Fabric and 96 for Forge**. Coverage includes vanilla NBT preservation, hidden entries and unknown fields, byte-exact undo, external-edit detection, corrupted input retention, JSON recovery, import/export, Horizon JSON migration, favorites and history, combined filtering, unknown-value sort order, persistent directory throttling and Retry-After, stale cache, the bundled catalog and a local TCP status-protocol fixture. The fixture is not a Minecraft dedicated server.

The final JARs passed `tools/package_release.py` validation: loader metadata, bundled Fabric modules and licenses, translations, catalog size and absence of Minecraft/Mojang classes. Checksums are included in the release. An independent Linux/JDK 25 build and both loader suites also passed on GitHub Actions: https://github.com/Musti3214/Horizon-Server-Browser/actions/runs/41805732877

## Runtime checks

The final Fabric distribution (SHA-256 `e052510b60b6013e1583bf87b6a383290e314f3ccd89f30b7ba7da110937261a`) was launched outside Gradle in an isolated standard Fabric Loader 0.20.1 profile with exactly one mod JAR. The bundled resource modules loaded. Multiplayer opened **Server Atlas** with German translations and the Horizon-style interface. Live server status, icons and MOTDs were visible. The 26.3 client JAR was verified against the official SHA-1; no personal account session or game profile was used. `docs/screenshots/server-atlas-fabric.png` in the repository is an actual unedited F2 capture.

The final Forge development build (Forge 66.1.5) was opened in game. Automatic replacement of Multiplayer, German resources, live status/icons/MOTDs, primary-button selection and details, favorite stars, the Favorites tab and the Status/Values filter pages were checked. `docs/screenshots/forge-filters.png` is an actual unedited F2 capture. Public directory requests returned valid cached pages from both providers. A separate standard installation of the packaged Forge JAR has not yet been runtime verified.

Minecraft 26.3 uses SDL primary-button identifier 1. Forge's bundled Mixin 0.8.7 requires compatibility level JAVA_21: our classes use bytecode 21, while dependencies and runtime use Java 25. Forge development runs explicitly enable the mixin configuration; distribution JARs declare MixinConfigs in their manifest. ForgeGradle source outputs are merged to keep metadata and classes together. The resource pack targets format 97.

## Limits

These releases are **public betas**. Full CRUD dialog coverage, compact GUI-scale details, Classic return and third-party modpack compatibility remain incomplete. Paid-account joining and multiplayer gameplay are not established by dummy-session test clients. The mod uses Minecraft's existing connection/authentication flow. LAN discovery still requires another advertising game to validate end to end.

Feedback and server suggestions save clearly labelled **local drafts only**; automatic email delivery is not configured. Expected development Realms authentication errors and OSHI performance-counter warnings do not establish a mod failure. No system configuration was changed.
