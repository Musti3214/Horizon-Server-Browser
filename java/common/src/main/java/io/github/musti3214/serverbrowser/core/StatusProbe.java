package io.github.musti3214.serverbrowser.core;

import com.google.gson.*;
import java.io.*;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.util.Base64;
import java.util.*;
import static io.github.musti3214.serverbrowser.core.Model.*;

/** Bounded Java status handshake; authentication and joins are handled exclusively by Minecraft. */
public final class StatusProbe {
    @FunctionalInterface public interface Resolver { InetSocketAddress resolve(Address address) throws IOException; }
    private StatusProbe() { }
    public static Status ping(String raw,int protocol,Resolver resolver) {
        try {
            Address address=Address.parse(raw);
            try(Socket socket=new Socket()) {
                socket.connect(resolver.resolve(address),3000);socket.setSoTimeout(3000);
                DataOutputStream out=new DataOutputStream(socket.getOutputStream());DataInputStream in=new DataInputStream(socket.getInputStream());
                ByteArrayOutputStream b=new ByteArrayOutputStream();DataOutputStream handshake=new DataOutputStream(b);
                varInt(handshake,0);varInt(handshake,protocol);string(handshake,address.host());handshake.writeShort(address.port());varInt(handshake,1);
                varInt(out,b.size());out.write(b.toByteArray());out.writeByte(1);out.writeByte(0);out.flush();
                byte[] packet=packet(in,1024*1024);DataInputStream data=new DataInputStream(new ByteArrayInputStream(packet));
                if(varInt(data)!=0)throw new IOException("status id");int size=varInt(data);if(size<0||size>1024*1024||size>data.available())throw new IOException("status size");
                JsonObject json=JsonParser.parseString(new String(data.readNBytes(size),StandardCharsets.UTF_8)).getAsJsonObject();
                JsonObject version=json.has("version")&&json.get("version").isJsonObject()?json.getAsJsonObject("version"):new JsonObject();
                JsonObject players=json.has("players")&&json.get("players").isJsonObject()?json.getAsJsonObject("players"):new JsonObject();
                Long ping=null;
                try {
                    long sent=System.nanoTime();out.writeByte(9);out.writeByte(1);out.writeLong(sent);out.flush();
                    DataInputStream pong=new DataInputStream(new ByteArrayInputStream(packet(in,32)));
                    if(varInt(pong)==1&&pong.readLong()==sent)ping=Math.max(0,(System.nanoTime()-sent)/1_000_000);
                } catch(IOException ignored) { }
                String favicon=null;
                if(json.has("favicon")&&json.get("favicon").isJsonPrimitive()) {
                    String rawIcon=json.get("favicon").getAsString();
                    if(rawIcon.startsWith("data:image/png;base64,")&&rawIcon.length()<32768) {
                        try {byte[] png=Base64.getDecoder().decode(rawIcon.substring(22));if(png.length>=24&&png[0]==(byte)137&&png[1]==80&&dimension(png,16)==64&&dimension(png,20)==64)favicon=rawIcon.substring(22);}catch(IllegalArgumentException ignored){}
                    }
                }
                String description=json.has("description")?json.get("description").toString():null;
                if(description!=null&&description.length()>16384)description=null;
                List<String> sample=new ArrayList<>();
                if(players.has("sample")&&players.get("sample").isJsonArray())for(JsonElement item:players.getAsJsonArray("sample")){if(sample.size()>=20)break;if(item.isJsonObject())sample.add(Directory.clean(text(item.getAsJsonObject().get("name")),64));}
                return new Status(Health.ONLINE,number(players,"online"),number(players,"max"),ping,text(version.get("name")),number(version,"protocol"),Directory.clean(text(json.get("description")),2048),favicon,description,sample);
            }
        } catch(Exception ex) {return Status.offline();}
    }
    private static int dimension(byte[] b,int pos) {return ((b[pos]&255)<<24)|((b[pos+1]&255)<<16)|((b[pos+2]&255)<<8)|(b[pos+3]&255);}
    private static Integer number(JsonObject o,String key) {try {return o.get(key).getAsInt();}catch(Exception ex){return null;}}
    private static String text(JsonElement value) {
        if(value==null||value.isJsonNull())return "";
        if(value.isJsonPrimitive())return value.getAsString();
        if(value.isJsonArray()){StringBuilder b=new StringBuilder();for(var v:value.getAsJsonArray())b.append(text(v));return b.toString();}
        var o=value.getAsJsonObject();return text(o.get("text"))+text(o.get("extra"));
    }
    static byte[] packet(DataInputStream in,int max) throws IOException {int length=varInt(in);if(length<1||length>max)throw new IOException("packet length");byte[] b=in.readNBytes(length);if(b.length!=length)throw new EOFException();return b;}
    static void string(DataOutputStream out,String value) throws IOException {byte[] b=value.getBytes(StandardCharsets.UTF_8);varInt(out,b.length);out.write(b);}
    static void varInt(DataOutputStream out,int value) throws IOException {do{int b=value&127;value>>>=7;out.writeByte(value==0?b:b|128);}while(value!=0);}
    static int varInt(DataInputStream in) throws IOException {int value=0;for(int i=0;i<5;i++){int b=in.readUnsignedByte();value|=(b&127)<<(i*7);if((b&128)==0)return value;}throw new IOException("VarInt overflow");}
}
