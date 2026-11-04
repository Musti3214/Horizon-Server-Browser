package io.github.musti3214.serverbrowser.client;
import net.minecraft.network.chat.Component;
import net.minecraft.client.resources.language.I18n;
import io.github.musti3214.serverbrowser.core.Model.Status;
import com.google.gson.*;
import java.util.List;
import net.minecraft.ChatFormatting;
import net.minecraft.network.chat.MutableComponent;
public final class Text {
    private Text() { }
    public static Component t(String key,Object...args){return Component.translatable("server_browser."+key,args);}
    public static String s(String key,Object...args){return I18n.get("server_browser."+key,args);}
    public static Component motd(Status status){
        if(status.descriptionJson()!=null)try{return safeText(JsonParser.parseString(status.descriptionJson()),0);}catch(Exception ignored){}
        return Component.literal(status.motd());
    }
    private static MutableComponent safeText(JsonElement value,int depth){
        if(value==null||value.isJsonNull()||depth>16)return Component.empty();
        if(value.isJsonPrimitive())return Component.literal(value.getAsString().substring(0,Math.min(2048,value.getAsString().length())));
        if(value.isJsonArray()){MutableComponent out=Component.empty();int i=0;for(var v:value.getAsJsonArray()){if(i++>64)break;out.append(safeText(v,depth+1));}return out;}
        JsonObject obj=value.getAsJsonObject();MutableComponent out=safeText(obj.get("text"),depth+1);
        if(obj.has("color")&&obj.get("color").isJsonPrimitive()){
            String color=obj.get("color").getAsString();
            if(color.matches("#[0-9a-fA-F]{6}"))out.withColor(Integer.parseInt(color.substring(1),16));
            else {if(List.of("black","dark_blue","dark_green","dark_aqua","dark_red","dark_purple","gold","gray","dark_gray","blue","green","aqua","red","light_purple","yellow","white").contains(color))out.withStyle(ChatFormatting.valueOf(color.toUpperCase(java.util.Locale.ROOT)));}
        }
        if(obj.has("bold")&&obj.get("bold").getAsBoolean())out.withStyle(ChatFormatting.BOLD);
        if(obj.has("italic")&&obj.get("italic").getAsBoolean())out.withStyle(ChatFormatting.ITALIC);
        out.append(safeText(obj.get("extra"),depth+1));return out;
    }
}
