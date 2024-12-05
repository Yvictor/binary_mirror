use proc_macro::TokenStream;
use quote::quote;
use syn::{parenthesized, parse_macro_input, Data, DeriveInput, Fields, LitStr};

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
            let field_type = get_bm_type(&field.attrs);

            field_type.map(|field_type| {
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
        if attr.path().is_ident("bm") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("type") {
                    let content;
                    parenthesized!(content in meta.input);
                    let lit = content.parse::<LitStr>()?;
                    println!("lit: {}", lit.value());
                    result = Some(lit.value());
                    return Ok(());
                }
                Ok(())
            });
        }
    }
    result
}
//if let Ok(Meta::List(meta_list)) = attr.parse_nested_meta() {
//     for nested in meta_list.nested.iter() {
//         if let NestedMeta::Meta(Meta::NameValue(nv)) = nested {
//             if nv.path.is_ident("type") {
//                 if let Lit::Str(lit_str) = &nv.lit {
//                     return lit_str.value();
//                 }
//             }
//         }
//     }
// }

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_derive() {
        // Tests will go here
    }
}
