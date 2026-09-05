use goldsrc::prelude::*;

pub struct VipCore;

#[plugin(
    name = "vip_core",
    version = "0.10.0",
    author = "GoldSrc.rs Team",
    description = "Core VIP authorization, capability registration, and player health/armor services",
    url = "https://github.com/goldsrc-rs/goldsrc-rs"
)]
impl VipCore {
    #[on_load]
    fn init() {
        log_info!("[VIP Core] Initializing VIP capabilities manager (v0.10.0)...");
        Auth::register_capability("vip.access", "Grants access to VIP features");
        Auth::register_capability("vip.give_armor", "Allows giving armor to a player");
        Auth::register_capability("vip.heal", "Allows healing a player to full HP");
    }

    /// Adds VIP status to player index (e.g. `vip_add 1`).
    #[command(
        name = "vip_add",
        description = "Grants full VIP capabilities to a target player",
        usage = "vip_add <player_index>"
    )]
    fn handle_vip_add(player: Player) {
        player.grant_capability("vip.access");
        player.grant_capability("vip.give_armor");
        player.grant_capability("vip.heal");

        engine::emit_sound(
            player.index(),
            0,
            "items/suitchargeno1.wav",
            1.0,
            0.8,
            0,
            100,
        );

        let name = player
            .name()
            .unwrap_or_else(|| format!("Player #{}", player.index()));
        log_info!(
            "[VIP Core] Granted VIP status to '{}' (idx={})",
            name,
            player.index()
        );
    }

    /// Checks if a player has VIP status (e.g. `vip_check 1`).
    #[command(
        name = "vip_check",
        description = "Checks VIP capability status for a target player",
        usage = "vip_check <player_index>"
    )]
    fn handle_vip_check(player: Player) {
        let is_vip = player.has_capability("vip.access");
        let name = player
            .name()
            .unwrap_or_else(|| format!("Player #{}", player.index()));

        log_info!(
            "[VIP Core] Player '{}' (idx={}): VIP status = {}",
            name,
            player.index(),
            if is_vip { "ACTIVE" } else { "NONE" }
        );
    }

    /// Heals living player to 100 HP (e.g. `vip_heal 1`).
    #[command(
        name = "vip_heal",
        capability = "vip.heal",
        description = "Restores target living player health to 100 HP",
        usage = "vip_heal <player_index>"
    )]
    fn handle_vip_heal(mut player: Alive<Player>) {
        player.set_health(100.0);
        log_info!("[VIP Core] Healed player #{} to 100 HP", player.index());
    }

    /// Gives 100 armor to living player (e.g. `vip_armor 1`).
    #[command(
        name = "vip_armor",
        capability = "vip.give_armor",
        description = "Restores target living player armor to 100 AP",
        usage = "vip_armor <player_index>"
    )]
    fn handle_give_armor(mut player: Alive<Player>) {
        player.give_item("item_assaultsuit");
        player.set_armorvalue(100.0);
        log_info!("[VIP Core] Given 100 armor to player #{}", player.index());
    }

    /// Passive ECS system running during player post-think to regenerate health for VIPs below 100 HP.
    #[system(stage = "post_think", phase = "modify")]
    fn vip_passive_regen(player: &mut Player) {
        if player.is_alive() && player.has_capability("vip.access") {
            let hp = player.health();
            if hp > 0.0 && hp < 100.0 {
                player.set_health((hp + 0.1).min(100.0));
            }
        }
    }
}
