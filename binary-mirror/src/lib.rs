use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr};

#[proc_macro_derive(BinaryMirror, attributes(bm))]
pub fn binary_mirror_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    impl_binary_mirror(&input)
}

fn impl_binary_mirror(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let methods: Vec<_> = fields
        .iter()
        .filter_map(|field| {
            let field_name = &field.ident;
            get_bm_type(&field.attrs).map(|field_type| {
                match field_type.as_str() {
                    "str" => {
                        quote! {
                            pub fn #field_name(&self) -> String {
                                String::from_utf8_lossy(&self.#field_name).trim().to_string()
                            }
                        }
                    }
                    "i32" => {
                        quote! {
                            pub fn #field_name(&self) -> Option<i32> {
                                String::from_utf8_lossy(&self.#field_name)
                                    .trim()
                                    .parse::<i32>()
                                    .ok()
                            }
                        }
                    }
                    "decimal" => {
                        quote! {
                            pub fn #field_name(&self) -> Option<rust_decimal::Decimal> {
                                String::from_utf8_lossy(&self.#field_name)
                                    .trim()
                                    .parse::<rust_decimal::Decimal>()
                                    .ok()
                            }
                        }
                    }
                    _ => panic!("Unsupported type: {}", field_type),
                }
            })
        })
        .collect();

    let gen = quote! {
        impl #name {
            #(#methods)*
        }
    };

    gen.into()
}

fn get_bm_type(attrs: &[syn::Attribute]) -> Option<String> {
    let mut result: Option<String> = None;
    for attr in attrs {
        println!("attr: {:?}", attr);
        if attr.path().is_ident("bm") {
            let r = attr.parse_nested_meta(|meta| {
                // println!("meta_path: {:?}", meta.path);
                // println!("meta_input: {:?}", meta.input);
                if meta.path.is_ident("type") {
                    // println!("meta is type: {:?}", meta.path.is_ident("type"));
                    let lit = meta.value()?.parse::<LitStr>()?;
                    // println!("lit: {:?}", lit.value());
                    result = Some(lit.value());
                    return Ok(());
                }
                Ok(())
            });
            println!("r: {:?}", r);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_derive() {
        // Tests will go here
    }
}
