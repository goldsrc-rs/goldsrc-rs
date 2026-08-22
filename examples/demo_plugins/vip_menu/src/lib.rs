use goldsrc::prelude::*;

pub struct VipMenu;

#[plugin(
    name = "vip_menu",
    version = "0.10.0",
    author = "GoldSrc.rs Team",
    description = "VIP equipment menu and daily supply package deployment",
    url = "https://github.com/goldsrc-rs/goldsrc-rs",
    dependencies = ["vip_core@>=0.10.0"]
)]
impl VipMenu {
    #[on_load]
    fn init() {
        log_info!("[VIP Menu] VIP Menu Plugin loaded successfully.");
    }

    /// Claims VIP daily kit (HP + Armor + Sound) (e.g. `vipmenu 1` or chat `/vip`).
    #[command(
        name = "vipmenu",
        aliases = ["vip", "/vip", "!vip"],
        capability = "vip.access",
        description = "Opens VIP kit and equips 120 HP + 100 AP",
        usage = "vipmenu <player_index>"
    )]
    fn handle_menu(mut player: Alive<Player>) {
        // Apply VIP Perks: 120 HP, 100 Armor
        player.set_health(120.0);
        player.set_armorvalue(100.0);

        #[cfg(target_arch = "wasm32")]
        {
            engine::emit_sound(player.index(), 0, "events/tutor_msg.wav", 1.0, 0.8, 0, 100);
        }

        let name = player
            .name()
            .unwrap_or_else(|| format!("Player #{}", player.index()));
        log_info!(
            "[VIP Menu] VIP Kit applied to '{}' (idx={}): HP=120, Armor=100, Sound emitted!",
            name,
            player.index()
        );
    }
}
