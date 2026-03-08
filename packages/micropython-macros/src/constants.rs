use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    Attribute, ExprMacro, ImplItemConst, Type,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

pub struct ConstantArgs {
    pub qstr_macro: ExprMacro,
}

impl Parse for ConstantArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let qstr_macro = input.parse::<ExprMacro>()?;
        if !qstr_macro.mac.path.is_ident("qstr") {
            return Err(syn::Error::new_spanned(
                &qstr_macro.mac.path,
                "expected macro call to `qstr`",
            ));
        }

        Ok(Self { qstr_macro })
    }
}

pub fn generate_constant(
    ty: &Type,
    constant: ImplItemConst,
    attr: Attribute,
) -> syn::Result<TokenStream> {
    let args = attr.parse_args::<ConstantArgs>()?;

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

    let qstr = &args.qstr_macro;
    let spanned_ty = quote_spanned! (constant.span()=> <#ty>);
    let constant_name = &constant.ident;
    Ok(match constructor {
        Some(cons) => {
            quote! { #qstr => ::micropython_rs::obj::Obj::#cons(#spanned_ty::#constant_name), }
        }
        None => {
            quote_spanned! {constant.ty.span()=> #qstr => ::micropython_rs::obj::Obj::from_static(&<#ty>::#constant_name), }
        }
    })
}
