//! ECS System handler parsing, typestate specification analysis, and code generation.

use crate::defs::SystemDefInfo;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, ExprArray, ExprLit, Lit};

/// Parses a `#[system]` attribute on a method and inspects refined parameter typestates.
pub fn parse_system(attr: &syn::Attribute, sig: &mut syn::Signature) -> syn::Result<SystemDefInfo> {
    let mut stage_name = "frame".to_string();
    let mut phase_name = "execute".to_string();
    let mut before_list: Vec<String> = Vec::new();
    let mut after_list: Vec<String> = Vec::new();

    if let Ok(Lit::Str(s)) = attr.parse_args::<Lit>() {
        stage_name = s.value();
    } else if let Ok(meta_list) = attr.meta.require_list() {
        meta_list.parse_nested_meta(|meta| {
            if meta.path.is_ident("stage") {
                if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                    stage_name = s.value();
                }
            } else if meta.path.is_ident("phase") {
                if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                    phase_name = s.value();
                }
            } else if meta.path.is_ident("before") {
                if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                    before_list.push(s.value());
                } else if let Ok(ExprArray { elems, .. }) = meta.value()?.parse::<ExprArray>() {
                    for elem in elems {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        }) = elem
                        {
                            before_list.push(s.value());
                        }
                    }
                }
            } else if meta.path.is_ident("after") {
                if let Ok(Lit::Str(s)) = meta.value()?.parse::<Lit>() {
                    after_list.push(s.value());
                } else if let Ok(ExprArray { elems, .. }) = meta.value()?.parse::<ExprArray>() {
                    for elem in elems {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        }) = elem
                        {
                            after_list.push(s.value());
                        }
                    }
                }
            }
            Ok(())
        })?;
    }

    let mut target_ty_name = String::new();
    let mut refined_specs: Vec<syn::Type> = Vec::new();
    let mut takes_refined_directly = false;

    if let Some(syn::FnArg::Typed(pat_type)) = sig.inputs.first_mut() {
        pat_type.attrs.retain(|attr| {
            if attr.path().is_ident("refined") {
                if let Ok(types) = attr.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Type, syn::Token![,]>::parse_terminated,
                ) {
                    for t in types {
                        refined_specs.push(t);
                    }
                }
                false
            } else {
                true
            }
        });

        let ty_str = quote!(#pat_type).to_string();
        if ty_str.contains("Player") {
            target_ty_name = "Player".to_string();
        } else if ty_str.contains("Client") {
            target_ty_name = "Client".to_string();
        } else if ty_str.contains("Entity") && !ty_str.contains("EntityId") {
            target_ty_name = "Entity".to_string();
        }

        if ty_str.contains("Refined") {
            takes_refined_directly = true;
            if target_ty_name.is_empty() {
                target_ty_name = "Player".to_string();
            }
            if refined_specs.is_empty() {
                if let syn::Type::Path(syn::TypePath { path, .. }) = &*pat_type.ty {
                    if let Some(seg) = path.segments.last() {
                        if seg.ident == "Refined" {
                            if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                                let type_args: Vec<_> = args
                                    .args
                                    .iter()
                                    .filter_map(|arg| {
                                        if let syn::GenericArgument::Type(t) = arg {
                                            Some(t.clone())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                if type_args.len() >= 2 {
                                    let first_target = &type_args[0];
                                    let target_arg_str = quote!(#first_target).to_string();
                                    if target_arg_str.contains("Client") {
                                        target_ty_name = "Client".to_string();
                                    } else if target_arg_str.contains("Entity") {
                                        target_ty_name = "Entity".to_string();
                                    } else {
                                        target_ty_name = "Player".to_string();
                                    }
                                    refined_specs.push(type_args[1].clone());
                                } else if type_args.len() == 1 {
                                    refined_specs.push(type_args[0].clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(SystemDefInfo {
        stage: stage_name,
        phase: phase_name,
        before: before_list,
        after: after_list,
        ident: sig.ident.clone(),
        inputs_len: sig.inputs.len(),
        target_ty_name,
        refined_specs,
        takes_refined_directly,
    })
}

/// Generates ECS system registrations for `on_load`.
pub fn generate_system_registrations(
    struct_name: &syn::Type,
    handlers: &[SystemDefInfo],
) -> Vec<TokenStream> {
    let mut registrations = Vec::new();
    for sys in handlers {
        let sys_ident = &sys.ident;
        let sys_name = sys_ident.to_string();
        let stage_str = &sys.stage;
        let phase_str = &sys.phase;
        let before_strs = &sys.before;
        let after_strs = &sys.after;

        let runner_fn = if sys.inputs_len == 0 {
            quote! { |_world: &mut ::goldsrc::ecs::World, _target: Option<::goldsrc::ecs::EntityId>| {
                #struct_name::#sys_ident();
            }}
        } else if sys.inputs_len == 1 {
            if !sys.target_ty_name.is_empty() {
                let target_ty: syn::Path = match sys.target_ty_name.as_str() {
                    "Client" => syn::parse_quote!(::goldsrc::Client),
                    "Entity" => syn::parse_quote!(::goldsrc::Entity),
                    _ => syn::parse_quote!(::goldsrc::Player),
                };

                if !sys.refined_specs.is_empty() {
                    let spec_ty = if sys.refined_specs.len() == 1 {
                        let single = &sys.refined_specs[0];
                        quote!(#single)
                    } else {
                        let specs = &sys.refined_specs;
                        quote!((#(#specs),*))
                    };

                    if sys.takes_refined_directly {
                        quote! { |_world: &mut ::goldsrc::ecs::World, target: Option<::goldsrc::ecs::EntityId>| {
                            if let Some(target_id) = target {
                                let mut target_obj = #target_ty::new(target_id.0 as i32);
                                use ::goldsrc::RefineExt;
                                if let Ok(refined) = target_obj.refine::<#spec_ty>() {
                                    #struct_name::#sys_ident(refined);
                                }
                            }
                        }}
                    } else {
                        quote! { |_world: &mut ::goldsrc::ecs::World, target: Option<::goldsrc::ecs::EntityId>| {
                            if let Some(target_id) = target {
                                let mut target_obj = #target_ty::new(target_id.0 as i32);
                                use ::goldsrc::RefineExt;
                                if let Ok(mut refined) = target_obj.refine::<#spec_ty>() {
                                    #struct_name::#sys_ident(&mut *refined);
                                }
                            }
                        }}
                    }
                } else if sys.takes_refined_directly {
                    quote! { |_world: &mut ::goldsrc::ecs::World, target: Option<::goldsrc::ecs::EntityId>| {
                        if let Some(target_id) = target {
                            let mut target_obj = #target_ty::new(target_id.0 as i32);
                            use ::goldsrc::RefineExt;
                            if let Ok(refined) = target_obj.refine() {
                                #struct_name::#sys_ident(refined);
                            }
                        }
                    }}
                } else {
                    quote! { |_world: &mut ::goldsrc::ecs::World, target: Option<::goldsrc::ecs::EntityId>| {
                        if let Some(target_id) = target {
                            let mut p = #target_ty::new(target_id.0 as i32);
                            #struct_name::#sys_ident(&mut p);
                        }
                    }}
                }
            } else {
                quote! { |world: &mut ::goldsrc::ecs::World, _target: Option<::goldsrc::ecs::EntityId>| {
                    #struct_name::#sys_ident(world);
                }}
            }
        } else {
            quote! { |world: &mut ::goldsrc::ecs::World, target: Option<::goldsrc::ecs::EntityId>| {
                #struct_name::#sys_ident(world, target);
            }}
        };

        registrations.push(quote! {
            ::goldsrc::ecs::System::builder(#sys_name)
                .stage(#stage_str.parse::<::goldsrc::ecs::Stage>().unwrap_or(::goldsrc::ecs::Stage::Frame))
                .phase(#phase_str.parse::<::goldsrc::ecs::SystemPhase>().unwrap_or(::goldsrc::ecs::SystemPhase::Execute))
                .before(vec![#(#before_strs),*])
                .after(vec![#(#after_strs),*])
                .register(#runner_fn);
        });
    }
    registrations
}
