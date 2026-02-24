use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, Path, Type};

#[derive(FromDeriveInput)]
#[darling(attributes(binary_struct))]
struct BinaryStructAttrs {
    ident: Ident,
    #[darling(default = "default_bit_order")]
    bit_order: Path,
    #[darling(default = "default_custom_error")]
    custom_error: String,
}

fn default_custom_error() -> String {
    "()".to_string()
}

fn default_bit_order() -> Path {
    syn::parse_str("shua_struct::Lsb0").expect("Failed to parse default bit order")
}

#[derive(FromField)]
#[darling(attributes(binary_field))]
struct BinaryFieldAttrs {
    #[darling(default)]
    count_field: Option<Ident>,
    #[darling(default)]
    count_func: Option<Ident>,
    #[darling(default)]
    align: Option<usize>,
    #[darling(default)]
    elem_align: Option<usize>,
    #[darling(default)]
    if_func: Option<Ident>,
    #[darling(default)]
    check_func: Option<Ident>,
}

struct FieldGenerator<'a> {
    idx: usize,
    name: &'a Ident,
    ty: &'a Type,
    attrs: BinaryFieldAttrs,
    error_enum: &'a Ident,
    bit_order: &'a Path,
}

impl<'a> FieldGenerator<'a> {
    fn gen_align_logic(&self) -> TokenStream2 {
        if let Some(align) = self.attrs.align {
            if align == 0 {
                quote! {}
            } else if align.is_power_of_two() {
                quote! {
                    l = (l + #align - 1) & !(#align - 1);
                }
            } else {
                quote! {
                    l = (l + #align - 1) / #align * #align;
                }
            }
        } else {
            quote! {}
        }
    }

    fn gen_ctx(&self, prefix: &TokenStream2) -> TokenStream2 {
        let has_count = self.attrs.count_field.is_some() || self.attrs.count_func.is_some();
        let has_align = self.attrs.align.is_some();
        let has_elem_align = self.attrs.elem_align.is_some();

        if !has_count && !has_align && !has_elem_align {
            return quote! { () };
        }

        let ctx_name = quote::format_ident!("{}Ctx", self.name);
        let mut struct_fields = Vec::new();
        let mut inst_fields = Vec::new();

        let count_impl = if has_count {
            let count_val = if let Some(f) = &self.attrs.count_field {
                quote! { #prefix.#f.into() }
            } else if let Some(f) = &self.attrs.count_func {
                quote! { #prefix.#f() }
            } else {
                quote! { 0 }
            };
            struct_fields.push(quote! { count: usize, });
            inst_fields.push(quote! { count: #count_val, });
            quote! {
                impl shua_struct::Count for #ctx_name {
                    #[inline]
                    fn get_count(&self) -> usize { self.count }
                }
            }
        } else {
            quote! {}
        };

        let align_impl = if has_align || has_elem_align {
            let align_val = self.attrs.align.or(self.attrs.elem_align).unwrap_or(0);
            struct_fields.push(quote! { align: usize, });
            inst_fields.push(quote! { align: #align_val, });
            quote! {
                impl shua_struct::Align for #ctx_name {
                    #[inline]
                    fn get_align(&self) -> usize { self.align }
                }
            }
        } else {
            quote! {
                impl shua_struct::Align for #ctx_name {
                    #[inline]
                    fn get_align(&self) -> usize { 0 }
                }
            }
        };

        let elem_ctx_impl = if let Some(elem_align_val) = self.attrs.elem_align {
            let elem_ctx_name = quote::format_ident!("{}ElemCtx", self.name);
            struct_fields.push(quote! { elem_align: usize, });
            inst_fields.push(quote! { elem_align: #elem_align_val, });
            quote! {
                #[allow(non_camel_case_types)]
                #[derive(Clone)]
                struct #elem_ctx_name { align: usize }

                impl shua_struct::Align for #elem_ctx_name {
                    #[inline]
                    fn get_align(&self) -> usize { self.align }
                }

                impl shua_struct::ElemCtx for #ctx_name {
                    type ElemCtx = #elem_ctx_name;
                    #[inline]
                    fn get_elem_ctx(&self) -> Self::ElemCtx {
                        #elem_ctx_name { align: self.elem_align }
                    }
                }
            }
        } else {
            quote! {
                impl shua_struct::ElemCtx for #ctx_name {
                    type ElemCtx = ();
                    #[inline]
                    fn get_elem_ctx(&self) -> Self::ElemCtx { () }
                }
            }
        };

        quote! {
            {
                #[allow(non_camel_case_types)]
                #[derive(Clone)]
                struct #ctx_name {
                    #(#struct_fields)*
                }
                #count_impl
                #align_impl
                #elem_ctx_impl
                #ctx_name {
                    #(#inst_fields)*
                }
            }
        }
    }

    fn gen_map_err(&self) -> TokenStream2 {
        let name = self.name;
        let idx = self.idx;
        let err_enum = self.error_enum;

        quote! {
            |_| {
                #[cfg(debug_assertions)]
                let err = shua_struct::BinaryError::At { index: stringify!(#name), source: #err_enum::#name };
                #[cfg(not(debug_assertions))]
                let err = shua_struct::BinaryError::At { index: #idx, source: #err_enum::#name };
                err
            }
        }
    }

    fn gen_parse_stmt(&self) -> TokenStream2 {
        let name = self.name;
        let ty = self.ty;
        let bit_order = self.bit_order;

        let ctx = self.gen_ctx(&quote! { s });
        let map_err = self.gen_map_err();
        let align = self.gen_align_logic();

        let core_logic = quote! {
            let field_ctx = #ctx;
            let val = <#ty as shua_struct::BinaryField<#bit_order, _>>::parse(
                &bits[offset..],
                &field_ctx
            ).map_err(#map_err)?;

            let field_len = <#ty as shua_struct::BinaryField<#bit_order, _>>::bit_len(&val, &field_ctx);
            let mut l = field_len;
            #align
            s.#name = val;
            offset += l;
        };

        let wrapped = if let Some(if_func) = &self.attrs.if_func {
            quote! { if s.#if_func() { #core_logic } }
        } else {
            core_logic
        };

        let check_logic = if let Some(check_func) = &self.attrs.check_func {
            quote! {
                if let Err(e) = s.#check_func() {
                    return Err(shua_struct::BinaryError::Custom(e));
                }
            }
        } else {
            quote! {}
        };

        quote! {
            #wrapped
            #check_logic
        }
    }

    fn gen_build_stmt(&self) -> TokenStream2 {
        let name = self.name;
        let ty = self.ty;
        let bit_order = self.bit_order;

        let ctx = self.gen_ctx(&quote! { self });
        let map_err = self.gen_map_err();
        let align = self.gen_align_logic();

        let core_logic = quote! {
            let field_ctx = #ctx;
            let field_len = <#ty as shua_struct::BinaryField<#bit_order, _>>::bit_len(&self.#name, &field_ctx);

            <#ty as shua_struct::BinaryField<#bit_order, _>>::build(
                &self.#name,
                &mut bits[offset..offset + field_len],
                &field_ctx
            ).map_err(#map_err)?;

            let mut l = field_len;
            #align
            offset += l;
        };

        if let Some(if_func) = &self.attrs.if_func {
            quote! { if self.#if_func() { #core_logic } }
        } else {
            core_logic
        }
    }

    fn gen_bit_len_stmt(&self) -> TokenStream2 {
        let name = self.name;
        let ty = self.ty;
        let bit_order = self.bit_order;

        let ctx = self.gen_ctx(&quote! { self });
        let align = self.gen_align_logic();

        let core_logic = quote! {
            let field_ctx = #ctx;
            let mut l = <#ty as shua_struct::BinaryField<#bit_order, _>>::bit_len(&self.#name, &field_ctx);
            #align
            total_len += l;
        };

        if let Some(if_func) = &self.attrs.if_func {
            quote! { if self.#if_func() { #core_logic } }
        } else {
            core_logic
        }
    }

    fn gen_error_variant(&self) -> TokenStream2 {
        let name = self.name;
        quote! {
            #name
        }
    }
}

#[proc_macro_derive(BinaryField, attributes(binary_struct, binary_field))]
pub fn binary_struct_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let vis = &input.vis;
    let attrs = match BinaryStructAttrs::from_derive_input(&input) {
        Ok(attrs) => attrs,
        Err(err) => return err.write_errors().into(),
    };

    let struct_name = &attrs.ident;
    let bit_order = attrs.bit_order;
    let error_enum_name = quote::format_ident!("{}Error", struct_name);

    let fields_named = match &input.data {
        Data::Struct(data) => {
            if let Fields::Named(fields) = &data.fields {
                fields.named.clone()
            } else {
                return syn::Error::new_spanned(
                    struct_name,
                    "BinaryField only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        }
        _ => {
            return syn::Error::new_spanned(struct_name, "BinaryField only works on structs")
                .to_compile_error()
                .into();
        }
    };

    let mut parse_stmts = Vec::new();
    let mut build_stmts = Vec::new();
    let mut bit_len_stmts = Vec::new();
    let mut error_variants = Vec::new();

    for (idx, field) in fields_named.iter().enumerate() {
        let attrs = match BinaryFieldAttrs::from_field(field) {
            Ok(attrs) => attrs,
            Err(err) => return err.write_errors().into(),
        };

        let generator = FieldGenerator {
            idx,
            name: field.ident.as_ref().unwrap(),
            ty: &field.ty,
            attrs,
            error_enum: &error_enum_name,
            bit_order: &bit_order,
        };

        error_variants.push(generator.gen_error_variant());
        parse_stmts.push(generator.gen_parse_stmt());
        build_stmts.push(generator.gen_build_stmt());
        bit_len_stmts.push(generator.gen_bit_len_stmt());
    }

    let custom_error: Type =
        syn::parse_str(&attrs.custom_error).expect("custom_error must be a valid Rust type");

    let expanded = quote! {
        #[derive(Debug, PartialEq, Eq)]
        #[allow(non_camel_case_types)]
        #vis enum #error_enum_name {
            #(#error_variants),*
        }

        #[allow(clippy::unused_unit)]
        impl<Ctx> shua_struct::BinaryField<#bit_order, Ctx> for #struct_name {
            #[cfg(debug_assertions)]
            type Error = shua_struct::BinaryError<#error_enum_name, &'static str, #custom_error>;

            #[cfg(not(debug_assertions))]
            type Error = shua_struct::BinaryError<#error_enum_name, usize, #custom_error>;

            #[inline]
            fn parse(
                bits: &shua_struct::BitSlice<u8, #bit_order>,
                _ctx: &Ctx,
            ) -> Result<Self, Self::Error> {
                let mut s = Self::default();
                let mut offset = 0;
                #(#parse_stmts)*
                Ok(s)
            }

            #[inline]
            fn build(
                &self,
                bits: &mut shua_struct::BitSlice<u8, #bit_order>,
                _ctx: &Ctx,
            ) -> Result<(), Self::Error> {
                let mut offset = 0;
                #(#build_stmts)*
                Ok(())
            }

            #[inline]
            fn bit_len(&self, _ctx: &Ctx) -> usize {
                let mut total_len = 0;
                #(#bit_len_stmts)*
                total_len
            }
        }
    };

    TokenStream::from(expanded)
}
