use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr};

#[proc_macro_derive(BinaryMirror, attributes(bm))]
pub fn binary_mirror_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    impl_binary_mirror(&input)
}

#[derive(Debug)]
struct FieldAttrs {
    type_name: String,
    alias: Option<String>,
    format: Option<String>,
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
            get_field_attrs(&field.attrs).map(|attrs| {
                let method_name = if let Some(alias) = attrs.alias {
                    quote::format_ident!("{}", alias)
                } else {
                    field_name.clone().unwrap()
                };

                match attrs.type_name.as_str() {
                    "str" => {
                        quote! {
                            pub fn #method_name(&self) -> String {
                                String::from_utf8_lossy(&self.#field_name).trim().to_string()
                            }
                        }
                    }
                    "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => {
                        let type_ident = quote::format_ident!("{}", attrs.type_name);
                        quote! {
                            pub fn #method_name(&self) -> Option<#type_ident> {
                                String::from_utf8_lossy(&self.#field_name)
                                    .trim()
                                    .parse::<#type_ident>()
                                    .ok()
                            }
                        }
                    }
                    "decimal" => {
                        quote! {
                            pub fn #method_name(&self) -> Option<rust_decimal::Decimal> {
                                String::from_utf8_lossy(&self.#field_name)
                                    .trim()
                                    .parse::<rust_decimal::Decimal>()
                                    .ok()
                            }
                        }
                    }
                    "date" => {
                        let format = attrs.format.unwrap_or_else(|| "%Y%m%d".to_string());
                        quote! {
                            pub fn #method_name(&self) -> Option<chrono::NaiveDate> {
                                chrono::NaiveDate::parse_from_str(
                                    String::from_utf8_lossy(&self.#field_name).trim(),
                                    #format
                                ).ok()
                            }
                        }
                    }
                    "time" => {
                        let format = attrs.format.unwrap_or_else(|| "%H%M%S".to_string());
                        quote! {
                            pub fn #method_name(&self) -> Option<chrono::NaiveTime> {
                                chrono::NaiveTime::parse_from_str(
                                    String::from_utf8_lossy(&self.#field_name).trim(),
                                    #format
                                ).ok()
                            }
                        }
                    }
                    _ => panic!("Unsupported type: {}", attrs.type_name),
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

fn get_field_attrs(attrs: &[syn::Attribute]) -> Option<FieldAttrs> {
    for attr in attrs {
        if attr.path().is_ident("bm") {
            let mut field_attrs = FieldAttrs {
                type_name: String::new(),
                alias: None,
                format: None,
            };

            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("type") {
                    let lit = meta.value()?.parse::<LitStr>()?;
                    field_attrs.type_name = lit.value();
                } else if meta.path.is_ident("alias") {
                    let lit = meta.value()?.parse::<LitStr>()?;
                    field_attrs.alias = Some(lit.value());
                } else if meta.path.is_ident("format") {
                    let lit = meta.value()?.parse::<LitStr>()?;
                    field_attrs.format = Some(lit.value());
                }
                Ok(())
            });

            if !field_attrs.type_name.is_empty() {
                return Some(field_attrs);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_derive() {
        // Tests will go here
    }
}
