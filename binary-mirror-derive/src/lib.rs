use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr};

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

#[derive(Debug, Clone)]
struct FieldAttrs {
    type_name: String,
    alias: Option<String>,
    format: Option<String>,
    datetime_with: Option<String>,
    skip: bool,
    enum_type: Option<String>,
}

#[derive(Debug, Clone)]
struct OriginField {
    name: syn::Ident,
    size: usize,
    attrs: Option<FieldAttrs>,
}

#[derive(Debug, Clone)]
struct NativeField {
    name: syn::Ident,
    ty: proc_macro2::TokenStream,
    origin_fields: Vec<OriginField>, // Store the complete OriginField for reference
    is_combined_datetime: bool,
}

#[derive(Debug)]
struct NativeField2OriginFieldMap {
    origin_field: OriginField,
    native_field: Option<NativeField>,
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

fn get_origin_fields(input: &DeriveInput) -> Vec<OriginField> {
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    fields
        .iter()
        .map(|field| {
            let name = field.ident.clone().unwrap();

            // Check if field is [u8] array and get size
            let size = if let syn::Type::Array(array) = &field.ty {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(ref lit_int),
                    ..
                }) = array.len
                {
                    lit_int
                        .base10_parse::<usize>()
                        .expect("Could not parse array length")
                } else {
                    panic!("Field {} array length must be a literal integer", name);
                }
            } else {
                panic!("Field {} must be a [u8] array", name);
            };

            OriginField {
                name,
                size,
                attrs: get_field_attrs(&field.attrs),
            }
        })
        .collect()
}

fn get_native_fields_and_map(
    origin_fields: &[OriginField],
) -> (Vec<NativeField>, Vec<NativeField2OriginFieldMap>) {
    let mut native_fields = Vec::new();
    let mut native_field_map = Vec::new();
    let mut processed = std::collections::HashSet::new();

    for field in origin_fields {
        if let Some(attrs) = &field.attrs {
            // Skip if this field has already been processed
            if processed.contains(&field.name.to_string()) {
                continue;
            }

            let field_name = if let Some(alias) = &attrs.alias {
                quote::format_ident!("{}", alias)
            } else {
                field.name.clone()
            };

            match attrs.type_name.as_str() {
                "date" if attrs.datetime_with.is_some() => {
                    let time_field_name = attrs.datetime_with.as_ref().unwrap();
                    let time_field = origin_fields
                        .iter()
                        .find(|f| f.name == quote::format_ident!("{}", time_field_name))
                        .expect("Could not find time field");

                    processed.insert(time_field.name.to_string());
                    let native_field = NativeField {
                        name: if let Some(alias) = &attrs.alias {
                            quote::format_ident!("{}", alias)
                        } else {
                            quote::format_ident!("datetime")
                        },
                        ty: quote!(Option<chrono::NaiveDateTime>),
                        origin_fields: vec![field.clone(), time_field.clone()],
                        is_combined_datetime: true,
                    };
                    native_fields.push(native_field.clone());
                    native_field_map.push(NativeField2OriginFieldMap {
                        origin_field: field.clone(),
                        native_field: Some(native_field.clone()),
                    });
                    native_field_map.push(NativeField2OriginFieldMap {
                        origin_field: time_field.clone(),
                        native_field: Some(native_field),
                    });
                }
                _ => {
                    let ty = match attrs.type_name.as_str() {
                        "str" => quote!(String),
                        "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => {
                            let type_ident = quote::format_ident!("{}", attrs.type_name);
                            quote!(Option<#type_ident>)
                        }
                        "decimal" => quote!(Option<rust_decimal::Decimal>),
                        "datetime" => quote!(Option<chrono::NaiveDateTime>),
                        "date" => quote!(Option<chrono::NaiveDate>),
                        "time" => quote!(Option<chrono::NaiveTime>),
                        "enum" => {
                            let enum_type = attrs.enum_type.as_ref();
                            match enum_type {
                                Some(enum_type) => {
                                    let enum_ident = quote::format_ident!("{}", enum_type);
                                    quote!(Option<#enum_ident>)
                                }
                                None => panic!("enum_type is required for enum field"),
                            }
                        }
                        _ => continue,
                    };
                    let native_field = NativeField {
                        name: field_name,
                        ty,
                        origin_fields: vec![field.clone()],
                        is_combined_datetime: false,
                    };

                    native_fields.push(native_field.clone());
                    native_field_map.push(NativeField2OriginFieldMap {
                        origin_field: field.clone(),
                        native_field: Some(native_field),
                    });
                }
            }
        } else {
            native_field_map.push(NativeField2OriginFieldMap {
                origin_field: field.clone(),
                native_field: None,
            });
        }
    }

    (native_fields, native_field_map)
}

fn get_debug_fields(origin_fields: &[OriginField]) -> Vec<proc_macro2::TokenStream> {
    origin_fields
        .iter()
        .map(|field| {
            let field_name = &field.name;
            quote! {
                .field(
                    stringify!(#field_name),
                    &format_args!("hex: [{}], bytes: \"{}\"",
                        binary_mirror::to_hex_repr(&self.#field_name),
                        binary_mirror::to_bytes_repr(&self.#field_name)
                    )
                )
            }
        })
        .collect()
}

fn get_methods(native_fields: &[NativeField]) -> Vec<proc_macro2::TokenStream> {
    native_fields
        .iter()
        .map(|field| {
            let name = &field.name;
            let origin_field = &field.origin_fields[0].name;

            let method_with_warn_name = quote::format_ident!("{}_with_warn", name);

            let debug_bytes = quote! {
                tracing::warn!("Failed to parse {} in {:?}", stringify!(#name), self);
            };

            if field.is_combined_datetime {
                let date_field = &field.origin_fields[0].name;
                let time_field = &field.origin_fields[1].name;
                let date_format = field.origin_fields[0]
                    .attrs
                    .as_ref()
                    .and_then(|attrs| attrs.format.as_ref())
                    .map(String::as_str)
                    .unwrap_or("%Y%m%d");
                let time_format = field.origin_fields[1]
                    .attrs
                    .as_ref()
                    .and_then(|attrs| attrs.format.as_ref())
                    .map(String::as_str)
                    .unwrap_or("%H%M%S");

                quote! {
                    pub fn #name(&self) -> Option<chrono::NaiveDateTime> {
                        let date = chrono::NaiveDate::parse_from_str(
                            String::from_utf8_lossy(&self.#date_field).trim(),
                            #date_format
                        ).ok()?;
                        let time = chrono::NaiveTime::parse_from_str(
                            String::from_utf8_lossy(&self.#time_field).trim(),
                            #time_format
                        ).ok()?;
                        Some(chrono::NaiveDateTime::new(date, time))
                    }

                    pub fn #method_with_warn_name(&self) -> Option<chrono::NaiveDateTime> {
                        match self.#name() {
                            Some(dt) => Some(dt),
                            None => {
                                #debug_bytes
                                return None;
                            }
                        }
                    }
                }
            } else {
                let attrs = field.origin_fields[0].attrs.as_ref().unwrap();
                match attrs.type_name.as_str() {
                    "str" => quote! {
                        pub fn #name(&self) -> String {
                            String::from_utf8_lossy(&self.#origin_field).trim().to_string()
                        } 

                        pub fn #method_with_warn_name(&self) -> String {
                            String::from_utf8_lossy(&self.#origin_field).trim().to_string()
                        }
                    },
                    "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => {
                        let type_ident = quote::format_ident!("{}", attrs.type_name);
                        quote! {
                            pub fn #name(&self) -> Option<#type_ident> {
                                String::from_utf8_lossy(&self.#origin_field)
                                    .trim()
                                    .parse::<#type_ident>()
                                    .ok()
                            }

                            pub fn #method_with_warn_name(&self) -> Option<#type_ident> {
                                match self.#name() {
                                    Some(val) => Some(val),
                                    None => {
                                        #debug_bytes
                                        None
                                    }
                                }
                            }
                        }
                    }
                    "decimal" => quote! {
                        pub fn #name(&self) -> Option<rust_decimal::Decimal> {
                            String::from_utf8_lossy(&self.#origin_field)
                                .trim()
                                .parse::<rust_decimal::Decimal>()
                                .ok()
                                .map(|d| d.normalize())
                        }
                        pub fn #method_with_warn_name(&self) -> Option<rust_decimal::Decimal> {
                            match self.#name() {
                                Some(d) => Some(d),
                                None => {
                                    #debug_bytes
                                    None
                                }
                            }
                        }

                    },
                    "datetime" => {
                        let format = attrs
                            .format
                            .as_ref()
                            .map(String::as_str)
                            .unwrap_or("%Y%m%d%H%M%S");
                        quote! {
                            pub fn #name(&self) -> Option<chrono::NaiveDateTime> {
                                chrono::NaiveDateTime::parse_from_str(
                                    String::from_utf8_lossy(&self.#origin_field).trim(),
                                    #format
                                ).ok()
                            }

                            pub fn #method_with_warn_name(&self) -> Option<chrono::NaiveDateTime> {
                                match self.#name() {
                                    Some(dt) => Some(dt),
                                    None => {
                                        #debug_bytes
                                        None
                                    }
                                }
                            }


                        }
                    }
                    "date" => {
                        let format = attrs
                            .format
                            .as_ref()
                            .map(String::as_str)
                            .unwrap_or("%Y%m%d");
                        quote! {
                            pub fn #name(&self) -> Option<chrono::NaiveDate> {
                                chrono::NaiveDate::parse_from_str(
                                    String::from_utf8_lossy(&self.#origin_field).trim(),
                                    #format
                                )
                                .ok()
                            }
                            pub fn #method_with_warn_name(&self) -> Option<chrono::NaiveDate> {
                                match self.#name() {
                                    Some(d) => Some(d),
                                    None => {
                                        #debug_bytes
                                        None
                                    }
                                }
                            }
                        }
                    }
                    "time" => {
                        let format = attrs
                            .format
                            .as_ref()
                            .map(String::as_str)
                            .unwrap_or("%H%M%S");
                        quote! {
                            pub fn #name(&self) -> Option<chrono::NaiveTime> {
                                chrono::NaiveTime::parse_from_str(
                                    String::from_utf8_lossy(&self.#origin_field).trim(),
                                    #format
                                )
                                .ok()
                            }
                            pub fn #method_with_warn_name(&self) -> Option<chrono::NaiveTime> {
                                match self.#name() {
                                    Some(t) => Some(t),
                                    None => {
                                        #debug_bytes
                                        None
                                    }
                                }
                            }
                        }
                    }
                    "enum" => {
                        let enum_type = attrs.enum_type.as_ref().unwrap();
                        let enum_ident = quote::format_ident!("{}", enum_type);
                        quote! {
                            pub fn #name(&self) -> Option<#enum_ident> {
                                #enum_ident::from_bytes(&self.#origin_field)
                            }

                            pub fn #method_with_warn_name(&self) -> Option<#enum_ident> {
                                match self.#name() {
                                    Some(v) => Some(v),
                                    None => {
                                        #debug_bytes
                                        None
                                    }
                                }
                            }

                        }
                    }
                    _ => panic!("Unsupported type: {}", attrs.type_name),
                }
            }
        })
        .collect()
}

fn get_display_fields(native_fields: &[NativeField]) -> Vec<proc_macro2::TokenStream> {
    native_fields
        .iter()
        .filter_map(|field| {
            let name = &field.name;
            let method_name = &field.name;
            let attrs = &field.origin_fields[0].attrs.as_ref()?;
            let origin_field = &field.origin_fields[0].name;

            // Skip if marked with skip (except datetime fields)
            if attrs.skip && !field.is_combined_datetime {
                return None;
            }

            Some(match attrs.type_name.as_str() {
                "str" => quote! {
                    write!(f, "{}: {}", stringify!(#name), self.#method_name())?;
                },
                "i32" | "i64" | "u32" | "u64" | "f32" | "f64" | "decimal" | "datetime" | "date"
                | "time" => quote! {
                    match self.#method_name() {
                        Some(val) => write!(f, "{}: {}", stringify!(#name), val)?,
                        None => write!(f, "{}: Error<hex: [{}], bytes: \"{}\">",
                            stringify!(#name),
                            binary_mirror::to_hex_repr(&self.#origin_field),
                            binary_mirror::to_bytes_repr(&self.#origin_field)
                        )?,
                    }
                },
                "enum" => quote! {
                    match self.#method_name() {
                        Some(val) => write!(f, "{}: {:?}", stringify!(#name), val)?,
                        None => write!(f, "{}: Error<hex: [{}], bytes: \"{}\">",
                            stringify!(#name),
                            binary_mirror::to_hex_repr(&self.#origin_field),
                            binary_mirror::to_bytes_repr(&self.#origin_field)
                        )?,
                    }
                },
                _ => quote! {},
            })
        })
        .collect()
}

fn get_native_fields_token(native_fields: &[NativeField]) -> Vec<proc_macro2::TokenStream> {
    native_fields
        .iter()
        .map(|field| {
            let name = &field.name;
            let ty = &field.ty;

            quote! {
                pub #name: #ty
            }
        })
        .collect()
}

fn get_to_native_fields(native_fields: &[NativeField]) -> Vec<proc_macro2::TokenStream> {
    native_fields
        .iter()
        .map(|field| {
            let name = &field.name;
            let method_name = quote::format_ident!("{}_with_warn", name);
            quote! { #name: self.#method_name() }
        })
        .collect()
}

fn get_from_native_fields(
    native_field_map: &[NativeField2OriginFieldMap],
) -> Vec<proc_macro2::TokenStream> {
    native_field_map.iter().map(|mapping| {
        let field_name = &mapping.origin_field.name;
        let size = mapping.origin_field.size;

        if let Some(native_field) = &mapping.native_field {
            let native_name = &native_field.name;
            let attrs = mapping.origin_field.attrs.as_ref().unwrap();

            match attrs.type_name.as_str() {
                "str" => quote! {
                    #field_name: {
                        let mut bytes = [b' '; #size];
                        let s = native.#native_name.as_bytes();
                        bytes[..s.len().min(#size)].copy_from_slice(&s[..s.len().min(#size)]);
                        bytes
                    }
                },
                "enum" => quote! {
                    #field_name: {
                        let mut bytes = [b' '; #size];
                        if let Some(enum_val) = &native.#native_name {
                            let s = enum_val.as_bytes();
                            bytes[..s.len().min(#size)].copy_from_slice(&s[..s.len().min(#size)]);
                        }
                        bytes
                    }
                },
                "date" if native_field.is_combined_datetime => {
                    let format = attrs.format.as_ref()
                        .map(String::as_str)
                        .unwrap_or("%Y%m%d");
                    quote! {
                        #field_name: {
                            let mut bytes = [b' '; #size];
                            if let Some(dt) = native.#native_name {
                                let s = dt.format(#format).to_string();
                                let b = s.as_bytes();
                                bytes[..b.len().min(#size)].copy_from_slice(&b[..b.len().min(#size)]);
                            }
                            bytes
                        }
                    }
                },
                "time" if native_field.is_combined_datetime => {
                    let format = attrs.format.as_ref()
                        .map(String::as_str)
                        .unwrap_or("%H%M%S");
                    quote! {
                        #field_name: {
                            let mut bytes = [b' '; #size];
                            if let Some(dt) = native.#native_name {
                                let s = dt.format(#format).to_string();
                                let b = s.as_bytes();
                                bytes[..b.len().min(#size)].copy_from_slice(&b[..b.len().min(#size)]);
                            }
                            bytes
                        }
                    }
                },
                _ => quote! {
                    #field_name: {
                        let mut bytes = [b' '; #size];
                        if let Some(val) = &native.#native_name {
                            let s = val.to_string();
                            let b = s.as_bytes();
                            bytes[..b.len().min(#size)].copy_from_slice(&b[..b.len().min(#size)]);
                        }
                        bytes
                    }
                }
            }
        } else {
            // Field without attributes, just fill with spaces
            quote! {
                #field_name: [b' '; #size]
            }
        }
    }).collect()
}

fn impl_binary_mirror(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let native_name = quote::format_ident!("{}Native", name);

    let origin_fields = get_origin_fields(input);
    let (native_fields, native_field_map) = get_native_fields_and_map(&origin_fields);
    let debug_fields_token = get_debug_fields(&origin_fields);
    let display_fields_token = get_display_fields(&native_fields);
    let methods = get_methods(&native_fields);
    let native_fields_token = get_native_fields_token(&native_fields);
    let to_native_fields_token = get_to_native_fields(&native_fields);
    let from_native_fields_token = get_from_native_fields(&native_field_map);

    let gen = quote! {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        pub struct #native_name {
            #(#native_fields_token,)*
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
                    #(#to_native_fields_token,)*
                }
            }

            pub fn from_native(native: &#native_name) -> Self {
                Self {
                    #(#from_native_fields_token,)*
                }
            }
        }

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(#name))
                    #(#debug_fields_token)*
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
                    #display_fields_token
                )*
                write!(f, " }}")
            }
        }
    };

    gen.into()
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

    let match_arms_from = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let byte_value = get_variant_value(&variant.attrs).unwrap_or_else(|| {
            let variant_str = variant_ident.to_string().to_uppercase();
            variant_str.chars().next().unwrap() as u8
        });

        quote! {
            Some(#byte_value) => Some(Self::#variant_ident),
        }
    });

    let match_arms_to = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let byte_value = get_variant_value(&variant.attrs).unwrap_or_else(|| {
            let variant_str = variant_ident.to_string().to_uppercase();
            variant_str.chars().next().unwrap() as u8
        });

        quote! {
            Self::#variant_ident => &[#byte_value],
        }
    });

    let gen = quote! {
        impl #name {
            pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
                match bytes.get(0) {
                    #(#match_arms_from)*
                    _ => None,
                }
            }

            pub fn as_bytes(&self) -> &'static [u8] {
                match self {
                    #(#match_arms_to)*
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
