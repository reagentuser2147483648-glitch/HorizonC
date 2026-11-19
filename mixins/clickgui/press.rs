fn clickgui_press(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    use crate::game::clickgui as gui;
    if *canceled || !game.native_on(gui::ID) || game.native_mod_screen() != Some(gui::ID) {
        return;
    }
    let [x, y] = screen.mouse;
    let [_, _, width, height] = screen.rect;
    if gui::world().is_none() {
        gui::choose_world(x, y, width, height);
        *canceled = true;
        return;
    }
    if gui::toggle_world(x, y, width) || gui::click(x, y, button) {
        *canceled = true;
    }
}
