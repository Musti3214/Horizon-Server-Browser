fn hook(game: &mut crate::game::Game<'_>, mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    let _ = game;
    mods.push(crate::game::native_hooks::NativeMod {
        id: "server_browser".into(),
        title: "Modern Server Browser 2.1.0".into(),
        description: "Native server browser: search, filters, favorites, LAN and opt-in discovery."
            .into(),
        keys: Vec::new(),
    });
}
