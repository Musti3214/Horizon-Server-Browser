"""Assemble our existing browser into a real, self-contained Horizon native mod.

No host patch, Cargo edit, WASM capability or external source path is shipped.
The official horizon-mixins tool performs all integration from this bundle.
"""
from pathlib import Path
import json
import re
import shutil
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT / "src"
OUT = ROOT / "mixins/server_browser"
REV = "6475f62f3c834b7489e1816bf47f47ea5eb2b879"
MIXINS = "a2af5c0dd4117d51ecd70066ba7a00b4a2c10dab"
VERSION = "2.2.1"


def items(text):
    return re.sub(r"(?m)^\s*//!", "//", text)


def read(path):
    if path.startswith("crates/horizon-server-browser/src/"):
        target = SOURCE / "core" / Path(path).name
    elif path.startswith("crates/horizon-client/src/server_browser/"):
        target = SOURCE / "ui" / Path(path).name
    elif "/lang/" in path:
        target = SOURCE / "lang" / Path(path).name
    elif path.endswith("catalog.rs"):
        target = SOURCE / "catalog.rs"
    else:
        raise ValueError(f"Unknown mod source: {path}")
    return target.read_text(encoding="utf-8")


def write(name, text):
    (OUT / name).write_text(text.rstrip() + "\n", encoding="utf-8", newline="\n")


def assemble():
    OUT.mkdir(parents=True, exist_ok=True)
    core_path = "crates/horizon-server-browser/src/"
    core = items(read(core_path + "lib.rs"))
    for name in ("address", "directory", "model", "persistence", "query", "service"):
        source = items(read(core_path + name + ".rs"))
        fixture = SOURCE / "state-v1.json"
        source = source.replace('include_bytes!("../tests/fixtures/state-v1.json")', "&[" + ",".join(map(str, fixture.read_bytes())) + "]")
        source = source.replace("crate::", "crate::server_browser::core::")
        source = source.replace("horizon_protocol::connection::server_addresses(", "horizon_protocol::connection::server_browser_resolver::addresses(")
        core = core.replace(f"pub mod {name};", f"pub mod {name} {{\n{source}\n}}")
    source = items(read("crates/horizon-client/src/server_browser/mod.rs"))
    for name in ("draw", "forms", "online", "tests"):
        body = items(read(f"crates/horizon-client/src/server_browser/{name}.rs"))
        source = source.replace(f"mod {name};", f"mod {name} {{\n{body}\n}}")
    source = source.replace("use horizon_server_browser::{\n    self as core,", "use self::core::{")
    source = source.replace("horizon_server_browser::", "crate::server_browser::core::")
    source = source.replace("use horizon_server_browser as core;", "use crate::server_browser::core;")
    source = source.replace("multiplayer::{ButtonSpec, Outcome}", "multiplayer::ButtonSpec")
    source = source.replace("crate::multiplayer::icon_pixels(bytes)", "super::icon_pixels(bytes)")
    source = source.replace("crate::multiplayer::draw_server_icon(", "super::draw_server_icon(")
    start = source.index("fn tr(key: &str) -> String {")
    end = source.index("\nfn label_value", start)
    languages = {}
    for code in ("en_us", "de_de"):
        values = json.loads(read(f"crates/horizon-client/assets/horizon/lang/{code}.json"))
        languages[code] = json.dumps({k: v for k, v in values.items() if k.startswith("horizon.browser.")}, ensure_ascii=False)
    translations = "\n".join(f'const {code.upper()}: &str = r###"{body}"###;' for code, body in languages.items())
    source = source[:start] + '''fn tr(key: &str) -> String {
        let key = format!("horizon.browser.{key}");
        if crate::lang::has(&key) { return crate::lang::get(&key); }
        static EN: std::sync::OnceLock<std::collections::HashMap<String,String>> = std::sync::OnceLock::new();
        static DE: std::sync::OnceLock<std::collections::HashMap<String,String>> = std::sync::OnceLock::new();
        let values = if crate::lang::code() == "de_de" {
            DE.get_or_init(|| serde_json::from_str(DE_DE).unwrap_or_default())
        } else { EN.get_or_init(|| serde_json::from_str(EN_US).unwrap_or_default()) };
        values.get(&key).cloned().unwrap_or(key)
    }
''' + source[end:]
    provider_start = source.index("#[derive(Clone, Default)]\npub struct Provider")
    provider_end = source.index("\n#[derive(Clone, Debug)]\nenum Modal", provider_start)
    source = source[:provider_start] + '''fn provider_servers() -> Vec<core::model::Server> {
        catalog::CURATED_SERVERS.iter().chain(catalog::MODRINTH_SNAPSHOT).map(|(name,address,tags)| {
            let mut server = core::model::Server { name: (*name).into(), address: (*address).into(), ..Default::default() };
            server.metadata.tags = tags.iter().map(|s| (*s).into()).collect();
            server
        }).collect()
    }
    pub fn open_support_url() {
        let url = "https://ko-fi.com/foryou41";
        #[cfg(windows)]
        if let Err(error) = open::that_detached(url) { log::warn!("Server Browser: {error}"); }
        #[cfg(not(windows))]
        {
            let program = if cfg!(target_os="macos") { "open" } else { "xdg-open" };
            if let Err(error) = std::process::Command::new(program).arg(url).spawn() { log::warn!("Server Browser: {error}"); }
        }
    }
''' + source[provider_end:]
    icons = (SOURCE / "icons.rs").read_text(encoding="utf-8").replace("pub(crate) fn", "fn")
    icons += '''\nfn icon_pixels(bytes: &[u8]) -> Option<std::sync::Arc<Vec<u8>>> {
        crate::hud::decode_bytes(bytes).ok().filter(|(w,h,_)| *w==64 && *h==64).map(|(_,_,p)| std::sync::Arc::new(p))
    }
'''
    catalog = items(read("mods/community/server-browser/src/catalog.rs"))
    if re.search(r'"(?:cracked|offline-mode)"', catalog, flags=re.I):
        raise ValueError("Remove offline-access listings from the bundled catalog")
    license_text = (ROOT / "LICENSE").read_text(encoding="utf-8")
    notice = "// Server Atlas — native mod by shuab. SPDX-License-Identifier: MIT\n"
    notice += "\n".join("// " + line for line in license_text.splitlines()) + "\n"
    payload = notice + f"pub mod core {{\n{core}\n}}\n" + source + "\n" + translations + "\n" + icons
    payload += f"\nmod catalog {{\n{catalog}\n}}\n"
    payload += "\n#[derive(Debug)]\npub enum Outcome { None, Back, Connect { address: String, lan: bool }, Classic }\n"
    payload += "\npub mod native {\n" + (ROOT / "native.rs").read_text(encoding="utf-8") + "\n}\n"
    write("runtime.rs", payload)

    manifest = f'''[mod]
id = "server_browser"
version = "{VERSION}"
priority = 100
[compatibility]
horizon_revision = "{REV}"
[[module]]
id = "runtime"
file = "crates/horizon-client/src/client_lib.rs"
name = "server_browser"
source = "runtime.rs"
visibility = "pub"
[[module]]
id = "resolver"
file = "crates/horizon-protocol/src/connection.rs"
name = "server_browser_resolver"
source = "resolver.rs"
visibility = "pub"
'''
    write("resolver.rs", "pub fn addresses(host: &str, port: u16) -> crate::Result<Vec<std::net::SocketAddr>> { crate::resolver::resolve(host, port).map(|resolved| super::ipv4_first(resolved.addresses)) }")

    def hook(name, target, params, body, result=None, kind="HEAD"):
        nonlocal manifest
        args = ", ".join(f"{n}: {t}" for n, t, _ in params)
        ret = (f" -> std::ops::ControlFlow<{result}>" if kind=="HEAD" else f" -> {result}") if result is not None else ""
        write(name + ".rs", f"fn hook({args}){ret} {{\n{body}\n}}")
        manifest += f'\n[[injection]]\nid = "{name}"\ntarget = "{target}"\nkind = "{kind}"\nhelper = "{name}.rs"\n'
        if result is not None and kind=="HEAD":
            manifest += "cancellable = true\n"
        manifest += "bindings = [\n" + "\n".join(f'  {{ name = {json.dumps(n)}, ty = {json.dumps(t)}, value = {json.dumps(v)} }},' for n, t, v in params) + "\n]\n"

    title = "horizon_client_lib::title::Title::"
    base = [("title", "&mut crate::title::Title", "&mut *self")]
    shared = [("title", "&crate::title::Title", "self")]
    def title_hook(name, params, result, call, target_name=None):
        action = ((call + ";\n") if call != "()" else "") + "return std::ops::ControlFlow::Break(());" if result == "()" else f"return std::ops::ControlFlow::Break({call});"
        body = f"if crate::server_browser::native::active(title) {{\n{action}\n}}\nstd::ops::ControlFlow::Continue(())"
        hook("title_" + name, title + (target_name or name), params, body, result)

    title_hook("draw", shared + [("atlas", "&crate::hud::HudAtlas", "atlas"),("batch", "&mut crate::hud::HudBatch", "&mut *batch"),("cursor", "(f32, f32)", "cursor")], "()", "crate::server_browser::native::draw(atlas,batch,cursor)", target_name="draw_with_items")
    hook("title_click", title + "click", base + [("x","f32","x"),("y","f32","y"),("size","(f32, f32)","size")], '''if crate::server_browser::native::active(title) {
        return std::ops::ControlFlow::Break(crate::server_browser::native::click(title,x,y,size));
    }
    if matches!(title.mode, crate::title::Mode::Main) {
        let multiplayer = title.buttons(size.0.ceil(),size.1.ceil()).into_iter().any(|b| b.id==4 && b.enabled && crate::ui::hovered((x,y),b.x,b.y,b.w,20.0));
        if multiplayer && crate::server_browser::native::enter(title) {return std::ops::ControlFlow::Break((crate::title::Action::None,true));}
    }
    if crate::server_browser::native::classic_click(title,(x,y),size) {return std::ops::ControlFlow::Break((crate::title::Action::None,true));}
    std::ops::ControlFlow::Continue(())''', "(Action, bool)")
    hook("classic_switch", "horizon_client_lib::multiplayer::Screen::draw_tooltip", [("screen","&crate::multiplayer::Screen","self"),("atlas","&crate::hud::HudAtlas","atlas"),("batch","&mut crate::hud::HudBatch","&mut *batch"),("cursor","(f32, f32)","cursor")], "crate::server_browser::native::draw_classic_switch(screen,atlas,batch,cursor);")
    hook("title_focused_field", title+"focused_field",base,'''if crate::server_browser::native::active(title) || title.rules.get_mut().is_some() { return None; }
    match &mut title.mode {
        crate::title::Mode::Multiplayer(screen) => screen.focused_text(),
        mode => caret_field(mode, title.search_focused, &mut title.search),
    }''',"Option<(crate::clipboard::FieldId, &mut String, usize)>",kind="OVERWRITE")
    title_hook("remember_direct_address",base,"()","()")
    title_hook("has_caret_field", base, "bool", "crate::server_browser::native::typing()")
    title_hook("type_text", base + [("text","&str","text")], "()", "crate::server_browser::native::type_text(text)")
    title_hook("field_key", base + [("key","winit::keyboard::KeyCode","key"),("ctrl","bool","ctrl"),("shift","bool","shift")], "bool", "crate::server_browser::native::field_key(key,ctrl,shift)")
    title_hook("field_shortcut", base + [("shortcut","crate::clipboard::EditShortcut","shortcut"),("board","&mut dyn crate::clipboard::Clipboard","&mut *board")], "()", "crate::server_browser::native::shortcut(shortcut,board)")
    title_hook("drag_field", base + [("atlas","&crate::hud::HudAtlas","atlas"),("x","f32","x")], "()", "crate::server_browser::native::drag_field(atlas,x)")
    title_hook("release_field", base, "()", "crate::server_browser::native::release_field()")
    title_hook("drag", base + [("cursor","(f32, f32)","cursor"),("size","(f32, f32)","size")], "()", "crate::server_browser::native::drag(cursor,size)")
    title_hook("release", base, "()", "crate::server_browser::native::release()")
    title_hook("scroll_worlds", base + [("delta","f32","delta"),("cursor","(f32, f32)","cursor"),("size","(f32, f32)","size")], "bool", "{ crate::server_browser::native::wheel(delta,cursor,size); false }")
    title_hook("escape", base, "()", "crate::server_browser::native::escape(title)")
    title_hook("refresh_servers", base, "()", "crate::server_browser::native::refresh()")
    title_hook("tab", base, "()", "crate::server_browser::native::key(title,winit::keyboard::KeyCode::Tab,false,false)")
    title_hook("confirm_edit", base, "()", "crate::server_browser::native::confirm(title)")
    # Hooks in the original game bin own authentication and connection handling.
    game = [("game", "&mut crate::game::Game<'_>", "&mut *self")]
    hook("mods", "horizon_client::game::native_hooks::Game<'_>::native_mods", game + [("mods", "&mut Vec<crate::game::native_hooks::NativeMod>", "&mut *mods")], '''let _ = game;
    mods.push(crate::game::native_hooks::NativeMod {
        id: "server_browser".into(), title: "Server Atlas 2.2.0".into(),
        description: "Native server browser: search, filters, favorites, LAN and public discovery.".into(), keys: Vec::new(),
    });'''.replace("2.2.0", VERSION))
    hook("frame", "horizon_client::game::native_hooks::Game<'_>::native_frame", game, '''if !game.native_window() { return; }
    if let Some(title) = &mut game.title {
        let on=!game.settings.mods.disabled.contains("server_browser");
        horizon_client::server_browser::native::sync(title, on);
    } else { horizon_client::server_browser::native::close(); }
    if let Some((address,lan)) = horizon_client::server_browser::native::take_connect()
        && let Err(error) = game.connect_server_with_options(&address, &game.options.player_name(),None,lan) {
        log::warn!("Server Browser: {error}");
    }''')
    hook("key_event", "horizon_client::game::<Game<'_> as ApplicationHandler>::window_event", game + [("event","&winit::event::WindowEvent","&event"),("window_id","winit::window::WindowId","id")], '''if game.window.as_ref().is_none_or(|window| window.id()!=window_id) { return std::ops::ControlFlow::Continue(()); }
    if game.screen!=crate::game::Screen::Playing {return std::ops::ControlFlow::Continue(());}
    let action = if let Some(title) = &mut game.title {
        horizon_client::server_browser::native::event(title, event, game.modifiers.control_key(), game.modifiers.shift_key(), game.modifiers.alt_key(), &mut game.clipboard)
    } else { None };
    if let Some(action) = action {
        if let horizon_client::title::Action::Connect { address,lan } = action
            && let Err(error) = game.connect_server_with_options(&address, &game.options.player_name(),None,lan) {
            log::warn!("Server Browser: {error}");
        }
        return std::ops::ControlFlow::Break(());
    }
    std::ops::ControlFlow::Continue(())''', "()")
    hook("startup_key", "horizon_client::game::startup_menu::StartupMenu::title_key", [("menu","&mut crate::game::startup_menu::StartupMenu","&mut *self"),("event","&winit::event::KeyEvent","event")], '''let action = if let Some(title) = &mut menu.title {
        horizon_client::server_browser::native::key_event(title,event,menu.modifiers.control_key(),menu.modifiers.shift_key(),menu.modifiers.alt_key(),&mut menu.clipboard)
    } else { None };
    if let Some(action) = action {
        return std::ops::ControlFlow::Break(action);
    }
    std::ops::ControlFlow::Continue(())''', "Action")
    hook("control", "horizon_client::game::control::Game<'_>::control_command", game + [("line","&str","line")], '''if game.title.is_some() && game.screen==crate::game::Screen::Playing {
        let words: Vec<_> = line.split_whitespace().collect();
        if words.first()==Some(&"key") && let Some(code)=words.get(1).and_then(|s|key_code(s)) {
                let action = {
                    let Some(title)=game.title.as_mut() else {return std::ops::ControlFlow::Break(Err("title closed".into()));};
                    horizon_client::server_browser::native::control_key(title,code,words.get(2).copied().unwrap_or("tap"),&mut game.clipboard)
                };
                if let horizon_client::title::Action::Connect{address,lan}=action
                    && let Err(error)=game.connect_server_with_options(&address,&game.options.player_name(),None,lan) {
                    return std::ops::ControlFlow::Break(Err(error));
                }
                return std::ops::ControlFlow::Break(Ok(Some(serde_json::json!({"ok":true}))));
        }
        if let Some(text)=line.strip_prefix("type ") {
            if let Some(title)=game.title.as_mut() {title.type_text(text);}
            return std::ops::ControlFlow::Break(Ok(Some(serde_json::json!({"ok":true}))));
        }
    }
    std::ops::ControlFlow::Continue(())''', "Result<Option<Value>>")
    hook("pad_typing", "horizon_client::game::pad_menu::Game<'_>::pad_typing", [("game","&crate::game::Game<'_>","self")], '''if game.screen==crate::game::Screen::Playing && game.title.as_ref().is_some_and(horizon_client::server_browser::native::active) {
        return std::ops::ControlFlow::Break(horizon_client::server_browser::native::typing());
    }
    std::ops::ControlFlow::Continue(())''',"bool")
    hook("pad_keyboard", "horizon_client::game::pad_menu::Game<'_>::pad_osk_output", game + [("out","&horizon_client::osk::Output","&out")], '''if game.screen==crate::game::Screen::Playing && game.title.as_ref().is_some_and(horizon_client::server_browser::native::active) {
        let mut action=horizon_client::title::Action::None;
        if let Some(title)=game.title.as_mut() {
            match out {
                horizon_client::osk::Output::Text(text)=>title.type_text(text),
                horizon_client::osk::Output::Backspace=>title.backspace(),
                horizon_client::osk::Output::Enter=>action=horizon_client::server_browser::native::control_key(title,winit::keyboard::KeyCode::Enter,"tap",&mut game.clipboard),
                horizon_client::osk::Output::Close=>{game.pad.menu.osk=None;game.pad.menu.osk_dismissed=true;}
            }
        }
        if let horizon_client::title::Action::Connect{address,lan}=action
            && let Err(error)=game.connect_server_with_options(&address,&game.options.player_name(),None,lan) {
            log::warn!("Server Browser: {error}");
        }
        return std::ops::ControlFlow::Break(());
    }
    std::ops::ControlFlow::Continue(())''',"()")
    write("horizon-mixin.toml", manifest)
    for obsolete in ("open.rs", "title_press_field.rs"):
        (OUT / obsolete).unlink(missing_ok=True)
    formatter=shutil.which("rustfmt")
    if not formatter:
        raise SystemExit("rustfmt (Rust 1.99.1) must be on PATH to build a reproducible bundle")
    subprocess.run([formatter,"--edition","2024","--config","newline_style=Unix",*map(str,sorted(OUT.glob("*.rs")))],check=True)
    native = {"schema":1,"id":"server_browser","version":VERSION,"game":"26.3","horizon":REV,"mixins":MIXINS}
    (ROOT / "horizon-native.json").write_text(json.dumps(native,indent=2)+"\n",encoding="utf-8",newline="\n")
    package = ROOT / f"server-browser-{VERSION}.hnmod"
    with zipfile.ZipFile(package,"w",zipfile.ZIP_DEFLATED,compresslevel=9) as z:
        for path in [ROOT / "horizon-native.json", *sorted(OUT.glob("*"))]:
            if path.is_file():
                info=zipfile.ZipInfo(path.relative_to(ROOT).as_posix(),date_time=(2026,11,1,0,0,0))
                info.compress_type=zipfile.ZIP_DEFLATED
                info.external_attr=0o100644<<16
                z.writestr(info,path.read_bytes())
        info=zipfile.ZipInfo("mixins/server_browser/LICENSE",date_time=(2026,11,1,0,0,0))
        info.compress_type=zipfile.ZIP_DEFLATED
        info.external_attr=0o100644<<16
        z.writestr(info,(ROOT/"LICENSE").read_bytes())
    print(package)


if __name__ == "__main__":
    assemble()
