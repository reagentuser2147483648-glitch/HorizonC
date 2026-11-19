fn clickgui_typed(game: &crate::game::Game<'_>, character: char, canceled: &mut bool) {
    use crate::game::clickgui as gui;
    if *canceled || !game.native_on(gui::ID) || game.native_mod_screen() != Some(gui::ID) {
        return;
    }
    if gui::type_char(character) {
        *canceled = true;
    }
}
