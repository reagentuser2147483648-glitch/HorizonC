use std::{
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU32, Ordering},
    },
    time::Instant,
};

pub const ID: &str = "clickgui";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str =
    "Futuristic ClickGUI for client modules. Open it with the menu key or /clickgui.";
pub const KEY: &str = "menu";
pub const DEFAULT_KEY: &str = "key.keyboard.right.shift";
pub const COMMAND: &str = "clickgui";

pub const PANEL_W: f32 = 112.0;
pub const HEAD_H: f32 = 22.0;
pub const ROW_H: f32 = 16.0;
pub const SETTING_H: f32 = 14.0;
pub const SETTING_PAD: f32 = 3.0;
pub const GAP: f32 = 10.0;
pub const LEFT: f32 = 14.0;
pub const TOP: f32 = 48.0;
pub const CUT: f32 = 5.0;
pub const BOX_W: f32 = 58.0;
pub const MAX_INPUT: usize = 24;
pub const ATTACK_REACH: f64 = 3.2;
pub const BLUR_RADIUS: f32 = 8.0;
pub const DIALOG_W: f32 = 220.0;
pub const DIALOG_H: f32 = 84.0;
pub const CHOICE_W: f32 = 90.0;
pub const CHOICE_H: f32 = 20.0;
pub const OPEN_SECS: f32 = 0.35;
pub const TOGGLE_SECS: f32 = 0.16;
pub const EXPAND_SECS: f32 = 0.2;

pub const MAX_CATEGORIES: usize = 4;
pub const MAX_MODULES: usize = 8;
pub const MAX_SETTINGS: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Worlds {
    Both,
    Single,
    Multi,
}

pub enum Kind {
    Choice {
        single: &'static [&'static str],
        multi: &'static [&'static str],
    },
    Number {
        default: f64,
        worlds: Worlds,
    },
}

pub struct Setting {
    pub name: &'static str,
    pub kind: Kind,
}

pub struct Module {
    pub name: &'static str,
    pub settings: &'static [Setting],
}

pub struct Category {
    pub name: &'static str,
    pub icon: &'static str,
    pub modules: &'static [Module],
}

pub static CATEGORIES: [Category; 1] = [Category {
    name: "Combat",
    icon: "minecraft:diamond_sword",
    modules: &[Module {
        name: "Criticals",
        settings: &[
            Setting {
                name: "Method",
                kind: Kind::Choice {
                    single: &["Hook"],
                    multi: &["Packet"],
                },
            },
            Setting {
                name: "Offset",
                kind: Kind::Number {
                    default: 0.0625,
                    worlds: Worlds::Multi,
                },
            },
        ],
    }],
}];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum World {
    Single,
    Multi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    Setting(usize),
    Option(usize, usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Nothing,
    Header(usize),
    Module(usize, usize),
    Setting(usize, usize, usize),
    Option(usize, usize, usize, usize),
}

struct State {
    on: [[bool; MAX_MODULES]; MAX_CATEGORIES],
    on_since: [[Option<Instant>; MAX_MODULES]; MAX_CATEGORIES],
    expanded: [[bool; MAX_MODULES]; MAX_CATEGORIES],
    expanded_since: [[Option<Instant>; MAX_MODULES]; MAX_CATEGORIES],
    choices: [[[u8; MAX_SETTINGS]; MAX_MODULES]; MAX_CATEGORIES],
    numbers: [[[Option<f64>; MAX_SETTINGS]; MAX_MODULES]; MAX_CATEGORIES],
    collapsed: [bool; MAX_CATEGORIES],
    scroll: f32,
    world: u8,
    dropdown: Option<(usize, usize, usize)>,
    editing: Option<(usize, usize, usize)>,
    buffer: String,
    fresh: bool,
    last_frame: Option<Instant>,
    open_since: Option<Instant>,
}

impl State {
    const INITIAL: State = State {
        on: [[false; MAX_MODULES]; MAX_CATEGORIES],
        on_since: [[None; MAX_MODULES]; MAX_CATEGORIES],
        expanded: [[false; MAX_MODULES]; MAX_CATEGORIES],
        expanded_since: [[None; MAX_MODULES]; MAX_CATEGORIES],
        choices: [[[0; MAX_SETTINGS]; MAX_MODULES]; MAX_CATEGORIES],
        numbers: [[[None; MAX_SETTINGS]; MAX_MODULES]; MAX_CATEGORIES],
        collapsed: [false; MAX_CATEGORIES],
        scroll: 0.0,
        world: 0,
        dropdown: None,
        editing: None,
        buffer: String::new(),
        fresh: false,
        last_frame: None,
        open_since: None,
    };
}

struct Global(Mutex<State>);

impl Global {
    fn with_borrow<R>(&self, f: impl FnOnce(&State) -> R) -> R {
        f(&self.0.lock().unwrap_or_else(|e| e.into_inner()))
    }

    fn with_borrow_mut<R>(&self, f: impl FnOnce(&mut State) -> R) -> R {
        f(&mut self.0.lock().unwrap_or_else(|e| e.into_inner()))
    }
}

static STATE: Global = Global(Mutex::new(State::INITIAL));
static PENDING: AtomicU32 = AtomicU32::new(0);
static START: OnceLock<Instant> = OnceLock::new();

pub fn spaced(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    for (index, c) in text.chars().enumerate() {
        if index > 0 {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

pub fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

pub fn scale_alpha(argb: u32, factor: f32) -> u32 {
    let alpha = ((argb >> 24) as f32 * factor.clamp(0.0, 1.0)).round() as u32;
    (alpha << 24) | (argb & 0x00FF_FFFF)
}

pub fn mix_rgb(from: u32, to: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let mut out = 0u32;
    for shift in [24, 16, 8, 0] {
        let a = ((from >> shift) & 0xFF) as f32;
        let b = ((to >> shift) & 0xFF) as f32;
        out |= ((a + (b - a) * t).round() as u32) << shift;
    }
    out
}

pub fn time() -> f32 {
    START.get_or_init(Instant::now).elapsed().as_secs_f32()
}

fn amount(on: bool, since: Option<Instant>, duration: f32) -> f32 {
    let progress = since.map_or(1.0, |t| ease_out(t.elapsed().as_secs_f32() / duration));
    if on { progress } else { 1.0 - progress }
}

pub fn begin_frame() -> f32 {
    STATE.with_borrow_mut(|s| {
        let now = Instant::now();
        let stale = s
            .last_frame
            .is_none_or(|t| now.duration_since(t).as_secs_f32() > 0.3);
        if stale {
            s.open_since = Some(now);
            s.dropdown = None;
            s.editing = None;
            s.buffer.clear();
            s.fresh = false;
        }
        s.last_frame = Some(now);
        s.open_since.map_or(1.0, |t| {
            ease_out(now.duration_since(t).as_secs_f32() / OPEN_SECS)
        })
    })
}

pub fn panel_fade(category: usize) -> f32 {
    let elapsed = STATE.with_borrow(|s| s.open_since.map(|t| t.elapsed().as_secs_f32()));
    match elapsed {
        Some(e) => ease_out((e - category as f32 * 0.06) / OPEN_SECS),
        None => 1.0,
    }
}

pub fn panel_top(category: usize) -> f32 {
    TOP + (1.0 - panel_fade(category)) * 14.0
}

fn def(category: usize, module: usize, setting: usize) -> &'static Setting {
    &CATEGORIES[category].modules[module].settings[setting]
}

fn options_of(kind: &Kind, world: Option<World>) -> &'static [&'static str] {
    match kind {
        Kind::Choice { single, multi } => {
            if world == Some(World::Multi) {
                *multi
            } else {
                *single
            }
        }
        Kind::Number { .. } => &[],
    }
}

fn is_visible(setting: &Setting, world: Option<World>) -> bool {
    match &setting.kind {
        Kind::Choice { .. } => !options_of(&setting.kind, world).is_empty(),
        Kind::Number { worlds, .. } => match worlds {
            Worlds::Both => true,
            Worlds::Single => world != Some(World::Multi),
            Worlds::Multi => world == Some(World::Multi),
        },
    }
}

pub fn is_on(category: usize, module: usize) -> bool {
    STATE.with_borrow(|s| s.on[category][module])
}

pub fn on_amount(category: usize, module: usize) -> f32 {
    STATE.with_borrow(|s| {
        amount(
            s.on[category][module],
            s.on_since[category][module],
            TOGGLE_SECS,
        )
    })
}

pub fn is_expanded(category: usize, module: usize) -> bool {
    STATE.with_borrow(|s| s.expanded[category][module])
}

pub fn expand_amount(category: usize, module: usize) -> f32 {
    STATE.with_borrow(|s| {
        amount(
            s.expanded[category][module],
            s.expanded_since[category][module],
            EXPAND_SECS,
        )
    })
}

pub fn is_collapsed(category: usize) -> bool {
    STATE.with_borrow(|s| s.collapsed[category])
}

pub fn scroll() -> f32 {
    STATE.with_borrow(|s| s.scroll)
}

pub fn enabled_count() -> usize {
    STATE.with_borrow(|s| s.on.iter().flatten().filter(|on| **on).count())
}

pub fn is_enabled(category: &str, module: &str) -> bool {
    CATEGORIES.iter().enumerate().any(|(c, cat)| {
        cat.name == category
            && cat
                .modules
                .iter()
                .enumerate()
                .any(|(m, def)| def.name == module && is_on(c, m))
    })
}

pub fn world() -> Option<World> {
    match STATE.with_borrow(|s| s.world) {
        1 => Some(World::Single),
        2 => Some(World::Multi),
        _ => None,
    }
}

pub fn set_world(world: World) {
    STATE.with_borrow_mut(|s| {
        s.world = match world {
            World::Single => 1,
            World::Multi => 2,
        };
        s.dropdown = None;
    });
}

pub fn world_label(world: Option<World>) -> &'static str {
    match world {
        Some(World::Single) => "Single Player",
        Some(World::Multi) => "Multi Player",
        None => "Unknown",
    }
}

pub fn visible_settings(category: usize, module: usize) -> Vec<usize> {
    let world = world();
    CATEGORIES[category].modules[module]
        .settings
        .iter()
        .enumerate()
        .filter(|(_, setting)| is_visible(setting, world))
        .map(|(index, _)| index)
        .collect()
}

pub fn block_rows(category: usize, module: usize) -> Vec<Row> {
    let dropdown = STATE.with_borrow(|s| s.dropdown);
    let world = world();
    let mut rows = Vec::new();
    for setting in visible_settings(category, module) {
        rows.push(Row::Setting(setting));
        if dropdown == Some((category, module, setting)) {
            let options = options_of(&def(category, module, setting).kind, world);
            for index in 0..options.len() {
                rows.push(Row::Option(setting, index));
            }
        }
    }
    rows
}

pub fn selected_choice(category: usize, module: usize, setting: usize) -> usize {
    let world = world();
    let len = options_of(&def(category, module, setting).kind, world).len();
    let raw = STATE.with_borrow(|s| usize::from(s.choices[category][module][setting]));
    if len == 0 { 0 } else { raw.min(len - 1) }
}

pub fn option_text(category: usize, module: usize, setting: usize, index: usize) -> &'static str {
    let world = world();
    options_of(&def(category, module, setting).kind, world)
        .get(index)
        .copied()
        .unwrap_or("")
}

pub fn is_dropdown(category: usize, module: usize, setting: usize) -> bool {
    STATE.with_borrow(|s| s.dropdown == Some((category, module, setting)))
}

fn default_number(category: usize, module: usize, setting: usize) -> f64 {
    match def(category, module, setting).kind {
        Kind::Number { default, .. } => default,
        Kind::Choice { .. } => 0.0,
    }
}

fn number_value(category: usize, module: usize, setting: usize) -> f64 {
    STATE
        .with_borrow(|s| s.numbers[category][module][setting])
        .unwrap_or_else(|| default_number(category, module, setting))
}

fn format_number(value: f64) -> String {
    format!("{value}")
}

pub fn setting_display(category: usize, module: usize, setting: usize) -> String {
    match def(category, module, setting).kind {
        Kind::Choice { .. } => option_text(
            category,
            module,
            setting,
            selected_choice(category, module, setting),
        )
        .to_owned(),
        Kind::Number { .. } => format_number(number_value(category, module, setting)),
    }
}

fn find(category: &str, module: &str, setting: &str) -> Option<(usize, usize, usize)> {
    let (c, cat) = CATEGORIES
        .iter()
        .enumerate()
        .find(|(_, cat)| cat.name == category)?;
    let (m, def) = cat
        .modules
        .iter()
        .enumerate()
        .find(|(_, def)| def.name == module)?;
    let (s, _) = def
        .settings
        .iter()
        .enumerate()
        .find(|(_, def)| def.name == setting)?;
    Some((c, m, s))
}

pub fn setting(category: &str, module: &str, setting: &str) -> Option<&'static str> {
    let (c, m, s) = find(category, module, setting)?;
    if !matches!(def(c, m, s).kind, Kind::Choice { .. }) {
        return None;
    }
    let text = setting_display(c, m, s);
    let options = options_of(&def(c, m, s).kind, world());
    options.iter().copied().find(|option| *option == text)
}

pub fn number(category: &str, module: &str, setting: &str) -> Option<f64> {
    let (c, m, s) = find(category, module, setting)?;
    if !matches!(def(c, m, s).kind, Kind::Number { .. }) {
        return None;
    }
    Some(number_value(c, m, s))
}

pub fn editing() -> bool {
    STATE.with_borrow(|s| s.editing.is_some())
}

pub fn edit_target() -> Option<(usize, usize, usize)> {
    STATE.with_borrow(|s| s.editing)
}

pub fn edit_buffer() -> String {
    STATE.with_borrow(|s| s.buffer.clone())
}

pub fn criticals_hook() -> bool {
    is_enabled("Combat", "Criticals") && world() != Some(World::Multi)
}

pub fn criticals_effects() -> bool {
    is_enabled("Combat", "Criticals") && world() == Some(World::Single)
}

pub fn criticals_packets() -> bool {
    is_enabled("Combat", "Criticals") && world() == Some(World::Multi)
}

pub fn contains(rect: [f32; 4], x: f32, y: f32) -> bool {
    x >= rect[0] && x < rect[0] + rect[2] && y >= rect[1] && y < rect[1] + rect[3]
}

pub fn dialog_rect(view_w: f32, view_h: f32) -> [f32; 4] {
    [
        (view_w - DIALOG_W) / 2.0,
        (view_h - DIALOG_H) / 2.0,
        DIALOG_W,
        DIALOG_H,
    ]
}

pub fn choice_rect(view_w: f32, view_h: f32, index: usize) -> [f32; 4] {
    let [x, y, w, h] = dialog_rect(view_w, view_h);
    let gap = (w - 2.0 * CHOICE_W) / 3.0;
    [
        x + gap + index as f32 * (CHOICE_W + gap),
        y + h - CHOICE_H - 10.0,
        CHOICE_W,
        CHOICE_H,
    ]
}

pub fn world_button(view_w: f32) -> [f32; 4] {
    [view_w - LEFT - 124.0, 8.0, 124.0, 16.0]
}

pub fn choose_world(x: f32, y: f32, view_w: f32, view_h: f32) -> bool {
    for (index, world) in [World::Single, World::Multi].into_iter().enumerate() {
        if contains(choice_rect(view_w, view_h, index), x, y) {
            set_world(world);
            return true;
        }
    }
    false
}

pub fn toggle_world(x: f32, y: f32, view_w: f32) -> bool {
    if !contains(world_button(view_w), x, y) {
        return false;
    }
    let next = if world() == Some(World::Single) {
        World::Multi
    } else {
        World::Single
    };
    set_world(next);
    true
}

pub fn panel_x(category: usize) -> f32 {
    LEFT + category as f32 * (PANEL_W + GAP) - scroll()
}

pub fn box_x(panel_x: f32) -> f32 {
    panel_x + PANEL_W - 8.0 - BOX_W
}

pub fn block_height(category: usize, module: usize) -> f32 {
    let rows = block_rows(category, module).len() as f32;
    if rows == 0.0 {
        return ROW_H;
    }
    ROW_H + (rows * SETTING_H + SETTING_PAD) * expand_amount(category, module)
}

pub fn module_top(category: usize, module: usize) -> f32 {
    let before: f32 = (0..module).map(|m| block_height(category, m)).sum();
    panel_top(category) + HEAD_H + before
}

pub fn panel_height(category: usize) -> f32 {
    if is_collapsed(category) {
        return HEAD_H;
    }
    let blocks: f32 = (0..CATEGORIES[category].modules.len())
        .map(|m| block_height(category, m))
        .sum();
    HEAD_H + blocks + 2.0
}

pub fn hit(x: f32, y: f32) -> Hit {
    for category in 0..CATEGORIES.len() {
        let left = panel_x(category);
        if x < left || x >= left + PANEL_W {
            continue;
        }
        let top = panel_top(category);
        if y >= top && y < top + HEAD_H {
            return Hit::Header(category);
        }
        if is_collapsed(category) {
            continue;
        }
        for module in 0..CATEGORIES[category].modules.len() {
            let module_y = module_top(category, module);
            if y >= module_y && y < module_y + ROW_H {
                return Hit::Module(category, module);
            }
            if !is_expanded(category, module) || expand_amount(category, module) < 0.99 {
                continue;
            }
            for (index, row) in block_rows(category, module).into_iter().enumerate() {
                let row_y = module_y + ROW_H + index as f32 * SETTING_H;
                if y >= row_y && y < row_y + SETTING_H {
                    return match row {
                        Row::Setting(setting) => Hit::Setting(category, module, setting),
                        Row::Option(setting, option) => {
                            Hit::Option(category, module, setting, option)
                        }
                    };
                }
            }
        }
    }
    Hit::Nothing
}

fn commit_edit() {
    STATE.with_borrow_mut(|s| {
        if let Some((c, m, i)) = s.editing.take() {
            if let Ok(value) = s.buffer.trim().parse::<f64>() {
                if value.is_finite() {
                    s.numbers[c][m][i] = Some(value);
                }
            }
        }
        s.buffer.clear();
        s.fresh = false;
    });
}

fn cancel_edit() {
    STATE.with_borrow_mut(|s| {
        s.editing = None;
        s.buffer.clear();
        s.fresh = false;
    });
}

fn start_edit(category: usize, module: usize, setting: usize) {
    if edit_target() == Some((category, module, setting)) {
        return;
    }
    let text = format_number(number_value(category, module, setting));
    STATE.with_borrow_mut(|s| {
        s.editing = Some((category, module, setting));
        s.buffer = text;
        s.fresh = true;
        s.dropdown = None;
    });
}

fn reset_number(category: usize, module: usize, setting: usize) {
    if edit_target() == Some((category, module, setting)) {
        cancel_edit();
    }
    STATE.with_borrow_mut(|s| s.numbers[category][module][setting] = None);
}

fn toggle_dropdown(category: usize, module: usize, setting: usize) {
    STATE.with_borrow_mut(|s| {
        if s.dropdown == Some((category, module, setting)) {
            s.dropdown = None;
        } else {
            s.dropdown = Some((category, module, setting));
        }
    });
}

fn settle(target: Hit) {
    let (editing, dropdown) = STATE.with_borrow(|s| (s.editing, s.dropdown));
    if let Some((c, m, i)) = editing {
        if target != Hit::Setting(c, m, i) {
            commit_edit();
        }
    }
    if let Some((c, m, i)) = dropdown {
        let keep = match target {
            Hit::Setting(tc, tm, ti) => (tc, tm, ti) == (c, m, i),
            Hit::Option(tc, tm, ti, _) => (tc, tm, ti) == (c, m, i),
            _ => false,
        };
        if !keep {
            STATE.with_borrow_mut(|s| s.dropdown = None);
        }
    }
}

pub fn click(x: f32, y: f32, button: i32) -> bool {
    let target = hit(x, y);
    settle(target);
    match target {
        Hit::Header(category) => {
            if button == 0 {
                STATE.with_borrow_mut(|s| s.collapsed[category] = !s.collapsed[category]);
            }
            true
        }
        Hit::Module(category, module) => {
            let has_settings = !visible_settings(category, module).is_empty();
            if button == 0 {
                STATE.with_borrow_mut(|s| {
                    s.on[category][module] = !s.on[category][module];
                    s.on_since[category][module] = Some(Instant::now());
                });
            } else if button == 1 && has_settings {
                STATE.with_borrow_mut(|s| {
                    s.expanded[category][module] = !s.expanded[category][module];
                    s.expanded_since[category][module] = Some(Instant::now());
                    s.dropdown = None;
                });
            }
            true
        }
        Hit::Setting(category, module, setting) => {
            match def(category, module, setting).kind {
                Kind::Choice { .. } => {
                    if button == 0 {
                        toggle_dropdown(category, module, setting);
                    }
                }
                Kind::Number { .. } => {
                    if button == 0 {
                        start_edit(category, module, setting);
                    } else if button == 1 {
                        reset_number(category, module, setting);
                    }
                }
            }
            true
        }
        Hit::Option(category, module, setting, index) => {
            if button == 0 {
                STATE.with_borrow_mut(|s| {
                    s.choices[category][module][setting] = index as u8;
                    s.dropdown = None;
                });
            }
            true
        }
        Hit::Nothing => false,
    }
}

pub fn type_char(c: char) -> bool {
    STATE.with_borrow_mut(|s| {
        if s.editing.is_none() {
            return false;
        }
        if c.is_ascii_digit() || c == '.' || c == '-' {
            if s.fresh {
                s.buffer.clear();
                s.fresh = false;
            }
            if s.buffer.len() < MAX_INPUT {
                s.buffer.push(c);
            }
        }
        true
    })
}

pub fn key_pressed(key: &str) -> bool {
    if !editing() {
        return false;
    }
    match key {
        "key.keyboard.backspace" => {
            STATE.with_borrow_mut(|s| {
                if s.fresh {
                    s.buffer.clear();
                    s.fresh = false;
                } else {
                    s.buffer.pop();
                }
            });
            true
        }
        "key.keyboard.enter" | "key.keyboard.keypad.enter" => {
            commit_edit();
            true
        }
        "key.keyboard.escape" => {
            cancel_edit();
            true
        }
        _ => false,
    }
}

pub fn pan(delta: f32, view_width: f32) {
    let content = LEFT * 2.0 + CATEGORIES.len() as f32 * (PANEL_W + GAP) - GAP;
    let max = (content - view_width).max(0.0);
    STATE.with_borrow_mut(|s| s.scroll = (s.scroll - delta * 24.0).clamp(0.0, max));
}

pub fn ray_hits_box(
    eye: [f64; 3],
    yaw: f32,
    pitch: f32,
    bounds: [[f64; 3]; 2],
    reach: f64,
) -> bool {
    let yaw = f64::from(yaw).to_radians();
    let pitch = f64::from(pitch).to_radians();
    let dir = [
        -yaw.sin() * pitch.cos(),
        -pitch.sin(),
        yaw.cos() * pitch.cos(),
    ];
    let mut near = 0.0_f64;
    let mut far = reach;
    for axis in 0..3 {
        let (lo, hi) = (bounds[0][axis], bounds[1][axis]);
        if dir[axis].abs() < 1e-9 {
            if eye[axis] < lo || eye[axis] > hi {
                return false;
            }
            continue;
        }
        let a = (lo - eye[axis]) / dir[axis];
        let b = (hi - eye[axis]) / dir[axis];
        near = near.max(a.min(b));
        far = far.min(a.max(b));
        if near > far {
            return false;
        }
    }
    true
}

pub fn request_open() {
    PENDING.store(300, Ordering::Relaxed);
}

pub fn take_pending(ready: bool) -> bool {
    let left = PENDING.load(Ordering::Relaxed);
    if left == 0 {
        return false;
    }
    if ready {
        PENDING.store(0, Ordering::Relaxed);
        return true;
    }
    PENDING.store(left - 1, Ordering::Relaxed);
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    static LOCK: Mutex<()> = Mutex::new(());
    const MODULE_X: f32 = LEFT + 8.0;
    const MODULE_Y: f32 = TOP + HEAD_H + 2.0;

    fn row_y(index: usize) -> f32 {
        TOP + HEAD_H + ROW_H + 2.0 + index as f32 * SETTING_H
    }

    fn fresh() -> std::sync::MutexGuard<'static, ()> {
        let guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        *STATE.0.lock().unwrap_or_else(|e| e.into_inner()) = State::INITIAL;
        PENDING.store(0, Ordering::Relaxed);
        guard
    }

    fn open_settings() {
        click(MODULE_X, MODULE_Y, 1);
        STATE.with_borrow_mut(|s| s.expanded_since[0][0] = None);
    }

    #[test]
    fn easing_and_colors() {
        assert_eq!(ease_out(0.0), 0.0);
        assert_eq!(ease_out(1.0), 1.0);
        assert!(ease_out(0.5) > 0.5);
        assert_eq!(scale_alpha(0xFF112233, 0.5), 0x80112233);
        assert_eq!(mix_rgb(0xFF000000, 0xFFFFFFFF, 0.0), 0xFF000000);
        assert_eq!(mix_rgb(0xFF000000, 0xFFFFFFFF, 1.0), 0xFFFFFFFF);
        assert_eq!(amount(true, None, 1.0), 1.0);
        assert_eq!(amount(false, None, 1.0), 0.0);
        assert_eq!(spaced("ABC"), "A B C");
    }

    #[test]
    fn toggling_a_module_starts_an_animation() {
        let _guard = fresh();
        set_world(World::Single);
        assert_eq!(on_amount(0, 0), 0.0);
        assert!(click(MODULE_X, MODULE_Y, 0));
        assert!(is_on(0, 0));
        assert!(on_amount(0, 0) < 1.0);
        STATE.with_borrow_mut(|s| s.on_since[0][0] = None);
        assert_eq!(on_amount(0, 0), 1.0);
    }

    #[test]
    fn the_method_list_opens_below_the_box() {
        let _guard = fresh();
        set_world(World::Multi);
        open_settings();
        assert_eq!(block_rows(0, 0), vec![Row::Setting(0), Row::Setting(1)]);
        assert!(click(MODULE_X, row_y(0), 0));
        assert_eq!(
            block_rows(0, 0),
            vec![Row::Setting(0), Row::Option(0, 0), Row::Setting(1)]
        );
        assert_eq!(hit(MODULE_X, row_y(1)), Hit::Option(0, 0, 0, 0));
        assert!(click(MODULE_X, row_y(1), 0));
        assert_eq!(block_rows(0, 0), vec![Row::Setting(0), Row::Setting(1)]);
        assert_eq!(setting("Combat", "Criticals", "Method"), Some("Packet"));
    }

    #[test]
    fn clicking_elsewhere_closes_the_method_list() {
        let _guard = fresh();
        set_world(World::Multi);
        open_settings();
        click(MODULE_X, row_y(0), 0);
        assert!(is_dropdown(0, 0, 0));
        click(MODULE_X, MODULE_Y, 0);
        assert!(!is_dropdown(0, 0, 0));
    }

    #[test]
    fn offset_accepts_any_number() {
        let _guard = fresh();
        set_world(World::Multi);
        open_settings();
        assert_eq!(number("Combat", "Criticals", "Offset"), Some(0.0625));
        assert!(click(MODULE_X, row_y(1), 0));
        assert!(editing());
        for c in "-12345.5".chars() {
            assert!(type_char(c));
        }
        assert!(key_pressed("key.keyboard.enter"));
        assert_eq!(number("Combat", "Criticals", "Offset"), Some(-12345.5));
        click(MODULE_X, row_y(1), 0);
        for c in "1e9".chars() {
            type_char(c);
        }
        key_pressed("key.keyboard.enter");
        assert_eq!(number("Combat", "Criticals", "Offset"), Some(19.0));
        click(MODULE_X, row_y(1), 0);
        type_char('-');
        type_char('-');
        key_pressed("key.keyboard.enter");
        assert_eq!(number("Combat", "Criticals", "Offset"), Some(19.0));
        click(MODULE_X, row_y(1), 1);
        assert_eq!(number("Combat", "Criticals", "Offset"), Some(0.0625));
    }

    #[test]
    fn escape_cancels_and_backspace_edits() {
        let _guard = fresh();
        set_world(World::Multi);
        open_settings();
        click(MODULE_X, row_y(1), 0);
        type_char('7');
        type_char('5');
        assert!(key_pressed("key.keyboard.backspace"));
        assert_eq!(edit_buffer(), "7");
        assert!(key_pressed("key.keyboard.escape"));
        assert!(!editing());
        assert_eq!(number("Combat", "Criticals", "Offset"), Some(0.0625));
        assert!(!key_pressed("key.keyboard.escape"));
    }

    #[test]
    fn single_player_hides_the_offset() {
        let _guard = fresh();
        set_world(World::Single);
        open_settings();
        assert_eq!(visible_settings(0, 0), vec![0]);
        assert_eq!(setting("Combat", "Criticals", "Method"), Some("Hook"));
        assert_eq!(hit(MODULE_X, row_y(1)), Hit::Nothing);
        click(MODULE_X, MODULE_Y, 0);
        assert!(criticals_hook());
        assert!(criticals_effects());
        assert!(!criticals_packets());
    }

    #[test]
    fn the_world_choice_dialog_and_button() {
        let _guard = fresh();
        assert_eq!(world(), None);
        let multi = choice_rect(400.0, 240.0, 1);
        assert!(choose_world(multi[0] + 2.0, multi[1] + 2.0, 400.0, 240.0));
        assert_eq!(world(), Some(World::Multi));
        let button = world_button(400.0);
        assert!(toggle_world(button[0] + 2.0, button[1] + 2.0, 400.0));
        assert_eq!(world(), Some(World::Single));
    }

    #[test]
    fn a_header_click_collapses_the_panel() {
        let _guard = fresh();
        set_world(World::Single);
        assert_eq!(hit(MODULE_X, TOP + 2.0), Hit::Header(0));
        assert!(click(MODULE_X, TOP + 2.0, 0));
        assert!(is_collapsed(0));
        assert_eq!(hit(MODULE_X, MODULE_Y), Hit::Nothing);
    }

    #[test]
    fn the_view_ray_finds_a_box_in_reach() {
        let eye = [0.0, 1.6, 0.0];
        let target = [[-0.3, 0.0, 2.0], [0.3, 1.8, 2.6]];
        assert!(ray_hits_box(eye, 0.0, 0.0, target, 3.0));
        assert!(!ray_hits_box(eye, 0.0, 0.0, target, 1.5));
        assert!(!ray_hits_box(eye, 90.0, 0.0, target, 3.0));
    }

    #[test]
    fn a_requested_open_waits_until_ready() {
        let _guard = fresh();
        request_open();
        assert!(!take_pending(false));
        assert!(take_pending(true));
        assert!(!take_pending(true));
    }
}
