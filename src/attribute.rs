use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, ImplItem, ImplItemFn, ItemImpl, Signature};


fn signature_to_struct(signature: &Signature) -> TokenStream {
    let struct_name = signature.ident.clone();

    let t: TokenStream = signature.inputs.iter().filter_map(|arg|
        match arg {
            FnArg::Receiver(this) => { None }
            FnArg::Typed(pat_type) => {
                let id = &pat_type.pat;
                let ty = &pat_type.ty;
                Some(quote! {
                    #id: #ty,
                })
            }
        }
    ).collect();

    quote!{
        struct #struct_name {
            #t
        }
    }
}
fn handle_function_item(mut f: ImplItemFn) -> (TokenStream, TokenStream) {
    let f_name = &f.sig.ident;

    let t = signature_to_struct(&f.sig);

    let sig = &f.sig;
    (quote! {
        fn #sig {
            todo!()
        }
    }, quote!{
        struct #f_name {
            #t
        }
    })
}

pub fn entrypoint(input: ItemImpl) -> TokenStream {
    let new = input.clone();
    let new_struct_name = syn::Ident::new("AServer", proc_macro2::Span::call_site());

    let (functions, structs): (TokenStream, TokenStream) = new.items.into_iter().filter_map(|item| {
        match item {
            ImplItem::Fn(f) => {
                Some(handle_function_item(f))
            }
            _ => None
        }
    }).unzip();

    let tokens = quote! {
        #input

        struct #new_struct_name {
        }

        #structs

        impl #new_struct_name {
            #functions
        }
    };

    tokens
}