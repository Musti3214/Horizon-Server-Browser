# Server Atlas 2.2.1 review validation

- Target: Minecraft Java 26.3 and Horizon Nightly 2026-11-04, commit `6475f62f3c834b7489e1816bf47f47ea5eb2b879`.
- Mixin tool: `a2af5c0dd4117d51ecd70066ba7a00b4a2c10dab`. Official application succeeded with 25 injections and two modules.
- Package: `server-browser-2.2.1.hnmod`, 145,083 bytes, SHA-256 `71279e980f3861608810f762fed93cb3264bc5b069b0200f3e9f46099418e8c3`.
- The official `cargo horizon-mod check` passed. The 30 entries match the public source byte for byte. The 50 upstream Cargo manifests, locks and WIT files match the pinned source.
- All 44 native browser Rust tests passed on the staged Horizon source. Coverage includes fresh and upgraded consent settings, a 2,500-row fixture exercising the visible probe window on load, scroll and refresh, 2,382 unique bundled addresses, and local TCP tests for modern and legacy server status.
- The published Fabric 2.2.0 beta was inspected separately. Both dependency files declared by `fabric.mod.json` exist under `META-INF/jars/` in the released JAR.

Online Discover starts after explicit consent, including when upgrading. The native browser queues status probes for the visible window and a small margin. The saved list still refreshes when the saved or favorites tab is active. Directory entries marked for offline access are excluded. The account mode of a saved server remains a private note. Feedback remains a local draft and the Ko-fi button opens the browser.
