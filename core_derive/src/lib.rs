// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

#[macro_use] extern crate nom;
#[macro_use] extern crate quote;
extern crate proc_macro;
extern crate syn;

use nom::{anychar, IResult};
use proc_macro::TokenStream;
use quote::Ident;
use syn::{Body, Lit, MetaItem, VariantData};

fn is_uppercase(a: u8) -> bool { (a as char).is_uppercase() }
named!(char_and_more_char<()>, do_parse!(
    anychar >>
    take_till!(is_uppercase) >>
    ()
));
named!(camel_case<(&str)>, map_res!(recognize!(char_and_more_char), std::str::from_utf8));
named!(p_camel_case<&[u8], Vec<&str>>, many0!(camel_case));

#[proc_macro_derive(FromMessage)]
pub fn from_message(input: TokenStream) -> TokenStream {
    let ast = syn::parse_derive_input(&input.to_string()).unwrap();
    let gen = impl_from_message(&ast);
    gen.parse().unwrap()
}

fn impl_from_message(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    quote! {
        impl #impl_generics ::message::FromMessage for #name #ty_generics #where_clause {
            fn from_msg(msg: ::message::InMessage) -> ::errors::Result<Self> {
                Ok(::serde_json::from_value(msg.into_inner())?)
            }
        }
    }
}

#[proc_macro_derive(IntoMessage)]
pub fn into_message(input: TokenStream) -> TokenStream {
    let ast = syn::parse_derive_input(&input.to_string()).unwrap();
    let gen = impl_into_message(&ast);
    gen.parse().unwrap()
}

fn impl_into_message(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    quote! {
        impl #impl_generics ::message::IntoMessage for #name #ty_generics #where_clause {
            fn into_msg(self, handle: &::tokio_core::reactor::Handle) -> ::errors::Result<::message::InMessage> {
                ::request::Request::#name(self).into_msg(handle)
            }
        }
    }
}

#[proc_macro_derive(Executable, attributes(response, future, hostarg))]
pub fn executable(input: TokenStream) -> TokenStream {
    let ast = syn::parse_derive_input(&input.to_string()).unwrap();
    let gen = impl_executable(ast);
    gen.parse().unwrap()
}

fn impl_executable(ast: syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // Break struct name into provider name and function name components
    let (provider, func) = match p_camel_case(name.as_ref().as_bytes()) {
        IResult::Done(_, slice) => (Ident::new(slice[0].to_lowercase()), Ident::new(slice[1].to_lowercase())),
        _ => panic!("Struct name does not match ProviderFn pattern"),
    };

    // Set args for method call
    let args = match ast.body {
        Body::Struct(data) => match data {
            VariantData::Struct(fields) => fields.into_iter().map(|f| f.ident.unwrap()).collect(),
            VariantData::Tuple(_) => panic!("Tuple structs are currently unsupported"),
            VariantData::Unit => Vec::new(),
        },
        _ => panic!("Only structs are currently supported"),
    };

    // Get attributes
    let mut response = None;
    let mut future = None;
    let mut hostarg = vec![syn::Ident::new("")];
    for attr in &ast.attrs {
        match attr.value {
            MetaItem::NameValue(ref i, Lit::Str(ref v, _)) if i == "hostarg" && v == "true" => hostarg.insert(0, syn::Ident::new("host")),
            MetaItem::NameValue(ref i, Lit::Str(ref v, _)) if i == "response" => response = Some(Ident::new(v.to_string())),
            MetaItem::NameValue(ref i, Lit::Str(ref v, _)) if i == "future" => future = Some(Ident::new(v.to_string())),
            _ => (),
        }
    }
    let response = response.expect("Missing attribute `response`");
    let future = future.unwrap_or(Ident::new("Box<::futures::Future<Item = Self::Response, Error = ::errors::Error>>"));

    quote! {
        impl #impl_generics ::request::Executable for #name #ty_generics #where_clause {
            type Response = #response;
            type Future = #future;

            fn exec(self, host: &::host::local::Local) -> Self::Future {
                host.#provider().#func(#(#hostarg),* #(&self.#args),*)
            }
        }
    }
}
