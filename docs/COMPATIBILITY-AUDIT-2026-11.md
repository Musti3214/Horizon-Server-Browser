# Compatibility upgrade audit — 2026-11-23

This is a planning-only change on a separate branch. No untested version bumps or release artifacts are included.

## Verified from repository

- Horizon native manifest: mod version 2.2.1, game 26.3, pinned Horizon commit `6475f62f3c834b7489e1816bf47f47ea5eb2b879`, pinned mixins commit `a2af5c0dd4117d51ecd70066ba7a00b4a2c10dab`.
- Fabric: Minecraft 26.3, Fabric Loader 0.20.1, Fabric Loom 1.19.3, Java toolchain 25; bundled Fabric API base and resource loader modules.
- Forge: Minecraft 26.3, Forge 66.1.5, ForgeGradle 7.0.40, Java toolchain 25.
- Java mod version: 2.2.0, public beta.

## Target decision

As of this audit, Minecraft 26.3 is the latest *stable* Java Edition release found; 26.4 snapshots are not production releases. All three editions already target Minecraft 26.3. Therefore do not claim a Minecraft version upgrade without a newer stable release and a supported Horizon build.

## Required next engineering steps

1. Check the latest compatible *released* Fabric Loader, Loom, Fabric API modules, Forge and Horizon commit against their authoritative release feeds.
2. Update dependencies separately, keeping Minecraft 26.3 and Java 25 unless an actual newer stable Minecraft release is supported.
3. Run `sh java/gradlew -p java :fabric:build :forge:build`, repository public validation, native Rust tests, native bundle build, and real client smoke tests.
4. Check UI integration, multiplayer screen interception, discovery, persistence, and privacy behavior on each loader.
5. Publish versioned artifacts and update Modrinth/CurseForge only after passing checks and explicit release approval.

## Safety

No change to `main`, no release, no claim that code has been built or tested. Dependency updates must not be made blindly from unverified version numbers.
