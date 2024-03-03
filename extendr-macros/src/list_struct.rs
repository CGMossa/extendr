use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput};

/// Implementation of the TryFromRobj macro. Refer to the documentation there
pub fn derive_try_from_robj(item: TokenStream) -> syn::Result<TokenStream> {
    // TODO: have an option on each field to `r_name`

    // FIXME: reject fields with r# field names or similar

    // Parse the tokens into a Struct
    let ast = syn::parse::<DeriveInput>(item)?;
    let inside = if let Data::Struct(inner) = ast.data {
        inner
    } else {
        return Err(syn::Error::new_spanned(ast, "Only struct is supported"));
    };
    let struct_name = ast.ident;

    // Iterate each struct field and capture a conversion from Robj for each field
    let mut tokens = Vec::<TokenStream2>::with_capacity(inside.fields.len());
    for field in inside.fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_str = field_name.to_string();
        // This is like `value$foo` in R

        //TODO: forming these strings is costly, even if R does interning,
        // we still need an R-string to compare by pointer directly.
        // Can't the STRSXP, must be the CHARSXP that is being compared directly.

        tokens.push(quote!(
            #field_name: value.dollar(#field_str)?.try_into()?
        ));
    }

    // Emit the conversion trait impl
    Ok(TokenStream::from(quote!(
        impl std::convert::TryFrom<&Robj> for #struct_name {
            type Error = extendr_api::Error;

            fn try_from(value: &Robj) -> extendr_api::Result<Self> {
                Ok(#struct_name {
                    #(#tokens),*
                })
            }
        }

        impl std::convert::TryFrom<Robj> for #struct_name {
            type Error = extendr_api::Error;

            fn try_from(value: Robj) -> extendr_api::Result<Self> {
                //TODO: remove this, and just piggyback on the &Robj implementation above
                Ok(#struct_name {
                    #(#tokens),*
                })
            }
        }
    )))
}

/// Implementation of the IntoRobj macro. Refer to the documentation there
pub fn derive_into_robj(item: TokenStream) -> syn::Result<TokenStream> {
    // TODO: have an option on each field to `r_name`

    // FIXME: reject fields with r# field names or similar

    // Parse the tokens into a Struct
    let ast = syn::parse::<DeriveInput>(item)?;
    let inside = if let Data::Struct(inner) = ast.data {
        inner
    } else {
        return Err(syn::Error::new_spanned(ast, "Only struct is supported"));
    };
    let struct_name = ast.ident;

    // Iterate each struct field and capture a token that creates a KeyValue pair (tuple) for
    // each field
    let mut tokens = Vec::<TokenStream2>::with_capacity(inside.fields.len());

    for field in inside.fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_str = field_name.to_string();
        tokens.push(quote!(
            (#field_str, (&value.#field_name).into())
        ));
    }

    // FIXME: these should really be implementing -> List and not -> Robj.

    // The only thing we emit from this macro is the conversion trait impl
    Ok(TokenStream::from(quote!(
        impl std::convert::From<&#struct_name> for Robj {
            fn from(value: &#struct_name) -> Self {
                extendr_api::List::from_pairs([#(#tokens),*]).into()
            }
        }
        impl std::convert::From<#struct_name> for Robj {
            fn from(value: #struct_name) -> Self {
                extendr_api::List::from_pairs([#(#tokens),*]).into()
            }
        }
    )))
}
