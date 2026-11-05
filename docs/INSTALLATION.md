# Install Server Browser in Horizon

Current native release: **2.2.1**, Minecraft Java **26.3**. Tested on Windows x64.
Use the official Horizon launcher and a separate instance for the first test.

## Install and build

1. Download the `.hnmod` from the website. Verify its checksum against
   `downloads/server-browser-2.2.1.sha256` when desired.
2. Open the selected instance's game folder. Put the single file in
   `horizon-native-mods`. Remove an older Server Browser copy from that folder
   after keeping a backup elsewhere. Leave unrelated mods alone.
3. Open **Mods** in the launcher and turn **server_browser 2.2.1** on. A newly
   created instance may have the local mod switched off.
4. Choose the ordinary **Build** action for the package's tested revision.
   The separate "rebuild on latest Nightly" action overrides that revision;
   use it only after porting and testing the mod against that Nightly.
5. Read the native-code notice. The player must make the trust/build decision.
   Wait for completion, then launch this exact instance and open Multiplayer.

The first native build downloads tools and compiles Horizon. It can take several
minutes. Native code runs with your user permissions. A later launch reuses the
finished build; changing the package or mod set requires a new build.

## Common problems

**Installed, not built into this game:** The package is present, but the running
client was built without it. Enable the mod, finish the native build and restart
the resulting instance. Selecting the newest ordinary game version does not build it.

**No native mod is enabled:** Turn on the switch beside `server_browser` in that
instance's Mods page. An installed file can still be disabled.

**The interface still looks like 2.0:** Close the older client and launch the
instance after its new build finishes. Copying the package alone cannot update
code already compiled into an existing executable.

**E_INVALID_HELPER or incompatible source revisions:** Check the exact Horizon
and horizon-mixins pins in `mod/horizon-native.json`. Use the ordinary Build action
and test with only this mod. Do not remove the pins or mark this as a stable-hook
mod to suppress the check: this browser integrates with additional menu functions.

**Other native mods fail together:** Their exact pins or hooks API requirements
may differ. Keep a dedicated instance, or port and test all involved mods against
one compatible revision. Updating an unrelated mod is outside this release.

**No public results:** New settings enable the online directory. A saved Off
remains Off; turn it on in Server Browser settings if desired. Each source is
rate-limited and cached. Temporary failures retain a usable prior list.

The package does not include game resources, a client executable, a companion
application or a separate host patch. Separate Fabric and Forge public beta JARs are available; see java/README.md.
