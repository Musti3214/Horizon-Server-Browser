"""One-time, marker-checked update of our renderer; no game textures are copied."""
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]
path = ROOT/'src/ui/draw.rs'
source = path.read_text(encoding='utf8')
backup = ROOT.parent/'horizon-server-browser/.local/native-browser/minecraft-20261103'
backup.mkdir(parents=True,exist_ok=True)
if not (backup/'draw-before.rs').exists(): shutil.copy2(path,backup/'draw-before.rs')

start=source.index('    pub(super) fn draw_buttons(')
end=source.index('    fn settings_width()',start)
source=source[:start]+'''    pub(super) fn draw_buttons(&self, atlas: &HudAtlas, batch: &mut HudBatch, cursor: (f32,f32)) {
        for (x,y,w,label,enabled,id) in self.buttons((batch.width.ceil(),batch.height.ceil())) {
            let hovered=enabled && crate::ui::hovered(cursor,x,y,w,20.0);
            // Use the game's real resource-pack-aware Minecraft button widget.
            crate::ui::button(batch,atlas,x,y,w,20.0,&Text::literal(label),enabled,hovered);
            let tab=matches!(self.modal,Modal::Main) && (10..15).contains(&id);
            let selected=tab && self.state.view.tab==Tab::ALL[(id-10) as usize];
            let sort=matches!(self.modal,Modal::Main) && matches!((id,self.state.view.sort),
                (52,Sort::Players)|(53,Sort::Ping)|(54,Sort::Name));
            if selected || sort {
                frame(batch,atlas,[x,y,w,20.0],rgba(255,255,255,220));
            }
        }
    }
''' + source[end:]
source=source.replace('let content = width.min(824.0);','let content = width.min(740.0);')
source=source.replace('const MUTED: u32 = 0xff8c96a6;','const MUTED: u32 = 0xffa0a0a0;')
source=source.replace('const SLATE_MUTED: u32 = 0xff8c96a6;','const SLATE_MUTED: u32 = 0xffa0a0a0;')
source=source.replace('const DARK_SLATE: u32 = 0xff525866;','const DARK_SLATE: u32 = 0xff707070;')

start=source.index('            // 3. Active Tab underline')
end=source.index('            let title_max_w',start)
source=source[:start]+source[end:]
start=source.index('                batch.fill(atlas, cx, 5.0, cw, 15.0')
end=source.index('                text(batch, atlas, &count',start)
source=source[:start]+source[end:]

start=source.index('                // Minecraft Item-Slot Frame')
end=source.index('                // Authentic Mojang Join',start)
source=source[:start]+'''                // Vanilla server icons are plain 32px squares, not inventory slots.
                let slot_x = left + 8.0;
                let slot_y = y + 10.0;
                crate::multiplayer::draw_server_icon(atlas,batch,pixels.as_deref().map(Vec::as_slice),
                    row-first,slot_x+2.0,slot_y+2.0);

''' +source[end:]
start=source.index('                batch.fill(atlas, pill_x, pill_y, pill_w, 13.0')
end=source.index('                text(\n                    batch,\n                    atlas,\n                    &count_str',start)
source=source[:start]+source[end:]
source=source.replace('if is_online { 0xffdce4f0 } else { DARK_SLATE }','if is_online { MUTED } else { DARK_SLATE }')
start=source.index('                batch.fill(\n                    atlas,\n                    ping_pill_x,')
end=source.index('                batch.sprite_scaled(\n                    atlas,\n                    &sprite',start)
source=source[:start]+source[end:]
start=source.index('                    let (fill, stroke, text_col) = if compatible')
end=source.index('                    text(\n                        batch,\n                        atlas,\n                        &chip_text',start)
source=source[:start]+'''                    let text_col=if compatible {EMERALD} else if status.is_none_or(|s|s.protocol.is_none()) {MUTED} else {RED};
''' +source[end:]
source=source.replace('0xff7e8e9e','0xffa0a0a0').replace('0xff94a6bc','0xffa0a0a0')
path.write_text(source,encoding='utf8',newline='\n')
print('Minecraft widgets, neutral text, plain icons and compact centered viewport applied.')
