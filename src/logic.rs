use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{self, ItemImpl, ReturnType, FnArg, Token, ImplItemFn, ImplItem};
use syn::punctuated::Punctuated;
use unzip_n::unzip_n;

unzip_n!(pub 4);
fn param_names(method: &ImplItemFn) -> TokenStream {
    let method_inputs = &method.sig.inputs.iter()
        .filter_map(|x| if let FnArg::Typed(a) = x { Some(&a.pat) } else { None })
        .collect::<Punctuated<_, Token![,]>>();

    quote!( #method_inputs )
}

fn enum_variant(method: &ImplItemFn) -> (TokenStream, TokenStream) {
    // We remove the self from the method as is not needed in the protocol
    let method_output = &method.sig.output;
    let method_name = &method.sig.ident;

    let method_params = &method.sig.inputs.iter()
        .filter_map(|x| if let FnArg::Typed(a) = x { Some(&a.ty) } else { None })
        .collect::<Vec<_>>();

    // If there is a return type we send a sender channel
    let returns = match method_output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(quote! {
                    flume::Sender<#ty>
                })
    };

    (quote! { #method_name }, quote! { #(#method_params),* #returns })
}

fn server_impls(input: &ImplItemFn, enum_type: &Ident) -> TokenStream {
    let function_name = &input.sig.ident;
    let inputs = param_names(input);
    let mut needs_return = None;
    let mut return_binding = None;
    let res = Ident::new("response", Span::call_site());
    if let ReturnType::Type(_, _) = &input.sig.output {
        return_binding = Some(Ident::new("return_channel", Span::call_site()));
        needs_return = Some(quote!(
            #return_binding.send(#res).unwrap();
        ));
    }

    let stream = quote! {
        #enum_type::#function_name(#inputs #return_binding) => {
            let #res = self.inner.#function_name(#inputs);
            #needs_return
        }
    };

    stream
}

fn client_impls(method: &ImplItemFn, enum_name: &Ident) -> TokenStream {
    let method_params = method.sig.inputs.iter()
        .filter(|x| matches!(x, FnArg::Typed(_)))
        .collect::<Punctuated<_, Token![,]>>();

    let function_name = &method.sig.ident;
    let inputs = param_names(method);
    let mut needs_return = None;
    let mut return_binding = None;
    let mut init_channel = None;
    let mut return_type = None;
    if let ReturnType::Type(_, rtype) = &method.sig.output {
        init_channel = Some(quote! {
           let (sender, recv) = flume::unbounded();
        });

        return_binding = Some(quote!(sender));

        needs_return = Some(quote!(
            recv.recv().unwrap()
        ));

        return_type = Some(quote! {
            -> #rtype
        });
    }

    let stream = quote! {
        pub fn #function_name(&self, #method_params) #return_type {
            #init_channel
            let message = #enum_name::#function_name(#inputs #return_binding);
            self.channel.send(message);
            #needs_return
        }
    };

    stream
}

pub fn actor(input: ItemImpl) -> TokenStream {
    let original_input = input.clone();
    let struct_type = &input.self_ty;
    let enum_type = Ident::new("AProt", Span::call_site());
    let server_type = Ident::new("AServer", Span::call_site());
    let client_type = Ident::new("AClient", Span::call_site());
    let (enum_variant_names, enum_variant_types, server_impls, client_impls): (Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>) = input
        .items
        .iter()
        .filter_map(|item| {
            let ImplItem::Fn(method) = item else { return None; };

            // If the function is not a instance function we ignore
            if let Some(FnArg::Typed(_)) = method.sig.inputs.first() {
                return None;
            }

            let (enum_variant_names, enum_variant_params) = enum_variant(method);
            let client_impls = client_impls(method, &enum_type);
            Some((enum_variant_names, enum_variant_params, server_impls(method, &enum_type), client_impls))
        }).unzip_n_vec();

    let enum_decl = quote! {
        enum #enum_type {
            #( #enum_variant_names(#enum_variant_types) ),*
        }
    };

    let server_impl = quote! {
        impl #server_type {
            pub fn event_loop(mut self) {
                while let Ok(message) = self.channel.recv() {
                    match message {
                        #( #server_impls )*
                    }
                }
            }
        }
    };

    let client_impl = quote! {
        impl #client_type {
            #( #client_impls )*
        }
    };

    quote! {
        #original_input

        pub mod actor {
            use super::*;

            #[allow(non_camel_case_types)]
            #enum_decl

            #[derive(Debug)]
            pub struct #server_type {
                inner: #struct_type,
                channel: flume::Receiver<#enum_type>,
            }

            #server_impl


            #[derive(Clone, Debug)]
            pub struct #client_type {
                channel: flume::Sender<#enum_type>,
            }

            #client_impl

            pub fn new_server_client(inner: #struct_type) -> (#client_type, #server_type) {
                let (sender, receiver) = flume::unbounded();
                (#client_type {
                    channel: sender
                },
                #server_type {
                    inner,
                    channel: receiver
                })
            }
        }
    }
}