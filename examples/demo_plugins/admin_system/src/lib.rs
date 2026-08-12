use goldsrc::{log_info, plugin, Player};

pub struct AdminSystem;

#[plugin(name = "admin_system", version = "1.0.0", author = "Oleg")]
impl AdminSystem {
    #[on_load]
    fn init() {
        log_info!("[Admin System] Initializing admin capabilities manager...");
        goldsrc::Auth::register_capability(
            "admin.grant",
            "Allows granting capabilities to players",
        );
    }

    #[command(name = "admin_grant")]
    fn handle_grant(_cmd: String, args: String) {
        // Pretend player 1 is executing
        let executer = Player::new(1);

        let mut parts = args.split_whitespace();
        let target_idx_str = parts.next();
        let cap_name = parts.next();

        if let (Some(target_idx_str), Some(cap_name)) = (target_idx_str, cap_name) {
            if let Ok(target_idx) = target_idx_str.parse::<i32>() {
                // Grant capability
                let target = Player::new(target_idx);
                if target.grant_capability(cap_name) {
                    log_info!(
                        "[Admin System] Granted {} to player {}",
                        cap_name,
                        target_idx
                    );
                    executer.print_chat(&format!("Granted {} to player {}", cap_name, target_idx));
                } else {
                    executer.print_chat(&format!("Failed to grant {}, does it exist?", cap_name));
                }
            }
        } else {
            executer.print_chat("Usage: admin_grant <player_index> <capability_name>");
        }
    }
}
