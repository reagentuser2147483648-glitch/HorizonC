fn clickgui_scroll(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    scroll: [f64; 2],
    canceled: &mut bool,
) {
    use crate::game::clickgui as gui;
    if !game.native_on(gui::ID) || game.native_mod_screen() != Some(gui::ID) {
        return;
    }
    gui::pan(scroll[1] as f32, screen.rect[2]);
    *canceled = true;
}
