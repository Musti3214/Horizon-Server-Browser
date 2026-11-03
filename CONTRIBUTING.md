# Contributing

Keep changes focused and describe the resulting behavior and validation. Use
original code and public documentation. Do not copy another mod's implementation.
Do not add Minecraft resources, game executables, account files or player data.

Run `python tools/validate_public.py`, build the website and run its validator.
For runtime changes, assemble the native bundle and apply/compile/test it on the
exact documented official Horizon and horizon-mixins revisions. Use fresh isolated
game homes and the contributor's own prepared resources. Never test against real
player worlds or publish them.

Do not remove pins or claim stable-hook compatibility without testing the actual
integration. Do not introduce automatic mail submissions or an undisclosed provider.
Keep Fabric/Forge availability truthful. Images must be actual unedited captures
or clearly identified examples.

The player decides whether to trust native code in the official launcher. Agents
may prepare and test isolated developer builds; they must not accept that player
trust prompt. Identify AI assistance in contributions and co-author agent commits.
