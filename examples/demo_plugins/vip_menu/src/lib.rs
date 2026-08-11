use goldsrc::{EntityId, World, command, event, log_info, plugin};

struct VipComponent {
    level: u8,
}

#[plugin(
    name = "vip_menu",
    version = "1.0.0",
    systems = ["MenuSystem"],
    dependencies = ["vip_core@^1.0.0"]
)]
pub struct VipMenu;

#[unsafe(no_mangle)]
pub extern "C" fn on_load() {
    let mut world = World::new();
    let player = EntityId(1);
    world.insert(player, VipComponent { level: 5 });

    if let Some(vip) = world.get::<VipComponent>(player) {
        log_info!(
            "[VIP Menu] Initialized ECS! Player 1 VIP Level: {}",
            vip.level
        );
    }
}

#[event]
pub fn handle_event(name: &str, data: &str) {
    log_info!("[VIP Menu] Received Event '{}': {}", name, data);
}

#[command(name = "vipmenu")]
pub fn handle_vipmenu(cmd: &str, args: &str) {
    log_info!("[VIP Menu] Command '{}' called with args: '{}'", cmd, args);
}
