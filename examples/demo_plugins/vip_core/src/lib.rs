use goldsrc::{log_info, plugin};

#[plugin(name = "vip_core", version = "1.0.0", systems = ["CoreSystem"])]
pub struct VipCore;

#[unsafe(no_mangle)]
pub extern "C" fn on_load() {
    log_info!("[VIP Core] Successfully initialized VIP Core System!");
}

#[unsafe(no_mangle)]
pub extern "C" fn on_frame() {
    // Frame tick
}
