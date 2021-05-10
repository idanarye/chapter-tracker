mod prop_sync_derive;
mod util;

#[proc_macro_derive(ProcSync, attributes(prop_sync))]
pub fn derive_prop_sync(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match prop_sync_derive::impl_prop_sync_derive(&input) {
        Ok(output) => output.into(),
        Err(error) => error.to_compile_error().into(),
    }
}
