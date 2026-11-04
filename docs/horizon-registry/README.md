# Server Atlas — Horizon registry review kit

This is a **review proposal**, not a registry acceptance or publication confirmation.

## Contents

- `mods/server_browser.toml`: proposed registry project and version, owned by Musti3214.
- `icons/server_browser.png`: original 256px logo, below the 256 KiB limit.
- `server_browser-2.1.0.hnmod`: the exact already released native bundle, renamed for the registry. Its bytes and SHA-256 are unchanged.
- `SHA256SUMS.txt`: bundle checksum.
- `NIGHTLY-PREFLIGHT.json`: latest source-planning result and its limits.
- `PULL-REQUEST.md`: draft review request with attribution and compatibility information.

## Review and publication

Official form: https://playhorizon.dev/mods/submit
Official workflow: https://playhorizon.dev/docs/publish-a-mod

Choose native mod, upload the .hnmod and icon, and use the proposed TOML for the project fields. Preserve `server_browser` / `2.1.0`, MIT, and owner Musti3214. Signing is optional; this file is unsigned and has no project signing key. Do not invent a signature.

For a manual registry pull request, copy `mods/server_browser.toml` and `icons/server_browser.png` to a current horizon-rust/horizon-mods checkout. The registry's fetch.sh also requires the exact file in its `staging` prerelease as `bb6bb8e5e89299a0d853ce85db520205564da8b77576b61fbf6054eb200bfd58.hnmod` before its checks can fetch a new version. A maintainer or the official submission service must handle that staging upload and the submitting-account/owner checks. Merely committing the TOML in our own repository does not submit or publish it.

The native file runs with the player's permissions and needs normal launcher compilation. The published source is MIT and contains the complete bundled Rust; no game assets, account files, keys or companion executable are included.

## Compatibility decision needed

The bundle pins Horizon 142388e1c136e622652ec7505af15d8ad195e041 and horizon-mixins a2af5c0dd4117d51ecd70066ba7a00b4a2c10dab. Its recorded native build and 44 Rust checks passed on those revisions.

A source-planning check against Nightly 67dad0d1a5adca139b426c8114e74a33b376ba64 failed with E_MODULE_CONFLICT: inserting server_browser beside the newer library's glob re-export is rejected by the pinned mixin tool. Only a disposable manifest copy was retargeted; the published file is unchanged. No successful current-Nightly build is claimed. Reviewers should decide whether the already validated pinned bundle is eligible, or whether the UI needs an API/mixin migration first. This kit must not be presented as current-Nightly compatible.

The public project is now Server Atlas; the 2.1.0 in-game title remains Server Browser. Feedback saves local drafts only; automatic email is not configured.

Source: https://github.com/Musti3214/Horizon-Server-Browser/tree/v2.1.0/mod
Native release: https://github.com/Musti3214/Horizon-Server-Browser/releases/tag/v2.1.0
Website and actual screenshots: https://musti3214.github.io/Horizon-Server-Browser/
