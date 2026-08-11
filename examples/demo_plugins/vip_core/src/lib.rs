use goldsrc::{log_info, on_load, plugin};

#[plugin(
    name = "vip_core",
    version = "1.0.0",
    author = "Oleg",
    systems = ["CoreSystem"]
)]
pub struct VipCore;

#[on_load]
fn init() {
    log_info!("[VIP Core] Successfully initialized VIP Core System!");
}
