// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

#[macro_use] extern crate quote;
extern crate proc_macro;
extern crate syn;

use quote::Ident;
use proc_macro::TokenStream;
use syn::{Lit, MetaItem};

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
        impl #impl_generics FromMessage for #name #ty_generics #where_clause {
            fn from_msg(msg: InMessage) -> Result<Self> {
                Ok(json::from_value(msg.into_inner())?)
            }
        }
    }
}

#[proc_macro_derive(IntoMessage, attributes(request))]
pub fn into_message(input: TokenStream) -> TokenStream {
    let ast = syn::parse_derive_input(&input.to_string()).unwrap();
    let gen = impl_into_message(&ast);
    gen.parse().unwrap()
}

fn impl_into_message(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let mut req_attr = None;

    for attr in &ast.attrs {
        match attr.value {
            MetaItem::NameValue(ref i, Lit::Str(ref v, _)) if i == "request" => req_attr = Some(v.to_string()),
            _ => (),
        }
    }

    let request = Ident::new(req_attr.expect("Missing `request` attribute"));

    quote! {
        impl #impl_generics IntoMessage for #name #ty_generics #where_clause {
            fn into_msg(self, handle: &Handle) -> Result<InMessage> {
                Request::#request(self).into_msg(handle)
            }
        }
    }
}
