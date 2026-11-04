# Add Server Atlas 2.2.0 native server browser

Owner: Musti3214. Please review this MIT-licensed native browser for publication through Horizon's normal registry workflow.

The new bundle builds on Nightly `6475f62f3c834b7489e1816bf47f47ea5eb2b879` and includes its full Rust source and license. It uses 25 injections and two modules with no Cargo manifest, dependency, WIT or post-build changes. Official package check, 44 browser Rust tests and isolated rendered UI/discovery tests pass. Exact results and checksums are in NIGHTLY-VALIDATION.json.

This supersedes the earlier pinned 2.1.0 compatibility proposal. Native menu targets were adapted to horizon-client-lib, scrolling and keyboard return values, and the DNS resolver API. The persistent mod ID and player data formats remain server_browser / vanilla servers.dat.

The registry staging asset and owner checks still need the official submission service or an authorized maintainer. No official listing is claimed by this prepared PR text.

Created with Codex assistance at the project owner's request. Actual source and runtime verification are documented; no Mojang game data is redistributed.
