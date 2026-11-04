package io.github.musti3214.serverbrowser.client;

import io.github.musti3214.serverbrowser.core.*;
import net.minecraft.nbt.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;
import java.util.function.Consumer;
import static io.github.musti3214.serverbrowser.core.Model.*;

/** Modify a fresh NBT tree for each operation, retaining hidden entries and unknown tags. */
public final class DatRepository {
    private final Path path;
    private byte[] undoBefore,undoAfter;
    public DatRepository(Path path) {this.path=path;}
    private CompoundTag read() throws IOException {
        if(!Files.exists(path))return new CompoundTag();
        if(Files.size(path)>JsonStore.MAX_FILE)throw new IOException("servers.dat is too large");
        try(DataInputStream in=new DataInputStream(new ByteArrayInputStream(Files.readAllBytes(path)))) {
            return NbtIo.read(in,NbtAccounter.create(JsonStore.MAX_FILE));
        } catch(RuntimeException ex){throw new IOException("Cannot read servers.dat; original retained",ex);}
    }
    public List<Entry> list() throws IOException {
        List<Entry> entries=new ArrayList<>();
        for(Tag tag:read().getListOrEmpty("servers")) {
            if(!(tag instanceof CompoundTag c)||c.getBooleanOr("hidden",false))continue;
            try {Entry e=new Entry(c.getStringOr("name","Minecraft"),c.getStringOr("ip",""));e.packs=c.getBoolean("acceptTextures").map(v->v?"on":"off").orElse("ask");e.icon=c.getString("icon").filter(v->v.length()<=32768).orElse(null);entries.add(e);}catch(IllegalArgumentException ignored){}
        }
        return entries;
    }
    public void upsert(Entry entry,String oldKey) throws IOException {
        Address.parse(entry.address);
        mutate(rows->{
            CompoundTag target=null;
            for(Tag t:rows)if(t instanceof CompoundTag c&&key(c).equals(oldKey==null?entry.key():oldKey)){target=c;break;}
            for(Tag t:rows)if(t instanceof CompoundTag c&&c!=target&&!c.getBooleanOr("hidden",false)&&key(c).equals(entry.key()))throw new IllegalArgumentException("duplicate");
            if(target==null){target=new CompoundTag();rows.add(target);}
            target.putString("name",entry.name);target.putString("ip",entry.address);target.putBoolean("hidden",false);
            if(entry.icon!=null&&entry.icon.length()<=32768)target.putString("icon",entry.icon);
            if(entry.packs!=null){if(entry.packs.equals("ask"))target.remove("acceptTextures");else target.putBoolean("acceptTextures",entry.packs.equals("on"));}
        });
    }
    public void delete(String key) throws IOException {mutate(rows->rows.removeIf(t->t instanceof CompoundTag c&&!c.getBooleanOr("hidden",false)&&key(c).equals(key)));}
    public int merge(List<CompoundTag> incoming) throws IOException {
        int[] count={0};mutate(rows->{Set<String> present=new HashSet<>();for(Tag t:rows)if(t instanceof CompoundTag c)present.add(key(c));for(CompoundTag c:incoming)if(!key(c).isEmpty()&&present.add(key(c))){rows.add(c.copy());count[0]++;}});return count[0];
    }
    public List<CompoundTag> compounds()throws IOException {List<CompoundTag> out=new ArrayList<>();for(Tag t:read().getListOrEmpty("servers"))if(t instanceof CompoundTag c)out.add(c.copy());return out;}
    public byte[] raw()throws IOException {if(!Files.exists(path))return new byte[0];if(Files.size(path)>JsonStore.MAX_FILE)throw new IOException("servers.dat too large");return Files.readAllBytes(path);}
    public void move(String key,int delta) throws IOException {
        mutate(rows->{
            List<Integer> visible=new ArrayList<>();int selected=-1;
            for(int i=0;i<rows.size();i++)if(rows.get(i) instanceof CompoundTag c&&!c.getBooleanOr("hidden",false)){if(key(c).equals(key))selected=visible.size();visible.add(i);}
            int destination=selected+delta;
            if(selected>=0&&destination>=0&&destination<visible.size())Collections.swap(rows,visible.get(selected),visible.get(destination));
        });
    }
    private void mutate(Consumer<ListTag> change) throws IOException {
        byte[] before=Files.exists(path)?Files.readAllBytes(path):null;
        CompoundTag root=read();ListTag rows=root.getListOrEmpty("servers");change.accept(rows);root.put("servers",rows);
        ByteArrayOutputStream buffer=new ByteArrayOutputStream();NbtIo.write(root,new DataOutputStream(buffer));byte[] after=buffer.toByteArray();
        // Do not replace a file another program changed while this edit was being prepared.
        byte[] current=Files.exists(path)?Files.readAllBytes(path):null;
        if(!Arrays.equals(before,current))throw new IOException("servers.dat changed; reload before saving");
        JsonStore.atomic(path,after,true);undoBefore=before;undoAfter=after;
    }
    public boolean canUndo() {return undoAfter!=null;}
    public void undo() throws IOException {
        if(undoAfter==null)return;
        byte[] current=Files.exists(path)?Files.readAllBytes(path):null;
        if(!Arrays.equals(current,undoAfter))throw new IOException("servers.dat changed; undo stopped");
        if(undoBefore==null) {
            // Preserve the newly created file as a backup and restore an empty server list.
            CompoundTag empty=new CompoundTag();empty.put("servers",new ListTag());ByteArrayOutputStream b=new ByteArrayOutputStream();NbtIo.write(empty,new DataOutputStream(b));JsonStore.atomic(path,b.toByteArray(),true);
        } else JsonStore.atomic(path,undoBefore,true);
        undoAfter=null;undoBefore=null;
    }
    private static String key(CompoundTag c) {try{return Address.key(c.getStringOr("ip",""));}catch(IllegalArgumentException ex){return "";}}
    public CompoundTag serverTag(String key) throws IOException {
        for(Tag t:read().getListOrEmpty("servers"))if(t instanceof CompoundTag c&&key(c).equals(key))return c.copy();
        return null;
    }
}
