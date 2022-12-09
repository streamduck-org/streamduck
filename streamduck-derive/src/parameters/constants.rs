use proc_macro2::{TokenStream};
use quote::quote;

pub struct Constants {
    pub trait_type: TokenStream,
    pub parameter_type: TokenStream,
    pub flatten_parameter_function: TokenStream,
    pub parameter_variant_type: TokenStream,
    pub preferred_parameter_variant_type: TokenStream,
    pub parameter_options_type: TokenStream,
    pub localized_string_type: TokenStream,
}

impl Constants {
    pub fn new() -> Constants {
        Self {
            trait_type: quote!(::streamduck_core::parameters::ParameterImpl),
            parameter_type: quote!(::streamduck_core::parameters::Parameter),
            flatten_parameter_function: quote!(::streamduck_core::parameters::flatten_parameter),
            parameter_variant_type: quote!(::streamduck_core::parameters::ParameterVariant),
            preferred_parameter_variant_type: quote!(::streamduck_core::parameters::PreferredParameterVariant),
            parameter_options_type: quote!(::streamduck_core::parameters::ParameterOptions),
            localized_string_type: quote!(::streamduck_core::localization::LocalizedString),
        }
    }
}