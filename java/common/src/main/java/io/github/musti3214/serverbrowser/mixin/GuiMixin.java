package io.github.musti3214.serverbrowser.mixin;

import io.github.musti3214.serverbrowser.client.*;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.multiplayer.JoinMultiplayerScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.*;

@Mixin(Gui.class)
public abstract class GuiMixin {
    @ModifyVariable(method="setScreen",at=@At("HEAD"),argsOnly=true,ordinal=0)
    private Screen serverbrowser$screen(Screen screen) {
        if(screen instanceof JoinMultiplayerScreen vanilla && !BrowserScreen.isClassic(vanilla)
                && ClientServices.get().state.settings.enabled) {
            Screen parent=((MultiplayerParent)vanilla).serverbrowser$parent();
            return new BrowserScreen(parent,vanilla);
        }
        return screen;
    }
}
