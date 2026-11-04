package io.github.musti3214.serverbrowser.client;

import com.mojang.blaze3d.platform.NativeImage;
import io.github.musti3214.serverbrowser.core.*;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.*;
import net.minecraft.client.gui.screens.*;
import net.minecraft.client.gui.screens.multiplayer.JoinMultiplayerScreen;
import net.minecraft.client.input.*;
import net.minecraft.client.multiplayer.resolver.ServerAddress;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.client.server.*;
import net.minecraft.network.chat.Component;
import java.io.*;
import java.net.URI;
import java.util.*;
import static io.github.musti3214.serverbrowser.core.Model.*;
import static io.github.musti3214.serverbrowser.client.Text.*;

public final class BrowserScreen extends Screen {
    private static final Set<JoinMultiplayerScreen> CLASSIC=Collections.newSetFromMap(new WeakHashMap<>());
    public static boolean isClassic(JoinMultiplayerScreen screen){return CLASSIC.contains(screen);}
    private final Screen parent;
    private final JoinMultiplayerScreen vanilla;
    public final ClientServices services;
    private Rows rows;
    private EditBox search;
    private int left,area,listWidth,listTop,listBottom,viewport;
    private List<Entry> source=List.of(),filtered=List.of();
    private String selectedKey="",notice="";
    private long revision=-1,nextUpdate,dirtySince;
    private LanServerDetection.LanServerList lanList;
    private LanServerDetection.LanServerDetector lanDetector;
    private final List<Entry> lanEntries=new ArrayList<>();
    private final Map<String,FaviconTexture> icons=new LinkedHashMap<>(32,0.75f,true);
    private final Map<String,String> iconData=new HashMap<>();
    private final List<Button> selectedButtons=new ArrayList<>();
    private final List<Button> contextButtons=new ArrayList<>();
    public BrowserScreen(Screen parent,JoinMultiplayerScreen vanilla){super(Component.literal("Server Atlas"));this.parent=parent;this.vanilla=vanilla;this.services=ClientServices.get();}
    private View view(){return services.state.view;}
    @Override protected void init() {
        selectedButtons.clear();contextButtons.clear();viewport=Math.min(740,width);area=viewport-24;left=(width-viewport)/2+12;listWidth=viewport>=720?viewport-256:area;
        int x=left+area;
        int sw=Math.max(88,font.width(t("settings"))+12);
        button("settings",x-sw,3,sw,()->settings());
        button("classic",x-sw-59,3,55,()->{CLASSIC.add(vanilla);minecraft.gui.setScreen(vanilla);});
        if(!services.state.settings.supportDismissed&&viewport>=360) {
            addRenderableWidget(Button.builder(Component.literal("×"),b->{services.state.settings.supportDismissed=true;safe(services::persist);rebuildWidgets();}).bounds(x-sw-81,3,18,20).build());
            button("feedback_icon",x-sw-101,3,18,this::feedback);
            int kofi=viewport<440?28:62;
            addRenderableWidget(Button.builder(Component.literal(viewport<440?"☕":"☕ Ko-fi"),ConfirmLinkScreen.confirmLink(this,URI.create("https://ko-fi.com/foryou41"))).bounds(x-sw-105-kofi,3,kofi,20).build());
        } else button("feedback_icon",x-sw-81,3,18,this::feedback);
        search=addRenderableWidget(new EditBox(font,left,27,area-58,20,t("search")));search.setMaxLength(120);search.setHint(t("search"));search.setValue(view().search);
        search.setResponder(value->{view().search=value;dirtySince=System.currentTimeMillis();rebuildRows();});
        button("clear",x-54,27,54,()->search.setValue(""));
        Tab[] tabs=Tab.values();int tw=(area-16)/5;
        for(int i=0;i<tabs.length;i++){Tab tab=tabs[i];Button b=button("tab."+tab.name().toLowerCase(Locale.ROOT),left+i*(tw+4),51,i==4?area-4*(tw+4):tw,()->{view().tab=tab;selectedKey="";safe(services::persist);rebuildWidgets();});if(view().tab==tab)b.setMessage(Component.literal("• ").append(t("tab."+tab.name().toLowerCase(Locale.ROOT))));}
        boolean narrow=viewport<420;int fw=narrow?52:64,pw=narrow?64:78,piw=narrow?44:56,nw=narrow?44:56;
        button("filters",left,77,fw,this::filters);
        quickSort("sort.players",Sort.PLAYERS,left+fw+4,pw);
        quickSort("sort.ping",Sort.PING,left+fw+pw+8,piw);
        quickSort("sort.name",Sort.NAME,left+fw+pw+piw+12,nw);
        addRenderableWidget(Button.builder(Component.literal("…"),b->more()).bounds(x-(narrow?56:64),77,narrow?26:30,20).build());
        addRenderableWidget(Button.builder(Component.literal("↺"),b->reset()).bounds(x-(narrow?26:30),77,narrow?26:30,20).build());
        listTop=view().tab==Tab.DISCOVER?117:103;listBottom=Math.max(listTop+1,height-(view().tab==Tab.DISCOVER?57:81));
        rows=addRenderableWidget(new Rows(minecraft,listWidth,listBottom-listTop,listTop,60));rows.setX(left);
        source=source();rebuildRows();
        int third=(area-8)/3;
        selectionButton("join",left,height-52,third,()->connect(selected()));
        button("direct",left+third+4,height-52,third,this::direct);
        button("add",left+2*(third+4),height-52,area-2*(third+4),()->edit(view().tab==Tab.DISCOVER?selected():null));
        int fourth=(area-12)/4;
        selectionButton(view().tab==Tab.DISCOVER?"info":"edit",left,height-28,fourth,()->{if(view().tab==Tab.DISCOVER)info();else edit(selected());});
        selectionButton("delete",left+fourth+4,height-28,fourth,this::remove);
        button("refresh",left+2*(fourth+4),height-28,fourth,this::refresh);
        button("back",left+3*(fourth+4),height-28,area-3*(fourth+4),this::onClose);
        if(view().tab!=Tab.DISCOVER) {
            String[] keys={"favorite","tags_short","move_up_short","move_down_short","info","reset","undo_short"};
            int[] sizes={40,40,20,20,40,48,48};int total=18;
            for(int i=0;i<sizes.length;i++){sizes[i]=Math.max(sizes[i],font.width(t(keys[i]))+10);total+=sizes[i];}
            double factor=Math.min(1.0,area/(double)total);int bx=left;
            for(int i=0;i<keys.length;i++){final int action=i;int w=(int)(sizes[i]*factor);Button b=button(keys[i],bx,height-77,w,()->{
                Entry e=selected();switch(action){case 0->{if(e!=null)safe(()->services.favorite(e));rebuildRows();}case 1->{if(e!=null)edit(e);}case 2,3->{if(sourceIsSaved(e))safe(()->services.saved.move(e.key(),action==2?-1:1));view().sort=Sort.CUSTOM;safe(services::persist);source=source();rebuildRows();}case 4->info();case 5->reset();case 6->{safe(services.saved::undo);source=source();rebuildRows();}}
            });contextButtons.add(b);bx+=w+3;}
        }
        updateButtons();if(view().tab==Tab.DISCOVER)services.discover();if(view().tab==Tab.LAN)startLan();else stopLan();
    }
    private void quickSort(String label,Sort sort,int x,int w){Button b=button(label,x,77,w,()->{if(view().sort==sort)view().descending=!view().descending;else{view().sort=sort;view().descending=sort==Sort.PLAYERS;}safe(services::persist);rebuildWidgets();});if(view().sort==sort)b.setMessage(Component.literal("• ").append(t(label)).append(view().descending?" ▼":" ▲"));}
    private void reset(){view().filter=new Filter();view().search="";safe(services::persist);rebuildWidgets();}
    private void refresh(){services.refresh();if(view().tab==Tab.DISCOVER)services.discover();rebuildRows();}
    private void info(){
        Entry e=selected();if(e==null)return;
        Status status=services.statuses.getOrDefault(e.key(),Status.unknown());
        List<Component> lines=new ArrayList<>();lines.add(Component.literal(e.address));lines.add(t("health."+status.health().name().toLowerCase(Locale.ROOT)));
        if(!status.version().isBlank())lines.add(t("version").copy().append(": "+status.version()+(status.protocol()==null?"":" / "+status.protocol())));
        if(status.players()!=null)lines.add(t("players_value",status.players(),status.max()==null?"?":status.max()));
        if(status.ping()!=null)lines.add(Component.literal(status.ping()+" ms"));
        lines.add(t("auth").copy().append(": ").append(t("value."+e.meta.auth)));
        if(!status.motd().isBlank())lines.add(Text.motd(status));
        if(!status.sampleNames().isEmpty())lines.add(Component.literal(String.join(", ",status.sampleNames())));
        for(var field:List.of(Map.entry("tags",String.join(", ",e.meta.tags)),Map.entry("group",e.meta.group),Map.entry("language",e.meta.language),Map.entry("region",e.meta.region),Map.entry("notes",e.meta.notes)))if(!field.getValue().isBlank())lines.add(t(field.getKey()).copy().append(": "+field.getValue()));
        minecraft.gui.setScreen(new Dialogs.Info(this,Component.literal(e.name),lines,List.of(new Dialogs.Action(t("edit"),()->edit(e)),new Dialogs.Action(t("copy_address"),()->{minecraft.keyboardHandler.setClipboard(e.address);notice=s("copied");}))));
    }
    private Button button(String key,int x,int y,int w,Runnable run){return addRenderableWidget(Button.builder(t(key),b->run.run()).bounds(x,y,w,20).build());}
    private void selectionButton(String key,int x,int y,int w,Runnable run){selectedButtons.add(button(key,x,y,w,run));}
    private void updateButtons(){Entry e=selected();for(Button b:selectedButtons)b.active=e!=null;if(selectedButtons.size()>2)selectedButtons.get(2).active=e!=null&&view().tab!=Tab.LAN&&sourceIsSaved(e);for(int i=0;i<contextButtons.size();i++)contextButtons.get(i).active=i==5||i==6&&services.saved.canUndo()||i<5&&e!=null&&(i<2||i==4||sourceIsSaved(e));}
    private boolean sourceIsSaved(Entry e){try{return e!=null&&services.saved.list().stream().anyMatch(s->s.key().equals(e.key()));}catch(IOException ex){return false;}}
    private Entry selected(){for(Entry e:filtered)if(e.key().equals(selectedKey))return e;return null;}
    private List<Entry> source(){
        if(view().tab==Tab.LAN)return new ArrayList<>(lanEntries);
        return services.entries(view().tab);
    }
    public void rebuildRows() {
        if(rows==null)return;
        double scroll=rows.scrollAmount();
        filtered=Query.select(source,services.statuses,view(),ClientServices.protocol());
        if(!services.state.settings.showOffline)filtered=filtered.stream().filter(e->services.statuses.getOrDefault(e.key(),Status.unknown()).health()!=Health.OFFLINE).toList();
        rows.populate(filtered);rows.setScrollAmount(scroll);updateButtons();
    }
    private void safe(IoAction action){try{action.run();notice="";}catch(Exception ex){notice=s("save_failed");services.error=ex.getMessage()==null?"":ex.getMessage();}}
    @FunctionalInterface private interface IoAction{void run()throws Exception;}
    @Override public void tick() {
        long now=System.currentTimeMillis();
        if(now>=nextUpdate){nextUpdate=now+200;if(revision!=services.revision.get()){revision=services.revision.get();source=source();rebuildRows();}}
        if(dirtySince!=0&&now-dirtySince>500){dirtySince=0;safe(services::persist);}
        if(view().tab==Tab.DISCOVER)services.discover();
        if(view().tab==Tab.LAN&&lanList!=null){var update=lanList.takeDirtyServers();if(update!=null){lanEntries.clear();for(LanServer server:update){try{Entry e=new Entry(server.getMotd(),server.getAddress());e.meta.group="LAN";lanEntries.add(e);}catch(IllegalArgumentException ignored){}}source=source();rebuildRows();}}
    }
    @Override public void extractRenderState(GuiGraphicsExtractor g,int mx,int my,float dt) {
        g.fill(left-4,0,left+area+4,103,0x5E000000);g.fill(left-4,listBottom,left+area+4,height,0x6A000000);
        super.extractRenderState(g,mx,my,dt);
        int titleWidth=Math.max(60,area-330);g.text(font,font.plainSubstrByWidth(title.getString(),titleWidth),left,9,0xFFFFFFFF);
        String status=t("count",filtered.size(),source.size()).getString();
        if(!notice.isEmpty())status=notice;
        else if(!services.error.isEmpty())status=s("local_error");
        else if(view().tab==Tab.DISCOVER)status+=(services.state.settings.onlineDirectory?" · "+s(services.directoryBusy?"loading":services.directoryPartial?"partial":"sources"):" · "+s("directory_off"));
        if(viewport>620)g.text(font,t("count",filtered.size(),source.size()),(width-font.width(t("count",filtered.size(),source.size())))/2,8,0xFFBFBFBF);
        if(view().tab==Tab.DISCOVER)g.text(font,font.plainSubstrByWidth(status,area),left,105,0xFFBFBFBF);
        else if(!notice.isEmpty()||!services.error.isEmpty())g.text(font,font.plainSubstrByWidth(status,area),left,listBottom-11,0xFFFFAAAA);
        if(filtered.isEmpty())g.text(font,font.plainSubstrByWidth(s(view().tab==Tab.LAN?"lan_wait":"empty"),listWidth-12),left+6,132,0xFFBFBFBF);
        if(listWidth<area)details(g);
        List<Entry> visible=new ArrayList<>();for(Row row:rows.children())if(row.getY()+row.getHeight()>rows.getY()&&row.getY()<rows.getBottom())visible.add(row.entry);
        services.ping(visible);
    }
    private void details(GuiGraphicsExtractor g) {
        int x=left+listWidth+20,w=area-listWidth-20,y=listTop+8;
        Entry e=selected();if(e==null){g.text(font,font.plainSubstrByWidth(s("select_server"),w),x,y,0xFFBFBFBF);return;}
        g.fill(x-8,listTop,left+area,listBottom,0x5C000000);
        List<Component> lines=new ArrayList<>();lines.add(Component.literal(e.name));lines.add(Component.literal(e.address));
        Status s=services.statuses.getOrDefault(e.key(),Status.unknown());
        lines.add(t("health."+s.health().name().toLowerCase(Locale.ROOT)));lines.add(Component.literal(s.version()));
        if(s.players()!=null)lines.add(t("players_value",s.players(),s.max()==null?"?":s.max()));
        if(s.ping()!=null)lines.add(Component.literal(s.ping()+" ms"));lines.add(t("value."+e.meta.auth));
        if(!s.motd().isBlank())lines.add(Text.motd(s));
        if(!s.sampleNames().isEmpty())lines.add(Component.literal(String.join(", ",s.sampleNames())));
        if(!e.meta.tags.isEmpty())lines.add(Component.literal(String.join(", ",e.meta.tags)));
        if(!e.meta.group.isBlank())lines.add(t("group").copy().append(": "+e.meta.group));
        if(!e.meta.notes.isBlank())lines.add(Component.literal(e.meta.notes));
        for(Component line:lines){for(var part:font.split(line,w-8)){if(y>listBottom-46)break;g.text(font,part,x,y,0xFFDDDDDD);y+=11;}y+=2;}
        g.text(font,t("ping_history"),x,listBottom-37,0xFFAAAAAA);List<Long> history=services.history(e.key());if(!history.isEmpty()){long max=Math.max(1,Collections.max(history));for(int i=0;i<history.size();i++){int bx=x+i*Math.max(1,(w-8)/32),bar=(int)(history.get(i)*22/max);g.fill(bx,listBottom-5-bar,bx+Math.max(1,(w-8)/32-1),listBottom-5,0xFF55AA55);}}
    }
    private void connect(Entry e) {
        if(e==null)return;
        safe(()->services.played(e));
        ConnectScreen.startConnecting(this,minecraft,ServerAddress.parseString(e.address),services.serverData(e,view().tab==Tab.LAN),false,null);
    }
    private void direct() {
        minecraft.gui.setScreen(new Dialogs.Form(this,t("direct"),List.of(new Dialogs.Field("address","",255)),"",v->{Entry e=new Entry(s("direct"),v.get("address"));connect(e);}));
    }
    private void edit(Entry existing) {
        Entry original=existing==null?new Entry():existing;
        boolean saved=sourceIsSaved(existing);Meta meta=JsonStore.GSON.fromJson(JsonStore.GSON.toJson(original.meta),Meta.class);
        List<Dialogs.Field> fields=List.of(new Dialogs.Field("name",original.name,256),new Dialogs.Field("address",original.address,255),new Dialogs.Field("tags",String.join(", ",meta.tags),1024),new Dialogs.Field("notes",meta.notes,4096),new Dialogs.Field("group",meta.group,64),new Dialogs.Field("language",meta.language,32),new Dialogs.Field("region",meta.region,64),new Dialogs.Field("auth",meta.auth,"unknown","online","offline"),new Dialogs.Field("resource_packs",original.packs==null?"ask":original.packs,"ask","on","off"));
        minecraft.gui.setScreen(new Dialogs.Form(this,t(saved?"edit":"add"),fields,s("local_note"),v->{
            if(v.get("name").isBlank())throw new IllegalArgumentException();
            Entry e=new Entry(v.get("name").trim(),v.get("address"));e.meta=meta;e.meta.tags=new ArrayList<>(Arrays.stream(v.get("tags").split(",")).map(String::trim).filter(a->!a.isEmpty()).distinct().limit(32).toList());
            e.meta.notes=v.get("notes");e.meta.group=v.get("group");e.meta.language=v.get("language");e.meta.region=v.get("region");e.meta.auth=v.get("auth");
            e.packs=v.get("resource_packs");services.save(e,saved?original.key():null);selectedKey=e.key();
        },List.of(List.of("name","address","tags","notes"),List.of("group","language","region","auth","resource_packs")),List.of(t("server"),t("details"))));
    }
    private void remove() {
        Entry e=selected();if(!sourceIsSaved(e))return;
        minecraft.gui.setScreen(new ConfirmScreen(yes->{if(yes)safe(()->services.saved.delete(e.key()));minecraft.gui.setScreen(this);},t("delete"),t("delete_confirm",e.name)));
    }
    private void filters() {
        Filter f=view().filter;
        List<Dialogs.Field> fields=List.of(new Dialogs.Field("online",f.online==null?"any":f.online?"online":"offline","any","online","offline"),bool("compatible",f.compatible),new Dialogs.Field("version",f.version,64),new Dialogs.Field("min_players",str(f.minPlayers),9),new Dialogs.Field("max_players",str(f.maxPlayers),9),new Dialogs.Field("max_ping",str(f.maxPing),9),bool("not_empty",f.notEmpty),bool("not_full",f.notFull),bool("favorites_only",f.favorites),new Dialogs.Field("tags",f.tags,1024),new Dialogs.Field("group",f.group,64),new Dialogs.Field("language",f.language,32),new Dialogs.Field("region",f.region,64),new Dialogs.Field("auth",f.auth,"any","unknown","online","offline"));
        minecraft.gui.setScreen(new Dialogs.Form(this,t("filters"),fields,s("and_filters"),v->{
            Filter n=new Filter();n.online=v.get("online").equals("any")?null:v.get("online").equals("online");n.compatible=on(v,"compatible");n.version=v.get("version");n.minPlayers=number(v.get("min_players"));n.maxPlayers=number(v.get("max_players"));Integer ping=number(v.get("max_ping"));n.maxPing=ping==null?null:ping.longValue();
            if(n.minPlayers!=null&&n.maxPlayers!=null&&n.minPlayers>n.maxPlayers)throw new IllegalArgumentException();
            n.notEmpty=on(v,"not_empty");n.notFull=on(v,"not_full");n.favorites=on(v,"favorites_only");n.tags=v.get("tags");n.group=v.get("group");n.language=v.get("language");n.region=v.get("region");n.auth=v.get("auth");view().filter=n;services.persist();
        },List.of(List.of("online","compatible","not_empty","not_full","favorites_only","auth"),List.of("min_players","max_players","max_ping","version"),List.of("tags","language","region","group")),List.of(t("status"),t("values"),t("tags_region"))));
    }
    private static String str(Object v){return v==null?"":v.toString();}
    private static Integer number(String v){if(v.isBlank())return null;int n=Integer.parseInt(v);if(n<0||n>1_000_000)throw new IllegalArgumentException();return n;}
    private static boolean on(Map<String,String> v,String key){return v.get(key).equals("on");}
    private static Dialogs.Field bool(String key,boolean value){return new Dialogs.Field(key,value?"on":"off","off","on");}
    private void sorts() {
        List<Dialogs.Action> actions=new ArrayList<>();for(Sort sort:Sort.values())actions.add(new Dialogs.Action(t("sort."+sort.name().toLowerCase(Locale.ROOT)),()->{view().sort=sort;safe(services::persist);minecraft.gui.setScreen(this);}));
        minecraft.gui.setScreen(new Dialogs.Menu(this,t("sort"),actions,t("unknown_last")));
    }
    private void settings() {
        Settings settings=services.state.settings;
        List<Dialogs.Field> fields=List.of(bool("enabled",settings.enabled),bool("online_directory",settings.onlineDirectory),bool("show_offline",settings.showOffline),new Dialogs.Field("refresh_seconds",str(settings.refreshSeconds),4),new Dialogs.Field("concurrent_pings",str(settings.concurrentPings),2));
        minecraft.gui.setScreen(new Dialogs.Form(this,t("settings"),fields,s("directory_privacy"),v->{settings.enabled=on(v,"enabled");settings.onlineDirectory=on(v,"online_directory");settings.showOffline=on(v,"show_offline");Integer refresh=number(v.get("refresh_seconds")),concurrency=number(v.get("concurrent_pings"));if(refresh==null||concurrency==null)throw new IllegalArgumentException();settings.refreshSeconds=refresh;settings.concurrentPings=concurrency;settings.sanitize();services.persist();services.configurePings();}));
    }
    private void profiles() {
        List<Dialogs.Action> actions=new ArrayList<>();
        actions.add(new Dialogs.Action(t("save_profile"),()->minecraft.gui.setScreen(new Dialogs.Form(this,t("save_profile"),List.of(new Dialogs.Field("profile_name","",48)),"",v->{String name=v.get("profile_name").trim();if(name.isEmpty())throw new IllegalArgumentException();services.state.presets.put(name,JsonStore.GSON.fromJson(JsonStore.GSON.toJson(view()),View.class));services.persist();}))));
        for(String name:new ArrayList<>(services.state.presets.keySet()))actions.add(new Dialogs.Action(Component.literal(name),()->minecraft.gui.setScreen(new Dialogs.Menu(this,Component.literal(name),List.of(
                new Dialogs.Action(t("load_profile"),()->{services.state.view=JsonStore.GSON.fromJson(JsonStore.GSON.toJson(services.state.presets.get(name)),View.class);safe(services::persist);minecraft.gui.setScreen(this);}),
                new Dialogs.Action(t("delete_profile"),()->{services.state.presets.remove(name);safe(services::persist);minecraft.gui.setScreen(this);})
        ),null))));
        minecraft.gui.setScreen(new Dialogs.Menu(this,t("profiles"),actions,null));
    }
    private void more() {
        List<Dialogs.Action> actions=new ArrayList<>();Entry e=selected();
        actions.add(new Dialogs.Action(t("sort"),this::sorts));actions.add(new Dialogs.Action(t("profiles"),this::profiles));
        for(String kind:List.of("import_json","import_nbt","export_json","export_nbt"))actions.add(new Dialogs.Action(t(kind),()->transfer(kind)));
        actions.add(new Dialogs.Action(t("reset_filters"),()->{view().filter=new Filter();view().search="";safe(services::persist);minecraft.gui.setScreen(this);}));
        if(sourceIsSaved(e))for(int direction:new int[]{-1,1})actions.add(new Dialogs.Action(t(direction<0?"move_up":"move_down"),()->{safe(()->services.saved.move(e.key(),direction));view().sort=Sort.CUSTOM;safe(services::persist);minecraft.gui.setScreen(this);}));
        if(services.saved.canUndo())actions.add(new Dialogs.Action(t("undo"),()->{safe(services.saved::undo);minecraft.gui.setScreen(this);}));
        if(e!=null)actions.add(new Dialogs.Action(t("copy_address"),()->{minecraft.keyboardHandler.setClipboard(e.address);notice=s("copied");minecraft.gui.setScreen(this);}));
        actions.add(new Dialogs.Action(t("feedback"),()->feedback()));
        actions.add(new Dialogs.Action(t("website"),()->ConfirmLinkScreen.confirmLinkNow(this,URI.create("https://musti3214.github.io/Horizon-Server-Browser/"))));
        minecraft.gui.setScreen(new Dialogs.Menu(this,t("more"),actions,null));
    }
    private void feedback() {
        Entry e=selected();
        minecraft.gui.setScreen(new Dialogs.Form(this,t("feedback"),List.of(new Dialogs.Field("category","feedback","feedback","server"),new Dialogs.Field("message","",4096),new Dialogs.Field("address",e==null?"":e.address,255)),s("mail_unconfigured"),v->{if(v.get("message").isBlank())throw new IllegalArgumentException();services.draft(v.get("category"),v.get("message"),v.get("address"));notice=s("draft_saved");}));
    }
    private void transfer(String kind) {
        boolean exporting=kind.startsWith("export");String initial=exporting?services.home.resolve("exports/server-browser-"+System.currentTimeMillis()+(kind.endsWith("json")?".json":".dat")).toAbsolutePath().toString():"";
        minecraft.gui.setScreen(new Dialogs.Form(this,t(kind),List.of(new Dialogs.Field("file_path",initial,4096)),s("transfer_note"),v->{
            java.nio.file.Path path=java.nio.file.Path.of(v.get("file_path")).toAbsolutePath().normalize();if(v.get("file_path").isBlank())throw new IllegalArgumentException();
            if(exporting)Transfers.destination(services,path);
            IoAction action=()->{switch(kind){case "import_json"->Transfers.importJson(services,path);case "import_nbt"->Transfers.importNbt(services,path);case "export_json"->Transfers.exportJson(services,path);case "export_nbt"->Transfers.exportNbt(services,path);}notice=s(exporting?"export_done":"import_done");};
            if(exporting&&java.nio.file.Files.exists(path))minecraft.gui.setScreen(new ConfirmScreen(yes->{if(yes)safe(action);minecraft.gui.setScreen(this);},t("overwrite"),Component.literal(path.toString())));else action.run();
        }));
    }
    private void startLan(){if(lanDetector!=null)return;try{lanList=new LanServerDetection.LanServerList();lanDetector=new LanServerDetection.LanServerDetector(lanList);lanDetector.setDaemon(true);lanDetector.start();}catch(IOException ex){notice=s("lan_error");}}
    private void stopLan(){if(lanDetector!=null){lanDetector.interrupt();lanDetector=null;lanList=null;}}
    @Override public void removed(){stopLan();if(dirtySince!=0){dirtySince=0;safe(services::persist);}for(FaviconTexture icon:icons.values())icon.close();icons.clear();iconData.clear();}
    @Override public void onClose(){minecraft.gui.setScreen(parent);}
    @Override public boolean keyPressed(KeyEvent event){if(event.isConfirmation()&&!search.isFocused()&&selected()!=null){connect(selected());return true;}return super.keyPressed(event);}
    private FaviconTexture icon(Entry e,Status status) {
        FaviconTexture icon=icons.get(e.key());
        if(icon==null){icon=FaviconTexture.forServer(minecraft.getTextureManager(),e.key());icons.put(e.key(),icon);if(icons.size()>128){String first=icons.keySet().iterator().next();icons.remove(first).close();iconData.remove(first);}}
        String encoded=status.icon()==null?e.icon:status.icon();
        if(encoded!=null&&!encoded.equals(iconData.get(e.key()))) {
            try {NativeImage image=NativeImage.read(Base64.getDecoder().decode(encoded));if(image.getWidth()!=64||image.getHeight()!=64)image.close();else icon.upload(image);iconData.put(e.key(),encoded);}catch(Exception ignored){iconData.put(e.key(),encoded);}
        }
        return icon;
    }
    private final class Rows extends ObjectSelectionList<Row> {
        Rows(Minecraft mc,int w,int h,int y,int row){super(mc,w,h,y,row);}
        void populate(List<io.github.musti3214.serverbrowser.core.Model.Entry> entries){clearEntries();for(io.github.musti3214.serverbrowser.core.Model.Entry e:entries){Row row=new Row(e);addEntry(row);if(e.key().equals(selectedKey))setSelected(row);}}
        @Override public int getRowWidth(){return getWidth()-8;}
        @Override protected void extractListBackground(GuiGraphicsExtractor g) { }
        @Override protected void extractListSeparators(GuiGraphicsExtractor g) { }
        @Override public void setSelected(Row row){super.setSelected(row);if(row!=null)selectedKey=row.entry.key();updateButtons();}
    }
    private final class Row extends ObjectSelectionList.Entry<Row> {
        final Entry entry;private long lastClick;
        Row(Entry entry){this.entry=entry;}
        @Override public Component getNarration(){Status s=services.statuses.getOrDefault(entry.key(),Status.unknown());return Component.literal(entry.name+", "+entry.address+", ").append(t("health."+s.health().name().toLowerCase(Locale.ROOT)));}
        @Override public void extractContent(GuiGraphicsExtractor g,int mx,int my,boolean hovered,float dt) {
            int x=getContentX(),y=getContentY(),w=getContentWidth();Status status=services.statuses.getOrDefault(entry.key(),Status.unknown());
            g.fill(getX(),getY(),getX()+getWidth(),getY()+getHeight()-4,rows.getSelected()==this?0x70000000:hovered?0x50000000:0x26000000);
            if(rows.getSelected()==this)g.outline(getX(),getY(),getWidth(),getHeight()-4,0xFFE1E1E1);
            g.blit(RenderPipelines.GUI_TEXTURED,icon(entry,status).textureLocation(),x+8,y+8,0,0,32,32,32,32);
            String right=status.health()==Health.ONLINE?(status.players()==null?"?":status.players())+"/"+(status.max()==null?"?":status.max())+"  "+(status.ping()==null?"?":status.ping())+" ms":s("health."+status.health().name().toLowerCase(Locale.ROOT));
            int rightWidth=font.width(right);String name=entry.name;
            g.text(font,font.plainSubstrByWidth(name,Math.max(40,w-64-rightWidth)),x+48,y+2,0xFFFFFFFF);
            g.text(font,right,getContentRight()-rightWidth-18,y+2,status.health()==Health.OFFLINE?0xFFFF5555:0xFFAAAAAA);
            g.text(font,entry.meta.favorite?"★":"☆",getContentRight()-10,y+2,entry.meta.favorite?0xFFFFAA00:0xFF707070);
            Component description=status.motd().isBlank()?Component.literal(entry.address):Text.motd(status);int line=0;
            for(var part:font.split(description,w-56)){if(line>=2)break;g.text(font,part,x+48,y+15+line*11,0xFFAAAAAA);line++;}
            String tags=(status.version().isBlank()?"":status.version()+" · "+(status.compatible(ClientServices.protocol())?s("compatible_short"):s("incompatible"))+" · ")+entry.address+(entry.meta.group.isBlank()?"":" · "+entry.meta.group)+(entry.meta.tags.isEmpty()?"":" · "+String.join(" · ",entry.meta.tags));
            g.text(font,font.plainSubstrByWidth(tags,w-56),x+48,y+43,status.compatible(ClientServices.protocol())?0xFF55FF55:0xFF909090);
            if(hovered)g.setTooltipForNextFrame(Component.literal(entry.name+" · "+entry.address+(entry.meta.notes.isBlank()?"":" · "+entry.meta.notes.substring(0,Math.min(120,entry.meta.notes.length())))),mx,my);
        }
        @Override public boolean mouseClicked(MouseButtonEvent event,boolean doubleClick){
            // 26.3 uses SDL button identifiers: primary/left is 1.
            if(event.button()!=1)return false;rows.setSelected(this);rows.setFocused(this);
            if(event.x()>=getContentRight()-16&&event.y()<getContentY()+16){safe(()->services.favorite(entry));rebuildRows();return true;}
            long now=System.currentTimeMillis();if(now-lastClick<250)connect(entry);lastClick=now;return true;
        }
    }
}
