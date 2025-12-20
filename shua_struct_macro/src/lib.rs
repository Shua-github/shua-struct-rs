use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, Data, DeriveInput, Fields, Ident, LitInt, parse_macro_input, Path, Token};

struct AttrArgs {
    bit_order: Ident,
}

impl Parse for AttrArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path: Path = input.parse()?;
        if !path.is_ident("bit_order") {
            return Err(input.error("expected `bit_order`"));
        }
        input.parse::<Token![=]>()?;
        let bit_order: Ident = input.parse()?;
        Ok(Self { bit_order })
    }
}

#[proc_macro_attribute]
pub fn binary_struct(attr: TokenStream, item: TokenStream) -> TokenStream {
    let order = if attr.is_empty() {
        Ident::new("Lsb0", proc_macro::Span::call_site().into())
    } else {
        let args = parse_macro_input!(attr as AttrArgs);
        args.bit_order
    };

    let mut input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;
    let fields_named = match &input.data {
        Data::Struct(data) => {
            if let Fields::Named(fields) = &data.fields {
                fields.named.clone()
            } else {
                panic!("binary_struct only supports structs with named fields");
            }
        }
        _ => panic!("binary_struct only works on structs"),
    };
    let mut parse_stmts = Vec::new();
    let mut build_stmts = Vec::new();
    let mut field_names = Vec::new();
    for field in fields_named.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        let field_name_str = field_name.to_string();
        let mut opt_size_field: Option<Ident> = None;
        let mut opt_size_func: Option<Ident> = None;
        let mut opt_align: Option<usize> = None;
        let mut opt_sub_align: Option<u8> = None;
        for attr in &field.attrs {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("size_field") {
                    let value = meta.value()?;
                    let field_ident: Ident = value.parse()?;
                    opt_size_field = Some(field_ident);
                    return Ok(());
                }
                if meta.path.is_ident("size_func") {
                    let value = meta.value()?;
                    let func_ident: Ident = value.parse()?;
                    opt_size_func = Some(func_ident);
                    return Ok(());
                }
                if meta.path.is_ident("align") {
                    let value = meta.value()?;
                    let align_lit: LitInt = value.parse()?;
                    let align_val: usize = align_lit.base10_parse()?;
                    opt_align = Some(align_val);
                    return Ok(());
                }
                if meta.path.is_ident("sub_align") {
                    let value = meta.value()?;
                    let align_lit: LitInt = value.parse()?;
                    let align_val: u8 = align_lit.base10_parse()?;
                    opt_sub_align = Some(align_val);
                    return Ok(());
                }
                Err(meta.error(
                    "expected `size_field = ...`, `size_func = ...`, `align = ...`, or `sub_align = ...`",
                ))
            });
        }
        field_names.push(field_name);
        let align_val = opt_align.unwrap_or(0);
        let sub_align_val = opt_sub_align.unwrap_or(0);
        let size_calc = if let Some(size_field) = opt_size_field.clone() {
            quote! { s.#size_field.into() }
        } else if let Some(size_func) = opt_size_func.clone() {
            quote! { s.#size_func() }
        } else {
            quote! { 0 }
        };
        let field_opts = quote! {
            Options {
                name: #field_name_str.to_string(),
                size: #size_calc,
                align: #align_val,
                sub_align: Cell::new(#sub_align_val),
            }
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
            let field_opts = Some(#field_opts);
            let (val, mut l) = <#field_type as BinaryField<#order>>::parse(
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
        let field_opts_build = quote! {
            Options {
                name: #field_name_str.to_string(),
                size: #size_calc_build,
                align: #align_val,
                sub_align: Cell::new(#sub_align_val),
            }
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
            let field_opts = Some(#field_opts_build);
            let mut field_bv = <#field_type as BinaryField<#order>>::build(&self.#field_name, &field_opts)?;
            #align_build_logic
            bv.extend(field_bv);
        });
    }
    if let Data::Struct(ref mut data) = input.data {
        if let Fields::Named(ref mut fields) = data.fields {
            for field in fields.named.iter_mut() {
                field
                    .attrs
                    .retain(|attr| !attr.path().is_ident("binary_field"));
            }
        }
    }
    let expanded = quote! {
        #input
        impl BinaryField<#order> for #struct_name {
            fn parse(
                bits: &BitSlice<u8, #order>,
                outer_opts: &Option<Options>,
            ) -> Result<(Self, usize), String> {
                let mut s = Self::default();
                let mut offset = 0;
                #(#parse_stmts)*
                Ok((s, offset))
            }
            fn build(&self, outer_opts: &Option<Options>) -> Result<BitVec<u8, #order>, String> {
                let mut bv = BitVec::new();
                #(#build_stmts)*
                Ok(bv)
            }
        }
    };
    TokenStream::from(expanded)
}