use quote::quote;
use syn::{DeriveInput, Fields, parse_macro_input, Type, Data, parenthesized, LitInt, Expr, Token, Ident, Lit, FieldsNamed, parse_str, Field, Attribute, PathArguments, AngleBracketedGenericArguments, ExprMethodCall, parse_quote};
use syn::__private::{Span, TokenStream};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Comma, Token};

struct DBArgs {
    values: Punctuated<Expr, Token![,]>
}

impl Parse for DBArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            values: input.parse_terminated(Expr::parse, Token![,])?
        })
    }
}

fn get_attr_value(attrs: &Vec<Attribute>, search_term: &str) -> Option<Expr>{
    let db_args = attrs.iter()
        .filter(|attr| attr.path().is_ident("db")).collect::<Vec<&Attribute>>().first()? //existence of db
        .parse_args::<DBArgs>().ok()?.values; //get arg values
    for arg in db_args {
        if let Expr::Assign(assignment) = arg {
            if let Expr::Path(expr_path) = *assignment.left {
                if let Some(ident) = expr_path.path.get_ident() {
                    if ident.eq(search_term) {
                        return Some(*assignment.right);
                    }
                }
            }
        }
    }
    None
}

fn get_attr_str(attrs: &Vec<Attribute>, search_term: &str) -> Option<String> {
    if let Some(Expr::Lit(expr_lit)) = get_attr_value(attrs, search_term){
        if let Lit::Str(lit_str) = expr_lit.lit {
            Some(lit_str.value())
        } else {
            None
        }
    } else {
        None
    }
}

fn determine_table_name(input: &DeriveInput) -> String {
    let attrs = input.attrs.clone();
    get_attr_str(&attrs, "table_name").unwrap_or(input.ident.to_string())
}

fn determine_table_columns(fields: &FieldsNamed) -> Vec<String> {
    fields.named.iter().map(|field| {
        field.ident.clone().unwrap().to_string()
    }).collect::<Vec<String>>()
    //TODO read if column different

    // let vals: String = fields.named.iter().map(|ref field| {
    //     // field.attrs.iter().map(|attr| attr.parse_args())
    //     // field.attrs.iter().map(|attr| attr.path().get_ident().and_then(|ident| Some(ident.to_string())).unwrap_or("UNKNOWN".to_string()))
    //     //     .collect::<Vec<String>>().join(",")
    //     match &field.ty {
    //         _ => "NYI",
    //         Type::Path(typepath) => "typepath",
    //     }
    // }).collect::<Vec<&str>>().join("|\n");
}

struct TableField {
    ///name of the field in the struct
    column: String,
    ///Name of the column
    value_expr: Expr
}

fn determine_fields(fields: &FieldsNamed) -> Vec<TableField> {
    fields.named.iter().filter_map(|field| {
        Some(TableField {
            column: determine_column_name(field),
            value_expr: determine_value_expr(&field)
        })
    }).collect()
}

fn determine_value_expr(field: &Field) -> Expr {
    let null_value: Option<Expr> = get_attr_value(&field.attrs, "null_value");
    let option = if let Type::Path(path) = &field.ty {
        if let Some(path_seg) = path.path.segments.last() {
            if path_seg.ident == "Option" {
                true
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    if(!option) {
        parse_str::<Expr>(format!("&self.{}", field.ident.clone().unwrap()).as_str()).unwrap()
    } else {
        let ident = field.ident.clone().unwrap();
        let self_expr: Expr = syn::parse_quote! { &self };
        let method_ident: Ident = Ident::new("unwrap_or", Span::call_site());
        let mut args: Punctuated<Expr, Comma> = Punctuated::new();
        args.push(null_value.unwrap());
        let field_access: Expr = parse_quote!(#self_expr.#ident.clone());
        let method_call = Expr::MethodCall(ExprMethodCall {
            attrs: vec![],
            receiver: Box::new(field_access),
            method: method_ident,
            args,
            dot_token: Default::default(),
            turbofish: None,
            paren_token: Default::default(),
        });
        method_call
    }

    // match option {
    //     true => quote!(&self.#ident.unwrap_or(#null_value)),
    //     false => Expr::MethodCall(ExprMethodCall {
    //         attrs: vec![],
    //         receiver: Box::new(),
    //         dot_token: Default::default(),
    //         method: (),
    //         turbofish: None,
    //         paren_token: Default::default(),
    //         args: Default::default(),
    //     }),
    //     // false => parse_str::<Expr>(format!("&self.{}", field.ident.clone().unwrap()).as_str()).unwrap()
    // }

}

fn determine_column_name(field: &Field) -> String {
    if let Some(column) = get_attr_str(&field.attrs, "column_name") {
        column
    } else {
        field.ident.clone().unwrap().to_string()
    }
}


#[proc_macro_derive(SerialiseDB, attributes(db))]
pub fn derive_generate_election_xml(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    if let syn::Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            let table_name = determine_table_name(&input);
            let name = input.ident;
            let table_fields = determine_fields(&fields);
            let values_expr: Punctuated<Expr, Token![,]> = table_fields.iter().map(|field| field.value_expr.clone()).collect();
            let table_columns: Vec<String> = table_fields.iter().map(|field| field.column.clone()).collect();
            let query = format!("INSERT INTO {table_name} ({table_columns}) VALUES ({value_brackets})",
                                table_columns=table_columns.join(", "),
                                value_brackets=values_expr.iter().map(|x| "$${}$$".to_string()).collect::<Vec<String>>().join(", "));

            TokenStream::from(quote!(
                impl SerialiseDB for #name {
                    async fn insert(&self, database: &mut MySQLDB) -> String {
                            format!(#query, #values_expr)
                    }
                }
            ))
        } else {
            TokenStream::from(
                syn::Error::new(input.ident.span(), "Only structs with named fields can derive `SerialiseDB`").to_compile_error()
            )
        }

    } else {
        TokenStream::from(
            syn::Error::new(input.ident.span(), "Only structs with named fields can derive `SerialiseDB`").to_compile_error()
        )
    }
}

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
                                panic!("Unable to parse {} in object {}", u, input.ident);
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
                fn postgres_create() -> String {
                        format!("CREATE TABLE {} ({});", stringify!(#name), #vals)
                }
                fn postgres_drop() -> String {
                        format!("DROP TABLE {};", stringify!(#name))
                }
            }));
        }
    }
    TokenStream::from(
        syn::Error::new(input.ident.span(), "Only structs with named fields can derive `PostGresObj`").to_compile_error()
    )
}