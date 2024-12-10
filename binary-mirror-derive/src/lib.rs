use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr, Field};

#[proc_macro_derive(BinaryMirror, attributes(bm))]
pub fn binary_mirror_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    impl_binary_mirror(&input)
}

#[proc_macro_derive(BinaryEnum, attributes(bv))]
pub fn binary_enum_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    impl_binary_enum(&input)
}

#[derive(Debug)]
struct FieldAttrs {
    type_name: String,
    alias: Option<String>,
    format: Option<String>,
    datetime_with: Option<String>,
    skip: bool,
    enum_type: Option<String>,
}

fn extract_array_size(field: &Field) -> usize {
    if let syn::Type::Array(array) = &field.ty {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(ref lit_int),
            ..
        }) = array.len {
            return lit_int.base10_parse::<usize>().expect("Could not parse array length");
        }
        panic!("Field is not an array");
    }
    panic!("Field is not an array");
}

fn get_field_size(input: &DeriveInput) -> Vec<(syn::Ident, usize)> {
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let out: Vec<(syn::Ident, usize)> = fields.iter().map(|f| (f.ident.clone().unwrap(), extract_array_size(f))).collect::<Vec<_>>();
    out
}


fn impl_binary_mirror(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let native_name = quote::format_ident!("{}Native", name);

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let field_debugs: Vec<_> = fields
        .iter()
        .filter_map(|f| {
            let field_name = &f.ident;
            Some(quote! {
                .field(
                    stringify!(#field_name),
                    &format_args!("hex: [{}], bytes: \"{}\"", 
                        self.#field_name
                            .iter()
                            .map(|b| format!("0x{:02x}", b))
                            .collect::<Vec<_>>()
                            .join(", "),
                        self.#field_name
                            .iter()
                            .map(|&b| {
                                match b {
                                    0x0A => "\\n".to_string(),
                                    0x0D => "\\r".to_string(),
                                    0x09 => "\\t".to_string(),
                                    0x20..=0x7E => (b as char).to_string(),
                                    _ => format!("\\x{:02x}", b),
                                }
                            })
                            .collect::<Vec<String>>()
                            .join("")
                    )
                )
            })
        })
        .collect();

    let methods: Vec<_> = fields
        .iter()
        .filter_map(|field| {
            let field_name = &field.ident;
            get_field_attrs(&field.attrs).map(|attrs| {
                let method_name = if let Some(alias) = &attrs.alias {
                    if attrs.type_name != "date" || attrs.datetime_with.is_none() {
                        quote::format_ident!("{}", alias)
                    } else {
                        field_name.clone().unwrap()
                    }
                } else {
                    field_name.clone().unwrap()
                };

                let base_method = match attrs.type_name.as_str() {
                    "str" => quote! {
                        pub fn #method_name(&self) -> String {
                            String::from_utf8_lossy(&self.#field_name).trim().to_string()
                        }
                    },
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
                    },
                    "decimal" => quote! {
                        pub fn #method_name(&self) -> Option<rust_decimal::Decimal> {
                            String::from_utf8_lossy(&self.#field_name)
                                .trim()
                                .parse::<rust_decimal::Decimal>()
                                .ok()
                                .map(|d| { d.normalize() })
                        }
                    },
                    "date" => {
                        let format = attrs.format.unwrap_or_else(|| "%Y%m%d".to_string());
                        
                        let date_method = quote! {
                            pub fn #method_name(&self) -> Option<chrono::NaiveDate> {
                                chrono::NaiveDate::parse_from_str(
                                    String::from_utf8_lossy(&self.#field_name).trim(),
                                    #format
                                ).ok()
                            }
                        };

                        if let Some(time_field) = &attrs.datetime_with {
                            let datetime_name = if let Some(datetime_alias) = attrs.alias {
                                quote::format_ident!("{}", datetime_alias)
                            } else {
                                quote::format_ident!("datetime")
                            };
                            
                            let time_ident = quote::format_ident!("{}", time_field);
                            
                            quote! {
                                #date_method

                                pub fn #datetime_name(&self) -> Option<chrono::NaiveDateTime> {
                                    let date = chrono::NaiveDate::parse_from_str(
                                        String::from_utf8_lossy(&self.#field_name).trim(),
                                        #format
                                    ).ok()?;
                                    let time = self.#time_ident()?;
                                    Some(chrono::NaiveDateTime::new(date, time))
                                }
                            }
                        } else {
                            date_method
                        }
                    },
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
                    },
                    "enum" => {
                        let enum_type = attrs.enum_type.as_ref()
                            .expect("enum_type attribute is required for enum type");
                        let enum_ident = quote::format_ident!("{}", enum_type);
                        
                        quote! {
                            pub fn #method_name(&self) -> Option<#enum_ident> {
                                #enum_ident::from_bytes(&self.#field_name)
                            }
                        }
                    },
                    _ => panic!("Unsupported type: {}", attrs.type_name),
                };

                base_method
            })
        })
        .collect();

    let display_fields: Vec<_> = fields
        .iter()
        .filter_map(|field| {
            let field_name = &field.ident;
            get_field_attrs(&field.attrs).and_then(|attrs| {
                if attrs.skip && (attrs.type_name != "date" || attrs.datetime_with.is_none()) {
                    return None;
                }

                let method_name = if let Some(alias) = &attrs.alias {
                    if attrs.type_name != "date" || attrs.datetime_with.is_none() {
                        quote::format_ident!("{}", alias)
                    } else {
                        field_name.clone().unwrap()
                    }
                } else {
                    field_name.clone().unwrap()
                };

                let tokens = match attrs.type_name.as_str() {
                    "str" => quote! {
                        write!(f, "{}: {}", stringify!(#method_name), self.#method_name())?;
                    },
                    "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => quote! {
                        match self.#method_name() {
                            Some(val) => write!(f, "{}: {}", stringify!(#method_name), val)?,
                            None => write!(f, "{}: <invalid>", stringify!(#method_name))?,
                        }
                    },
                    "decimal" => quote! {
                        match self.#method_name() {
                            Some(val) => write!(f, "{}: {}", stringify!(#method_name), val.normalize())?,
                            None => write!(f, "{}: <invalid>", stringify!(#method_name))?,
                        }
                    },
                    "date" => {
                        if attrs.skip {
                            if let Some(_) = &attrs.datetime_with {
                                let datetime_name = if let Some(datetime_alias) = attrs.alias {
                                    quote::format_ident!("{}", datetime_alias)
                                } else {
                                    quote::format_ident!("datetime")
                                };
                                
                                quote! {
                                    match self.#datetime_name() {
                                        Some(val) => write!(f, "{}: {}", stringify!(#datetime_name), val.format("%Y-%m-%dT%H:%M:%S"))?,
                                        None => write!(f, "{}: <invalid>", stringify!(#datetime_name))?,
                                    }
                                }
                            } else {
                                return None;
                            }
                        } else {
                            let mut tokens = quote! {
                                match self.#method_name() {
                                    Some(val) => write!(f, "{}: {}", stringify!(#method_name), val)?,
                                    None => write!(f, "{}: <invalid>", stringify!(#method_name))?,
                                }
                            };

                            if let Some(_) = &attrs.datetime_with {
                                let datetime_name = if let Some(datetime_alias) = attrs.alias {
                                    quote::format_ident!("{}", datetime_alias)
                                } else {
                                    quote::format_ident!("datetime")
                                };
                                
                                tokens = quote! {
                                    #tokens
                                    write!(f, ", ")?;
                                    match self.#datetime_name() {
                                        Some(val) => write!(f, "{}: {}", stringify!(#datetime_name), val.format("%Y-%m-%dT%H:%M:%S"))?,
                                        None => write!(f, "{}: <invalid>", stringify!(#datetime_name))?,
                                    }
                                };
                            }
                            tokens
                        }
                    },
                    "time" => quote! {
                        match self.#method_name() {
                            Some(val) => write!(f, "{}: {}", stringify!(#method_name), val)?,
                            None => write!(f, "{}: <invalid>", stringify!(#method_name))?,
                        }
                    },
                    "enum" => quote! {
                        match self.#method_name() {
                            Some(val) => write!(f, "{}: {:?}", stringify!(#method_name), val)?,
                            None => write!(f, "{}: <invalid>", stringify!(#method_name))?,
                        }
                    },
                    _ => quote! {},
                };
                Some(tokens)
            })
        })
        .collect();

    let native_fields = fields.iter().filter_map(|field| {
        let field_name = &field.ident;
        get_field_attrs(&field.attrs).map(|attrs| {
            if attrs.skip && attrs.datetime_with.is_none() {
                return None;
            }

            let field_ident = if let Some(alias) = &attrs.alias {
                quote::format_ident!("{}", alias)
            } else {
                field_name.clone().unwrap()
            };

            let field_type = match attrs.type_name.as_str() {
                "str" => quote!(String),
                "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => {
                    let type_ident = quote::format_ident!("{}", attrs.type_name);
                    quote!(Option<#type_ident>)
                },
                "decimal" => quote!(Option<rust_decimal::Decimal>),
                "date" => {
                    if attrs.datetime_with.is_some() {
                        quote!(Option<chrono::NaiveDateTime>)
                    } else {
                        quote!(Option<chrono::NaiveDate>)
                    }
                },
                "time" => quote!(Option<chrono::NaiveTime>),
                "enum" => {
                    let enum_type = attrs.enum_type.as_ref()
                        .expect("enum_type attribute is required for enum type");
                    let enum_ident = quote::format_ident!("{}", enum_type);
                    quote!(Option<#enum_ident>)
                },
                _ => panic!("Unsupported type: {}", attrs.type_name),
            };

            Some(quote! {
                pub #field_ident: #field_type
            })
        }).flatten()
    });

    let to_native_fields = fields.iter().filter_map(|field| {
        let field_name = &field.ident;
        get_field_attrs(&field.attrs).map(|attrs| {
            if attrs.skip && attrs.datetime_with.is_none() {
                return None;
            }

            let (native_field, getter_method) = if let Some(alias) = &attrs.alias {
                let ident = quote::format_ident!("{}", alias);
                (ident.clone(), ident)
            } else {
                let ident = field_name.clone().unwrap();
                (ident.clone(), ident)
            };

            Some(match attrs.type_name.as_str() {
                "str" => quote!(#native_field: self.#getter_method()),
                "date" if attrs.datetime_with.is_some() => {
                    if let Some(alias) = &attrs.alias {
                        let datetime_method = quote::format_ident!("{}", alias);
                        quote!(#native_field: self.#datetime_method())
                    } else {
                        quote!(#native_field: self.datetime())
                    }
                },
                _ => quote!(#native_field: self.#getter_method()),
            })
        }).flatten()
    });

    let _native_field_size = get_field_size(input);
    // let from_native_fields = native_field_size.iter().map(|field| {
    //     quote!(#field.0: #field.1)
    // });


    let gen = quote! {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        pub struct #native_name {
            #(#native_fields,)*
        }

        impl #name {
            #(#methods)*

            /// Create a new instance from bytes
            /// Returns Err if the bytes length doesn't match the struct size
            pub fn from_bytes(bytes: &[u8]) -> Result<&Self, binary_mirror::BytesSizeError> {
                let expected = Self::size();
                let actual = bytes.len();
                if actual != expected {
                    return Err(binary_mirror::BytesSizeError::new(
                        expected, 
                        actual,
                        bytes.iter()
                        .map(|&b| {
                            match b {
                                0x0A => "\\n".to_string(),
                                0x0D => "\\r".to_string(),
                                0x09 => "\\t".to_string(),
                                0x20..=0x7E => (b as char).to_string(),
                                _ => format!("\\x{:02x}", b),
                            }
                        })
                        .collect::<Vec<String>>()
                        .join("")
                    ));
                }
                // Safety: 
                // 1. We've verified the size matches
                // 2. The struct is #[repr(C)]
                // 3. The alignment is handled by the compiler
                Ok(unsafe { &*(bytes.as_ptr() as *const Self) })
            }

            /// Convert the struct back to its binary representation
            pub fn to_bytes(&self) -> &[u8] {
                // Safety: 
                // 1. The struct is #[repr(C)]
                // 2. We're reading the exact size of the struct
                // 3. All fields are byte arrays
                // 4. The returned slice lifetime is tied to self
                unsafe {
                    std::slice::from_raw_parts(
                        (self as *const Self) as *const u8,
                        Self::size()
                    )
                }
            }

            pub fn to_bytes_owned(&self) -> Vec<u8> {
                self.to_bytes().to_vec()
            }

            /// Get the size of the struct in bytes
            pub fn size() -> usize {
                std::mem::size_of::<Self>()
            }

            pub fn to_native(&self) -> #native_name {
                #native_name {
                    #(#to_native_fields,)*
                }
            }

            // pub fn from_native(native: &#native_name) -> Self {
            //     Self {
            //         #(#from_native_fields,)*
            //     }
            // }
        }

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(#name))
                    #(#field_debugs)*
                    .finish()
            }
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{} {{ ", stringify!(#name))?;
                let mut first = true;
                #(
                    if first {
                        first = false;
                    } else {
                        write!(f, ", ")?;
                    }
                    #display_fields
                )*
                write!(f, " }}")
            }
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
                datetime_with: None,
                skip: false,
                enum_type: None,
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
                } else if meta.path.is_ident("datetime_with") {
                    let lit = meta.value()?.parse::<LitStr>()?;
                    field_attrs.datetime_with = Some(lit.value());
                } else if meta.path.is_ident("skip") {
                    field_attrs.skip = meta.value()?.parse::<syn::LitBool>()?.value();
                } else if meta.path.is_ident("enum_type") {
                    let lit = meta.value()?.parse::<LitStr>()?;
                    field_attrs.enum_type = Some(lit.value());
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



fn get_variant_value(attrs: &[syn::Attribute]) -> Option<u8> {
    for attr in attrs {
        if attr.path().is_ident("bv") {
            let mut byte_value = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("value") {
                    let lit = meta.value()?.parse::<syn::LitByteStr>()?;
                    if let Some(&value) = lit.value().first() {
                        byte_value = Some(value);
                    }
                }
                Ok(())
            });
            return byte_value;
        }
    }
    None
}

fn impl_binary_enum(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => panic!("BinaryEnum can only be derived for enums"),
    };

    let match_arms = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let byte_value = get_variant_value(&variant.attrs)
            .unwrap_or_else(|| {
                let variant_str = variant_ident.to_string().to_uppercase();
                variant_str.chars().next().unwrap() as u8
            });
        
        quote! {
            Some(#byte_value) => Some(Self::#variant_ident),
        }
    });

    let gen = quote! {
        impl #name {
            pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
                match bytes.get(0) {
                    #(#match_arms)*
                    _ => None,
                }
            }
        }
    };

    gen.into()
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_derive() {
        // Tests will go here
    }
}
