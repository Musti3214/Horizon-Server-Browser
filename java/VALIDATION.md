# Validation record — 2.2.0 / Minecraft 26.3

Test date: 4 November 2026. Windows 11 x64, Temurin JDK 25.0.5+8, NVIDIA GTX 1650. All development game profiles and test files are isolated in this workspace.

## Automated checks

Both loader builds run the same integration suite. Coverage includes real vanilla NBT preservation, hidden entries and unknown fields, byte-exact undo, external-edit detection, corrupted input retention, JSON recovery, import/export, Horizon JSON migration, favorites and recent history, combined filtering, unknown-value sort order, persistent source throttling and Retry-After, stale cache, the bundled catalog and an actual local TCP status-protocol peer. The test peer is a fixture, not a Minecraft server.

Final build: **96 checks passed for Fabric and 96 for Forge**. The final distribution JARs passed `tools/package_release.py` validation. Checksums are in `release/SHA256SUMS.txt`.

The package script additionally checks loader metadata, expected bundled Fabric modules and their licenses, translations, catalog size and absence of Minecraft/Mojang classes in the released JARs.

## Runtime checks

Fabric Loader 0.20.1 and Forge 66.1.5 used separate development profiles. Fabric was opened in game and its translations, native widgets, translucent directory rows, live status, colored MOTDs and server icons were visually checked. After final common-source changes, both loaders rebuilt successfully; the final Fabric JAR has not yet had its separate packaged-profile launch verified.

The final Forge development build was opened in game. Automatic replacement of Multiplayer, German resources, live server status/icons/MOTDs, primary-button selection and detail panel, favorite stars, the Favorites tab and the Status/Values filter pages were checked. `proof/forge-filter.png` is an actual unedited Minecraft F2 screenshot. Public directory requests produced valid cached pages from both providers.

Minecraft 26.3 uses SDL's primary button identifier 1; row selection and favorite stars use it. Forge's bundled Mixin 0.8.7 requires compatibility level JAVA_21, so our own classes use bytecode 21 while dependency resolution and runtime remain Java 25. Forge development runs explicitly enable the mixin config because directory outputs have no JAR manifest; distribution JARs have the MixinConfigs manifest entry. ForgeGradle source outputs are merged to keep metadata and classes together, and pack.mcmeta targets resource format 97.

The user stopped Computer Use with physical Escape after these checks. Further UI tests (including a standard packaged-JAR install launch, full CRUD dialogs, compact-scale details and Classic return) were not completed. A standalone Fabric test profile containing only the final Fabric JAR was prepared but not launched after that stop. No claim of exhaustive runtime coverage is made.

## Limits

Paid-account multiplayer joining and server gameplay are not established by the development clients. They use loader-provided dummy sessions. The mod uses Minecraft's existing connection/authentication flow; it does not bypass it. LAN discovery requires another game advertising on the local network to validate end to end. Email submission is unconfigured and remains a clearly labelled local draft. A third-party modpack compatibility matrix has not been tested.

The local Windows performance-counter warnings come from the game's OSHI system-information library. The tests do not change system configuration. Expected Realms authentication errors are from development dummy accounts.

## Server Atlas publication check

The final Fabric distribution (SHA-256 e052510b60b6013e1583bf87b6a383290e314f3ccd89f30b7ba7da110937261a) was launched outside Gradle in an isolated standard Fabric Loader 0.20.1 profile with exactly one mod JAR. The bundled resource modules loaded, and Multiplayer opened Server Atlas with German translations and the matching Horizon-style interface. The 26.3 client JAR was verified against the official SHA-1. No account session or personal game profile was used.
