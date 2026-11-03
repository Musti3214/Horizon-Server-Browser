fn hook(
    app: &mut crate::game::App,
    event: &winit::event::WindowEvent,
) -> std::ops::ControlFlow<()> {
    let crate::game::App::Ready(game) = app else {
        return std::ops::ControlFlow::Continue(());
    };
    if game.screen != crate::game::Screen::Playing {
        return std::ops::ControlFlow::Continue(());
    }
    let action = if let Some(title) = &mut game.title {
        horizon_client::server_browser::native::event(
            title,
            event,
            game.modifiers.control_key(),
            game.modifiers.shift_key(),
            game.modifiers.alt_key(),
            &mut game.clipboard,
        )
    } else {
        None
    };
    if let Some(action) = action {
        if let horizon_client::title::Action::Connect { address } = action
            && let Err(error) = game.connect_server(&address, &game.options.player_name())
        {
            log::warn!("Server Browser: {error}");
        }
        return std::ops::ControlFlow::Break(());
    }
    std::ops::ControlFlow::Continue(())
}
