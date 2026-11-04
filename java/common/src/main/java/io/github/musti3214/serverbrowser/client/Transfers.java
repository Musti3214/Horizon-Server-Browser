package io.github.musti3214.serverbrowser.client;

import com.google.gson.*;
import io.github.musti3214.serverbrowser.core.*;
import net.minecraft.nbt.*;
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import static io.github.musti3214.serverbrowser.core.Model.*;

public final class Transfers {
    private Transfers() { }
    public static int importJson(ClientServices services,Path file)throws IOException {
        try {
            JsonElement input=JsonParser.parseString(JsonStore.read(file));
            // Horizon 2.1 exports a plain array; Java exports a versioned envelope.
            JsonArray rows;
            if(input.isJsonArray())rows=input.getAsJsonArray();
            else {JsonObject root=input.getAsJsonObject();if(!root.has("schema")||root.get("schema").getAsInt()!=1)throw new IOException("unsupported import schema");rows=root.getAsJsonArray("servers");}
            if(rows==null||rows.size()>20000)throw new IOException("invalid server list");
            List<CompoundTag> compounds=new ArrayList<>();Map<String,Meta> metadata=new HashMap<>();
            Set<String> existing=new HashSet<>();for(Entry entry:services.saved.list())existing.add(entry.key());
            for(JsonElement raw:rows) {
                JsonObject row=raw.getAsJsonObject();Entry entry=new Entry(row.get("name").getAsString(),row.get("address").getAsString());
                if(entry.name.isBlank()||entry.name.length()>256)throw new IOException("invalid name");
                Meta meta=row.has("metadata")?JsonStore.GSON.fromJson(row.get("metadata"),Meta.class):new Meta();
                if(meta==null||meta.tags==null||meta.notes==null||meta.group==null||meta.language==null||meta.region==null||meta.auth==null||meta.notes.length()>4096||meta.tags.size()>32)throw new IOException("invalid metadata");
                CompoundTag tag=new CompoundTag();tag.putString("name",entry.name);tag.putString("ip",entry.address);
                JsonElement icon=row.get("icon");if(icon!=null&&!icon.isJsonNull()){String encoded=icon.getAsString();if(encoded.length()>32768)throw new IOException("icon too large");Base64.getDecoder().decode(encoded);tag.putString("icon",encoded);}
                JsonElement packs=row.get("packs");if(packs!=null&&!packs.isJsonNull()){if(packs.isJsonPrimitive()&&packs.getAsJsonPrimitive().isBoolean())tag.putBoolean("acceptTextures",packs.getAsBoolean());else if(Set.of("on","off").contains(packs.getAsString()))tag.putBoolean("acceptTextures",packs.getAsString().equals("on"));}
                compounds.add(tag);if(existing.add(entry.key()))metadata.put(entry.key(),meta);
            }
            if(!services.writable())throw new IOException("configuration read-only");
            int count=services.saved.merge(compounds);services.state.metadata.putAll(metadata);services.persist();return count;
        } catch(JsonParseException|IllegalArgumentException|IllegalStateException|NullPointerException ex){throw new IOException("invalid JSON import; no input rows applied",ex);}
    }
    public static int importNbt(ClientServices services,Path file)throws IOException {
        if(!services.writable())throw new IOException("configuration read-only");
        if(!Files.isRegularFile(file))throw new IOException("import file does not exist");
        DatRepository from=new DatRepository(file);List<CompoundTag> rows=from.compounds();if(rows.size()>20000)throw new IOException("too many servers");
        for(CompoundTag c:rows)Address.parse(c.getStringOr("ip",""));
        int count=services.saved.merge(rows);services.persist();return count;
    }
    public static void exportJson(ClientServices services,Path file)throws IOException {
        destination(services,file);JsonObject root=new JsonObject();root.addProperty("schema",1);JsonArray rows=new JsonArray();
        for(Entry entry:services.entries(Tab.SAVED)) {
            JsonObject row=new JsonObject();row.addProperty("name",entry.name);row.addProperty("address",entry.address);row.add("metadata",JsonStore.GSON.toJsonTree(entry.meta));
            row.add("packs",entry.packs==null||entry.packs.equals("ask")?JsonNull.INSTANCE:new JsonPrimitive(entry.packs.equals("on")));rows.add(row);
            if(entry.icon!=null)row.addProperty("icon",entry.icon);
        }
        root.add("servers",rows);JsonStore.atomic(file,JsonStore.GSON.toJson(root).getBytes(StandardCharsets.UTF_8),true);
    }
    public static void exportNbt(ClientServices services,Path file)throws IOException {
        destination(services,file);byte[] raw=services.saved.raw();if(raw.length==0){CompoundTag empty=new CompoundTag();empty.put("servers",new ListTag());ByteArrayOutputStream bytes=new ByteArrayOutputStream();NbtIo.write(empty,new DataOutputStream(bytes));raw=bytes.toByteArray();}JsonStore.atomic(file,raw,true);
    }
    public static void destination(ClientServices services,Path file)throws IOException {
        Path target=file.toAbsolutePath().normalize(),home=services.home.toAbsolutePath().normalize();
        if(target.equals(home.getParent().getParent().resolve("servers.dat"))||target.startsWith(home)&&!target.startsWith(home.resolve("exports"))||Files.isSymbolicLink(target))throw new IOException("reserved destination");
    }
}
