use goldsrc::prelude::*;

pub struct VipCore;

#[plugin(name = "vip_core", version = "0.10.0", author = "GoldSrc.rs Team")]
impl VipCore {
    #[on_load]
    fn init() {
        log_info!("[VIP Core] Initializing VIP capabilities manager (v0.10.0)...");
        Auth::register_capability("vip.access", "Grants access to VIP features");
        Auth::register_capability("vip.give_armor", "Allows giving armor to a player");
        Auth::register_capability("vip.heal", "Allows healing a player to full HP");
    }

    /// Adds VIP status to player index (e.g. `vip_add 1`).
    #[command(name = "vip_add")]
    fn handle_vip_add(_cmd: String, args: String) {
        let idx = args.trim().parse::<i32>().unwrap_or(1);
        let player = Player::new(idx);
        if !player.is_valid() {
            log_warn!("[VIP Core] Player #{} is not connected/valid!", idx);
            return;
        }

        player.grant_capability("vip.access");
        player.grant_capability("vip.give_armor");
        player.grant_capability("vip.heal");

        #[cfg(target_arch = "wasm32")]
        {
            engine::emit_sound(idx, 0, "items/suitchargeno1.wav", 1.0, 0.8, 0, 100);
        }

        let name = player.name().unwrap_or_else(|| format!("Player #{}", idx));
        log_info!("[VIP Core] Granted VIP status to '{}' (idx={})", name, idx);
    }

    /// Checks if a player has VIP status (e.g. `vip_check 1`).
    #[command(name = "vip_check")]
    fn handle_vip_check(_cmd: String, args: String) {
        let idx = args.trim().parse::<i32>().unwrap_or(1);
        let player = Player::new(idx);
        let is_vip = player.has_capability("vip.access");
        let name = player.name().unwrap_or_else(|| format!("Player #{}", idx));

        log_info!(
            "[VIP Core] Player '{}' (idx={}): VIP status = {}",
            name,
            idx,
            if is_vip { "ACTIVE" } else { "NONE" }
        );
    }

    /// Heals player to 100 HP (e.g. `vip_heal 1`).
    #[command(name = "vip_heal")]
    fn handle_vip_heal(_cmd: String, args: String) {
        let idx = args.trim().parse::<i32>().unwrap_or(1);
        let mut player = Player::new(idx);
        if !player.has_capability("vip.heal") {
            log_warn!("[VIP Core] Player #{} lacks 'vip.heal' capability!", idx);
            return;
        }

        player.set_health(100.0);
        log_info!("[VIP Core] Healed player #{} to 100 HP", idx);
    }

    /// Gives 100 armor to player (e.g. `vip_armor 1`).
    #[command(name = "vip_armor")]
    fn handle_give_armor(_cmd: String, args: String) {
        let idx = args.trim().parse::<i32>().unwrap_or(1);
        let mut player = Player::new(idx);
        if !player.has_capability("vip.give_armor") {
            log_warn!(
                "[VIP Core] Player #{} lacks 'vip.give_armor' capability!",
                idx
            );
            return;
        }

        player.set_armorvalue(100.0);
        log_info!("[VIP Core] Set armor to 100 AP for player #{}", idx);
    }
}
