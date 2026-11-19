fn clickgui_draw(
    game: &crate::game::Game<'_>,
    batch: &mut crate::hud::HudBatch,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
) {
    use crate::game::clickgui as gui;

    const SCRIM: u32 = 0x99020510;
    const PANEL: u32 = 0xF00A1222;
    const DEEP: u32 = 0xF0060B18;
    const EDGE: u32 = 0xFF3B82F6;
    const EDGE_SOFT: u32 = 0xFF1E3A8A;
    const ACCENT: u32 = 0xFF60A5FA;
    const ON_FILL: u32 = 0xF01E3A8A;
    const HOVER: u32 = 0xF0102244;
    const GLOW: u32 = 0x403B82F6;
    const TEXT: u32 = 0xE6EEFF;
    const DIM: u32 = 0x7C8DB5;
    const BRIGHT: u32 = 0x93C5FD;

    fn tint(rgb: u32, a: f32) -> u32 {
        crate::game::clickgui::mix_rgb(0xFF050A14, 0xFF000000 | rgb, a) & 0x00FF_FFFF
    }

    fn fit(text: &str, max: usize, tail: bool) -> String {
        let count = text.chars().count();
        if count <= max {
            return text.to_owned();
        }
        if tail {
            text.chars().skip(count - max).collect()
        } else {
            let mut head: String = text.chars().take(max - 1).collect();
            head.push('~');
            head
        }
    }

    fn chamfer(
        game: &crate::game::Game<'_>,
        batch: &mut crate::hud::HudBatch,
        rect: [f32; 4],
        cut: f32,
        argb: u32,
    ) {
        let [x, y, w, h] = rect;
        game.native_fill(batch, [x + cut, y, w - cut, cut], argb);
        game.native_fill(batch, [x, y + cut, w, h - 2.0 * cut], argb);
        game.native_fill(batch, [x, y + h - cut, w - cut, cut], argb);
        game.native_quad(
            batch,
            [[x, y + cut], [x + cut, y], [x + cut, y + cut], [x, y + cut]],
            argb,
        );
        game.native_quad(
            batch,
            [
                [x + w, y + h - cut],
                [x + w - cut, y + h],
                [x + w - cut, y + h - cut],
                [x + w, y + h - cut],
            ],
            argb,
        );
    }

    fn framed(
        game: &crate::game::Game<'_>,
        batch: &mut crate::hud::HudBatch,
        rect: [f32; 4],
        cut: f32,
        edge: u32,
        fill: u32,
        glow: u32,
    ) {
        let [x, y, w, h] = rect;
        chamfer(game, batch, [x - 1.0, y - 1.0, w + 2.0, h + 2.0], cut + 1.0, glow);
        chamfer(game, batch, rect, cut, edge);
        chamfer(game, batch, [x + 1.0, y + 1.0, w - 2.0, h - 2.0], cut - 1.0, fill);
    }

    fn centered(
        game: &crate::game::Game<'_>,
        batch: &mut crate::hud::HudBatch,
        text: &str,
        rect: [f32; 4],
        dy: f32,
        rgb: u32,
    ) {
        let width = game.native_text_width(text);
        game.native_text(batch, text, [rect[0] + (rect[2] - width) / 2.0, rect[1] + dy], rgb, true);
    }

    if !game.native_on(gui::ID) || game.native_mod_screen() != Some(gui::ID) {
        return;
    }
    let open = gui::begin_frame();
    let t = gui::time();
    let pulse = 0.7 + 0.3 * (t * 2.4).sin();
    let blink = (t * 2.0) as i32 % 2 == 0;
    let [mx, my] = screen.mouse;
    let (w, h) = (batch.width, batch.height);
    game.native_fill(batch, [0.0, 0.0, w, h], gui::scale_alpha(SCRIM, open));

    let bracket = gui::scale_alpha(EDGE, open * pulse);
    for (bx, by, sx, sy) in [
        (6.0, 6.0, 1.0, 1.0),
        (w - 6.0, 6.0, -1.0, 1.0),
        (6.0, h - 6.0, 1.0, -1.0),
        (w - 6.0, h - 6.0, -1.0, -1.0),
    ] {
        let hx = if sx > 0.0 { bx } else { bx - 16.0 };
        let hy = if sy > 0.0 { by } else { by - 2.0 };
        game.native_fill(batch, [hx, hy, 16.0, 2.0], bracket);
        let vx = if sx > 0.0 { bx } else { bx - 2.0 };
        let vy = if sy > 0.0 { by } else { by - 16.0 };
        game.native_fill(batch, [vx, vy, 2.0, 16.0], bracket);
    }

    if gui::world().is_none() {
        let slide = (1.0 - open) * 10.0;
        let mut dialog = gui::dialog_rect(w, h);
        dialog[1] += slide;
        framed(
            game,
            batch,
            dialog,
            gui::CUT,
            gui::scale_alpha(EDGE, open),
            gui::scale_alpha(PANEL, open),
            gui::scale_alpha(GLOW, open * pulse),
        );
        centered(game, batch, &gui::spaced("SELECT WORLD TYPE"), dialog, 12.0, tint(TEXT, open));
        centered(
            game,
            batch,
            "SYS // MODULES ADAPT TO YOUR CHOICE",
            dialog,
            28.0,
            tint(DIM, open),
        );
        for (index, label) in ["SINGLE PLAYER", "MULTI PLAYER"].into_iter().enumerate() {
            let mut rect = gui::choice_rect(w, h, index);
            rect[1] += slide;
            let fill = if gui::contains(rect, mx, my) { ON_FILL } else { DEEP };
            framed(
                game,
                batch,
                rect,
                3.0,
                gui::scale_alpha(EDGE, open),
                gui::scale_alpha(fill, open),
                gui::scale_alpha(GLOW, open),
            );
            centered(game, batch, label, rect, 6.0, tint(TEXT, open));
        }
        return;
    }

    game.native_text(
        batch,
        &gui::spaced("SYS // ONLINE"),
        [gui::LEFT, 9.0],
        tint(DIM, open),
        false,
    );
    let title = gui::spaced("HORIZON CLIENT");
    game.native_text(batch, &title, [gui::LEFT, 21.0], tint(TEXT, open), true);
    game.native_text(batch, &title, [gui::LEFT + 1.0, 21.0], tint(TEXT, open), false);
    let shift = (t * 14.0) % 7.0;
    let mut stripe = gui::LEFT + shift - 7.0;
    while stripe + 7.0 < w - gui::LEFT {
        if stripe >= gui::LEFT {
            game.native_quad(
                batch,
                [
                    [stripe + 3.0, 34.0],
                    [stripe + 6.0, 34.0],
                    [stripe + 3.0, 38.0],
                    [stripe, 38.0],
                ],
                gui::scale_alpha(EDGE_SOFT, open),
            );
        }
        stripe += 7.0;
    }

    let button = gui::world_button(w);
    let button_fill = if gui::contains(button, mx, my) { ON_FILL } else { DEEP };
    framed(
        game,
        batch,
        button,
        3.0,
        gui::scale_alpha(EDGE, open),
        gui::scale_alpha(button_fill, open),
        gui::scale_alpha(GLOW, open * pulse),
    );
    let label = gui::world_label(gui::world()).to_uppercase();
    centered(game, batch, &label, button, 4.0, tint(TEXT, open));
    let chip = [button[0], button[1] + button[3] + 4.0, button[2], 12.0];
    framed(
        game,
        batch,
        chip,
        2.0,
        gui::scale_alpha(EDGE_SOFT, open),
        gui::scale_alpha(DEEP, open),
        gui::scale_alpha(GLOW, open * 0.5),
    );
    let active = format!("ACTIVE // {}", gui::enabled_count());
    centered(game, batch, &active, chip, 2.0, tint(BRIGHT, open));

    for (c, category) in gui::CATEGORIES.iter().enumerate() {
        let pf = gui::panel_fade(c);
        let x = gui::panel_x(c);
        let py = gui::panel_top(c);
        if x + gui::PANEL_W < 0.0 || x > w {
            continue;
        }
        let panel = [x, py, gui::PANEL_W, gui::panel_height(c)];
        framed(
            game,
            batch,
            panel,
            gui::CUT,
            gui::scale_alpha(EDGE, pf),
            gui::scale_alpha(PANEL, pf),
            gui::scale_alpha(GLOW, pf * pulse),
        );

        let cx = x + 13.0;
        let cy = py + gui::HEAD_H / 2.0;
        game.native_quad(
            batch,
            [[cx, cy - 8.0], [cx + 8.0, cy], [cx, cy + 8.0], [cx - 8.0, cy]],
            gui::scale_alpha(EDGE, pf),
        );
        game.native_quad(
            batch,
            [[cx, cy - 7.0], [cx + 7.0, cy], [cx, cy + 7.0], [cx - 7.0, cy]],
            gui::scale_alpha(DEEP, pf),
        );
        game.native_item(batch, category.icon, [cx - 4.0, cy - 4.0], 8.0);
        let name = gui::spaced(&category.name.to_uppercase());
        game.native_text(batch, &name, [x + 26.0, py + 7.0], tint(TEXT, pf), true);
        let mark = if gui::is_collapsed(c) { "+" } else { "-" };
        let mark_w = game.native_text_width(mark);
        game.native_text(
            batch,
            mark,
            [x + gui::PANEL_W - mark_w - 8.0, py + 7.0],
            tint(DIM, pf),
            true,
        );
        game.native_fill(
            batch,
            [x + 4.0, py + gui::HEAD_H, gui::PANEL_W - 8.0, 1.0],
            gui::scale_alpha(EDGE_SOFT, pf),
        );
        if gui::is_collapsed(c) {
            continue;
        }

        for (m, module) in category.modules.iter().enumerate() {
            let top = gui::module_top(c, m);
            let oa = gui::on_amount(c, m);
            let row = [x + 4.0, top + 1.0, gui::PANEL_W - 8.0, gui::ROW_H - 2.0];
            if oa > 0.01 {
                chamfer(game, batch, row, 3.0, gui::scale_alpha(ON_FILL, oa * pf));
                game.native_fill(
                    batch,
                    [row[0], row[1] + 3.0, 2.0, (row[3] - 6.0) * oa],
                    gui::scale_alpha(ACCENT, pf),
                );
            } else if gui::contains(row, mx, my) {
                chamfer(game, batch, row, 3.0, gui::scale_alpha(HOVER, pf));
            }
            let name_color = gui::mix_rgb(0xFF000000 | DIM, 0xFF000000 | TEXT, oa) & 0x00FF_FFFF;
            game.native_text(batch, module.name, [x + 11.0, top + 4.0], tint(name_color, pf), true);

            let track = [x + gui::PANEL_W - 8.0 - 16.0, top + 4.0, 14.0, 8.0];
            let outer = gui::mix_rgb(EDGE_SOFT, ACCENT, oa);
            let inner = gui::mix_rgb(DEEP, 0xFF1D4ED8, oa);
            let knob = gui::mix_rgb(0xFF5B6B8C, 0xFFFFFFFF, oa);
            game.native_fill(batch, track, gui::scale_alpha(outer, pf));
            game.native_fill(
                batch,
                [track[0] + 1.0, track[1] + 1.0, 12.0, 6.0],
                gui::scale_alpha(inner, pf),
            );
            game.native_fill(
                batch,
                [track[0] + 2.0 + 6.0 * oa, track[1] + 2.0, 4.0, 4.0],
                gui::scale_alpha(knob, pf),
            );

            let rows = gui::block_rows(c, m);
            if rows.is_empty() {
                continue;
            }
            let expanded = gui::is_expanded(c, m);
            let arrow = if expanded { "v" } else { ">" };
            let arrow_w = game.native_text_width(arrow);
            game.native_text(
                batch,
                arrow,
                [track[0] - arrow_w - 5.0, top + 4.0],
                tint(DIM, pf),
                true,
            );
            let ea = gui::expand_amount(c, m);
            if ea < 0.01 {
                continue;
            }

            let block_top = top + gui::ROW_H;
            let full = rows.len() as f32 * gui::SETTING_H + gui::SETTING_PAD;
            let now_h = full * ea;
            game.native_fill(
                batch,
                [x + 4.0, block_top, gui::PANEL_W - 8.0, now_h],
                gui::scale_alpha(DEEP, pf),
            );
            game.native_fill(
                batch,
                [x + 4.0, block_top, 1.0, now_h],
                gui::scale_alpha(EDGE, pf),
            );
            let fade = ea * pf;
            let box_x = gui::box_x(x);
            for (i, entry) in rows.iter().enumerate() {
                let y = block_top + i as f32 * gui::SETTING_H;
                if y + gui::SETTING_H > block_top + now_h + 0.5 {
                    break;
                }
                match *entry {
                    gui::Row::Setting(s) => {
                        let setting = &module.settings[s];
                        let strip = [x + 5.0, y, gui::PANEL_W - 9.0, gui::SETTING_H];
                        if gui::contains(strip, mx, my) {
                            game.native_fill(batch, strip, gui::scale_alpha(HOVER, fade));
                        }
                        game.native_text(
                            batch,
                            setting.name,
                            [x + 9.0, y + 3.0],
                            tint(DIM, fade),
                            true,
                        );
                        let boxed = [box_x, y + 2.0, gui::BOX_W, gui::SETTING_H - 4.0];
                        let editing_here = gui::edit_target() == Some((c, m, s));
                        let active = gui::is_dropdown(c, m, s) || editing_here;
                        let edge = if active { ACCENT } else { EDGE_SOFT };
                        game.native_fill(batch, boxed, gui::scale_alpha(edge, fade));
                        game.native_fill(
                            batch,
                            [boxed[0] + 1.0, boxed[1] + 1.0, boxed[2] - 2.0, boxed[3] - 2.0],
                            gui::scale_alpha(DEEP, fade),
                        );
                        let is_choice = matches!(setting.kind, gui::Kind::Choice { .. });
                        let mut text = if editing_here {
                            fit(&gui::edit_buffer(), 9, true)
                        } else {
                            fit(&gui::setting_display(c, m, s), 9, false)
                        };
                        if editing_here && blink {
                            text.push('_');
                        }
                        game.native_text(
                            batch,
                            &text,
                            [boxed[0] + 4.0, y + 3.0],
                            tint(BRIGHT, fade),
                            true,
                        );
                        if is_choice {
                            let chevron = if gui::is_dropdown(c, m, s) { "^" } else { "v" };
                            let chevron_w = game.native_text_width(chevron);
                            game.native_text(
                                batch,
                                chevron,
                                [boxed[0] + gui::BOX_W - chevron_w - 4.0, y + 3.0],
                                tint(ACCENT & 0x00FF_FFFF, fade),
                                true,
                            );
                        }
                    }
                    gui::Row::Option(s, index) => {
                        let rect = [box_x, y, gui::BOX_W, gui::SETTING_H];
                        let selected = gui::selected_choice(c, m, s) == index;
                        game.native_fill(batch, rect, gui::scale_alpha(EDGE_SOFT, fade));
                        let fill = if gui::contains(rect, mx, my) { HOVER } else { PANEL };
                        game.native_fill(
                            batch,
                            [rect[0] + 1.0, rect[1], rect[2] - 2.0, rect[3]],
                            gui::scale_alpha(fill, fade),
                        );
                        if selected {
                            game.native_text(
                                batch,
                                ">",
                                [rect[0] + 4.0, y + 3.0],
                                tint(ACCENT & 0x00FF_FFFF, fade),
                                true,
                            );
                        }
                        let color = if selected { TEXT } else { DIM };
                        game.native_text(
                            batch,
                            gui::option_text(c, m, s, index),
                            [rect[0] + 12.0, y + 3.0],
                            tint(color, fade),
                            true,
                        );
                    }
                }
            }
        }
    }

    centered(
        game,
        batch,
        "TAP TO TOGGLE // RIGHT CLICK FOR SETTINGS // SCROLL TO PAN",
        [0.0, h - 16.0, w, 10.0],
        0.0,
        tint(DIM, open),
    );
}
