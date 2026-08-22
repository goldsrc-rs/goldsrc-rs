use goldsrc::prelude::*;

pub struct AdminSystem;

#[plugin(
    name = "admin_system",
    version = "0.10.0",
    author = "GoldSrc.rs Team",
    description = "Administration utilities and capability-based player management"
)]
impl AdminSystem {
    #[on_load]
    fn init() {
        log_info!("[Admin System] Initializing admin capabilities manager (v0.10.0)...");
        Auth::register_capability("admin.grant", "Allows granting capabilities to players");
        Auth::register_capability("admin.slay", "Allows slaying players");
        Auth::register_capability("admin.teleport", "Allows teleporting players");
        Auth::register_capability("admin.cvar", "Allows changing server cvars");
    }

    /// Grants a capability to a player (e.g. `admin_grant 1 admin.slay`).
    #[command(name = "admin_grant")]
    fn handle_grant(_cmd: String, args: String) {
        let mut parts = args.split_whitespace();
        let target_idx_str = parts.next();
        let cap_name = parts.next();

        if let (Some(target_idx_str), Some(cap_name)) = (target_idx_str, cap_name) {
            if let Ok(target_idx) = target_idx_str.parse::<i32>() {
                let target = Player::new(target_idx);
                if target.grant_capability(cap_name) {
                    log_info!(
                        "[Admin System] Granted '{}' to player #{}",
                        cap_name,
                        target_idx
                    );
                } else {
                    log_warn!(
                        "[Admin System] Capability '{}' is not registered!",
                        cap_name
                    );
                }
            }
        } else {
            log_info!("[Admin System] Usage: admin_grant <player_index> <capability_name>");
        }
    }

    /// Slays a player (sets HP to 0) (e.g. `admin_slay 1`).
    #[command(name = "admin_slay")]
    fn handle_slay(_cmd: String, args: String) {
        let idx = args.trim().parse::<i32>().unwrap_or(1);
        let mut player = Player::new(idx);
        if !player.is_valid() {
            log_warn!("[Admin System] Player #{} is not connected/valid!", idx);
            return;
        }

        player.set_health(0.0);
        let name = player.name().unwrap_or_else(|| format!("Player #{}", idx));
        log_info!("[Admin System] Slayed player '{}' (#{})", name, idx);
    }

    /// Teleports a player to target coordinates (e.g. `admin_teleport 1 0 0 100`).
    #[command(name = "admin_teleport")]
    fn handle_teleport(_cmd: String, args: String) {
        let mut parts = args.split_whitespace();
        let idx = parts
            .next()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1);
        let x = parts
            .next()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);
        let y = parts
            .next()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);
        let z = parts
            .next()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(50.0);

        let mut player = Player::new(idx);
        if !player.is_valid() {
            log_warn!("[Admin System] Player #{} is not connected/valid!", idx);
            return;
        }

        player.set_origin(Vector3 { x, y, z });
        player.set_velocity(Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        log_info!(
            "[Admin System] Teleported player #{} to ({:.1}, {:.1}, {:.1})",
            idx,
            x,
            y,
            z
        );
    }

    /// Changes server gravity (e.g. `admin_gravity 400`).
    #[command(name = "admin_gravity")]
    fn handle_gravity(_cmd: String, args: String) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(g) = args.trim().parse::<f32>() {
                engine::cvar_set_float("sv_gravity", g);
                log_info!("[Admin System] Set server sv_gravity to {:.0}", g);
            } else {
                let current = engine::cvar_get_float("sv_gravity");
                log_info!(
                    "[Admin System] Current sv_gravity: {:.0} (Usage: admin_gravity <val>)",
                    current
                );
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            log_info!("[Admin System] admin_gravity executed with: {}", args);
        }
    }
}
