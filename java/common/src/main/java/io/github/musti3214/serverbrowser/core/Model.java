package io.github.musti3214.serverbrowser.core;

import java.util.*;
import com.google.gson.annotations.SerializedName;

public final class Model {
    private Model() { }
    public enum Tab { SAVED, DISCOVER, FAVORITES, RECENT, LAN }
    public enum Sort { CUSTOM, NAME, PLAYERS, FREE_SLOTS, PING, VERSION, LAST_PLAYED, DATE_ADDED, FAVORITES }
    public enum Health { UNKNOWN, PINGING, ONLINE, OFFLINE }
    public static final class Entry {
        public String name="", address="";
        public String packs, icon;
        @SerializedName(value="metadata",alternate={"meta"}) public Meta meta=new Meta();
        public Entry() { }
        public Entry(String name,String address) { this.name=name; this.address=Address.key(address); }
        public String key() { return Address.key(address); }
    }
    public static final class Meta {
        public boolean favorite;
        public List<String> tags=new ArrayList<>();
        public String notes="", group="", language="", region="", auth="unknown";
        public long added=System.currentTimeMillis()/1000;
        @SerializedName(value="last_played",alternate={"lastPlayed"}) public long lastPlayed;
    }
    public record Status(Health health, Integer players, Integer max, Long ping, String version, Integer protocol, String motd, String icon,String descriptionJson,List<String> sampleNames) {
        public Status(Health health,Integer players,Integer max,Long ping,String version,Integer protocol,String motd,String icon){this(health,players,max,ping,version,protocol,motd,icon,null,List.of());}
        public static Status unknown() { return new Status(Health.UNKNOWN,null,null,null,"",null,"",null); }
        public static Status pinging() { return new Status(Health.PINGING,null,null,null,"",null,"",null); }
        public static Status offline() { return new Status(Health.OFFLINE,null,null,null,"",null,"",null); }
        public boolean compatible(int current) { return health==Health.ONLINE && protocol!=null && protocol==current; }
    }
    public static final class Filter {
        public Boolean online;
        public boolean compatible, notEmpty, notFull, favorites;
        public String version="", tags="", group="", language="", region="", auth="any";
        public Integer minPlayers, maxPlayers;
        public Long maxPing;
    }
    public static final class View {
        public String search="";
        public Tab tab=Tab.SAVED;
        public Sort sort=Sort.CUSTOM;
        public boolean descending;
        public Filter filter=new Filter();
    }
    public static final class Settings {
        public boolean enabled=true, onlineDirectory=true, showOffline=true, supportDismissed;
        public int refreshSeconds=60, concurrentPings=8;
        public void sanitize() { concurrentPings=Math.clamp(concurrentPings,1,16); refreshSeconds=refreshSeconds==0?0:Math.clamp(refreshSeconds,15,3600); }
    }
    public static final class State {
        public int schema=1;
        public Settings settings=new Settings();
        public View view=new View();
        public Map<String,Meta> metadata=new LinkedHashMap<>();
        public Map<String,View> presets=new TreeMap<>();
        public List<Entry> recent=new ArrayList<>();
    }
}
