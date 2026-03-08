use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    Attribute, ExprMacro, FnArg, Ident, LitInt, Meta, Signature, Token, Type,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

#[derive(Default)]
pub enum FunType {
    #[default]
    Fixed,
    FunVar {
        min_args: LitInt,
    },
    FunVarBetween {
        min_args: LitInt,
        max_args: LitInt,
    },
    FunVarKw {
        min_args: LitInt,
    },
}

#[derive(Default)]
pub struct MethodArgs {
    pub ty: FunType,
    pub qstr: Option<ExprMacro>,
}

impl Parse for MethodArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let mut ty = None;
        let mut qstr = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "name" => {
                    if qstr.replace(input.parse()?).is_some() {
                        return Err(syn::Error::new(ident.span(), "multiple `name` definitions"));
                    }
                }
                "ty" => {
                    if ty.replace(input.parse()?).is_some() {
                        return Err(syn::Error::new(ident.span(), "multiple `ty` definitions"));
                    }
                }
                _ => {
                    return Err(syn::Error::new(ident.span(), "unknown argument"));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(MethodArgs {
            ty: ty.unwrap_or_default(),
            qstr,
        })
    }
}

impl Parse for FunType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        match ident.to_string().as_str() {
            "var" => {
                let content;
                syn::parenthesized!(content in input);

                let arg: Ident = content.parse()?;
                if arg != "min" {
                    return Err(syn::Error::new(arg.span(), "expected `min`"));
                }

                content.parse::<Token![=]>()?;
                let min_args: LitInt = content.parse()?;

                Ok(FunType::FunVar { min_args })
            }

            "var_between" => {
                let content;
                syn::parenthesized!(content in input);

                let min_ident: Ident = content.parse()?;
                if min_ident != "min" {
                    return Err(syn::Error::new(min_ident.span(), "expected `min`"));
                }

                content.parse::<Token![=]>()?;
                let min_args: LitInt = content.parse()?;

                content.parse::<Token![,]>()?;

                let max_ident: Ident = content.parse()?;
                if max_ident != "max" {
                    return Err(syn::Error::new(max_ident.span(), "expected `max`"));
                }

                content.parse::<Token![=]>()?;
                let max_args: LitInt = content.parse()?;

                Ok(FunType::FunVarBetween { min_args, max_args })
            }

            "kw" => {
                let content;
                syn::parenthesized!(content in input);

                let arg: Ident = content.parse()?;
                if arg != "min" {
                    return Err(syn::Error::new(arg.span(), "expected `min`"));
                }

                content.parse::<Token![=]>()?;
                let min_args: LitInt = content.parse()?;

                Ok(FunType::FunVarKw { min_args })
            }

            _ => Err(syn::Error::new(ident.span(), "unknown function type")),
        }
    }
}

fn generate_fixed_fun(ty: &Type, sig: &Signature) -> syn::Result<TokenStream> {
    if sig.inputs.len() > 3 {
        return Err(syn::Error::new(
            sig.inputs.span(),
            "fixed functions can only accept up to 3 arguments",
        ));
    }

    let first_arg_def = if let Some(arg) = sig.inputs.get(0) {
        match arg {
            FnArg::Typed(v) => {
                let ty = &v.ty;
                quote! {
                    let a_argvalue = ::argparse::ArgValue::from_obj(&a);
                    if !<#ty as ::argparse::ArgTrait>::coercable(a_argvalue.ty()) {
                        ::micropython_rs::except::raise_type_error(
                            ::micropython_rs::init::token(),
                            ::argparse::error_msg!(
                                "expected <{}> for argument #1, found <{}>",
                                <#ty as ::argparse::ArgTrait>::ty(),
                                a_value.ty()
                            ),
                        );
                    }
                    let a_value = unsafe { <#ty as ::argparse::ArgTrait>::from_arg_value(a_argvalue).unwrap_unchecked() };
                }
            }
            FnArg::Receiver(v) => {
                if v.reference.is_none() || v.mutability.is_some() {
                    return Err(syn::Error::new(
                        v.span(),
                        "receiver argument can only be `&self`",
                    ));
                }

                quote! {
                    let a_value = unsafe { a.try_as_obj().unwrap_unchecked() };
                }
            }
        }
    } else {
        quote! {}
    };

    let is_method = matches!(sig.inputs.first(), Some(FnArg::Receiver(_)));

    let second_arg_def = if let Some(arg) = sig.inputs.get(1) {
        let FnArg::Typed(v) = arg else { unreachable!() };
        let ty = &v.ty;
        let arg_number = if is_method {
            quote! { "1" }
        } else {
            quote! { "2" }
        };
        quote! {
            let b_argvalue = ::argparse::ArgValue::from_obj(&b);
            if !<#ty as ::argparse::ArgTrait>::coercable(b_argvalue.ty()) {
                ::micropython_rs::except::raise_type_error(
                    ::micropython_rs::init::token(),
                    ::argparse::error_msg!(
                        "expected <{}> for argument #{}, found <{}>",
                        <#ty as ::argparse::ArgTrait>::ty(),
                        #arg_number,
                        b_argvalue.ty()
                    ),
                );
            }
            let b_value = unsafe { <#ty as ::argparse::ArgTrait>::from_arg_value(b_argvalue).unwrap_unchecked() };
        }
    } else {
        quote! {}
    };

    let third_arg_def = if let Some(arg) = sig.inputs.get(2) {
        let FnArg::Typed(v) = arg else { unreachable!() };
        let ty = &v.ty;
        let arg_number = if is_method {
            quote! { "2" }
        } else {
            quote! { "3" }
        };
        quote! {
            let c_argvalue = ::argparse::ArgValue::from_obj(&c);
            if !<#ty as ::argparse::ArgTrait>::coercable(c_argvalue.ty()) {
                ::micropython_rs::except::raise_type_error(
                    ::micropython_rs::init::token(),
                    ::argparse::error_msg!(
                        "expected <{}> for argument #{}, found <{}>",
                        <#ty as ::argparse::ArgTrait>::ty(),
                        #arg_number,
                        c_argvalue.ty()
                    ),
                );
            }
            let c_value = unsafe { <#ty as ::argparse::ArgTrait>::from_arg_value(c_argvalue).unwrap_unchecked() };
        }
    } else {
        quote! {}
    };

    let fn_name = &sig.ident;
    let (args, call_args, fun_ty) = match sig.inputs.len() {
        0 => (quote! {}, quote! {}, quote! { Fun0 }),
        1 => (
            quote! { a: ::micropython_rs::obj::Obj },
            quote! { a_value },
            quote! { Fun1 },
        ),
        2 => (
            quote! { a: ::micropython_rs::obj::Obj, b: ::micropython_rs::obj::Obj },
            quote! { a_value, b_value },
            quote! { Fun2 },
        ),
        3 => (
            quote! { a: ::micropython_rs::obj::Obj, b: ::micropython_rs::obj::Obj, c: ::micropython_rs::obj::Obj },
            quote! { a_value, b_value, c_value },
            quote! { Fun3 },
        ),
        _ => unreachable!(),
    };

    Ok(quote! {{
        extern "C" fn trampoline(#args) -> ::micropython_rs::obj::Obj {
            #first_arg_def
            #second_arg_def
            #third_arg_def

            #ty::#fn_name(#call_args).into()
        }

        ::micropython_rs::fun::#fun_ty::new(trampoline)
    }})
}

fn generate_var_fun(ty: &Type, sig: &Signature, fun_type: FunType) -> syn::Result<TokenStream> {
    let spanned_ty = quote_spanned! (sig.span()=> <#ty>);

    let f_ty = match fun_type {
        FunType::FunVar { .. } | FunType::FunVarBetween { .. } => quote! {
            for<'a> Fn(&'a [::micropython_rs::obj::Obj]) -> R
        },
        FunType::FunVarKw { .. } => quote! {
            for<'a> Fn(&'a [::micropython_rs::obj::Obj], &'a ::micropython_rs::map::Map) -> R
        },
        _ => unreachable!(),
    };

    let fn_name = &sig.ident;

    let (map_arg, map_def, map_call_arg) = if let FunType::FunVarKw { .. } = fun_type {
        (
            quote! { map: *mut ::micropython_rs::map::Map },
            quote! {
                let map = unsafe { &*map };
            },
            quote! { map },
        )
    } else {
        (quote! {}, quote! {}, quote! {})
    };

    let trampoline = quote_spanned! {sig.span()=>
        unsafe fn trampoline_inner<F, R>(f: F, n_args: usize, ptr: *const ::micropython_rs::obj::Obj, #map_arg) -> ::micropython_rs::obj::Obj
        where
            F: #f_ty,
            R: Into<::micropython_rs::obj::Obj>,
        {
            let args = unsafe { ::std::slice::from_raw_parts(ptr, n_args) };
            #map_def
            f(args, #map_call_arg).into()
        }

        unsafe extern "C" fn trampoline(n_args: usize, ptr: *const ::micropython_rs::obj::Obj, #map_arg) -> ::micropython_rs::obj::Obj {
            unsafe { trampoline_inner(#spanned_ty::#fn_name, n_args, ptr, #map_call_arg) }
        }
    };

    let make_ret = match fun_type {
        FunType::FunVar { min_args } => quote! {
            FunVar::new(trampoline, #min_args)
        },
        FunType::FunVarBetween { min_args, max_args } => quote! {
            FunVarBetween::new(trampoline, #min_args, #max_args)
        },
        FunType::FunVarKw { min_args } => quote! {
            FunVarKw::new(trampoline, #min_args)
        },
        _ => unreachable!(),
    };

    Ok(quote! {{
        #trampoline

        ::micropython_rs::fun::#make_ret
    }})
}

pub fn generate_fun(ty: &Type, sig: &Signature, attr: &Attribute) -> syn::Result<TokenStream> {
    let args = match &attr.meta {
        Meta::Path(_) => MethodArgs::default(),
        Meta::List(_) => attr.parse_args()?,
        Meta::NameValue(_) => return Err(syn::Error::new(attr.span(), "unsupported `=` format")),
    };

    let fun_expr = match args.ty {
        FunType::Fixed => generate_fixed_fun(ty, sig)?,
        _ => generate_var_fun(ty, sig, args.ty)?,
    };
    let attr_qstr = match &args.qstr {
        Some(m) => {
            let mac = &m.mac;
            quote! { #mac }
        }
        None => {
            let fn_ident = &sig.ident;
            quote! { qstr!(#fn_ident) }
        }
    };

    Ok(quote! {
        #attr_qstr => ::micropython_rs::obj::Obj::from_static(&#fun_expr),
    })
}
