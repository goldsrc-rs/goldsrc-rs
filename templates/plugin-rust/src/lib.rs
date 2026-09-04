use goldsrc::prelude::*;

pub struct MyPlugin;

#[plugin(
    name = "{{project-name}}",
    version = "0.1.0",
    author = "{{authors}}",
    description = "GoldSrc plugin written in Rust",
    url = "https://github.com/goldsrc-rs/goldsrc-rs"
)]
impl MyPlugin {
    #[on_load]
    fn init() {
        log_info!("[{{project-name}}] Plugin loaded successfully.");
    }

    #[on_unload]
    fn shutdown() {
        log_info!("[{{project-name}}] Plugin unloaded.");
    }

    /// Example console / chat command: `my_cmd` or `/my_cmd`.
    #[command(
        name = "my_cmd",
        aliases = ["/my_cmd", "!my_cmd"],
        description = "Example command greeting the player",
        usage = "my_cmd"
    )]
    fn handle_cmd(player: Player) {
        let name = player.name().unwrap_or_else(|| "Player".to_string());
        player.print_center(&format!("Hello, {name}!"));
        player.print_chat(&format!("^4[{{project-name}}]^1 Welcome, ^3{name}!"));
    }
}
