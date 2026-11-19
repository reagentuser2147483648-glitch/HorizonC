fn clickgui_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    use crate::game::clickgui as gui;
    if *ran || !game.native_on(gui::ID) || command.trim() != gui::COMMAND {
        return;
    }
    gui::request_open();
    *ran = true;
}
