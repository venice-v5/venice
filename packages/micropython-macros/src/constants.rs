use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    Attribute, ExprMacro, ImplItemConst, Meta, Type,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

#[derive(Default)]
pub struct ConstantArgs {
    pub qstr_macro: Option<ExprMacro>,
}

impl Parse for ConstantArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let qstr_macro = input.parse::<ExprMacro>()?;
        if !qstr_macro.mac.path.is_ident("qstr") {
            return Err(syn::Error::new_spanned(
                &qstr_macro.mac.path,
                "expected macro call to `qstr`",
            ));
        }

        Ok(Self {
            qstr_macro: Some(qstr_macro),
        })
    }
}

pub fn generate_constant(
    ty: &Type,
    constant: ImplItemConst,
    attr: Attribute,
) -> syn::Result<TokenStream> {
    let args = match &attr.meta {
        Meta::Path(_) => ConstantArgs::default(),
        Meta::List(_) => attr.parse_args()?,
        Meta::NameValue(_) => return Err(syn::Error::new(attr.span(), "unsupported `=` format")),
    };

    let constructor = if let Type::Path(path) = &constant.ty
        && let Some(ident) = path.path.get_ident()
    {
        if ident == "i32" {
            Some(quote! { from_int })
        } else if ident == "f32" {
            Some(quote! { from_float })
        } else if ident == "bool" {
            Some(quote! { from_bool })
        } else {
            None
        }
    } else {
        None
    };

    let spanned_ty = quote_spanned! (constant.span()=> <#ty>);

    let constant_name = &constant.ident;
    let qstr = match &args.qstr_macro {
        Some(m) => {
            let mac = &m.mac;
            quote! { #mac }
        }
        None => {
            quote! { qstr!(#constant_name) }
        }
    };

    Ok(match constructor {
        Some(cons) => {
            quote! { #qstr => ::micropython_rs::obj::Obj::#cons(#spanned_ty::#constant_name), }
        }
        None => {
            quote_spanned! {constant.ty.span()=> #qstr => ::micropython_rs::obj::Obj::from_static(&<#ty>::#constant_name), }
        }
    })
}
