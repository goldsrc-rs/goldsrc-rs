#![allow(clippy::collapsible_if, clippy::single_match)]

extern crate proc_macro;

mod command_def;
mod plugin_attr;
mod plugin_impl;
mod system_def;
mod utils;

use crate::plugin_attr::parse_plugin_attr;
use crate::plugin_impl::expand_plugin;
use crate::utils::marker_outside_plugin;
use proc_macro::TokenStream;
use syn::{ItemImpl, parse_macro_input};

/// Attribute macro that marks a struct implementation as a GoldSrc WASM plugin.
///
/// Generates the WIT Component Model export bindings, event dispatcher, command registry,
/// and metadata header for the WASM runtime.
#[proc_macro_attribute]
pub fn plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = match parse_plugin_attr(attr.into()) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };

    let input_impl = parse_macro_input!(item as ItemImpl);
    expand_plugin(attr, input_impl)
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
    use super::*;
    use crate::plugin_attr::PluginAttr;
    use crate::utils::toml_escape;

    fn parse(s: &str) -> syn::Result<PluginAttr> {
        let ts: proc_macro2::TokenStream = s.parse().unwrap();
        parse_plugin_attr(ts)
    }

    #[test]
    fn parses_all_attrs() {
        let a = parse(
            r#"name = "x", version = "2.0", author = "A", description = "Test Desc", license = "MIT", url = "https://github.com", require = ["plugin:b@>=1", "plugin:c@1.0"]"#,
        )
        .unwrap();
        assert_eq!(a.name, "x");
        assert_eq!(a.version, "2.0");
        assert_eq!(a.author, "A");
        assert_eq!(a.description, "Test Desc");
        assert_eq!(a.license, "MIT");
        assert_eq!(a.url, "https://github.com");
        assert_eq!(a.require, vec!["plugin:b@>=1", "plugin:c@1.0"]);
    }

    #[test]
    fn rejects_unknown_attr() {
        assert!(parse("banana = 1").is_err());
    }

    #[test]
    fn rejects_non_string_value() {
        assert!(parse("name = 42").is_err());
    }

    #[test]
    fn toml_escape_quotes_and_slashes() {
        assert_eq!(toml_escape("a\"b\\c"), "a\\\"b\\\\c");
        assert_eq!(toml_escape("line\nbreak"), "line\\nbreak");
    }
}
