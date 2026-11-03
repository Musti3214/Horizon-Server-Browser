pub(crate) fn draw_server_icon(
    atlas: &HudAtlas,
    batch: &mut HudBatch,
    pixels: Option<&[u8]>,
    slot: usize,
    x: f32,
    y: f32,
) {
    let sprite = format!("horizon/world-icon{}", slot % crate::hud::WORLD_ICON_SLOTS);
    if let Some(pixels) = pixels
        && let Some(rect) = atlas.sprites.get(&sprite)
    {
        batch.patches.push(crate::font::Patch {
            x: (rect.u0 * atlas.width as f32).round() as u32,
            y: (rect.v0 * atlas.height as f32).round() as u32,
            width: 64,
            height: 64,
            rgba: pixels.to_vec(),
        });
        batch.sprite_scaled(atlas, &sprite, x, y, 32.0, 32.0, u32::MAX);
    } else {
        batch.sprite_scaled(atlas, "misc/unknown_server", x, y, 32.0, 32.0, u32::MAX);
    }
}
