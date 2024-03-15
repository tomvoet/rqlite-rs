use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(FromRow)]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let syn::Data::Struct(data) = &input.data {
        if let syn::Fields::Named(fields) = &data.fields {
            let field_vals = fields.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_string = field_name.to_string();
                quote!(#field_name: row.get(#field_name_string)?)
            });

            let struct_name = &input.ident;

            return TokenStream::from(quote!(
                impl rqlite_rs::FromRow for #struct_name {
                    fn from_row(row: rqlite_rs::Row) -> anyhow::Result<Self> {
                        Ok(#struct_name {
                            #(#field_vals),*
                        })
                    }
                }
            ));
        }

        // TODO: Add support for unit and tuple structs
    }

    TokenStream::from(
        syn::Error::new(
            input.ident.span(),
            "Only structs with named fields are supported for `#[derive(FromRow)]`",
        )
        .to_compile_error(),
    )
}
