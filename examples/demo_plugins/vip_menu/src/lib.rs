use goldsrc::prelude::*;

pub struct VipMenu;

#[plugin(
    name = "vip_menu",
    version = "0.10.0",
    author = "GoldSrc.rs Team",
    description = "VIP equipment menu and daily supply package deployment",
    dependencies = ["vip_core@>=0.10.0"]
)]
impl VipMenu {
    #[on_load]
    fn init() {
        log_info!("[VIP Menu] VIP Menu Plugin loaded successfully.");
    }

    /// Claims VIP daily kit (HP + Armor + Sound) (e.g. `vipmenu 1` or chat `/vip`).
    #[command(name = "vipmenu")]
    fn handle_menu(_cmd: String, args: String) {
        Self::apply_vip_kit(args);
    }

    /// Alias for `vipmenu` (e.g. `/vip`).
    #[command(name = "vip")]
    fn handle_vip_alias(_cmd: String, args: String) {
        Self::apply_vip_kit(args);
    }

    fn apply_vip_kit(args: String) {
        let idx = args.trim().parse::<i32>().unwrap_or(1);
        let mut player = Player::new(idx);
        if !player.is_valid() {
            log_warn!("[VIP Menu] Player #{} is not connected/valid!", idx);
            return;
        }

        if !player.has_capability("vip.access") {
            log_warn!(
                "[VIP Menu] Access denied for Player #{}: missing 'vip.access' capability. Use `vip_add {}` first!",
                idx,
                idx
            );
            return;
        }

        // Apply VIP Perks: 120 HP, 100 Armor
        player.set_health(120.0);
        player.set_armorvalue(100.0);

        #[cfg(target_arch = "wasm32")]
        {
            engine::emit_sound(idx, 0, "events/tutor_msg.wav", 1.0, 0.8, 0, 100);
        }

        let name = player.name().unwrap_or_else(|| format!("Player #{}", idx));
        log_info!(
            "[VIP Menu] VIP Kit applied to '{}' (idx={}): HP=120, Armor=100, Sound emitted!",
            name,
            idx
        );
    }
}
