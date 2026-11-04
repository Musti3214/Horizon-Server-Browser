# Server Atlas — Fabric and Forge

Formerly Horizon Server Browser. The client mod brings the Horizon Server Browser's Minecraft interface to Java Edition: the same five tabs, translucent server rows, native Minecraft controls, filter pages, action bar and server details.

**Public beta:** see [VALIDATION.md](VALIDATION.md) for tested behavior and remaining runtime checks. Existing `server_browser` identifiers and filenames are retained to preserve settings.

## Install

Version **2.2.0** targets **Minecraft 26.3** and **Java 25**.

| Edition | Required loader | Mod file |
| --- | --- | --- |
| Fabric | Fabric Loader 0.20.1 or newer for 26.3 | `server-browser-fabric-2.2.0+mc26.3.jar` |
| Forge | Minecraft Forge 66.1.5 or newer in the 66.x series for 26.3 | `server-browser-forge-2.2.0+mc26.3.jar` |

1. Install the matching loader in your Minecraft launcher.
2. Open that instance's `mods` folder and add **one** matching JAR. Remove older Server Browser JARs from that instance.
3. Start Minecraft and choose **Multiplayer**. Server Atlas opens automatically. **Classic** returns to the original screen; its Server Atlas button opens this mod again.

These are client mods: servers need no installation. Fabric's two small resource-loading modules are included inside the Fabric JAR; a separate Fabric API download is unnecessary. This port does not require Horizon or a native Horizon build. The Horizon `.hnmod` is a separate edition.

## Features

- My Servers, Discover, Favorites, Recent and LAN, with search across names, addresses, tags, groups and notes.
- Combined status, protocol, player count, ping, authentication labels, tag, language and region filters; nine sort modes and saved filter profiles.
- 2,835 bundled public addresses plus paginated Modrinth and minecraftserve.rs discovery. Online discovery is on by default and can be switched off.
- Actual server status, colored MOTDs, server icons, player samples, local ping history and the game's normal connection/authentication flow.
- Add, edit, delete, custom order, undo, favorites, notes and resource-pack preferences. Existing vanilla `servers.dat` entries, hidden entries and unknown NBT fields are retained.
- JSON and vanilla NBT import/export. Imports add new addresses and preserve existing entries. Export overwrites require confirmation.
- English and German, selected by Minecraft's language setting. Native widgets respect Minecraft GUI scaling and resource packs.

Discovery requests only public directory pages. Personal addresses, notes and search text are never sent to directory providers. Server status requests go directly to the visible servers. Each directory has a persistent one-request-per-minute limit, a five-minute cache and stale-cache fallback. Unknown compatibility/authentication information stays unknown.

**Feedback and server suggestions are local drafts only. Automatic email delivery is not configured.** Drafts are stored in `config/server-browser/drafts`; the interface states that no email was sent.

Settings, metadata and profiles live in `config/server-browser/state-v1.json`, separately from vanilla `servers.dat`. File writes use backups and atomic replacement. Undo stops if another program has changed the server file in the meantime. Keep normal instance backups.

## Build and checks

Use a JDK 25 and the included Gradle wrapper:

```powershell
.\gradlew.bat :fabric:build :forge:build
```

On Linux/macOS use `sh gradlew :fabric:build :forge:build`. Outputs are in each loader's `build/libs`. Install the regular JAR, not `-sources.jar`. No game assets or game JARs are part of this repository.

`build` runs a shared integration suite against both loaders. It tests actual NBT and JSON files, import/export (including Horizon's native JSON array), corruption recovery, stable filtering/sorting, persistent directory throttling, stale-cache fallback and a local TCP status-protocol fixture. `:fabric:runClient` and `:forge:runClient` use separate development profiles; they do not use the player's existing worlds or server lists. Development clients do not establish a verified paid-account multiplayer session. ForgeGradle's development launcher additionally needs a discoverable JDK 8 toolchain for its helper; configure `org.gradle.java.installations.paths` if Gradle does not detect it. Players only need Java 25.

The source is MIT licensed. See [THIRD-PARTY.md](THIRD-PARTY.md) for bundled Fabric components. Project: https://github.com/Musti3214/Horizon-Server-Browser · Website: https://musti3214.github.io/Horizon-Server-Browser/ · Support: https://ko-fi.com/foryou41

## Deutsch

Für **Minecraft 26.3 / Java 25**: Fabric Loader ab **0.20.1** oder Forge ab **66.1.5** installieren. Genau die passende Mod-JAR in den `mods`-Ordner dieser Instanz legen und ältere Server-Browser-Versionen dort entfernen. Danach **Mehrspieler** öffnen. Fabric und Forge sind zwei getrennte Dateien; verwende pro Instanz nur die passende Version. Fabric API ist bereits mit den benötigten Modulen enthalten. Kein Horizon-Build nötig.

Design und Bedienung folgen unserer Horizon-Mod: fünf Reiter, transparente Serverzeilen, Suchleiste, Filter, Favoriten, Details und Aktionsleiste. Die Sprache folgt Minecraft. Eigene Server und Notizen bleiben lokal; Feedback wird derzeit nur als Entwurf gespeichert. Die Version benötigt keine Installation auf dem Server.
