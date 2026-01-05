use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, LitInt, Path, Token, parse_macro_input};

#[proc_macro_derive(BinaryStruct, attributes(binary_struct, binary_field))]
pub fn binary_struct_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut bit_order: Path = syn::parse_str("shua_struct::Lsb0").unwrap();

    for attr in input.attrs.iter() {
        if attr.path().is_ident("binary_struct") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("bit_order") {
                    meta.input.parse::<Token![=]>()?;
                    bit_order = meta.input.parse()?;
                } else {
                    return Err(meta.error("expected `bit_order`"));
                }
                Ok(())
            })
            .unwrap();
        }
    }

    let struct_name = &input.ident;
    let fields_named = match &input.data {
        Data::Struct(data) => {
            if let Fields::Named(fields) = &data.fields {
                fields.named.clone()
            } else {
                panic!("BinaryStruct only supports structs with named fields");
            }
        }
        _ => panic!("BinaryStruct only works on structs"),
    };
    let mut parse_stmts = Vec::new();
    let mut build_stmts = Vec::new();
    let mut field_names = Vec::new();
    for field in fields_named.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        let mut opt_size_field: Option<Ident> = None;
        let mut opt_size_func: Option<Ident> = None;
        let mut opt_align: Option<usize> = None;
        let mut opt_sub_align: Option<u8> = None;
        let has_binary_field_attr = field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("binary_field"));
        for attr in &field.attrs {
            if attr.path().is_ident("binary_field") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("size_field") {
                        meta.input.parse::<Token![=]>()?;
                        opt_size_field = Some(meta.input.parse()?);
                        return Ok(());
                    }
                    if meta.path.is_ident("size_func") {
                        meta.input.parse::<Token![=]>()?;
                        opt_size_func = Some(meta.input.parse()?);
                        return Ok(());
                    }
                    if meta.path.is_ident("align") {
                        meta.input.parse::<Token![=]>()?;
                        let align_lit: LitInt = meta.input.parse()?;
                        let align_val: usize = align_lit.base10_parse()?;
                        opt_align = Some(align_val);
                        return Ok(());
                    }
                    if meta.path.is_ident("sub_align") {
                        meta.input.parse::<Token![=]>()?;
                        let align_lit: LitInt = meta.input.parse()?;
                        let align_val: u8 = align_lit.base10_parse()?;
                        opt_sub_align = Some(align_val);
                        return Ok(());
                    }
                    Err(meta.error(
                        "expected `size_field = ...`, `size_func = ...`, `align = ...`, or `sub_align = ...`",
                    ))
                });
            }
        }
        field_names.push(field_name);
        let has_opts = opt_size_field.is_some()
            || opt_size_func.is_some()
            || opt_align.is_some()
            || opt_sub_align.is_some()
            || has_binary_field_attr;
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
        parse_stmts.push(quote! {
            let field_opts = #field_opts_parse;
            let (val, mut l) = <#field_type as shua_struct::BinaryField<#bit_order>>::parse(
                &bits[offset..],
                &field_opts
            )?;
            #align_parse_logic
            s.#field_name = val;
            offset += l;
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
        build_stmts.push(quote! {
            let field_opts = #field_opts_build;
            let mut field_bv = <#field_type as shua_struct::BinaryField<#bit_order>>::build(&self.#field_name, &field_opts)?;
            #align_build_logic
            bv.extend(field_bv);
        });
    }
    let expanded = quote! {
        impl shua_struct::BinaryField<#bit_order> for #struct_name {
            fn parse(
                bits: &shua_struct::BitSlice<u8, #bit_order>,
                outer_opts: &Option<shua_struct::Options>,
            ) -> Result<(Self, usize), String> {
                let mut s = Self::default();
                let mut offset = 0;
                #(#parse_stmts)*
                Ok((s, offset))
            }
            fn build(&self, outer_opts: &Option<shua_struct::Options>) -> Result<shua_struct::BitVec<u8, #bit_order>, String> {
                let mut bv = shua_struct::BitVec::new();
                #(#build_stmts)*
                Ok(bv)
            }
        }
    };
    TokenStream::from(expanded)
}
