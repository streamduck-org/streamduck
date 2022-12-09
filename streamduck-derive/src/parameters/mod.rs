mod constants;

use proc_macro::{TokenStream as OGTokenStream};
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, DeriveInput, Data, Field, Path, Meta, NestedMeta, Lit, Variant};
use syn::spanned::Spanned;
use crate::parameters::constants::Constants;

pub fn generate_impl(input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let cst = Constants::new();

    let trait_type = &cst.trait_type;
    let ident = &input.ident;

    let body = generate_body(&cst, &ident, &input.data)?;

    Ok(quote!{
        impl #trait_type for #ident {
            #body
        }
    })
}

fn generate_body(cst: &Constants, ident: &Ident, data: &Data) -> Result<TokenStream, syn::Error> {
    let parameter_type = &cst.parameter_type;
    let parameter_variant_type = &cst.parameter_variant_type;
    let parameter_options_type = &cst.parameter_options_type;
    let localized_string_type = &cst.localized_string_type;

    match data {
        Data::Struct(str) => {
            let field_tokens = str.fields.iter()
                .enumerate()
                .map(|(index, x)| generate_get_field(cst,x, index, true))
                .collect::<Result<Vec<TokenStream>, syn::Error>>()?;

            Ok(quote!(
                fn parameter(&self, options: #parameter_options_type) -> #parameter_type {
                    #parameter_type::new_from_options(
                        &options,
                        #parameter_variant_type::CollapsableMenu(vec![
                            #(#field_tokens),*
                        ].into_iter().flatten().collect())
                    )
                }
            ))
        },
        Data::Enum(enm) => {
            let enum_variants = enm.variants.iter()
                .map(|x| generate_variant(cst, ident, x))
                .collect::<Result<Vec<TokenStream>, syn::Error>>()?;

            let msg = format!("{:#?}", enm);

            Ok(quote!(
                #[doc = #msg]
                fn parameter(&self, options: #parameter_options_type) -> #parameter_type {
                    #parameter_type::new_from_options(
                        &options,
                        #parameter_variant_type::CollapsableMenu(match self {
                            #(#enum_variants),*
                        }.into_iter().flatten().collect())
                    )
                }
            ))
        },
        Data::Union(union) => {
            Err(syn::Error::new(union.union_token.span, "unions are not supported"))
        }
    }
}

fn generate_variant(cst: &Constants, enum_ident: &Ident, variant: &Variant) -> Result<TokenStream, syn::Error> {
    let variant_ident = &variant.ident;

    let field_tokens = variant.fields.iter()
        .enumerate()
        .map(|(index, x)| generate_get_field(cst, x, index, false))
        .collect::<Result<Vec<TokenStream>, syn::Error>>()?;

    Ok(quote!{
        #enum_ident::#variant_ident => vec![
            #(#field_tokens),*
        ].into_iter().flatten().collect()
    })
}

fn check_path(search: &str, path: &Path) -> bool {
    for segment in &path.segments {
        let ident_str = segment.ident.to_string();

        if ident_str == search {
            return true;
        }
    }

    false
}

fn infer_options(cst: &Constants, ident: &Ident, field: &Field, field_index: usize) -> Result<(TokenStream, bool), syn::Error> {
    let ident = ident.to_string();

    let mut should_flatten = false;

    let name_field = quote!(#ident.to_string());
    let mut display_name_field = quote!("".to_string());
    let mut description_field = quote!("".to_string());
    let mut disabled_field = quote!(false);

    let preferred_type = &cst.preferred_parameter_variant_type;
    let mut preference_field = quote!(#preferred_type::NoPreference);

    for attr in &field.attrs {
        let meta = &attr.parse_meta()?;

        if let Meta::List(meta_list) = meta {
            if check_path("param", &meta_list.path) {
                for nested_meta in &meta_list.nested {
                    match nested_meta {
                        NestedMeta::Meta(meta) => {
                            match meta {
                                Meta::Path(path) => {
                                    let ident = &path.segments.first()
                                        .map_or_else(
                                            || Err(syn::Error::new(path.span(), "cannot find segment")),
                                            |s| Ok(s)
                                        )?.ident;

                                    let ident_str = ident.to_string().to_lowercase();

                                    match ident_str.as_str() {
                                        "disabled" => {
                                            disabled_field = quote!(true);
                                        }

                                        "choice" => {
                                            preference_field = quote!(#preferred_type::Choice);
                                        }

                                        "label" => {
                                            preference_field = quote!(#preferred_type::Label);
                                        }

                                        "textinput" => {
                                            preference_field = quote!(#preferred_type::TextInput);
                                        }

                                        "toggle" => {
                                            preference_field = quote!(#preferred_type::Toggle);
                                        }

                                        "checkbox" => {
                                            preference_field = quote!(#preferred_type::Checkbox);
                                        }

                                        "flatten" => {
                                            should_flatten = true;
                                        }

                                        _ => {
                                            return Err(syn::Error::new(ident.span(), "not supported attribute"))
                                        }
                                    }
                                }
                                Meta::List(meta_list) => {
                                    return Err(syn::Error::new(meta_list.span(), "there's no attributes that use this kind of syntax"))
                                }
                                Meta::NameValue(pair) => {
                                    let ident = &pair.path.segments.first()
                                        .map_or_else(
                                            || Err(syn::Error::new(pair.path.span(), "cannot find segment")),
                                            |s| Ok(s)
                                        )?.ident;

                                    let ident_str = ident.to_string().to_lowercase();

                                    match ident_str.as_str() {
                                        "loc_key" => {
                                            if let Lit::Str(str) = &pair.lit {
                                                let display_format = format!("{}.name", str.value());
                                                let desc_format = format!("{}.desc", str.value());

                                                display_name_field = quote!(#display_format.to_string());
                                                description_field = quote!(#desc_format.to_string());
                                            } else {
                                                return Err(syn::Error::new(pair.lit.span(), "not supported value for this attribute"))
                                            }
                                        }

                                        _ => {
                                            return Err(syn::Error::new(ident.span(), "not supported attribute"))
                                        }
                                    }
                                }
                            }
                        }
                        NestedMeta::Lit(literal) => {
                            return Err(syn::Error::new(literal.span(), "random literals are not allowed"))
                        }
                    }
                }
            }
        }


    }

    let options_type = &cst.parameter_options_type;

    Ok((quote!{
        #options_type {
            name: #name_field,
            display_name: #display_name_field,
            description: #description_field,
            disabled: #disabled_field,
            preferred_variant: #preference_field
        }
    }, should_flatten))
}

fn generate_get_field(cst: &Constants, field: &Field, field_index: usize, use_self: bool) -> Result<TokenStream, syn::Error> {
    let ident = field.ident.clone().unwrap_or_else(|| {
        Ident::new(&field_index.to_string(), field.span())
    });

    let (options, flatten) = infer_options(cst, &ident, field, field_index)?;

    let field_type = &field.ty;
    let field_type_span = field_type.span();
    let trait_type = &cst.trait_type;

    let value_token = if use_self {
        quote!(&self.#ident)
    } else {
        quote!(&#ident)
    };

    if flatten {
        let flatten_func = &cst.flatten_parameter_function;

        Ok(quote_spanned! {field_type_span=>
            #flatten_func(<#field_type as #trait_type>::parameter(#value_token, #options))
        })
    } else {
        Ok(quote_spanned! {field_type_span=>
            vec![<#field_type as #trait_type>::parameter(#value_token, #options)]
        })
    }
}

