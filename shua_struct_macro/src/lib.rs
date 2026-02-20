use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, Path};

#[derive(FromDeriveInput)]
#[darling(attributes(binary_struct))]
struct BinaryStructAttrs {
    ident: Ident,
    #[darling(default = "default_bit_order")]
    bit_order: Path,
}

fn default_bit_order() -> Path {
    syn::parse_str("shua_struct::Lsb0").unwrap()
}

#[derive(FromField)]
#[darling(attributes(binary_field))]
struct BinaryFieldAttrs {
    #[darling(default)]
    size_field: Option<Ident>,
    #[darling(default)]
    size_func: Option<Ident>,
    #[darling(default)]
    align: Option<usize>,
    #[darling(default)]
    sub_align: Option<u8>,
    #[darling(default)]
    if_func: Option<Ident>,
    #[darling(default)]
    check_func: Option<Ident>,
}

#[proc_macro_derive(BinaryField, attributes(binary_struct, binary_field))]
pub fn binary_struct_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let attrs = match BinaryStructAttrs::from_derive_input(&input) {
        Ok(attrs) => attrs,
        Err(err) => return err.write_errors().into(),
    };

    let struct_name = &attrs.ident;
    let bit_order = attrs.bit_order;

    let fields_named = match &input.data {
        Data::Struct(data) => {
            if let Fields::Named(fields) = &data.fields {
                fields.named.clone()
            } else {
                return syn::Error::new_spanned(
                    &struct_name,
                    "BinaryField only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        }
        _ => {
            return syn::Error::new_spanned(&struct_name, "BinaryField only works on structs")
                .to_compile_error()
                .into();
        }
    };

    let mut parse_stmts = Vec::new();
    let mut build_stmts = Vec::new();
    let mut bit_len_stmts = Vec::new();

    for (field_idx, field) in fields_named.iter().enumerate() {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        let field_attrs = match BinaryFieldAttrs::from_field(field) {
            Ok(attrs) => attrs,
            Err(err) => return err.write_errors().into(),
        };

        let opt_size_field = field_attrs.size_field;
        let opt_size_func = field_attrs.size_func;
        let opt_align = field_attrs.align;
        let opt_sub_align = field_attrs.sub_align;
        let opt_if_func = field_attrs.if_func;
        let opt_check_func = field_attrs.check_func;

        let has_opts = opt_size_field.is_some()
            || opt_size_func.is_some()
            || opt_align.is_some()
            || opt_sub_align.is_some();

        let align_val = opt_align.unwrap_or(0);
        let sub_align_val = opt_sub_align.unwrap_or(0);

        let size_calc = if let Some(size_field) = opt_size_field.clone() {
            quote! { s.#size_field.into() }
        } else if let Some(size_func) = opt_size_func.clone() {
            quote! { s.#size_func() }
        } else {
            quote! { 0 }
        };

        let field_opts_parse = if has_opts {
            quote! {
                Some(shua_struct::Options {
                    size: #size_calc,
                    align: #align_val,
                    sub_align: std::cell::Cell::new(#sub_align_val),
                })
            }
        } else {
            quote! { None }
        };

        let align_parse_logic = if opt_align.is_some() && opt_sub_align.is_none() {
            quote! {
                let remainder = l % #align_val;
                if remainder != 0 {
                    l += #align_val - remainder;
                }
            }
        } else {
            quote! {}
        };

        let check_func_logic = if let Some(ref check_func) = opt_check_func {
            quote! {
                if let Some(err) = s.#check_func() {
                    return Err(err);
                }
            }
        } else {
            quote! {}
        };

        let parse_field_logic = if let Some(ref if_func) = opt_if_func {
            quote! {
                if s.#if_func() {
                    let field_opts = #field_opts_parse;
                    let val = <#field_type as shua_struct::BinaryField<#bit_order>>::parse(
                        &bits[offset..],
                        &field_opts
                    ).map_err(|e| {
                        #[cfg(debug_assertions)]
                        {
                            format!("{} parse error: {}", stringify!(#field_name), e)
                        }
                        #[cfg(not(debug_assertions))]
                        {
                            format!("{} parse error: {}", #field_idx, e)
                        }
                    })?;
                    let field_len = <#field_type as shua_struct::BinaryField<#bit_order>>::bit_len(&val, &field_opts);
                    let mut l = field_len;
                    #align_parse_logic
                    s.#field_name = val;
                    offset += l;
                }
            }
        } else {
            quote! {
                let field_opts = #field_opts_parse;
                let val = <#field_type as shua_struct::BinaryField<#bit_order>>::parse(
                    &bits[offset..],
                    &field_opts
                ).map_err(|e| {
                    #[cfg(debug_assertions)]
                    {
                        format!("{} parse error: {}", stringify!(#field_name), e)
                    }
                    #[cfg(not(debug_assertions))]
                    {
                        format!("{} parse error: {}", #field_idx, e)
                    }
                })?;
                let field_len = <#field_type as shua_struct::BinaryField<#bit_order>>::bit_len(&val, &field_opts);
                let mut l = field_len;
                #align_parse_logic
                s.#field_name = val;
                offset += l;
            }
        };

        parse_stmts.push(quote! {
            #parse_field_logic
            #check_func_logic
        });

        let size_calc_build = if let Some(size_field) = opt_size_field {
            quote! { self.#size_field.into() }
        } else if let Some(size_func) = opt_size_func {
            quote! { self.#size_func() }
        } else {
            quote! { 0 }
        };

        let field_opts_build = if has_opts {
            quote! {
                Some(shua_struct::Options {
                    size: #size_calc_build,
                    align: #align_val,
                    sub_align: std::cell::Cell::new(#sub_align_val),
                })
            }
        } else {
            quote! { None }
        };

        let align_build_logic = if opt_align.is_some() && opt_sub_align.is_none() {
            quote! {
                let remainder = field_bv.len() % #align_val;
                if remainder != 0 {
                    field_bv.resize(field_bv.len() + (#align_val - remainder), false);
                }
            }
        } else {
            quote! {}
        };

        let build_field_logic = if let Some(ref if_func) = opt_if_func {
            quote! {
                if self.#if_func() {
                    let field_opts = #field_opts_build;
                    let field_bv = <#field_type as shua_struct::BinaryField<#bit_order>>::build(&self.#field_name, &field_opts)?;
                    let mut field_bv = field_bv;
                    #align_build_logic
                    bv.extend(field_bv);
                }
            }
        } else {
            quote! {
                let field_opts = #field_opts_build;
                let field_bv = <#field_type as shua_struct::BinaryField<#bit_order>>::build(&self.#field_name, &field_opts)?;
                let mut field_bv = field_bv;
                #align_build_logic
                bv.extend(field_bv);
            }
        };

        build_stmts.push(quote! {
            #build_field_logic
        });

        let bit_len_field_logic = if let Some(ref if_func) = opt_if_func {
            quote! {
                if self.#if_func() {
                    let field_opts = #field_opts_build;
                    let field_len = <#field_type as shua_struct::BinaryField<#bit_order>>::bit_len(&self.#field_name, &field_opts);
                    let mut l = field_len;
                    #align_parse_logic
                    total_len += l;
                }
            }
        } else {
            quote! {
                let field_opts = #field_opts_build;
                let field_len = <#field_type as shua_struct::BinaryField<#bit_order>>::bit_len(&self.#field_name, &field_opts);
                let mut l = field_len;
                #align_parse_logic
                total_len += l;
            }
        };

        bit_len_stmts.push(quote! {
            #bit_len_field_logic
        });
    }

    let expanded = quote! {
        impl shua_struct::BinaryField<#bit_order> for #struct_name {
            #[inline]
            fn parse(
                bits: &shua_struct::BitSlice<u8, #bit_order>,
                outer_opts: &Option<shua_struct::Options>,
            ) -> Result<Self, String> {
                let mut s = Self::default();
                let mut offset = 0;
                #(#parse_stmts)*
                Ok(s)
            }

            #[inline]
            fn build(&self, outer_opts: &Option<shua_struct::Options>) -> Result<shua_struct::BitVec<u8, #bit_order>, String> {
                let total_bits = self.bit_len(outer_opts);
                let mut bv = shua_struct::BitVec::with_capacity(total_bits);
                #(#build_stmts)*
                Ok(bv)
            }

            #[inline]
            fn bit_len(&self, outer_opts: &Option<shua_struct::Options>) -> usize {
                let mut total_len = 0;
                #(#bit_len_stmts)*
                total_len
            }
        }
    };
    TokenStream::from(expanded)
}
