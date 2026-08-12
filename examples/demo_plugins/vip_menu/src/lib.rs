use goldsrc::{log_info, plugin, Player};

pub struct VipMenu;

#[plugin(
    name = "vip_menu",
    version = "1.0.0",
    author = "Oleg",
    dependencies = ["vip_core@>=1.0.0"]
)]
impl VipMenu {
    #[on_load]
    fn init() {
        log_info!("[Vip Menu] VIP Menu Plugin loaded successfully.");
    }

    #[command(name = "vip_menu")]
    fn handle_menu(_cmd: String, _args: String) {
        log_info!("[Vip Menu] Opening VIP Menu... (Mock)");
        // Logic to show menu to player 1
        let _player = Player::new(1);
        // player.print_chat("Welcome to VIP Menu!");
    }
}
