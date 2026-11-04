package io.github.musti3214.serverbrowser.client;

import io.github.musti3214.serverbrowser.core.*;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ServerData;
import net.minecraft.client.multiplayer.resolver.*;
import net.minecraft.SharedConstants;
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicLong;
import static io.github.musti3214.serverbrowser.core.Model.*;

public final class ClientServices {
    private static ClientServices instance;
    public static synchronized ClientServices get() {if(instance==null)instance=new ClientServices(Minecraft.getInstance().gameDirectory.toPath());return instance;}
    public final State state;
    public final Path home;
    public final DatRepository saved;
    public final Map<String,Status> statuses=new ConcurrentHashMap<>();
    public final AtomicLong revision=new AtomicLong();
    private final Map<String,Long> checked=new ConcurrentHashMap<>();
    private final Map<String,Deque<Long>> pingHistory=new ConcurrentHashMap<>();
    private final Set<String> inFlight=ConcurrentHashMap.newKeySet();
    private final JsonStore store;
    private boolean writable=true;
    public volatile String error="";
    public volatile boolean directoryBusy;
    public volatile boolean directoryPartial;
    public volatile int receivedPages;
    private final Directory directory;
    private final List<Entry> discovered=new ArrayList<>();
    private final int[] nextPage={1,1},lastPage={200,200};
    private long nextDirectoryCheck;
    private final ThreadPoolExecutor pingPool;
    private final ExecutorService directoryPool=Executors.newSingleThreadExecutor(r->{Thread t=new Thread(r,"server-browser-directory");t.setDaemon(true);return t;});
    public ClientServices(Path game) {
        home=game.resolve("config/server-browser");saved=new DatRepository(game.resolve("servers.dat"));store=new JsonStore(home.resolve("state-v1.json"));
        State loaded;
        try {loaded=store.load();}catch(IOException ex){loaded=new State();writable=false;error=ex.getMessage();}
        state=loaded;
        directory=new Directory(home.resolve("directory"));
        pingPool=new ThreadPoolExecutor(1,16,30,TimeUnit.SECONDS,new ArrayBlockingQueue<>(64),r->{Thread t=new Thread(r,"server-browser-ping");t.setDaemon(true);return t;},new ThreadPoolExecutor.AbortPolicy());
        pingPool.setCorePoolSize(state.settings.concurrentPings);pingPool.setMaximumPoolSize(state.settings.concurrentPings);
        try {discovered.addAll(Directory.bundled());}catch(IOException ex){error=ex.getMessage();}
    }
    public boolean writable() {return writable;}
    public void configurePings() {
        int n=state.settings.concurrentPings;
        if(n>pingPool.getMaximumPoolSize()){pingPool.setMaximumPoolSize(n);pingPool.setCorePoolSize(n);}
        else {pingPool.setCorePoolSize(n);pingPool.setMaximumPoolSize(n);}
    }
    public void persist() throws IOException {
        if(!writable)throw new IOException("Configuration could not be read. Original retained; repair state-v1.json before editing.");
        state.settings.sanitize();store.save(state);revision.incrementAndGet();
    }
    public List<Entry> entries(Tab tab) {
        List<Entry> entries;
        try {
            if(tab==Tab.DISCOVER) {synchronized(discovered){entries=new ArrayList<>(discovered);}}
            else if(tab==Tab.RECENT)entries=new ArrayList<>(state.recent);
            else entries=saved.list();
            if(tab==Tab.FAVORITES) {
                Map<String,Entry> combined=new LinkedHashMap<>();for(Entry e:entries)combined.put(e.key(),e);
                synchronized(discovered){for(Entry e:discovered)if(state.metadata.containsKey(e.key()))combined.putIfAbsent(e.key(),e);}
                for(Entry e:state.recent)combined.putIfAbsent(e.key(),e);entries=new ArrayList<>(combined.values());
            }
        } catch(IOException ex){error=ex.getMessage();return List.of();}
        for(Entry e:entries){Meta local=state.metadata.get(e.key());if(local!=null)e.meta=local;}
        return entries;
    }
    public void save(Entry entry,String oldKey) throws IOException {
        if(!writable)throw new IOException("Configuration is read-only");
        saved.upsert(entry,oldKey);
        if(oldKey!=null&&!entry.key().equals(oldKey))state.metadata.remove(oldKey);
        state.metadata.put(entry.key(),entry.meta);persist();
    }
    public void favorite(Entry e) throws IOException {if(!writable)throw new IOException("Configuration is read-only");e.meta.favorite=!e.meta.favorite;state.metadata.put(e.key(),e.meta);persist();}
    public void played(Entry e) throws IOException {
        e.meta.lastPlayed=System.currentTimeMillis()/1000;state.metadata.put(e.key(),e.meta);
        state.recent.removeIf(r->r.key().equals(e.key()));state.recent.addFirst(e);while(state.recent.size()>100)state.recent.removeLast();persist();
    }
    public ServerData serverData(Entry e,boolean lan) {
        ServerData data=new ServerData(e.name,e.address,lan?ServerData.Type.LAN:ServerData.Type.OTHER);
        try {var tag=saved.serverTag(e.key());if(tag!=null){data=ServerData.read(tag);data.name=e.name;data.ip=e.address;}}catch(IOException ex){error=ex.getMessage();}
        return data;
    }
    public void ping(List<Entry> entries) {
        long now=System.currentTimeMillis();int ttl=state.settings.refreshSeconds==0?Integer.MAX_VALUE:state.settings.refreshSeconds*1000;
        for(Entry e:entries) {
            String key=e.key();if(checked.getOrDefault(key,0L)+ttl>now||!inFlight.add(key))continue;
            try {
                pingPool.execute(()->{
                    statuses.put(key,Status.pinging());revision.incrementAndGet();
                    Status status=StatusProbe.ping(e.address,protocol(),address->ServerNameResolver.DEFAULT.resolveAddress(new ServerAddress(address.host(),address.port())).orElseThrow(()->new IOException("Unknown server")).asInetSocketAddress());
                    statuses.put(key,status);if(status.ping()!=null){Deque<Long> values=pingHistory.computeIfAbsent(key,k->new ArrayDeque<>());synchronized(values){values.addLast(status.ping());while(values.size()>32)values.removeFirst();}}
                    checked.put(key,System.currentTimeMillis());inFlight.remove(key);revision.incrementAndGet();
                });
            } catch(RejectedExecutionException ex){inFlight.remove(key);}
        }
    }
    public List<Long> history(String key){Deque<Long> values=pingHistory.get(key);if(values==null)return List.of();synchronized(values){return List.copyOf(values);}}
    public void refresh() {checked.clear();nextDirectoryCheck=0;revision.incrementAndGet();}
    public static int protocol() {return SharedConstants.getProtocolVersion();}
    public void discover() {
        long now=System.currentTimeMillis()/1000;
        if(!state.settings.onlineDirectory||directoryBusy||now<nextDirectoryCheck)return;
        directoryBusy=true;nextDirectoryCheck=now+60;
        directoryPool.execute(()->{
            int succeeded=0,attempted=0;
            for(Directory.Source source:Directory.Source.values()) {
                if(!state.settings.onlineDirectory)break;
                int i=source.ordinal();if(nextPage[i]>lastPage[i])continue;attempted++;
                try {
                    Directory.Page page=directory.page(source,nextPage[i],state.settings.onlineDirectory);
                    synchronized(discovered) {
                        Set<String> keys=new HashSet<>();for(Entry e:discovered)keys.add(e.key());
                        for(Entry e:page.entries())if(keys.add(e.key()))discovered.add(e);
                    }
                    lastPage[i]=page.pages();nextPage[i]++;receivedPages++;succeeded++;
                    directoryPartial=page.stale();
                } catch(InterruptedException ex){Thread.currentThread().interrupt();break;}catch(IOException ex){directoryPartial=true;}
            }
            if(attempted>0&&succeeded<attempted)directoryPartial=true;
            directoryBusy=false;revision.incrementAndGet();
        });
    }
    public Path draft(String kind,String text,String address) throws IOException {
        if(text.isBlank())throw new IOException("empty message");
        var object=new com.google.gson.JsonObject();object.addProperty("category",kind);object.addProperty("message",text);object.addProperty("address",address);object.addProperty("version","2.2.0");
        Path file=home.resolve("drafts").resolve("draft-"+System.currentTimeMillis()+"-"+UUID.randomUUID()+".json");
        JsonStore.atomic(file,JsonStore.GSON.toJson(object).getBytes(StandardCharsets.UTF_8),false);return file;
    }
}
