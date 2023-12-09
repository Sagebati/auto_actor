use proc_macro2::TokenStream;
use quote::quote;
use syn::{self, ItemImpl, ReturnType, FnArg, Token};

pub fn actor(input: ItemImpl) -> TokenStream {
    let original_input = input.clone();
    let struct_name = &input.self_ty;
    let enum_variants = input
        .items
        .iter()
        .filter_map(|item| {
            let syn::ImplItem::Fn(method) = item else { return None; };

            // If the function is not a instance function we ignore
            if let Some(FnArg::Typed(_)) = method.sig.inputs.first() {
                return None;
            }

            // We remove the self from the method as is not needed in the protocol
            let method_inputs = &method.sig.inputs.iter()
                .filter(|x| matches!(x,FnArg::Typed(_)))
                .collect::<syn::punctuated::Punctuated<_, Token![,]>>();
            let method_output = &method.sig.output;
            let method_name = &method.sig.ident;

            // If there is a return type we send a sender channel
            let returns = match method_output {
                ReturnType::Default => None,
                ReturnType::Type(_, ty) => Some(quote! {
                    std::sync::mpsc::Sender<#ty>
                })
            };

            Some(quote! {
                #method_name(#method_inputs, #returns),
            })
        })
        .collect::<Vec<_>>();

    let prot_enum = quote! {
        pub enum AProt {
            #( #enum_variants )*
        }
    };

    let server_impl = quote! {
        impl #struct_name {
            fn event_loop(self) {
                while let Ok(message) = self.channel.recv() {
                    match message {
                        #( #struct_name::#enum_variants )*
                    }
                }
            }
        }
    };

    let result = quote! {
        #original_input

        #prot_enum

        /*
        struct AServer {
            inner: #struct_name,
            channel: std::sync::mpsc::Receiver<AProt>,
        }
        #server_impl

        #[derive(Clone)]
        struct AClient {
            channel: std::sync::mpsc::Sender<AProt>,
        }

        impl AClient {
            #( #enum_variants )*
        }
         */
    };

    result
}