mod constants;
mod methods;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    ExprMacro, ImplItem, ImplItemConst, ItemImpl, ItemStruct, Signature,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
};

use crate::{constants::generate_constant, methods::generate_fun};

struct ClassArgs {
    pub qstr_macro: ExprMacro,
}

impl Parse for ClassArgs {
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

#[proc_macro_attribute]
pub fn class(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ClassArgs);
    let qstr_macro = &args.qstr_macro;

    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;

    // obj must have static lifetime, meaning no generic lifetimes
    if input.generics.lifetimes().next().is_some() {
        return syn::Error::new(input.generics.span(), "type must have static lifetime")
            .into_compile_error()
            .into();
    }

    let first_field = match input.fields.iter().next() {
        Some(v) => v,
        None => {
            return syn::Error::new(input.fields.span(), "first field must be an `ObjBase`")
                .into_compile_error()
                .into();
        }
    };

    let first_field_accessor = match &first_field.ident {
        Some(i) => quote! { &s.#i },
        None => quote! { &s.0 },
    };

    quote! {
        #[repr(C)]
        #input

        // ObjBase identity check
        // (check that the first field is an ObjBase)
        const _: () = {
            fn assert_type(_: &::micropython_rs::obj::ObjBase) {}
            fn validate(s: &#struct_name) {
                assert_type(#first_field_accessor);
            }
        };

        unsafe impl ::micropython_rs::obj::ObjTrait for #struct_name {
            const OBJ_TYPE: &::micropython_rs::obj::ObjType = {
                static TY: ::micropython_rs::obj::ObjFullType = {
                    let ty = ::micropython_rs::obj::ObjFullType::new(::micropython_rs::obj::TypeFlags::empty(), #qstr_macro);

                    macro_rules! set_slot {
                        ($ty:expr, $slot:ident, $setter:ident) => {{
                            if let Some(v) = <#struct_name as ::micropython_rs::class::Class>::$slot {
                                $ty.$setter(v)
                            } else {
                                $ty
                            }
                        }};
                    }

                    let ty = set_slot!(ty, MAKE_NEW, set_make_new);
                    let ty = set_slot!(ty, PARENT, set_parent);
                    let ty = set_slot!(ty, LOCALS_DICT, set_locals_dict);
                    let ty = set_slot!(ty, ATTR, set_attr);
                    let ty = set_slot!(ty, ITER, set_iter);
                    let ty = set_slot!(ty, STREAM, set_stream);
                    let ty = set_slot!(ty, SUBSCR, set_subscr);
                    let ty = set_slot!(ty, UNARY_OP, set_unary_op_raw);
                    let ty = set_slot!(ty, BINARY_OP, set_binary_op_raw);

                    ty
                };

                TY.as_obj_type()
            };
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn class_methods(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);
    let ty = &input.self_ty;

    let mut methods = Vec::new();
    let mut constants = Vec::new();
    let mut make_new = None;
    let mut parent = None;
    let mut iter = None;
    let mut attr = None;
    let mut subscr = None;
    let mut stream = None;
    let mut unary_op = None;
    let mut binary_op = None;

    let replace_err = |span, attr_name| {
        syn::Error::new(span, format!("multiple `{attr_name}` functions"))
            .into_compile_error()
            .into()
    };

    for item in input.items.iter_mut() {
        match item {
            ImplItem::Const(c) => {
                let mut attr_idx = None;
                for (idx, a) in c.attrs.iter().enumerate() {
                    let Some(i) = a.path().get_ident() else {
                        continue;
                    };
                    match i.to_string().as_str() {
                        "constant" => {
                            constants.push((c.clone(), a.clone()));
                        }
                        "stream" => {
                            let span = a.span();
                            if stream.replace(c.clone()).is_some() {
                                return replace_err(span, "stream");
                            }
                        }
                        "parent" => {
                            let span = a.span();
                            if parent.replace(c.clone()).is_some() {
                                return replace_err(span, "stream");
                            }
                        }
                        _ => continue,
                    }
                    attr_idx = Some(idx);
                    break;
                }

                if let Some(idx) = attr_idx {
                    c.attrs.remove(idx);
                }
            }
            ImplItem::Fn(f) => {
                let mut attr_idx = None;
                let mut opt = None;
                let mut method_attr = None;
                for (idx, a) in f.attrs.iter().enumerate() {
                    let Some(i) = a.path().get_ident() else {
                        continue;
                    };
                    match i.to_string().as_str() {
                        "method" => {
                            method_attr = Some(a.clone());
                        }
                        "make_new" => {
                            opt = Some(("make_new", a.span(), &mut make_new));
                        }
                        "iter" => {
                            opt = Some(("iter", a.span(), &mut iter));
                        }
                        "attr" => {
                            opt = Some(("attr", a.span(), &mut attr));
                        }
                        "subscr" => {
                            opt = Some(("subscr", a.span(), &mut subscr));
                        }
                        "unary_op" => {
                            opt = Some(("unary_op", a.span(), &mut unary_op));
                        }
                        "binary_op" => {
                            opt = Some(("binary_op", a.span(), &mut binary_op));
                        }
                        _ => continue,
                    }
                    attr_idx = Some(idx);
                    break;
                }
                if let Some(idx) = attr_idx {
                    f.attrs.remove(idx);
                }
                if let Some((name, span, opt)) = opt {
                    if opt.replace(f.sig.clone()).is_some() {
                        return replace_err(span, name);
                    }
                } else if let Some(method_attr) = method_attr {
                    methods.push((f.sig.clone(), method_attr));
                }
            }
            _ => {}
        }
    }

    let none_tokens = quote! { None };

    let map_item = |span, ident| {
        // angle brackets around ty are necessary to prevent span contamination
        // without them the tokens span would start at ty.span()
        quote_spanned!(span=> <#ty>::#ident)
    };

    let map_fn_item = |sig: Signature| map_item(sig.span(), sig.ident);
    let map_const_item = |c: ImplItemConst| map_item(c.ty.span(), c.ident);

    let parent_tokens = parent
        .map(map_const_item)
        .map(|c| {
            quote! { Some(#c) }
        })
        .unwrap_or_else(|| none_tokens.clone());

    let make_new_tokens = make_new
        .map(map_fn_item)
        .map(|f| quote! {Some(::micropython_rs::make_new_from_fn!(#f))})
        .unwrap_or_else(|| none_tokens.clone());

    let iter_tokens = iter
        .map(map_fn_item)
        .map(|f| quote! { Some(::micropython_rs::obj::Iter::IterNext(#f)) })
        .unwrap_or_else(|| none_tokens.clone());

    let attr_tokens = attr
        .map(map_fn_item)
        .map(|f| quote! { Some(::micropython_rs::attr_from_fn!(#f)) })
        .unwrap_or_else(|| none_tokens.clone());

    let subscr_tokens = subscr
        .map(map_fn_item)
        .map(|f| quote! { Some(::micropython_rs::subscr_from_fn!(#f)) })
        .unwrap_or_else(|| none_tokens.clone());

    let stream_tokens = stream
        .map(map_const_item)
        .map(|c| quote! { Some(&#c) })
        .unwrap_or_else(|| none_tokens.clone());

    let unary_op_tokens = unary_op
        .map(map_fn_item)
        .map(|f| quote! { Some(#f) })
        .unwrap_or_else(|| none_tokens.clone());

    let binary_op_tokens = binary_op
        .map(map_fn_item)
        .map(|f| quote! { Some(#f) })
        .unwrap_or(none_tokens);

    let method_tokens = match methods
        .into_iter()
        .map(|(sig, attr)| generate_fun(&ty, &sig, &attr))
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(tokens) => tokens,
        Err(e) => return e.into_compile_error().into(),
    };

    let constant_tokens = match constants
        .into_iter()
        .map(|(constant, attr)| generate_constant(&ty, constant, attr))
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(tokens) => tokens,
        Err(e) => return e.into_compile_error().into(),
    };

    quote! {
        #input

        impl ::micropython_rs::class::Class for #ty {
            const PARENT: Option<&::micropython_rs::obj::ObjType> = #parent_tokens;
            const MAKE_NEW: Option<::micropython_rs::obj::MakeNew> = #make_new_tokens;
            const ITER: Option<::micropython_rs::obj::Iter> = #iter_tokens;
            const ATTR: Option<::micropython_rs::obj::Attr> = #attr_tokens;
            const SUBSCR: Option<::micropython_rs::obj::Subscr> = #subscr_tokens;
            const STREAM: Option<&::micropython_rs::stream::Stream> = #stream_tokens;
            const UNARY_OP: Option<::micropython_rs::obj::UnaryOpFn> = #unary_op_tokens;
            const BINARY_OP: Option<::micropython_rs::obj::BinaryOpFn> = #binary_op_tokens;

            const LOCALS_DICT: Option<&::micropython_rs::map::Dict> = Some(::micropython_rs::const_dict![
                #(#method_tokens)*
                #(#constant_tokens)*
            ]);
        }
    }
    .into()
}

// macro_rules! dummy_macro {
//     ($name:ident) => {
//         #[proc_macro_attribute]
//         pub fn $name(_: TokenStream, item: TokenStream) -> TokenStream {
//             item
//         }
//     };
// }

// dummy_macro!(method);
// dummy_macro!(constant);
// dummy_macro!(make_new);
// dummy_macro!(parent);
// dummy_macro!(iter);
// dummy_macro!(attr);
// dummy_macro!(subscr);
// dummy_macro!(stream);
// dummy_macro!(unary_op);
// dummy_macro!(binary_op);
