fn clickgui_blur(game: &crate::game::Game<'_>, radius: &mut Option<f32>) {
    use crate::game::clickgui as gui;
    if game.native_on(gui::ID) && game.native_mod_screen() == Some(gui::ID) {
        *radius = Some(gui::BLUR_RADIUS);
    }
}
