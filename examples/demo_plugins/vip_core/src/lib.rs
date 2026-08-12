use goldsrc::{log_info, plugin, Player};

pub struct VipCore;

#[plugin(name = "vip_core", version = "1.0.0", author = "Oleg")]
impl VipCore {
    #[on_load]
    fn init() {
        log_info!("[Vip Core] Initializing VIP system...");
        // Register the capability globally
        goldsrc::Auth::register_capability("vip.give_armor", "Allows giving armor to a player");
    }

    #[command(name = "vip_give_armor")]
    fn handle_give_armor(_cmd: String, args: String) {
        // We will pretend player 1 is the one executing this command
        let executer = Player::new(1);

        if !executer.has_capability("vip.give_armor") {
            executer.print_chat("You do not have access to this command.");
            return;
        }

        if let Ok(index) = args.parse::<i32>() {
            let mut player = Player::new(index);
            player.set_armorvalue(100.0);
            log_info!("[Vip Core] Gave armor to player {}", index);
        }
    }
}
