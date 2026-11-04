package io.github.musti3214.serverbrowser;

import com.google.gson.*;
import io.github.musti3214.serverbrowser.core.*;
import io.github.musti3214.serverbrowser.client.*;
import net.minecraft.nbt.*;
import java.io.*;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.*;
import static io.github.musti3214.serverbrowser.core.Model.*;

/** Exercises real files/NBT, persistent rate limits and a local status-protocol peer. */
public final class CoreSelfTest {
    private static int checks;
    private static void check(boolean value,String description){if(!value)throw new AssertionError(description);checks++;}
    private static void rejects(Throwing action,String description)throws Exception{try{action.run();}catch(Exception ex){checks++;return;}throw new AssertionError(description);}
    @FunctionalInterface interface Throwing{void run()throws Exception;}
    public static void main(String[] args)throws Exception {
        Path root=Path.of(args[0]).toAbsolutePath();Files.createDirectories(root);
        net.minecraft.SharedConstants.tryDetectVersion();
        addresses();queries();json(root.resolve("json"));directories(root.resolve("directories"));nbt(root.resolve("nbt"));services(root.resolve("services"));transfers(root.resolve("transfers"));wire();
        check(Directory.bundled().size()==2835,"complete bundled catalog");
        System.out.println("PASS: "+checks+" checks; address/query, NBT preservation/undo, JSON recovery, directory cache/rate limits, actual TCP status protocol.");
    }
    private static void addresses()throws Exception {
        check(Address.key(" EXAMPLE.org.:25565 ").equals("example.org"),"default-port and case identity");
        check(Address.key("example.org:25566").equals("example.org:25566"),"nondefault port");
        check(Address.key("[::1]:25565").startsWith("["),"IPv6 identity");
        check(Address.key("münchen.example").contains("xn--"),"IDN");
        for(String bad:List.of("","example.org:0","example.org:65536","https://example.org","host/attack","a\nb","host:abc","1.2.3.999","[invalid]"))rejects(()->Address.parse(bad),"invalid address "+bad);
        for(String local:List.of("localhost","192.168.1.1","127.0.0.1","10.1.2.3","172.16.0.1","100.64.0.1","[::1]","[fd00::1]","test.local"))check(!Address.parse(local).publicListing(),"private directory rejection");
        check(Address.parse("play.example.org").publicListing(),"public DNS listing");
    }
    private static void queries() {
        Entry e=new Entry("§aTest SMP","play.example.org");e.meta.tags.add("survival");e.meta.group="Friends";e.meta.notes="weekend world";
        Status good=new Status(Health.ONLINE,12,20,35L,"26.3",777,"Welcome",null);View v=new View();
        v.search="test #surv @friends weekend";check(Query.matches(e,good,v,777),"AND words, tag/group prefixes and formatting");
        v.search="absent";check(!Query.matches(e,good,v,777),"missing term");v.search="";
        v.filter.minPlayers=10;check(Query.matches(e,good,v,777),"player minimum");check(!Query.matches(e,null,v,777),"unknown does not satisfy minimum");
        v.filter.maxPing=30L;check(!Query.matches(e,good,v,777),"ping AND minimum");v.filter.maxPing=40L;v.filter.compatible=true;check(Query.matches(e,good,v,777),"compatible protocol");check(!Query.matches(e,good,v,778),"incompatible protocol");
        v.filter=new Filter();v.tab=Tab.FAVORITES;check(!Query.matches(e,good,v,777),"favorites filter");e.meta.favorite=true;check(Query.matches(e,good,v,777),"favorite included");
        Entry unknown=new Entry("Unknown","unknown.example.org");Map<String,Status> statuses=Map.of(e.key(),good);v=new View();v.sort=Sort.PING;
        check(Query.select(List.of(unknown,e),statuses,v,777).getLast()==unknown,"unknown last ascending");v.descending=true;check(Query.select(List.of(unknown,e),statuses,v,777).getLast()==unknown,"unknown last descending");
        v.sort=Sort.CUSTOM;check(Query.select(List.of(unknown,e),statuses,v,777).getFirst()==unknown,"stable custom order");
    }
    private static void json(Path root)throws Exception {
        Files.createDirectories(root);Path file=root.resolve("state.json");
        Files.writeString(file,"{\"schema\":1,\"future\":{\"keep\":true},\"settings\":{\"onlineDirectory\":false},\"presets\":{\"delete-me\":{}}}");
        JsonStore store=new JsonStore(file);State state=store.load();check(!state.settings.onlineDirectory,"explicit Off persists");state.presets.remove("delete-me");store.save(state);
        JsonObject raw=JsonParser.parseString(Files.readString(file)).getAsJsonObject();check(raw.getAsJsonObject("future").get("keep").getAsBoolean(),"unknown JSON retained");check(!raw.getAsJsonObject("presets").has("delete-me"),"profile deletion persists");check(Files.exists(root.resolve("state.json.bak")),"backup written");
        Files.writeString(file,"{ broken");byte[] broken=Files.readAllBytes(file);rejects(()->new JsonStore(file).load(),"corrupt config rejected");check(Arrays.equals(broken,Files.readAllBytes(file)),"corrupt original unchanged");
        state.settings.concurrentPings=200;state.settings.refreshSeconds=2;state.settings.sanitize();check(state.settings.concurrentPings==16&&state.settings.refreshSeconds==15,"settings bounds");
    }
    private static final byte[] MODRINTH=("{\"total_hits\":101,\"hits\":[{\"project_types\":[\"minecraft_java_server\"],\"name\":\"Test\",\"minecraft_java_server\":{\"address\":\"PLAY.EXAMPLE.ORG:25565\"},\"categories\":[\"survival\"]},{\"project_types\":[\"minecraft_java_server\"],\"minecraft_java_server\":{\"address\":\"play.example.org\"}},{\"project_types\":[\"minecraft_java_server\"],\"minecraft_java_server\":{\"address\":\"127.0.0.1\"}},{\"project_types\":[\"mod\"],\"minecraft_java_server\":{\"address\":\"wrong.example.org\"}}]}").getBytes(StandardCharsets.UTF_8);
    private static void directories(Path root)throws Exception {
        Directory.Page parsed=Directory.parse(Directory.Source.MODRINTH,MODRINTH);check(parsed.entries().size()==1&&parsed.pages()==3,"Modrinth dedup/private/type validation");
        check(parsed.entries().getFirst().meta.auth.equals("unknown"),"listing does not invent authentication");
        var mc=Directory.parse(Directory.Source.MINECRAFTSERVERS,"{\"pages\":2,\"servers\":[{\"edition\":\"java\",\"host\":\"mc.example.org\",\"port\":25565,\"name\":\"Server\"},{\"edition\":\"bedrock\",\"host\":\"wrong.example.org\"}]}".getBytes(StandardCharsets.UTF_8));
        check(mc.entries().size()==1&&mc.pages()==2,"minecraftserve.rs actual response shape");
        rejects(()->Directory.parse(Directory.Source.MODRINTH,"{\"hits\":null}".getBytes()),"bad schema rejected");
        rejects(()->Directory.parse(Directory.Source.MODRINTH,new byte[JsonStore.MAX_FILE+1]),"bounded response");
        AtomicLong time=new AtomicLong(1000);AtomicInteger calls=new AtomicInteger();
        Directory.Transport transport=uri->{calls.incrementAndGet();return uri.getHost().equals("api.modrinth.com")?MODRINTH:"{\"pages\":1,\"servers\":[]}".getBytes(StandardCharsets.UTF_8);};Directory d=new Directory(root,transport,time::get);
        d.page(Directory.Source.MODRINTH,1,true);d.page(Directory.Source.MODRINTH,1,true);check(calls.get()==1,"fresh cache avoids HTTP");
        rejects(()->new Directory(root,transport,time::get).page(Directory.Source.MODRINTH,2,true),"rate survives restart");check(calls.get()==1,"rate blocks transport");
        rejects(()->d.page(Directory.Source.MODRINTH,1,false),"Off blocks even cached discovery");
        d.page(Directory.Source.MINECRAFTSERVERS,1,true);check(calls.get()==2,"providers have independent rate limits");
        time.set(1400);Directory stale=new Directory(root,uri->{throw new IOException("offline fixture");},time::get);
        check(stale.page(Directory.Source.MODRINTH,1,true).stale(),"expired good cache survives network failure");
        Path throttled=root.resolve("throttled");AtomicInteger throttledCalls=new AtomicInteger();
        Directory limited=new Directory(throttled,uri->{throttledCalls.incrementAndGet();throw new Directory.Limited(300);},time::get);
        rejects(()->limited.page(Directory.Source.MODRINTH,1,true),"HTTP 429 retained");time.addAndGet(61);
        rejects(()->new Directory(throttled,uri->{throttledCalls.incrementAndGet();return MODRINTH;},time::get).page(Directory.Source.MODRINTH,1,true),"Retry-After survives restart");check(throttledCalls.get()==1,"Retry-After prevents another HTTP request");
    }
    private static CompoundTag entry(String name,String ip,boolean hidden){CompoundTag t=new CompoundTag();t.putString("name",name);t.putString("ip",ip);t.putBoolean("hidden",hidden);t.putString("futureField","keep");t.putBoolean("acceptTextures",true);return t;}
    private static void nbt(Path root)throws Exception {
        Files.createDirectories(root);Path file=root.resolve("servers.dat");CompoundTag original=new CompoundTag();original.putString("futureRoot","🧱 Grüß dich");ListTag rows=new ListTag();rows.add(entry("One","one.example.org",false));rows.add(entry("Hidden","hidden.example.org",true));rows.add(entry("Two","two.example.org",false));original.put("servers",rows);NbtIo.write(original,file);byte[] before=Files.readAllBytes(file);
        DatRepository dat=new DatRepository(file);check(dat.list().size()==2,"hidden server remains hidden");
        dat.upsert(new Entry("Renamed","one.example.org"),"one.example.org");CompoundTag changed=NbtIo.read(file);check(changed.getStringOr("futureRoot","").equals("🧱 Grüß dich"),"unknown UTF16 root retained");check(changed.getListOrEmpty("servers").getCompoundOrEmpty(0).getStringOr("futureField","").equals("keep"),"unknown entry fields retained");check(changed.getListOrEmpty("servers").getCompoundOrEmpty(0).getBooleanOr("acceptTextures",false),"resource pack setting retained");
        dat.undo();check(Arrays.equals(before,Files.readAllBytes(file)),"undo restores exact original bytes");
        dat.move("two.example.org",-1);check(dat.list().getFirst().name.equals("Two"),"custom ordering");check(NbtIo.read(file).getListOrEmpty("servers").getCompoundOrEmpty(1).getBooleanOr("hidden",false),"moving skips hidden entries");
        dat.delete("two.example.org");check(dat.list().size()==1,"delete selected only");dat.undo();check(dat.list().size()==2,"undo delete");
        rejects(()->dat.upsert(new Entry("Duplicate","one.example.org"),"two.example.org"),"duplicate edit rejected");
        dat.delete("two.example.org");Files.write(file,before);rejects(dat::undo,"concurrent external edit stops undo");
        Files.writeString(file,"broken NBT");byte[] broken=Files.readAllBytes(file);rejects(dat::list,"invalid NBT rejected");check(Arrays.equals(broken,Files.readAllBytes(file)),"invalid NBT retained");
    }
    private static void services(Path root)throws Exception {
        ClientServices services=new ClientServices(root);Entry e=new Entry("Saved","saved.example.org");e.meta.notes="Private note";services.save(e,null);services.favorite(e);services.played(e);
        ClientServices reopened=new ClientServices(root);check(reopened.entries(Tab.SAVED).getFirst().meta.favorite,"favorite persisted");check(reopened.entries(Tab.RECENT).getFirst().meta.notes.equals("Private note"),"recent metadata persisted");
        services.state.settings.onlineDirectory=false;services.persist();check(!new ClientServices(root).state.settings.onlineDirectory,"discovery Off survives reopening");
        Path draft=services.draft("feedback","A test message","");check(Files.isRegularFile(draft),"local feedback draft");
        check(Files.list(root.resolve("config/server-browser/drafts")).count()==1,"one explicit draft");
    }
    private static void wire()throws Exception {
        try(ServerSocket peer=new ServerSocket(0,1,InetAddress.getLoopbackAddress())) {
            ExecutorService worker=Executors.newSingleThreadExecutor();
            Future<?> server=worker.submit(()->{try(Socket s=peer.accept()) {
                s.setSoTimeout(5000);DataInputStream in=new DataInputStream(s.getInputStream());DataOutputStream out=new DataOutputStream(s.getOutputStream());
                DataInputStream handshake=new DataInputStream(new ByteArrayInputStream(packet(in)));check(readVar(handshake)==0&&readVar(handshake)==777,"wire protocol handshake");int host=readVar(handshake);handshake.skipNBytes(host);check(handshake.readUnsignedShort()==peer.getLocalPort()&&readVar(handshake)==1,"wire port and status state");check(packet(in)[0]==0,"status request");
                String json="{\"version\":{\"name\":\"26.3\",\"protocol\":777},\"players\":{\"online\":12,\"max\":20},\"description\":{\"text\":\"Fixture \",\"extra\":[{\"text\":\"MOTD\"}]}}";
                ByteArrayOutputStream b=new ByteArrayOutputStream();DataOutputStream data=new DataOutputStream(b);data.writeByte(0);byte[] body=json.getBytes(StandardCharsets.UTF_8);writeVar(data,body.length);data.write(body);writeVar(out,b.size());out.write(b.toByteArray());out.flush();
                byte[] ping=packet(in);writeVar(out,ping.length);out.write(ping);out.flush();
            }catch(Exception ex){throw new RuntimeException(ex);}});
            Status result=StatusProbe.ping("127.0.0.1:"+peer.getLocalPort(),777,a->new InetSocketAddress(InetAddress.getLoopbackAddress(),a.port()));
            server.get(10,TimeUnit.SECONDS);worker.shutdownNow();
            check(result.health()==Health.ONLINE&&result.players()==12&&result.max()==20,"actual local status response");check(result.ping()!=null&&result.motd().equals("Fixture MOTD")&&result.compatible(777),"ping echo and structured MOTD");
        }
        Status failed=StatusProbe.ping("127.0.0.1:1",777,a->{throw new IOException("fixture DNS failure");});check(failed.health()==Health.OFFLINE,"DNS failure becomes offline");
    }
    private static void transfers(Path root)throws Exception {
        ClientServices source=new ClientServices(root.resolve("source"));Entry e=new Entry("Exported","export.example.org");e.meta.notes="Original note 🧱";e.meta.tags.add("survival");e.meta.favorite=true;e.packs="off";e.icon=Base64.getEncoder().encodeToString(new byte[]{1,2,3});source.save(e,null);
        Path json=root.resolve("export.json");Transfers.exportJson(source,json);
        ClientServices target=new ClientServices(root.resolve("target"));check(Transfers.importJson(target,json)==1,"JSON import count");
        Entry imported=target.entries(Tab.SAVED).getFirst();check(imported.meta.notes.equals(e.meta.notes)&&imported.meta.favorite&&imported.packs.equals("off"),"JSON round trip metadata and resource packs");
        check(e.icon.equals(imported.icon),"saved icon retained through JSON transfer");
        imported.meta.notes="Do not overwrite";target.save(imported,imported.key());check(Transfers.importJson(target,json)==0&&target.entries(Tab.SAVED).getFirst().meta.notes.equals("Do not overwrite"),"duplicate import preserves existing metadata");
        byte[] before=target.saved.raw();Path invalid=root.resolve("invalid.json");Files.writeString(invalid,"{\"schema\":1,\"servers\":[{\"name\":\"First\",\"address\":\"first.example.org\"},{\"name\":\"Bad\",\"address\":\"https://invalid\"}]}");
        rejects(()->Transfers.importJson(target,invalid),"invalid later row rejects whole import");check(Arrays.equals(before,target.saved.raw()),"failed import writes no earlier rows");
        rejects(()->Transfers.exportJson(source,root.resolve("source/servers.dat")),"cannot export over game server file");rejects(()->Transfers.exportNbt(source,source.home.resolve("state-v1.json")),"cannot export over own configuration");
        Path nbt=root.resolve("export.dat");Transfers.exportNbt(source,nbt);check(Arrays.equals(source.saved.raw(),Files.readAllBytes(nbt)),"NBT export retains exact bytes");
        ClientServices nbtTarget=new ClientServices(root.resolve("nbt-target"));check(Transfers.importNbt(nbtTarget,nbt)==1&&nbtTarget.saved.list().getFirst().packs.equals("off"),"NBT import uses actual server format");
        Path horizon=root.resolve("horizon.json");Files.writeString(horizon,"[{\"name\":\"Horizon\",\"address\":\"horizon.example.org\",\"packs\":false,\"metadata\":{\"notes\":\"From Horizon\",\"favorite\":true,\"last_played\":123}}]");
        check(Transfers.importJson(nbtTarget,horizon)==1,"Horizon native JSON array accepted");Entry fromHorizon=nbtTarget.entries(Tab.SAVED).stream().filter(a->a.address.equals("horizon.example.org")).findFirst().orElseThrow();check(fromHorizon.meta.favorite&&fromHorizon.meta.lastPlayed==123&&fromHorizon.packs.equals("off"),"Horizon metadata and packs retained");
        rejects(()->Transfers.importNbt(nbtTarget,root.resolve("missing.dat")),"missing NBT import rejected");
        Path readOnlyRoot=root.resolve("read-only");Files.createDirectories(readOnlyRoot.resolve("config/server-browser"));Files.writeString(readOnlyRoot.resolve("config/server-browser/state-v1.json"),"broken");ClientServices readOnly=new ClientServices(readOnlyRoot);rejects(()->Transfers.importJson(readOnly,json),"read-only recovery prevents imported writes");check(!Files.exists(readOnlyRoot.resolve("servers.dat")),"read-only import creates no game file");
    }
    private static int readVar(DataInputStream in)throws IOException{int v=0;for(int i=0;i<5;i++){int b=in.readUnsignedByte();v|=(b&127)<<(i*7);if((b&128)==0)return v;}throw new IOException();}
    private static void writeVar(DataOutputStream out,int n)throws IOException{do{int b=n&127;n>>>=7;out.writeByte(n==0?b:b|128);}while(n!=0);}
    private static byte[] packet(DataInputStream in)throws IOException{int n=readVar(in);if(n<0||n>1024*1024)throw new IOException();return in.readNBytes(n);}
}
