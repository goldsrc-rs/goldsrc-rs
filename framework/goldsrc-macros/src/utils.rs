//! Common utilities and string helpers for GoldSrc procedural macros.

use proc_macro::TokenStream;
use syn::ImplItemFn;

/// Escapes a string for embedding inside a TOML double-quoted literal.
pub fn toml_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// Error raised when a marker attribute is used outside a `#[plugin]` impl.
pub fn marker_outside_plugin(name: &str) -> TokenStream {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        format!("#[{name}] can only be used on methods inside a #[plugin(...)] impl block"),
    )
    .to_compile_error()
    .into()
}

/// Verifies that a handler method accepts an allowed number of arguments.
pub fn check_handler_args(method: &ImplItemFn, name: &str, allowed: &[usize]) -> syn::Result<()> {
    let count = method.sig.inputs.len();
    if !allowed.contains(&count) {
        let allowed_str: Vec<String> = allowed.iter().map(|n| n.to_string()).collect();
        return Err(syn::Error::new_spanned(
            &method.sig,
            format!(
                "handler for '{name}' must take {} arguments, but takes {count}",
                allowed_str.join(" or ")
            ),
        ));
    }
    Ok(())
}
