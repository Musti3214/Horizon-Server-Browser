# Server Atlas

**Find your next Minecraft server. Keep your favorite worlds close.**

A Minecraft Java server browser for **Horizon, Fabric and Forge**, with a familiar Minecraft interface, public discovery, practical filters and a personal server list. Built by shuab. Formerly Horizon Server Browser.

<img src="docs/server-atlas-icon.png" alt="Server Atlas compass logo" width="128" height="128">

[Website & screenshots](https://musti3214.github.io/Horizon-Server-Browser/) ·
[Download Horizon 2.2.1](https://musti3214.github.io/Horizon-Server-Browser/downloads/server-browser-2.2.1.hnmod) ·
[Installation](docs/INSTALLATION.md) · [Deutsch](README.de.md)

Website languages: [English](https://musti3214.github.io/Horizon-Server-Browser/),
[Deutsch](https://musti3214.github.io/Horizon-Server-Browser/de/),
[Français](https://musti3214.github.io/Horizon-Server-Browser/fr/),
[Español](https://musti3214.github.io/Horizon-Server-Browser/es/).
If you would like to support development: [Support this project on Ko-fi](https://ko-fi.com/foryou41).

![Actual Server Browser 2.2.0 public discovery screen in Horizon](docs/screenshots/entdecken.png)

*An unedited capture of the built mod. Server status and player counts are momentary values.*

## One browser. Your next adventure.

- **Discover public Java servers** from minecraftserve.rs and Modrinth.
  Online discovery starts after you accept the privacy screen. Status probes stay within the visible rows and a small margin.
- **Search and filter** names, addresses, MOTDs, tags and personal notes. Combine
  status, version, player count, ping, language, region and group filters.
- **Keep your own list** with favorites, notes, groups, custom ordering and reusable
  filter profiles. Personal data stays in the local game profile.
- **Feel at home** with actual Minecraft buttons, resource-pack support, translucent
  rows and a centered layout. Classic mode remains available.
- **Stay in control** with cached public directories, bounded status checks and
  persistent discovery preferences. An explicit Off remains Off.

## Available now

| Platform | Status | Download |
| --- | --- | --- |
| Horizon, Minecraft Java 26.3 | **2.2.1 · native Rust mod**; tested on Windows x64 | `.hnmod` |
| Fabric · Minecraft 26.3 | **2.2.0 public beta** · Java 25 · Loader 0.20.1+ | [Fabric JAR](downloads/server-browser-fabric-2.2.0+mc26.3.jar) |
| Forge · Minecraft 26.3 | **2.2.0 public beta** · Java 25 · Forge 66.1.5+ | [Forge JAR](downloads/server-browser-forge-2.2.0+mc26.3.jar) |

The Horizon package uses the official native-mod build process. It includes its
Rust sources and requires no companion program or separately installed host patch.
It is pinned to the exact Horizon commit documented in [validation](mod/VALIDATION-2.2.1.md).

## A closer look

### Your servers, your details

![Personal example server list with favorites and the details panel](docs/screenshots/meine-server.png)

### Filter the way you play

![Actual player-count and maximum-ping filter fields](docs/screenshots/werte-filter.png)

<details>
<summary><strong>More real screenshots: editing, profiles and settings</strong></summary>

![Editing a local example server and its notes](docs/screenshots/server-bearbeiten.png)

![Preparing a reusable filter profile](docs/screenshots/profile.png)

![Actual Server Browser settings](docs/screenshots/einstellungen.png)

</details>

Function screenshots use disposable example profiles and local status fixtures.
The discovery image comes from a real public-directory test. No private player
worlds, accounts or personal server lists appear in these images.

## Install Fabric or Forge

Minecraft **26.3 / Java 25**: install Fabric Loader **0.20.1+** or Minecraft Forge **66.1.5+**, put only the matching JAR in that instance’s `mods` folder and open **Multiplayer**. Required Fabric API modules are bundled. No Horizon build is needed for these Java editions. They are public betas; see [Java installation and validation](java/README.md) and [test limits](java/VALIDATION.md).

## Install in Horizon

1. Download `server-browser-2.2.1.hnmod` and select a dedicated Horizon instance.
2. Put the file in that instance's `horizon-native-mods` folder.
3. On the launcher's Mods page, enable **server_browser 2.2.1**. Choose **Build**
   for the package's tested revision and review the native-code trust notice.
4. After the build finishes, launch that instance and open **Multiplayer**.

An existing 2.0 client cannot hot-load the 2.2 source update. Rebuild after changing
the package. Other native mods may target different Horizon commits; test this
mod on its own first. [Full instructions and troubleshooting](docs/INSTALLATION.md).

## Privacy and current limits

Your saved servers, favorites and notes stay local. Public-directory requests go
only to the named sources; normal status pings contact the servers being checked.
The mod uses Horizon's existing login and connection implementation.

Feedback and server-proposal drafts can be saved locally. The native toolbar’s Ko-fi button opens the project’s support page in your browser. Fabric and Forge 2.2.0 are now available as public betas.
The web catalog is a dated address list and does not claim live player counts.
Read [privacy](docs/PRIVACY.md) and [server sources](docs/SERVER-SOURCES.md).

## Development

`java/` contains the original shared Java code, Fabric and Forge build configurations and integration suite (96 checks per loader). `mod/` contains the original Rust implementation, translations, native manifest
and official-mixin helpers. `website/` contains the static website and all 13 real
screenshots. `downloads/` contains the tested package and SHA-256 checksum.

```sh
python tools/validate_public.py
python website/build_site.py
python website/tools/validate_site.py
```

Rebuilding the native bundle additionally requires Rust 1.99.1's `rustfmt` on PATH:
`python mod/build_bundle.py`. Running native game tests requires a separately
built pinned Horizon client and your own prepared Minecraft resources.

The release passed **44 Rust tests**, native client build
and actual isolated game/UI/directory tests. [Exact scope and inputs](docs/VALIDATION.md).

MIT licensed. Independent community project; not affiliated with or endorsed by
Mojang, Microsoft or the Minecraft modloader projects. [License](LICENSE) · [Notices](NOTICE.md).
