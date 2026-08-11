use goldsrc::{EntityId, World, command, log_info, plugin};

struct VipComponent {
    level: u8,
}

#[plugin(name = "vip_menu", version = "1.0.0", systems = ["MenuSystem"])]
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

/// # Safety
/// Pointers must be valid WASM memory regions.
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn on_event(
    name_ptr: *const u8,
    name_len: usize,
    data_ptr: *const u8,
    data_len: usize,
) {
    let name_slice = unsafe { std::slice::from_raw_parts(name_ptr, name_len) };
    let data_slice = unsafe { std::slice::from_raw_parts(data_ptr, data_len) };

    if let (Ok(event_name), Ok(event_data)) = (
        std::str::from_utf8(name_slice),
        std::str::from_utf8(data_slice),
    ) {
        log_info!("[VIP Menu] Received Event '{}': {}", event_name, event_data);
    }
}

#[command(name = "vipmenu")]
pub fn handle_vipmenu(cmd: &str, args: &str) {
    log_info!("[VIP Menu] Command '{}' called with args: '{}'", cmd, args);
}
