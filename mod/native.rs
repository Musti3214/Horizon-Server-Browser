use super::{Browser, Outcome, core};
use crate::{clipboard::{Clipboard, EditShortcut}, hud::{HudAtlas, HudBatch}, title::{Action, Mode, Title}, ui::Caret};
use std::{cell::{Cell, RefCell}, path::Path, time::Instant};
use winit::{event::{ElementState, KeyEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

const MARKER: &str = "\0server_browser:2.2.0";
struct State { browser: Browser, caret: Caret, focused_at: Instant, press: Option<((f32,f32),(f32,f32),bool)>, mouse_down: bool }
thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
    static BYPASS: Cell<bool> = const { Cell::new(false) };
    static CONNECT: RefCell<Option<(String,bool)>> = const { RefCell::new(None) };
    static CLASSIC: Cell<bool> = const { Cell::new(false) };
    static ON: Cell<bool> = const { Cell::new(false) };
    static HELD: Cell<(bool,bool)> = const { Cell::new((false,false)) };
}
fn with<T>(f: impl FnOnce(&mut State)->T) -> Option<T> {
    STATE.with_borrow_mut(|state| state.as_mut().map(f))
}
pub fn active(title: &Title) -> bool {
    matches!(&title.mode, Mode::Multiplayer(crate::multiplayer::Screen::Direct { address }) if address==MARKER)
        && STATE.with_borrow(|s| s.is_some())
}
fn enabled(saves: &Path) -> bool {
    let home=saves.parent().unwrap_or(Path::new("."));
    std::fs::read_to_string(home.join("horizon-mods.txt")).ok().is_none_or(|s| {
        s.lines().filter_map(|l|l.split_once(':')).rfind(|(key,_)|key.trim()=="mod.server_browser")
            .is_none_or(|(_,value)|value.trim()!="false")
    })
}
fn marker() -> crate::multiplayer::Screen {
    crate::multiplayer::Screen::Direct { address: MARKER.into() }
}
pub fn open(saves: &Path, force: bool) -> Option<crate::multiplayer::Screen> {
    let on=enabled(saves);ON.set(on);
    if BYPASS.get() || !on { return None; }
    let home=saves.parent().unwrap_or(Path::new("."));
    let settings=core::persistence::load_state(home).ok()?;
    if !force && (!settings.settings.modern || settings.settings.classic_menu) { return None; }
    match Browser::new(saves) {
        Ok(mut browser) => {
            if force {
                browser.state.settings.modern=true;browser.state.settings.classic_menu=false;
                if let Err(error)=browser.persist() {log::warn!("Server Browser settings: {error}");}
            }
            STATE.set(Some(State {browser,caret:Caret::default(),focused_at:Instant::now(),press:None,mouse_down:false}));
            log::info!("[mod server_browser] Server Atlas 2.2.0 opened");
            Some(marker())
        }
        Err(e) => { log::warn!("Server Browser: {e}"); None }
    }
}
pub fn close() { CONNECT.take(); STATE.set(None); }
pub fn enter(title:&mut Title)->bool {
    if let Some(screen)=open(&title.saves,false) { CLASSIC.set(false);title.mode=Mode::Multiplayer(screen);true } else {false}
}
fn classic(saves: &Path) -> crate::multiplayer::Screen {
    struct Guard;
    impl Drop for Guard { fn drop(&mut self) { BYPASS.set(false); } }
    BYPASS.set(true);
    let _guard=Guard;
    crate::multiplayer::Screen::list(saves)
}
fn apply(title: &mut Title, outcome: Outcome) -> Action {
    match outcome {
        Outcome::Connect {address,lan} => { title.last_server=address.clone(); Action::Connect { address, lan } }
        Outcome::Back => { close(); title.mode=Mode::Main; Action::None }
        Outcome::Classic => { close(); CLASSIC.set(true);title.mode=Mode::Multiplayer(classic(&title.saves)); Action::None }
        Outcome::None => Action::None,
    }
}
pub fn sync(title: &mut Title, on: bool) {
    ON.set(on);
    if active(title) && !on {
        close(); title.mode=Mode::Multiplayer(classic(&title.saves));
    } else if !active(title) {
        close();
        if matches!(title.mode,Mode::Main) { CLASSIC.set(false); }
        if on && !CLASSIC.get() && matches!(title.mode,Mode::Multiplayer(crate::multiplayer::Screen::List{..})) && !enter(title) {
            CLASSIC.set(true);
        }
    }
}
pub fn draw_classic_switch(screen:&crate::multiplayer::Screen,atlas:&HudAtlas,batch:&mut HudBatch,cursor:(f32,f32)) {
    if ON.get() && matches!(screen,crate::multiplayer::Screen::List{..}) {
        let x=(batch.width.ceil()-112.0).max(12.0);
        let label=super::tr("title");
        crate::ui::button(batch,atlas,x,6.0,100.0,20.0,&crate::text::Text::literal(label),true,crate::ui::hovered(cursor,x,6.0,100.0,20.0));
    }
}
pub fn classic_click(title:&mut Title,cursor:(f32,f32),size:(f32,f32))->bool {
    if !ON.get() || !matches!(title.mode,Mode::Multiplayer(crate::multiplayer::Screen::List{..})) {return false;}
    if !crate::ui::hovered(cursor,(size.0.ceil()-112.0).max(12.0),6.0,100.0,20.0) {return false;}
    if let Some(screen)=open(&title.saves,true) {
        CLASSIC.set(false);title.mode=Mode::Multiplayer(screen);return true;
    }
    false
}
pub fn draw(atlas:&HudAtlas,batch:&mut HudBatch,cursor:(f32,f32)) {
    with(|s| {
        if let Some((cursor,size,shift))=s.press.take()
            && let Some(rect @ [x,y,w,h])=s.browser.focused_box(size)
            && crate::ui::hovered(cursor,x,y,w,h)
            && let Some(field)=s.browser.focused_text() {
                s.caret.press(atlas,field,rect,cursor.0,shift);
                if !s.mouse_down {s.caret.release();}
        }
        s.browser.draw(atlas,batch,&mut s.caret,s.focused_at,cursor);
        s.browser.draw_buttons(atlas,batch,cursor);
        s.browser.tooltip(atlas,batch,cursor);
    });
}
pub fn click(title:&mut Title,x:f32,y:f32,size:(f32,f32))->(Action,bool) {
    let Some((outcome,sound))=with(|s| {
        let hit=s.browser.buttons(size).into_iter().find(|(bx,by,w,_,on,_)| *on && crate::ui::hovered((x,y),*bx,*by,*w,20.0)).map(|b|b.5);
        let outcome=s.browser.click(hit,(x,y),size);
        s.press=Some(((x,y),size,s.browser.modifiers.1));s.mouse_down=true;
        (outcome,hit.is_some())
    }) else { return (Action::None,false); };
    (apply(title,outcome),sound)
}
pub fn typing()->bool { with(|s| s.browser.focused_text().is_some()).unwrap_or(false) }
fn edit(f:impl FnOnce(&mut crate::ui::EditBox)) {
    with(|s| {
        if let Some((id,value,max))=s.browser.focused_text() { s.caret.edit(id,value,max,f); }
    });
}
pub fn type_text(text:&str) { edit(|e| e.insert_text(text)); }
pub fn field_key(key:KeyCode,ctrl:bool,shift:bool)->bool {
    let mut handled=false; edit(|e| handled=e.key(key,ctrl,shift)); handled
}
pub fn shortcut(shortcut:EditShortcut,board:&mut dyn Clipboard) { edit(|e| { e.shortcut(shortcut,board); }); }
pub fn press_field(atlas:&HudAtlas,cursor:(f32,f32),size:(f32,f32),shift:bool) {
    with(|s| {
        if let Some(rect @ [x,y,w,h])=s.browser.focused_box(size)
            && crate::ui::hovered(cursor,x,y,w,h)
            && let Some(field)=s.browser.focused_text() {
                s.caret.press(atlas,field,rect,cursor.0,shift);
            }
    });
}
pub fn drag_field(atlas:&HudAtlas,x:f32) { with(|s| { if let Some(field)=s.browser.focused_text() { s.caret.drag(atlas,field,x); } }); }
pub fn release_field() { with(|s| {s.caret.release();s.mouse_down=false;}); }
pub fn drag(cursor:(f32,f32),size:(f32,f32)) { with(|s| s.browser.drag(cursor.1,size)); }
pub fn release() { with(|s| {s.browser.release();s.caret.release();s.mouse_down=false;}); }
pub fn wheel(delta:f32,cursor:(f32,f32),size:(f32,f32)) { with(|s| s.browser.wheel(delta,cursor,size)); }
pub fn refresh() { with(|s| s.browser.refresh()); }
pub fn escape(title:&mut Title) { CONNECT.take(); if with(|s| s.browser.escape()).unwrap_or(true) { close(); title.mode=Mode::Main; } }
fn key_action(title:&mut Title,key:KeyCode,ctrl:bool,shift:bool)->Option<Action> {
    let outcome=with(|s| s.browser.key(key,ctrl,shift)).flatten()?;
    Some(apply(title,outcome))
}
pub fn queue_action(action:Action) { if let Action::Connect{address,lan}=action { CONNECT.set(Some((address,lan))); } }
pub fn key(title:&mut Title,key:KeyCode,ctrl:bool,shift:bool) { if let Some(action)=key_action(title,key,ctrl,shift) {queue_action(action);} }
pub fn confirm(title:&mut Title) { key(title,KeyCode::Enter,false,false); }
pub fn take_connect()->Option<(String,bool)> { CONNECT.take() }
pub fn event(title:&mut Title,event:&WindowEvent,ctrl:bool,shift:bool,alt:bool,board:&mut dyn Clipboard)->Option<Action> {
    if let WindowEvent::ModifiersChanged(modifiers)=event {with(|s| s.browser.set_modifiers(modifiers.state().control_key(),modifiers.state().shift_key()));}
    if let WindowEvent::KeyboardInput{event,..}=event { key_event(title,event,ctrl,shift,alt,board) } else {None}
}
pub fn control_key(title:&mut Title,key:KeyCode,action:&str,board:&mut dyn Clipboard)->Action {
    if matches!(key,KeyCode::ControlLeft|KeyCode::ControlRight|KeyCode::ShiftLeft|KeyCode::ShiftRight) {
        let (mut ctrl,mut shift)=HELD.get();
        if matches!(key,KeyCode::ControlLeft|KeyCode::ControlRight) {ctrl=action!="up";} else {shift=action!="up";}
        HELD.set((ctrl,shift));with(|s| s.browser.set_modifiers(ctrl,shift));return Action::None;
    }
    if action=="up" {return Action::None;}
    let (ctrl,shift)=HELD.get();
    let letter=match key {KeyCode::KeyA=>Some('a'),KeyCode::KeyC=>Some('c'),KeyCode::KeyV=>Some('v'),KeyCode::KeyX=>Some('x'),_=>None};
    if !active(title) {
        if let Some(edit)=crate::clipboard::edit_shortcut(letter,ctrl,shift,false) {title.field_shortcut(edit,board);return Action::None;}
        match key {
            KeyCode::Escape=>title.escape(),KeyCode::F5=>title.refresh_servers(),KeyCode::Tab=>title.tab(),
            KeyCode::Enter|KeyCode::NumpadEnter=>title.confirm_edit(),
            _=>{title.field_key(key,ctrl,shift);}
        }
        return Action::None;
    }
    if let Some(edit)=crate::clipboard::edit_shortcut(letter,ctrl,shift,false) && typing() {shortcut(edit,board);return Action::None;}
    if key==KeyCode::Escape {escape(title);return Action::None;}
    if let Some(action)=key_action(title,key,ctrl,shift) {return action;}
    field_key(key,ctrl,shift);Action::None
}
pub fn key_event(title:&mut Title,event:&KeyEvent,ctrl:bool,shift:bool,alt:bool,board:&mut dyn Clipboard)->Option<Action> {
    if !active(title) || event.state!=ElementState::Pressed {return None;}
    let PhysicalKey::Code(key)=event.physical_key else {return None;};
    if let Some(edit)=crate::clipboard::edit_shortcut(crate::clipboard::shortcut_letter(&event.logical_key,event.physical_key),ctrl,shift,alt) && typing() {
        shortcut(edit,board);return Some(Action::None);
    }
    if key==KeyCode::Escape {escape(title);return Some(Action::None);}
    if let Some(action)=key_action(title,key,ctrl,shift) {return Some(action);}
    if typing() && field_key(key,ctrl,shift) {return Some(Action::None);}
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancelling_early_join_drops_pending_connection() {
        let scratch=std::env::temp_dir().join(format!("browser-native-cancel-{}",std::process::id()));
        std::fs::create_dir_all(&scratch).unwrap();
        let mut title=Title::new(scratch.join("saves"),&scratch);
        assert!(enter(&mut title));
        // Closing a dialog must cancel an early-startup connection even while
        // the browser itself remains open for the next resource-loading frame.
        with(|s| s.browser.modal=super::super::Modal::Direct(String::new()));
        queue_action(Action::Connect { address:"127.0.0.1:25565".into(), lan:true });
        assert_eq!(take_connect(),Some(("127.0.0.1:25565".into(),true)));
        queue_action(Action::Connect { address:"127.0.0.1:25565".into(), lan:true });
        escape(&mut title);
        assert!(active(&title));
        assert!(take_connect().is_none());
        queue_action(Action::Connect { address:"127.0.0.1:25565".into(), lan:false });
        close();
        assert!(take_connect().is_none());
        assert!(!active(&title));
        std::fs::remove_dir_all(scratch).unwrap();
    }
    #[test]
    fn explicitly_disabled_native_mod_keeps_classic() {
        let scratch=std::env::temp_dir().join(format!("browser-native-{}",std::process::id()));
        std::fs::create_dir_all(&scratch).unwrap();
        std::fs::write(scratch.join("horizon-mods.txt"),"mod.server_browser:false\n").unwrap();
        assert!(open(&scratch.join("saves"),false).is_none());
        std::fs::remove_file(scratch.join("horizon-mods.txt")).unwrap();
        std::fs::remove_dir(scratch).unwrap();
    }
    #[test]
    fn bundled_catalog_has_unique_valid_addresses() {
        let mut addresses=std::collections::HashSet::new();
        for (_,address,_) in super::super::catalog::CURATED_SERVERS.iter().chain(super::super::catalog::DIRECTORY_SNAPSHOT).chain(super::super::catalog::MODRINTH_SNAPSHOT) {
            assert!(core::address::parse(address).is_ok());
            assert!(addresses.insert(core::address::key(address).unwrap()));
        }
        assert_eq!(addresses.len(),2835);
    }
}
