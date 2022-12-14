mod parameters;

extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Auto-generates ParameterImpl implementation for structs and enums
#[proc_macro_derive(ParameterImpl, attributes(param))]
pub fn process_parameter_impl(input: TokenStream) -> TokenStream {
    parameters::generate_impl(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
