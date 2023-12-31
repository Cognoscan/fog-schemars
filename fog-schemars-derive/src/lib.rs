#![forbid(unsafe_code)]

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;

mod ast;
mod attr;
mod validator_exprs;

use ast::*;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;

#[proc_macro_derive(FogValidate, attributes(serde, fog))]
pub fn derive_fog_validate_wrapper(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    derive_fog_validate(input)
        .unwrap_or_else(|errs| {
            let compile_errors = errs.iter().map(syn::Error::to_compile_error);
            quote! {
                #(#compile_errors)*
            }
        })
        .into()
}

fn derive_fog_validate(mut input: syn::DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    attr::process_serde_attrs(&mut input)?;

    let mut cont = Container::from_ast(&input)?;
    add_trait_bounds(&mut cont);

    let crate_alias = cont.attrs.crate_name.as_ref().map(|path| {
        quote_spanned! { path.span() => use #path as fog_schemars; }
    });

    let type_name = &cont.ident;
    let (impl_generics, ty_generics, where_clause) = cont.generics.split_for_impl();

    if let Some(transparent_field) = cont.transparent_field() {
        let (ty, type_def) = validator_exprs::type_for_field_validator(transparent_field);
        return Ok(quote! {
            const _: () = {
                #crate_alias
                #type_def

                #[automatically_derived]
                impl #impl_generics fog_schemars::FogValidate for #type_name #ty_generics #where_clause {
                    fn should_reference(opt: bool) -> bool {
                        <#ty as fog_schemars::FogValidate>::should_reference(opt)
                    }

                    fn has_opt() -> bool {
                        <#ty as fog_schemars::FogValidate>::has_default_opt()
                    }

                    fn validator_name(opt: bool) -> String {
                        <#ty as fog_schemars::FogValidate>::validator_name(opt)
                    }

                    fn validator(gen: &mut SchemaGenerator, opt: bool) -> Validator {
                        <#ty as fog_schemars::FogValidate>::validator(gen)
                    }

                    fn validator_type_id() -> std::any::TypeId {
                        <#ty as fog_schemars::FogValidate>::validator_type_id()
                    }
                }
            };
        });
    }

    let mut base_name = cont.name();
    if !cont.attrs.is_renamed {
        if let Some(path) = cont.serde_attrs.remote() {
            if let Some(segment) = path.segments.last() {
                base_name = segment.ident.to_string();
            }
        }
    }

    let type_params: Vec<_> = cont.generics.type_params().map(|ty| &ty.ident).collect();
    let validator_name = if type_params.is_empty() || (cont.attrs.is_renamed && !base_name.contains('{')) {
        quote! { #base_name.to_owned() }
    }
    else if cont.attrs.is_renamed {
        let mut name_fmt = base_name;
        for typ in &type_params {
            name_fmt.push_str(&format!("{{{}:.0}}", typ));
        }
        quote! {
            format!(#name_fmt #(,#type_params=#type_params::validator_name())*)
        }
    }
    else {
        let mut name_fmt = base_name;
        name_fmt.push_str(&"_{}".repeat(type_params.len()));
        quote! {
            format!(#name_fmt #(,#type_params::validator_name())*)
        }
    };

    let validator_expr = validator_exprs::expr_for_container(&cont);

    Ok(quote! {
        const _: () = {
            #crate_alias

            #[automatically_derived]
            #[allow(unused_braces)]
            impl #impl_generics fog_schemars::FogSchema for #type_name #ty_generics #where_clause {
                fn validator_name(_: bool) -> std::string::String {
                    #validator_name
                }

                fn validator(gen: &mut fog_schemars::SchemaGenerator, _:bool) -> _fog_pack::validator::Validator {
                    #validator_expr
                }
            }
        }
    })

}

fn add_trait_bounds(cont: &mut Container) {
    if let Some(bounds) = cont.serde_attrs.ser_bound() {
        let where_clause = cont.generics.make_where_clause();
        where_clause.predicates.extend(bounds.iter().cloned());
    } else {
        // No explicit trait bounds specified, assume the Rust convention of adding the trait to each type parameter
        // TODO consider also adding trait bound to associated types when used as fields - I think Serde does this?
        for param in &mut cont.generics.params {
            if let syn::GenericParam::Type(ref mut type_param) = *param {
                type_param.bounds.push(parse_quote!(schemars::JsonSchema));
            }
        }
    }
}
