#![warn(clippy::pedantic, clippy::all)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Type};

#[proc_macro_derive(FromRow)]
/// Derives the `FromRow` trait for a struct.
///
/// # Panics
///
/// This function will panic if the field name cannot be converted to a string.
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let syn::Data::Struct(data) = &input.data {
        if let syn::Fields::Named(fields) = &data.fields {
            let field_vals = fields.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_string = field_name.to_string();

                match &field.ty {
                    // If the field is an Option, we need to use `get_opt` instead of `get`
                    Type::Path(type_path)
                        if type_path.path.segments.last().unwrap().ident == "Option" =>
                    {
                        quote! {
                            #field_name: row.get_opt(#field_name_string).unwrap_or(None)
                        }
                    }
                    _ => quote! {
                        #field_name: row.get(#field_name_string)?
                    },
                }
            });

            let struct_name = &input.ident;

            return TokenStream::from(quote!(
                impl rqlite_rs::FromRow for #struct_name {
                    fn from_row(row: rqlite_rs::Row) -> Result<Self, rqlite_rs::IntoTypedError> {
                        Ok(#struct_name {
                            #(#field_vals),*
                        })
                    }
                }
            ));
        }

        if let syn::Fields::Unnamed(fields) = &data.fields {
            let field_vals = fields.unnamed.iter().enumerate().map(|(index, field)| {
                let index = syn::Index::from(index);

                match &field.ty {
                    Type::Path(type_path)
                        if type_path.path.segments.last().unwrap().ident == "Option" =>
                    {
                        quote! { row.get_by_index_opt(#index)? }
                    }
                    _ => quote! { row.get_by_index(#index)? },
                }
            });

            let struct_name = &input.ident;

            return TokenStream::from(quote!(
                impl rqlite_rs::FromRow for #struct_name {
                    fn from_row(row: rqlite_rs::Row) -> Result<Self, rqlite_rs::IntoTypedError> {
                        Ok(#struct_name(
                            #(#field_vals),*
                        ))
                    }
                }
            ));
        }
    }

    TokenStream::from(
        syn::Error::new(
            input.ident.span(),
            "Only structs with named fields are supported for `#[derive(FromRow)]`",
        )
        .to_compile_error(),
    )
}
