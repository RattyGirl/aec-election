extern crate proc_macro;
use proc_macro::{TokenStream};
use quote::quote;
use syn::{DeriveInput, Fields, parse_macro_input, Type};

#[proc_macro_derive(PostGresObj)]
pub fn derive_generate_postgres(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    if let syn::Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            let vals = fields.named.iter().map(|field| {
                let name = &field.ident;
                let ty = if let Type::Path(type_path) = &field.ty {
                    if type_path.path.segments.len() == 1 {
                        match type_path.path.segments.last().unwrap().ident.to_string().as_str() {
                            "i32" => "INTEGER",
                            "String" => "VARCHAR",
                            u => {
                                panic!("Unable to parse {} in object {}", input.ident, u);
                            }
                        }
                    } else {
                        "Unknown"
                    }
                } else {
                    "Unknown"
                };

                format!("{} {}", quote!(#name), ty.to_string())
            }).collect::<Vec<String>>().join(", ");
            let name = input.ident;

            return TokenStream::from(quote!(
            impl PostGresObj for #name {
                fn postgres_create(&self) -> String {
                        format!("CREATE TABLE {} ({});", stringify!(#name), #vals)
                }
                fn postgres_drop(&self) -> String {
                        format!("DROP TABLE {};", stringify!(#name))
                }
            }));
        }
    }
    TokenStream::from(
        syn::Error::new(input.ident.span(), "Only structs with named fields can derive `PostGresObj`").to_compile_error()
    )
}