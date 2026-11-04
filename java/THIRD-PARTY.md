# Third-party components

The Fabric distribution bundles these unmodified official Fabric API modules as nested JARs:

| Module | Version | License |
| --- | --- | --- |
| fabric-api-base | 2.0.6+fcdff87f5d | Apache License 2.0 |
| fabric-resource-loader-v1 | 3.1.0+5af7744b5d | Apache License 2.0 |

Copyright FabricMC and contributors. Source: https://github.com/FabricMC/fabric · Artifacts: https://maven.fabricmc.net/ · License: https://www.apache.org/licenses/LICENSE-2.0

The modules' own license files remain in their original nested JARs. They register mod resources, including translations, with Minecraft. Fabric Loader and Minecraft Forge are provided by the player's launcher and are not bundled. The Gradle wrapper is from the official loader templates and licensed under Apache License 2.0.

Minecraft, its textures, fonts, sounds, game code and game assets are not included. The mod draws the player's installed Minecraft resources at runtime. This project is an independent community project and is not an official Minecraft product.
