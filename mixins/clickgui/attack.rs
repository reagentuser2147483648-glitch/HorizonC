fn clickgui_attack_packets(game: &mut crate::game::Game<'_>, button: i32, canceled: bool) {
    use crate::game::clickgui as gui;
    use horizon_client::particles::{ParticleOptions, ParticleType};
    use horizon_protocol::packets::play::{MovePlayerPos, Serverbound};
    if button != 0 || canceled || !game.native_on(gui::ID) {
        return;
    }
    if !gui::criticals_effects() && !gui::criticals_packets() {
        return;
    }
    let Some(state) = game.native_client_state() else {
        return;
    };
    let Some(eye) = state.eye_position else {
        return;
    };
    let Some(target) = game.native_client_entities().into_iter().find(|entity| {
        entity.alive
            && gui::ray_hits_box(eye, state.yaw, state.pitch, entity.bounds, gui::ATTACK_REACH)
    }) else {
        return;
    };
    if gui::criticals_effects() {
        let [low, high] = target.bounds;
        let center = [
            (low[0] + high[0]) / 2.0,
            (low[1] + high[1]) / 2.0,
            (low[2] + high[2]) / 2.0,
        ];
        for i in 0..12 {
            let angle = f64::from(i) * 0.5236;
            game.particles.add(
                &ParticleOptions::simple(ParticleType::Crit),
                center,
                [
                    angle.cos() * 0.25,
                    0.15 + f64::from(i % 3) * 0.08,
                    angle.sin() * 0.25,
                ],
            );
        }
        game.native_play_local_sound("minecraft:entity.player.attack.crit", center, 1.0, 1.0, 0);
    }
    if !gui::criticals_packets() {
        return;
    }
    let step = gui::number("Combat", "Criticals", "Offset").unwrap_or(0.0625);
    let Some(player) = game.native_player() else {
        return;
    };
    for offset in [step, 0.0] {
        let _ = game.native_send_packet(Serverbound::MovePlayerPos(MovePlayerPos {
            x: player.x,
            y: player.y + offset,
            z: player.z,
            flags: 0,
        }));
    }
}
