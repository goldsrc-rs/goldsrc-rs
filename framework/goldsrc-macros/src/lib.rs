#![allow(clippy::collapsible_if, clippy::single_match)]

extern crate proc_macro;

mod defs;
mod plugin;
mod utils;

use crate::plugin::attr::parse_plugin_and_helpers;
use crate::plugin::expand_plugin;
use crate::utils::marker_outside_plugin;
use proc_macro::TokenStream;
use syn::{ItemImpl, parse_macro_input};

/// Attribute macro that marks a struct implementation as a GoldSrc WASM plugin.
///
/// Generates the WIT Component Model export bindings, event dispatcher, command registry,
/// and metadata header for the WASM runtime.
#[proc_macro_attribute]
pub fn plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_impl = parse_macro_input!(item as ItemImpl);
    let plugin_attr = match parse_plugin_and_helpers(attr.into(), &mut input_impl) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };

    expand_plugin(plugin_attr, input_impl)
}

/// Helper attribute for declaring plugin bundle membership (`#[bundle("bundle_name")]`).
#[proc_macro_attribute]
pub fn bundle(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("bundle")
}

/// Helper attribute for declaring plugin / command requirements (`#[require("plugin@^1.0")]`).
#[proc_macro_attribute]
pub fn require(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("require")
}

/// Helper attribute for declaring WASM sandbox permissions (`#[permissions("fs:read", "chat:broadcast")]`).
#[proc_macro_attribute]
pub fn permissions(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("permissions")
}

/// Helper attribute for declaring WASM sandbox single permission (`#[permission("fs:read")]`).
#[proc_macro_attribute]
pub fn permission(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("permission")
}

/// Helper attribute for declaring plugin lifecycle constraints (`#[lifecycle(load = anytime, unload = never)]`).
#[proc_macro_attribute]
pub fn lifecycle(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("lifecycle")
}

/// Marker attribute for the plugin's `on_load` lifecycle hook.
#[proc_macro_attribute]
pub fn on_load(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("on_load")
}

/// Marker attribute for the plugin's `on_unload` lifecycle hook.
#[proc_macro_attribute]
pub fn on_unload(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("on_unload")
}

/// Marker attribute for the plugin's per-frame `on_frame` hook.
#[proc_macro_attribute]
pub fn on_frame(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("on_frame")
}

/// Marker attribute for event handlers (`#[event("event_name")]`).
#[proc_macro_attribute]
pub fn event(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("event")
}

/// Marker attribute for console / client command handlers (`#[command("cmd_name")]`).
#[proc_macro_attribute]
pub fn command(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("command")
}

/// Marker attribute for menu action handlers (`#[menu_action(id = 1)]` or `#[menu_action(action = "buy_m4")]`).
#[proc_macro_attribute]
pub fn menu_action(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("menu_action")
}

/// Marker attribute for ECS system handlers (`#[system(stage = "frame", order = 10)]`).
#[proc_macro_attribute]
pub fn system(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("system")
}

/// Marker attribute for contextual placeholder handlers (`#[placeholder(name = "rank", usage = "{rank}")]`).
#[proc_macro_attribute]
pub fn placeholder(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    marker_outside_plugin("placeholder")
}

#[cfg(test)]
mod tests {
    use crate::defs::PluginAttr;
    use crate::plugin::attr::parse_plugin_and_helpers;
    use crate::utils::toml_escape;
    use syn::ItemImpl;

    fn parse_attr(s: &str, impl_code: &str) -> syn::Result<PluginAttr> {
        let ts: proc_macro2::TokenStream = s.parse().unwrap();
        let mut input_impl: ItemImpl = syn::parse_str(impl_code).unwrap();
        parse_plugin_and_helpers(ts, &mut input_impl)
    }

    #[test]
    fn parses_all_attrs() {
        let a = parse_attr(
            r#"name = "x", version = "2.0", author = "A", description = "Test Desc", license = "MIT", url = "https://github.com", require = ["plugin:b@>=1", "plugin:c@1.0"], permissions = ["fs:read", "chat:broadcast"]"#,
            "impl MyPlugin {}",
        )
        .unwrap();
        assert_eq!(a.name, "x");
        assert_eq!(a.version, "2.0");
        assert_eq!(a.author, "A");
        assert_eq!(a.description, "Test Desc");
        assert_eq!(a.license, "MIT");
        assert_eq!(a.url, "https://github.com");
        assert_eq!(a.require, vec!["plugin:b@>=1", "plugin:c@1.0"]);
        assert_eq!(a.permissions, vec!["fs:read", "chat:broadcast"]);
    }

    #[test]
    fn parses_stacked_helper_attributes() {
        let a = parse_attr(
            "",
            r#"
            #[bundle("vip_system")]
            #[require("vip_core")]
            #[require("cstrike@^1.0")]
            #[permissions("fs:read('configs/*.toml')", "storage:wal")]
            #[lifecycle(load = anytime, unload = never)]
            impl MyPlugin {}
            "#,
        )
        .unwrap();

        assert_eq!(a.bundle, Some("vip_system".to_string()));
        assert_eq!(a.require, vec!["vip_core", "cstrike@^1.0"]);
        assert_eq!(
            a.permissions,
            vec!["fs:read('configs/*.toml')", "storage:wal"]
        );
        assert_eq!(a.load_time, "anytime");
        assert_eq!(a.unload_time, "never");
    }

    #[test]
    fn rejects_unknown_attr() {
        assert!(parse_attr("banana = 1", "impl MyPlugin {}").is_err());
    }

    #[test]
    fn rejects_non_string_value() {
        assert!(parse_attr("name = 42", "impl MyPlugin {}").is_err());
    }

    #[test]
    fn toml_escape_quotes_and_slashes() {
        assert_eq!(toml_escape("a\"b\\c"), "a\\\"b\\\\c");
        assert_eq!(toml_escape("line\nbreak"), "line\\nbreak");
    }
}
