fn clickgui_frame(game: &mut crate::game::Game<'_>) {
    use crate::game::clickgui as gui;
    if game.native_on(gui::ID) && gui::take_pending(game.native_playing()) {
        game.native_mod_screen_open(gui::ID);
    }
}
