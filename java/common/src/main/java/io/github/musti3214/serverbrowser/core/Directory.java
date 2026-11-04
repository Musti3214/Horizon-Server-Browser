package io.github.musti3214.serverbrowser.core;

import com.google.gson.*;
import java.io.*;
import java.net.*;
import java.net.http.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.time.Duration;
import java.util.*;
import java.util.function.LongSupplier;
import static io.github.musti3214.serverbrowser.core.Model.*;

/** Public directory only: neither search text nor personal entries enter these requests. */
public final class Directory {
    public enum Source { MINECRAFTSERVERS, MODRINTH }
    public record Page(List<Entry> entries,int pages,boolean stale) { }
    @FunctionalInterface public interface Transport { byte[] get(URI uri) throws IOException,InterruptedException; }
    public static final class Limited extends IOException {
        public final long seconds;
        public Limited(long seconds) { super("directory rate limit");this.seconds=Math.clamp(seconds,60,86400); }
    }
    private final Path home;
    private final Transport transport;
    private final LongSupplier clock;
    private final Object[] locks={new Object(),new Object()};
    public Directory(Path home) { this(home,http(),()->System.currentTimeMillis()/1000); }
    public Directory(Path home,Transport transport,LongSupplier clock) { this.home=home;this.transport=transport;this.clock=clock; }
    public static URI url(Source source,int page) {
        page=Math.clamp(page,1,200);
        return URI.create(source==Source.MODRINTH
                ?"https://api.modrinth.com/v3/search?limit=50&offset="+((page-1)*50)+"&index=relevance&facets=%5B%5B%22project_types%3Aminecraft_java_server%22%5D%5D"
                :"https://minecraftserve.rs/api/servers?edition=java&per=50&page="+page);
    }
    public Page page(Source source,int number,boolean enabled) throws IOException,InterruptedException {
        if(!enabled)throw new IOException("directory disabled");
        synchronized(locks[source.ordinal()]) {
            number=Math.clamp(number,1,200);long now=clock.getAsLong();
            Path folder=home.resolve(source.name().toLowerCase(Locale.ROOT));
            Path cache=folder.resolve("page-"+number+".json"),rate=folder.resolve("rate.json");
            JsonObject cached=readObject(cache);
            if(cached!=null&&cached.has("expires")&&cached.get("expires").getAsLong()>now)return fromCache(cached,false);
            JsonObject previousRate=readObject(rate);
            if(previousRate!=null&&previousRate.get("next").getAsLong()>now) {
                if(cached!=null)return fromCache(cached,true);
                throw new Limited(previousRate.get("next").getAsLong()-now);
            }
            writeRate(rate,now+60);
            try {
                Page parsed=parse(source,transport.get(url(source,number)));
                JsonObject object=JsonStore.GSON.toJsonTree(parsed).getAsJsonObject();object.addProperty("expires",now+300);
                JsonStore.atomic(cache,JsonStore.GSON.toJson(object).getBytes(StandardCharsets.UTF_8),false);
                return parsed;
            } catch(Limited ex) {
                writeRate(rate,now+ex.seconds);
                if(cached!=null)return fromCache(cached,true);throw ex;
            } catch(IOException|JsonParseException|IllegalStateException ex) {
                if(cached!=null)return fromCache(cached,true);
                throw ex instanceof IOException io?io:new IOException("invalid directory response",ex);
            }
        }
    }
    private static JsonObject readObject(Path p) {
        try {return JsonParser.parseString(JsonStore.read(p)).getAsJsonObject();}catch(Exception ex){return null;}
    }
    private static Page fromCache(JsonObject data,boolean stale) throws IOException {
        try {
            Page p=JsonStore.GSON.fromJson(data,Page.class);
            if(p.entries()==null||p.entries().size()>50||p.pages()<1||p.pages()>200)throw new IOException("invalid cache");
            for(Entry e:p.entries())Address.parse(e.address);
            return new Page(p.entries(),p.pages(),stale);
        } catch(JsonParseException|IllegalArgumentException ex){throw new IOException("invalid cache",ex);}
    }
    private static void writeRate(Path p,long next) throws IOException {JsonStore.atomic(p,("{\"next\":"+next+"}").getBytes(StandardCharsets.UTF_8),false);}
    public static Page parse(Source source,byte[] bytes) throws IOException {
        if(bytes.length>JsonStore.MAX_FILE)throw new IOException("directory too large");
        try {
            JsonObject root=JsonParser.parseString(new String(bytes,StandardCharsets.UTF_8)).getAsJsonObject();
            JsonArray rows=source==Source.MODRINTH?root.getAsJsonArray("hits"):root.getAsJsonArray("servers");
            if(rows==null||rows.size()>50)throw new IOException("invalid directory rows");
            int pages=source==Source.MODRINTH?(int)Math.ceil(number(root,"total_hits",0)/50.0):number(root,"pages",1);
            pages=Math.clamp(pages,1,200);
            Map<String,Entry> entries=new LinkedHashMap<>();
            for(JsonElement raw:rows) {
                if(!raw.isJsonObject())continue;JsonObject row=raw.getAsJsonObject();
                JsonObject details=source==Source.MODRINTH?obj(row,"minecraft_java_server"):row;
                if(source==Source.MODRINTH&&!strings(row,"project_types").contains("minecraft_java_server"))continue;
                if(source==Source.MINECRAFTSERVERS&&!str(row,"edition","").equals("java"))continue;
                String host=str(row,"host","");
                String value=source==Source.MODRINTH?str(details,"address",""):(host.contains(":")?"["+host+"]":host)+":"+number(row,"port",25565);
                try {
                    Address addr=Address.parse(value);if(!addr.publicListing())continue;
                    Entry e=new Entry(clean(str(row,"name",str(row,"title",addr.key())),256),addr.key());
                    e.meta.tags=strings(row,source==Source.MODRINTH?"categories":"tags");
                    e.meta.group=source==Source.MODRINTH?"Modrinth":"minecraftserve.rs";
                    JsonObject shared=source==Source.MODRINTH?obj(row,"minecraft_server"):row;
                    e.meta.region=clean(str(shared,"region",""),64);
                    List<String> languages=strings(shared,"languages");e.meta.language=languages.isEmpty()?clean(str(shared,"language",""),32):languages.getFirst();
                    if(str(obj(details,"content"),"kind","").equals("modpack")&&!e.meta.tags.contains("modded"))e.meta.tags.add("modded");
                    entries.putIfAbsent(e.key(),e);
                } catch(IllegalArgumentException ignored) { }
            }
            return new Page(new ArrayList<>(entries.values()),pages,false);
        } catch(JsonParseException|IllegalStateException ex){throw new IOException("invalid directory response",ex);}
    }
    public static List<Entry> bundled() throws IOException {
        try(InputStream in=Directory.class.getResourceAsStream("/assets/server_browser/catalog.json")) {
            if(in==null)throw new IOException("catalog missing");
            byte[] bytes=in.readNBytes(JsonStore.MAX_FILE+1);if(bytes.length>JsonStore.MAX_FILE)throw new IOException("catalog too large");
            JsonArray rows=JsonParser.parseString(new String(bytes,StandardCharsets.UTF_8)).getAsJsonObject().getAsJsonArray("servers");
            Map<String,Entry> out=new LinkedHashMap<>();
            for(var raw:rows) {var r=raw.getAsJsonObject();try {Entry e=new Entry(str(r,"name","Minecraft"),str(r,"address",""));if(!Address.parse(e.address).publicListing())continue;e.meta.tags=strings(r,"tags");out.putIfAbsent(e.key(),e);}catch(IllegalArgumentException ignored){}}
            return new ArrayList<>(out.values());
        }
    }
    private static Transport http() {
        HttpClient client=HttpClient.newBuilder().connectTimeout(Duration.ofSeconds(8)).followRedirects(HttpClient.Redirect.NEVER).build();
        return uri->{
            var req=HttpRequest.newBuilder(uri).timeout(Duration.ofSeconds(8)).header("User-Agent","ServerBrowser/2.2.0 (+https://github.com/Musti3214/Horizon-Server-Browser)").header("Accept","application/json").GET().build();
            // The body limit applies during collection; the request deadline covers the complete body.
            var response=client.send(req,HttpResponse.BodyHandlers.limiting(HttpResponse.BodyHandlers.ofByteArray(),JsonStore.MAX_FILE));
                if(response.statusCode()==429) {long seconds=60;try{seconds=Long.parseLong(response.headers().firstValue("Retry-After").orElse("60"));}catch(NumberFormatException ignored){}throw new Limited(seconds);}
                if(response.statusCode()!=200)throw new IOException("directory HTTP "+response.statusCode());
                if(response.headers().firstValueAsLong("Content-Length").orElse(0)>JsonStore.MAX_FILE)throw new IOException("directory too large");
                return response.body();
        };
    }
    public static String clean(String value,int max) {return value.replaceAll("[\\p{Cntrl}]","").codePoints().limit(max).collect(StringBuilder::new,StringBuilder::appendCodePoint,StringBuilder::append).toString();}
    private static String str(JsonObject o,String key,String fallback) {var v=o.get(key);return v!=null&&v.isJsonPrimitive()&&v.getAsJsonPrimitive().isString()?v.getAsString():fallback;}
    private static int number(JsonObject o,String key,int fallback) {try{return o.get(key).getAsInt();}catch(Exception ignored){return fallback;}}
    private static JsonObject obj(JsonObject o,String key) {var v=o.get(key);return v!=null&&v.isJsonObject()?v.getAsJsonObject():new JsonObject();}
    private static List<String> strings(JsonObject o,String key) {List<String> out=new ArrayList<>();var v=o.get(key);if(v!=null&&v.isJsonArray())for(var item:v.getAsJsonArray()){if(out.size()==32)break;if(item.isJsonPrimitive()&&item.getAsJsonPrimitive().isString())out.add(clean(item.getAsString(),64));}return out;}
}
