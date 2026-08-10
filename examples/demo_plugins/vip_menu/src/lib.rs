use goldsrc::plugin;

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn server_print(ptr: *const u8, len: usize);
}

fn log(msg: &str) {
    unsafe {
        server_print(msg.as_ptr(), msg.len());
    }
}

#[plugin(name = "vip_menu", version = "1.0.0", systems = ["MenuSystem"])]
pub struct VipMenu;

#[unsafe(no_mangle)]
pub extern "C" fn on_load() {
    log("[VIP Menu] Initialized VIP Menu Sub-System!\n");
}

#[unsafe(no_mangle)]
pub extern "C" fn on_event(
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
        let msg = format!(
            "[VIP Menu] Received Event '{}': {}\n",
            event_name, event_data
        );
        log(&msg);
    }
}
