use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    Attribute, ExprMacro, FnArg, Ident, LitInt, LitStr, Meta, Signature, Token, Type,
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
pub enum Binding {
    Static,
    Class,
    #[default]
    Instance,
}

#[derive(Default)]
pub struct MethodArgs {
    pub ty: FunType,
    pub qstr: Option<ExprMacro>,
    pub binding: Binding,
}

impl Parse for MethodArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let mut ty = None;
        let mut qstr = None;
        let mut binding = None;

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
                "binding" => {
                    if binding.replace(input.parse()?).is_some() {
                        return Err(syn::Error::new(
                            ident.span(),
                            "multiple `binding` definitions",
                        ));
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
            binding: binding.unwrap_or_default(),
            qstr,
        })
    }
}

impl Parse for Binding {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let str = input.parse::<LitStr>()?;
        match str.value().as_str() {
            "static" => Ok(Self::Static),
            "class" => Ok(Self::Class),
            "instance" => Ok(Self::Instance),
            _ => Err(syn::Error::new(str.span(), "unknown binding")),
        }
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
                    let a_parser = <#ty as ::argparse::DefaultParser>::Parser::default();
                    let a_value = match ::argparse::ArgParser::parse(&a_parser, &a) {
                        ::std::result::Result::Ok(v) => v,
                        ::std::result::Result::Err(e) => ::micropython_rs::except::Exception::from(match e {
                            ::argparse::ParseError::TypeError { expected } => ::argparse::PositionalError::TypeError { n: 1, expected, found: ::argparse::type_name(&a) },
                            ::argparse::ParseError::ValueError { mk_msg } => ::argparse::PositionalError::ValueError { msg: mk_msg("argument #1") },
                        }).raise(::micropython_rs::init::token()),
                    };
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
            quote! { 1 }
        } else {
            quote! { 2 }
        };
        quote! {
            let b_parser = <#ty as ::argparse::DefaultParser>::Parser::default();
            let b_value = match ::argparse::ArgParser::parse(&b_parser, &b) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => ::micropython_rs::except::Exception::from(match e {
                    ::argparse::ParseError::TypeError { expected } => ::argparse::PositionalError::TypeError { n: #arg_number, expected, found: ::argparse::type_name(&b) },
                    ::argparse::ParseError::ValueError { mk_msg } => ::argparse::PositionalError::ValueError { msg: mk_msg(&format!("argument #{}", #arg_number)) },
                }).raise(::micropython_rs::init::token()),
            };
        }
    } else {
        quote! {}
    };

    let third_arg_def = if let Some(arg) = sig.inputs.get(2) {
        let FnArg::Typed(v) = arg else { unreachable!() };
        let ty = &v.ty;
        let arg_number = if is_method {
            quote! { 2 }
        } else {
            quote! { 3 }
        };
        quote! {
            let c_parser = <#ty as ::argparse::DefaultParser>::Parser::default();
            let c_value = match ::argparse::ArgParser::parse(&c_parser, &c) {
                ::std::result::Result::Ok(v) => v,
                ::std::result::Result::Err(e) => ::micropython_rs::except::Exception::from(match e {
                    ::argparse::ParseError::TypeError { expected } => ::argparse::PositionalError::TypeError { n: #arg_number, expected, found: ::argparse::type_name(&c) },
                    ::argparse::ParseError::ValueError { mk_msg } => ::argparse::PositionalError::ValueError { msg: mk_msg(&format!("argument #{}", #arg_number)) },
                }).raise(::micropython_rs::init::token()),
            };
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

    let mut fun_expr = match args.ty {
        FunType::Fixed => generate_fixed_fun(ty, sig)?,
        _ => generate_var_fun(ty, sig, args.ty)?,
    };
    match args.binding {
        Binding::Instance => {}
        Binding::Static => {
            fun_expr = quote! { ::micropython_rs::fun::StaticMethod::new(&#fun_expr) }
        }
        Binding::Class => fun_expr = quote! { ::micropython_rs::fun::ClassMethod::new(&#fun_expr) },
    }

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
