package io.github.musti3214.serverbrowser.core;

import java.util.*;
import static io.github.musti3214.serverbrowser.core.Model.*;

/** AND filters; unavailable status values never satisfy a numeric constraint. */
public final class Query {
    private Query() { }
    public static String plain(String s) { return (s==null?"":s).replaceAll("§.","").toLowerCase(Locale.ROOT); }
    public static boolean matches(Entry e, Status s, View v, int protocol) {
        Meta m=e.meta; Filter f=v.filter;
        if ((v.tab==Tab.FAVORITES||f.favorites)&&!m.favorite || v.tab==Tab.RECENT&&m.lastPlayed==0) return false;
        if (!f.group.isBlank()&&!f.group.equalsIgnoreCase(m.group) || !f.language.isBlank()&&!f.language.equalsIgnoreCase(m.language)
                || !f.region.isBlank()&&!f.region.equalsIgnoreCase(m.region) || !f.auth.equals("any")&&!f.auth.equalsIgnoreCase(m.auth)) return false;
        for(String tag:f.tags.split(",")) if(!tag.isBlank()&&m.tags.stream().noneMatch(t->t.equalsIgnoreCase(tag.trim()))) return false;
        if (f.online!=null && (s==null || (f.online ? s.health()!=Health.ONLINE : s.health()!=Health.OFFLINE))) return false;
        if (f.compatible&&(s==null||!s.compatible(protocol)) || !f.version.isBlank()&&(s==null||!f.version.equalsIgnoreCase(s.version()))) return false;
        if (f.minPlayers!=null&&(s==null||s.players()==null||s.players()<f.minPlayers)
                || f.maxPlayers!=null&&(s==null||s.players()==null||s.players()>f.maxPlayers)
                || f.maxPing!=null&&(s==null||s.ping()==null||s.ping()>f.maxPing)
                || f.notEmpty&&(s==null||s.players()==null||s.players()==0)
                || f.notFull&&(s==null||s.players()==null||s.max()==null||s.players()>=s.max())) return false;
        String hay=plain(e.name+" "+e.address+" "+m.notes+" "+m.group+" "+String.join(" ",m.tags)+" "+(s==null?"":s.motd()));
        for(String word:plain(v.search).trim().split("\\s+")) {
            if(word.startsWith("#")) { if(m.tags.stream().noneMatch(t->plain(t).contains(word.substring(1))))return false; }
            else if(word.startsWith("@")) { if(!plain(m.group).contains(word.substring(1)))return false; }
            else if(!hay.contains(word)) return false;
        }
        return true;
    }
    public static List<Entry> select(List<Entry> source, Map<String,Status> statuses, View view,int protocol) {
        List<Entry> result=new ArrayList<>();
        for(Entry e:source) if(matches(e,statuses.get(e.key()),view,protocol))result.add(e);
        Comparator<Entry> cmp=(a,b)->0;
        switch(view.sort) {
            case NAME -> cmp=Comparator.comparing(e->plain(e.name));
            case VERSION -> cmp=(a,b)->optional(version(statuses.get(a.key())),version(statuses.get(b.key())),view.descending);
            case PLAYERS, FREE_SLOTS, PING -> cmp=(a,b)->optional(value(statuses.get(a.key()),view.sort),value(statuses.get(b.key()),view.sort),view.descending);
            case LAST_PLAYED -> cmp=Comparator.comparingLong(e->e.meta.lastPlayed);
            case DATE_ADDED -> cmp=Comparator.comparingLong(e->e.meta.added);
            case FAVORITES -> cmp=Comparator.comparing(e->!e.meta.favorite);
            default -> { }
        }
        if(view.descending && !Set.of(Sort.PLAYERS,Sort.FREE_SLOTS,Sort.PING,Sort.VERSION,Sort.CUSTOM).contains(view.sort))cmp=cmp.reversed();
        result.sort(cmp); return result;
    }
    private static String version(Status s) { return s==null||s.version().isBlank()?null:s.version(); }
    private static Long value(Status s,Sort sort) {
        if(s==null)return null;
        return switch(sort) {
            case PING -> s.ping();
            case PLAYERS -> s.players()==null?null:s.players().longValue();
            case FREE_SLOTS -> s.players()==null||s.max()==null?null:Math.max(0L,s.max().longValue()-s.players());
            default -> null;
        };
    }
    private static <T extends Comparable<T>> int optional(T a,T b,boolean desc) {
        if(a==null)return b==null?0:1;
        if(b==null)return -1;
        return desc?b.compareTo(a):a.compareTo(b);
    }
}
