//! Procedural macro implementation for `#[derive(ConfigModel)]`.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields};

/// Expands the `#[derive(ConfigModel)]` macro.
pub fn expand_derive_config_model(input: DeriveInput) -> Result<TokenStream, Error> {
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let Data::Struct(data_struct) = &input.data else {
        return Err(Error::new_spanned(
            &input,
            "ConfigModel can only be derived for structs with named fields",
        ));
    };

    let Fields::Named(fields_named) = &data_struct.fields else {
        return Err(Error::new_spanned(
            &input,
            "ConfigModel can only be derived for structs with named fields",
        ));
    };

    struct FieldSpec {
        ident: syn::Ident,
        cvar_name: String,
        toml_key: String,
        description: String,
        flags_expr: TokenStream,
    }

    let mut parsed_fields = Vec::new();

    for field in &fields_named.named {
        let ident = field.ident.clone().unwrap();
        let mut cvar_name = ident.to_string();
        let mut toml_key = ident.to_string();
        let mut description = String::new();
        let mut flags_expr = quote! { ::goldsrc_api::cvar::CvarFlags::ARCHIVE };

        for attr in &field.attrs {
            if attr.path().is_ident("cvar") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("name") {
                        let value: syn::LitStr = meta.value()?.parse()?;
                        cvar_name = value.value();
                    } else if meta.path.is_ident("toml_key") {
                        let value: syn::LitStr = meta.value()?.parse()?;
                        toml_key = value.value();
                    } else if meta.path.is_ident("description") {
                        let value: syn::LitStr = meta.value()?.parse()?;
                        description = value.value();
                    } else if meta.path.is_ident("flags") {
                        let value: syn::LitStr = meta.value()?.parse()?;
                        let flags_str = value.value();
                        let mut flag_tokens = Vec::new();
                        for part in flags_str.split('|') {
                            let part = part.trim().to_uppercase();
                            match part.as_str() {
                                "ARCHIVE" => flag_tokens
                                    .push(quote! { ::goldsrc_api::cvar::CvarFlags::ARCHIVE }),
                                "SERVER" | "NOTIFY" => flag_tokens
                                    .push(quote! { ::goldsrc_api::cvar::CvarFlags::SERVER }),
                                "USERINFO" => flag_tokens
                                    .push(quote! { ::goldsrc_api::cvar::CvarFlags::USERINFO }),
                                "PROTECTED" => flag_tokens
                                    .push(quote! { ::goldsrc_api::cvar::CvarFlags::PROTECTED }),
                                "SP_ONLY" | "READ_ONLY" => flag_tokens
                                    .push(quote! { ::goldsrc_api::cvar::CvarFlags::SP_ONLY }),
                                "PRINTABLE_ONLY" => flag_tokens.push(
                                    quote! { ::goldsrc_api::cvar::CvarFlags::PRINTABLE_ONLY },
                                ),
                                "UNLOGGED" => flag_tokens
                                    .push(quote! { ::goldsrc_api::cvar::CvarFlags::UNLOGGED }),
                                "NO_EXTRA_WHITESPACE" => flag_tokens.push(
                                    quote! { ::goldsrc_api::cvar::CvarFlags::NO_EXTRA_WHITESPACE },
                                ),
                                _ => {}
                            }
                        }
                        if !flag_tokens.is_empty() {
                            flags_expr = quote! { #(#flag_tokens)|* };
                        }
                    }
                    Ok(())
                })?;
            }
        }

        parsed_fields.push(FieldSpec {
            ident,
            cvar_name,
            toml_key,
            description,
            flags_expr,
        });
    }

    let toml_lines = parsed_fields.iter().map(|f| {
        let ident = &f.ident;
        let key = &f.toml_key;
        let desc = &f.description;
        let desc_comment = if !desc.is_empty() {
            quote! { out.push_str(&format!("# {}\n", #desc)); }
        } else {
            quote! {}
        };
        quote! {
            #desc_comment
            out.push_str(&format!("{} = {}\n", #key, ::goldsrc_api::cvar::ToTomlVal::to_toml_val(&self.#ident)));
        }
    });

    let cvar_lines = parsed_fields.iter().map(|f| {
        let ident = &f.ident;
        let cvar_name = &f.cvar_name;
        let desc = &f.description;
        quote! {
            out.push_str(&format!("{} \"{}\" // {}\n", #cvar_name, self.#ident, #desc));
        }
    });

    let reg_lines = parsed_fields.iter().map(|f| {
        let ident = &f.ident;
        let cvar_name = &f.cvar_name;
        let flags = &f.flags_expr;
        quote! {
            engine.cvar_register(#cvar_name, &format!("{}", self.#ident), #flags);
        }
    });

    let sync_from_lines = parsed_fields.iter().map(|f| {
        let ident = &f.ident;
        let cvar_name = &f.cvar_name;
        quote! {
            ::goldsrc_api::cvar::FromCvarEngine::read_cvar(engine, #cvar_name, &mut self.#ident);
        }
    });

    let sync_to_lines = parsed_fields.iter().map(|f| {
        let ident = &f.ident;
        let cvar_name = &f.cvar_name;
        quote! {
            ::goldsrc_api::cvar::FromCvarEngine::write_cvar(&self.#ident, engine, #cvar_name);
        }
    });

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics ::goldsrc_api::cvar::ConfigModel for #struct_name #ty_generics #where_clause {
            fn to_toml(&self) -> ::std::string::String {
                let mut out = ::std::string::String::new();
                #(#toml_lines)*
                out
            }

            fn to_cvars(&self) -> ::std::string::String {
                let mut out = ::std::string::String::new();
                #(#cvar_lines)*
                out
            }

            fn register_cvars(&self, engine: &dyn ::goldsrc_api::cvar::CvarEngine) {
                #(#reg_lines)*
            }

            fn sync_from_cvars(&mut self, engine: &dyn ::goldsrc_api::cvar::CvarEngine) {
                #(#sync_from_lines)*
            }

            fn sync_to_cvars(&self, engine: &dyn ::goldsrc_api::cvar::CvarEngine) {
                #(#sync_to_lines)*
            }
        }
    };

    Ok(expanded)
}
