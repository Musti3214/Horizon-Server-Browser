package io.github.musti3214.serverbrowser.core;

import com.google.gson.*;
import java.io.*;
import java.nio.ByteBuffer;
import java.nio.channels.FileChannel;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import static java.nio.file.StandardCopyOption.*;
import static java.nio.file.StandardOpenOption.*;

public final class JsonStore {
    public static final Gson GSON=new GsonBuilder().setPrettyPrinting().disableHtmlEscaping().create();
    public static final int MAX_FILE=8*1024*1024;
    private final Path path;
    private JsonObject original=new JsonObject();
    public JsonStore(Path path) { this.path=path; }
    public Model.State load() throws IOException {
        if(!Files.exists(path)) return new Model.State();
        try {
            original=JsonParser.parseString(read(path)).getAsJsonObject();
            Model.State state=GSON.fromJson(original,Model.State.class);
            if(state==null||state.schema!=1||state.settings==null||state.view==null||state.view.filter==null||state.metadata==null||state.presets==null||state.recent==null)throw new IOException("Unsupported or incomplete state");
            state.settings.sanitize();return state;
        } catch(JsonParseException|IllegalStateException ex) { throw new IOException("Unreadable state; original retained",ex); }
    }
    public void save(Model.State state) throws IOException {
        JsonObject next=original.deepCopy();
        JsonObject serialized=GSON.toJsonTree(state).getAsJsonObject();
        for(String map:new String[]{"metadata","presets"}) {
            if(next.has(map)&&next.get(map).isJsonObject())next.getAsJsonObject(map).keySet().retainAll(serialized.getAsJsonObject(map).keySet());
        }
        merge(next,serialized);
        atomic(path,GSON.toJson(next).getBytes(StandardCharsets.UTF_8),true); original=next;
    }
    private static void merge(JsonObject target,JsonObject source) {
        for(var entry:source.entrySet()) {
            var old=target.get(entry.getKey()); var value=entry.getValue();
            if(old!=null&&old.isJsonObject()&&value.isJsonObject())merge(old.getAsJsonObject(),value.getAsJsonObject());
            else target.add(entry.getKey(),value.deepCopy());
        }
    }
    public static String read(Path path) throws IOException {
        if(Files.size(path)>MAX_FILE)throw new IOException("File too large");
        try(InputStream in=Files.newInputStream(path)) {byte[] b=in.readNBytes(MAX_FILE+1);if(b.length>MAX_FILE)throw new IOException("File too large");return new String(b,StandardCharsets.UTF_8);}
    }
    public static void atomic(Path path,byte[] bytes,boolean backup) throws IOException {
        if(bytes.length>MAX_FILE)throw new IOException("File too large");
        Path parent=path.toAbsolutePath().getParent();Files.createDirectories(parent);
        Path temp=Files.createTempFile(parent,".server-browser-",".tmp");
        try {
            try(FileChannel ch=FileChannel.open(temp,WRITE,TRUNCATE_EXISTING)){ByteBuffer b=ByteBuffer.wrap(bytes);while(b.hasRemaining())ch.write(b);ch.force(true);}
            if(backup&&Files.exists(path))Files.copy(path,path.resolveSibling(path.getFileName()+".bak"),REPLACE_EXISTING);
            try {Files.move(temp,path,ATOMIC_MOVE,REPLACE_EXISTING);}catch(AtomicMoveNotSupportedException ex){Files.move(temp,path,REPLACE_EXISTING);}
        } finally { Files.deleteIfExists(temp); }
    }
}
