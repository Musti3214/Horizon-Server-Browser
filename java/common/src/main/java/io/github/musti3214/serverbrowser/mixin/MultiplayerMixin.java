package io.github.musti3214.serverbrowser.mixin;

import io.github.musti3214.serverbrowser.client.*;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.multiplayer.JoinMultiplayerScreen;
import net.minecraft.network.chat.Component;
import org.spongepowered.asm.mixin.*;
import org.spongepowered.asm.mixin.injection.*;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(JoinMultiplayerScreen.class)
public abstract class MultiplayerMixin extends Screen implements MultiplayerParent {
    @Shadow @Final private Screen lastScreen;
    protected MultiplayerMixin(Component title){super(title);}
    @Unique public Screen serverbrowser$parent(){return lastScreen;}
    @Inject(method="init",at=@At("TAIL"))
    private void serverbrowser$button(CallbackInfo ci) {
        addRenderableWidget(Button.builder(Component.literal("Server Atlas"),b->minecraft.gui.setScreen(new BrowserScreen(lastScreen,(JoinMultiplayerScreen)(Object)this))).bounds(5,5,110,20).build());
    }
}
