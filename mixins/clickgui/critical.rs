fn clickgui_critical(
    game: &crate::game::Game<'_>,
    vanilla_critical: bool,
    critical: &mut bool,
    damage_multiplier: &mut f32,
) {
    use crate::game::clickgui as gui;
    if vanilla_critical || !game.native_on(gui::ID) || !gui::criticals_hook() {
        return;
    }
    *critical = true;
    *damage_multiplier = 1.5;
}
