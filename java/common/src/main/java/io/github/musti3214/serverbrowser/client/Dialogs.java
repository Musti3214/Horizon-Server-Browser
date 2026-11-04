package io.github.musti3214.serverbrowser.client;

import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.*;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;
import net.minecraft.util.FormattedCharSequence;
import java.util.*;
import static io.github.musti3214.serverbrowser.client.Text.*;

/** Native Minecraft widgets, paginated so dialogs remain usable at every GUI scale. */
public final class Dialogs {
    private Dialogs() { }
    public record Field(String key,String value,int max,List<String> choices) {
        public Field(String key,String value,int max){this(key,value,max,List.of());}
        public Field(String key,String value,String...choices){this(key,value,128,List.of(choices));}
    }
    @FunctionalInterface public interface Save {void save(Map<String,String> values)throws Exception;}
    public record Action(Component label,Runnable run) { }
    /** Full, wrapped server details remain accessible when the side panel does not fit. */
    public static final class Info extends Screen {
        private final Screen parent;
        private final List<Component> content;
        private final List<Action> actions;
        public Info(Screen parent,Component title,List<Component> content,List<Action> actions){super(title);this.parent=parent;this.content=content;this.actions=actions;}
        @Override protected void init() {
            int area=Math.min(480,width-24),left=(width-area)/2;
            class Lines extends ObjectSelectionList<Line> {
                Lines(){super(Info.this.minecraft,area,Math.max(20,Info.this.height-106),34,13);for(Component block:content){for(FormattedCharSequence line:font.split(block,area-32))addEntry(new Line(line,block));addEntry(new Line(FormattedCharSequence.EMPTY,Component.empty()));}}
                @Override public int getRowWidth(){return getWidth()-16;}
                @Override protected void extractListBackground(GuiGraphicsExtractor g) { }
                @Override protected void extractListSeparators(GuiGraphicsExtractor g) { }
            }
            Lines list=addRenderableWidget(new Lines());list.setX(left);
            int bw=(area-4*Math.max(0,actions.size()-1))/Math.max(1,actions.size());
            for(int i=0;i<actions.size();i++){Action action=actions.get(i);addRenderableWidget(Button.builder(action.label(),b->action.run().run()).bounds(left+i*(bw+4),height-53,bw,20).build());}
            addRenderableWidget(Button.builder(t("back"),b->onClose()).bounds(left,height-28,area,20).build());
        }
        private final class Line extends ObjectSelectionList.Entry<Line> {
            private final FormattedCharSequence text;private final Component narration;
            Line(FormattedCharSequence text,Component narration){this.text=text;this.narration=narration;}
            @Override public Component getNarration(){return narration;}
            @Override public void extractContent(GuiGraphicsExtractor g,int mx,int my,boolean hovered,float dt){g.text(font,text,getContentX()+4,getContentY()+2,0xFFDDDDDD);}
        }
        @Override public void extractRenderState(GuiGraphicsExtractor g,int mx,int my,float dt){super.extractRenderState(g,mx,my,dt);g.text(font,font.plainSubstrByWidth(title.getString(),width-24),12,12,0xFFFFFFFF);}
        @Override public void onClose(){minecraft.gui.setScreen(parent);}
    }
    public static final class Form extends Screen {
        private final Screen parent;
        private final List<Field> fields;
        private final Map<String,String> values=new LinkedHashMap<>();
        private final Save save;
        private final String note;
        private int page,perPage,left,area,group,firstY;
        private final List<List<String>> groups;
        private final List<Component> groupNames;
        private List<Field> currentFields;
        private String error="";
        private final List<String> visibleKeys=new ArrayList<>();
        public Form(Screen parent,Component title,List<Field> fields,String note,Save save) {
            this(parent,title,fields,note,save,List.of(),List.of());
        }
        public Form(Screen parent,Component title,List<Field> fields,String note,Save save,List<List<String>> groups,List<Component> groupNames) {
            super(title);this.parent=parent;this.fields=fields;this.save=save;this.note=note;this.groups=groups;this.groupNames=groupNames;
            for(Field f:fields)values.put(f.key(),f.value());
        }
        @Override protected void init() {
            visibleKeys.clear();area=Math.min(360,width-24);left=(width-area)/2;firstY=groups.isEmpty()?40:57;perPage=Math.max(1,(height-firstY-74)/34);
            currentFields=groups.isEmpty()?fields:fields.stream().filter(f->groups.get(group).contains(f.key())).sorted(Comparator.comparingInt(f->groups.get(group).indexOf(f.key()))).toList();
            if(!groups.isEmpty()) {int tw=(area-4*(groups.size()-1))/groups.size();for(int i=0;i<groups.size();i++){final int selected=i;addRenderableWidget(Button.builder(groupNames.get(i),b->{group=selected;page=0;rebuildWidgets();}).bounds(left+i*(tw+4),25,tw,20).build());}}
            int pages=Math.max(1,(currentFields.size()+perPage-1)/perPage);page=Math.min(page,pages-1);
            for(int i=page*perPage;i<Math.min(currentFields.size(),(page+1)*perPage);i++) {
                Field f=currentFields.get(i);int y=firstY+(i-page*perPage)*34;visibleKeys.add(f.key());
                if(f.choices().isEmpty()) {
                    EditBox edit=addRenderableWidget(new EditBox(font,left,y,area,20,t(f.key())));edit.setMaxLength(f.max());edit.setValue(values.get(f.key()));edit.setResponder(v->values.put(f.key(),v));
                } else {
                    Button option=Button.builder(choice(f.key(),values.get(f.key())),b->{int n=f.choices().indexOf(values.get(f.key()));String value=f.choices().get((n+1)%f.choices().size());values.put(f.key(),value);b.setMessage(choice(f.key(),value));}).bounds(left,y,area,20).build();addRenderableWidget(option);
                }
            }
            if(pages>1) {
                addRenderableWidget(Button.builder(Component.literal("←"),b->{page--;rebuildWidgets();}).bounds(left,height-54,50,20).build()).active=page>0;
                addRenderableWidget(Button.builder(Component.literal("→"),b->{page++;rebuildWidgets();}).bounds(left+area-50,height-54,50,20).build()).active=page<pages-1;
            }
            addRenderableWidget(Button.builder(t("save"),b->{try{save.save(Collections.unmodifiableMap(values));if(minecraft.gui.screen()==this)minecraft.gui.setScreen(parent);}catch(Exception ex){error=ex instanceof IllegalArgumentException?s("invalid_value"):s("save_failed");}}).bounds(left,height-28,area/2-2,20).build());
            addRenderableWidget(Button.builder(t("cancel"),b->onClose()).bounds(left+area/2+2,height-28,area/2-2,20).build());
        }
        private Component choice(String key,String value){return t(key).copy().append(": ").append(t("value."+value));}
        @Override public void extractRenderState(GuiGraphicsExtractor g,int mx,int my,float dt) {
            super.extractRenderState(g,mx,my,dt);
            g.text(font,title,12,9,0xFFFFFFFF);
            for(int i=0;i<visibleKeys.size();i++)g.text(font,t(visibleKeys.get(i)),left,firstY-10+i*34,0xFFBFBFBF);
            String message=error.isEmpty()?note:error;
            if(!message.isEmpty())g.text(font,font.plainSubstrByWidth(message,area),left,height-72,error.isEmpty()?0xFFBFBFBF:0xFFFF5555);
            if(currentFields.size()>perPage){String position=(page+1)+" / "+((currentFields.size()+perPage-1)/perPage);g.text(font,position,(width-font.width(position))/2,height-48,0xFFBFBFBF);}
        }
        @Override public void onClose(){minecraft.gui.setScreen(parent);}
    }
    public static final class Menu extends Screen {
        private final Screen parent;
        private final List<Action> actions;
        private int page,perPage;
        private final Component description;
        public Menu(Screen parent,Component title,List<Action> actions,Component description){super(title);this.parent=parent;this.actions=actions;this.description=description;}
        @Override protected void init() {
            int area=Math.min(330,width-24),left=(width-area)/2;perPage=Math.max(1,(height-112)/25);int pages=Math.max(1,(actions.size()+perPage-1)/perPage);page=Math.min(page,pages-1);
            for(int i=page*perPage;i<Math.min(actions.size(),(page+1)*perPage);i++){Action action=actions.get(i);addRenderableWidget(Button.builder(action.label(),b->action.run().run()).bounds(left,51+(i-page*perPage)*25,area,20).build());}
            if(pages>1){addRenderableWidget(Button.builder(Component.literal("←"),b->{page--;rebuildWidgets();}).bounds(left,height-53,50,20).build()).active=page>0;addRenderableWidget(Button.builder(Component.literal("→"),b->{page++;rebuildWidgets();}).bounds(left+area-50,height-53,50,20).build()).active=page<pages-1;}
            addRenderableWidget(Button.builder(t("back"),b->onClose()).bounds(left,height-28,area,20).build());
        }
        @Override public void extractRenderState(GuiGraphicsExtractor g,int mx,int my,float dt){super.extractRenderState(g,mx,my,dt);g.text(font,title,(width-font.width(title))/2,12,0xFFFFFFFF);if(description!=null){String line=font.plainSubstrByWidth(description.getString(),width-24);g.text(font,line,(width-font.width(line))/2,31,0xFFBFBFBF);}}
        @Override public void onClose(){minecraft.gui.setScreen(parent);}
    }
}
