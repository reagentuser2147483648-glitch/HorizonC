fn clickgui_keypress(game: &crate::game::Game<'_>, key: &str, canceled: &mut bool) {
    use crate::game::clickgui as gui;
    if *canceled || !game.native_on(gui::ID) || game.native_mod_screen() != Some(gui::ID) {
        return;
    }
    if gui::key_pressed(key) {
        *canceled = true;
    }
}
