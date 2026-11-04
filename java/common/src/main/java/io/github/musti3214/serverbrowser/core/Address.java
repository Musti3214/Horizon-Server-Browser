package io.github.musti3214.serverbrowser.core;

import java.net.IDN;
import java.net.InetAddress;
import java.util.Locale;

/** Address identity never performs a DNS lookup. Local addresses remain valid for saved servers. */
public record Address(String host, int port) {
    public static Address parse(String raw) {
        String value = raw == null ? "" : raw.trim();
        if (value.isEmpty() || value.length() > 255 || value.chars().anyMatch(c -> c <= 32 || c == 127)
                || value.matches(".*[/\\\\?#@].*") || value.contains("://")) throw new IllegalArgumentException("address");
        String host = value;
        int port = 25565;
        if (value.startsWith("[")) {
            int end = value.indexOf(']');
            if (end < 2) throw new IllegalArgumentException("IPv6");
            host = value.substring(1, end);
            if (!host.contains(":")) throw new IllegalArgumentException("IPv6");
            if (end + 1 < value.length()) {
                if (value.charAt(end + 1) != ':') throw new IllegalArgumentException("port");
                port = port(value.substring(end + 2));
            }
        } else if (value.indexOf(':') == value.lastIndexOf(':') && value.contains(":")) {
            int split = value.indexOf(':'); host = value.substring(0, split); port = port(value.substring(split + 1));
        }
        if (host.contains(":")) {
            if (!host.matches("[0-9a-fA-F:.]+")) throw new IllegalArgumentException("IPv6");
            try { host = InetAddress.getByName(host).getHostAddress(); }
            catch (Exception ex) { throw new IllegalArgumentException("IPv6", ex); }
        } else {
            if (host.endsWith(".")) host = host.substring(0, host.length()-1);
            host = IDN.toASCII(host, IDN.USE_STD3_ASCII_RULES).toLowerCase(Locale.ROOT);
            if (host.isBlank() || host.length() > 253) throw new IllegalArgumentException("host");
            if (host.matches("[0-9.]+")) {
                String[] parts = host.split("\\.", -1);
                if (parts.length != 4) throw new IllegalArgumentException("IPv4");
                for (String part : parts) if (part.isEmpty() || Integer.parseInt(part)>255) throw new IllegalArgumentException("IPv4");
            }
        }
        return new Address(host, port);
    }
    private static int port(String value) {
        if (!value.matches("[0-9]{1,5}")) throw new IllegalArgumentException("port");
        int n = Integer.parseInt(value);
        if (n < 1 || n > 65535) throw new IllegalArgumentException("port");
        return n;
    }
    public String key() { return (host.contains(":") ? "["+host+"]" : host) + (port==25565 ? "" : ":"+port); }
    public boolean publicListing() {
        if (host.contains(":")) {
            try {
                InetAddress ip=InetAddress.getByName(host);
                return !(ip.isAnyLocalAddress() || ip.isLoopbackAddress() || ip.isLinkLocalAddress() || ip.isSiteLocalAddress() || ip.isMulticastAddress())
                        && !host.startsWith("fc") && !host.startsWith("fd");
            } catch (Exception ex) { return false; }
        }
        if (host.matches("[0-9.]+")) {
            String[] p=host.split("\\."); int a=Integer.parseInt(p[0]), b=Integer.parseInt(p[1]);
            return a!=0 && a!=10 && a!=127 && a<224 && !(a==169 && b==254) && !(a==172 && b>=16 && b<=31)
                    && !(a==192 && b==168) && !(a==100 && b>=64 && b<=127);
        }
        return host.contains(".") && !host.endsWith(".local") && !host.endsWith(".localhost") && !host.endsWith(".internal") && !host.endsWith(".home.arpa");
    }
    public static String key(String value) { return parse(value).key(); }
}
