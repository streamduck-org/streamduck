extern crate proc_macro;
use proc_macro::TokenStream;

fn add_trait(attr: TokenStream, mut item: TokenStream, trait_path: &str) -> TokenStream {
    let mut item_iter = item.clone().into_iter();

    let mut struct_name = None;
    while struct_name.is_none() {
        if let Some(tkn) = item_iter.next() {
            if tkn.to_string() == "struct" {
                if let Some(name) = item_iter.next() {
                    struct_name = Some(name.to_string());
                }
                break;
            }
        } else {
            break;
        }
    }

    let result = if let Some(component_name) = attr.into_iter().next() {
        if let Some(struct_name) = struct_name {
            format!(r#"impl {} for {} {{ const NAME: &'static str = "{}"; }}"#, trait_path, struct_name, component_name.to_string().replace("\"", ""))
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    }.parse::<TokenStream>().unwrap();

    item.extend(result);

    item
}

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    add_trait(attr, item, "::streamduck_core::core::button::Component")
}

#[proc_macro_attribute]
pub fn socket_data(attr: TokenStream, item: TokenStream) -> TokenStream {
    add_trait(attr, item, "::streamduck_daemon::SocketData")
}