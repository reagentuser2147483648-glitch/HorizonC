fn clickgui_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::clickgui as gui;
    if !pressed || !game.native_on(gui::ID) || gui::editing() {
        return;
    }
    if key != game.native_mod_key(gui::ID, gui::KEY, gui::DEFAULT_KEY) {
        return;
    }
    if game.native_mod_screen() == Some(gui::ID) {
        game.native_mod_screen_close(gui::ID);
    } else if game.native_playing() {
        game.native_mod_screen_open(gui::ID);
    }
}
