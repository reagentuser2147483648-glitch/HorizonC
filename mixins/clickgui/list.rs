fn clickgui_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::clickgui as gui;
    mods.push(crate::game::native_hooks::NativeMod {
        id: gui::ID.into(),
        title: format!("Horizon ClickGUI {}", gui::VERSION),
        description: gui::DESCRIPTION.into(),
        keys: vec![[gui::KEY.into(), "Menu key".into(), gui::DEFAULT_KEY.into()]],
    });
}
