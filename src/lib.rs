mod logic;

extern crate proc_macro;
use proc_macro::{TokenStream};

use syn::{parse_macro_input, ItemImpl};
use quote::quote;

/// [1]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-mode-macros
#[proc_macro_derive(MyDerive)]
pub fn my_derive(_input: TokenStream) -> TokenStream {
    let tokens = quote! {
        struct Hello;
    };

    tokens.into()
}

/// This will generate a second impl for the
///
/// [1]: https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros
#[proc_macro_attribute]
pub fn actrix(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input: syn::ItemImpl = parse_macro_input!(input as ItemImpl);

    let stream = logic::actor(input);
    TokenStream::from(stream)
}
