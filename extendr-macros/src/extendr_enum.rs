use quote::{format_ident, quote};

use crate::extendr_options::ExtendrOptions;

//TODO: Variants with Named structs, that happens to be ExternalPtr<NamedStruct>
// could be supported. The API needs investigation though..

/// Adds the ability to take an `enum` of plain variants and turn them into
/// an R factor.
///
/// The order of the enums listed in Rust dictates the order in `levels`.
/// We do not use the discriminant value (if specified) for anything.
///
///
pub(crate) fn extendr_enum(
    item_enum: syn::ItemEnum,
    opts: &ExtendrOptions,
) -> proc_macro::TokenStream {
    //TODO: error on opts that isn't used here:
    // first, inherent &opts, and see if any value is provided..

    //FIXME: sanitize field names, as sometimes they have r# etc.
    let enum_name = &item_enum.ident;

    assert!(
        item_enum.generics.lt_token.is_none(),
        "generic enums are not supported"
    );

    if item_enum.variants.is_empty() {
        return quote!(compile_error!("Empty enums are not supported")).into();
    }

    let mut literal_field_names = Vec::with_capacity(item_enum.variants.len());
    let mut field_names = Vec::with_capacity(item_enum.variants.len());
    for ele in item_enum.variants.iter() {
        match ele.fields {
            syn::Fields::Named(_) | syn::Fields::Unnamed(_) => {
                return quote!(compile_error!("`#[extendr]` only supports plain enums")).into()
            }
            syn::Fields::Unit => {}
        }

        //TODO: process ele.attrs, and see if it has #[extendr(r_name)] to use
        // as field identifier instead of direct field names

        let field_name = &ele.ident;
        //FIXME: sanitize field names, as sometimes they have r# etc.
        literal_field_names.push(syn::LitStr::new(
            field_name.to_string().as_str(),
            field_name.span(),
        ));
        // field_names.push(format!("{enum_name}::{field_name}"));
        field_names.push(field_name);
    }
    let literal_field_names = literal_field_names;
    let field_names = field_names;

    let enum_name_upper = enum_name.to_string().to_uppercase();
    let enum_levels_name_strings = format_ident!("__{}_R_LEVELS", enum_name_upper);
    let enum_levels_name = format_ident!("__{}_LEVELS", enum_name_upper);
    let enum_levels_name_str = format_ident!("__{}_LEVELS_STR", enum_name_upper);
    let n_variants = item_enum.variants.len();
    let field_name_number: Vec<usize> = (0..n_variants).collect();

    let item_enum = &item_enum;

    //TODO: consider using a secret module to hide this even further?
    quote!(

        #item_enum

        #[doc(hidden)]
        const fn is_clone<T: Clone>(){}

        #[doc(hidden)]
        const _: () = is_clone::<#enum_name>();

        #[doc(hidden)]
        const #enum_levels_name: [#enum_name; #n_variants] = [#(#enum_name::#field_names),*];
        #[doc(hidden)]
        const #enum_levels_name_str: [&str; #n_variants] = [#(#literal_field_names),*];

        #[doc(hidden)]
        thread_local! {
            static #enum_levels_name_strings: extendr_api::prelude::once_cell::unsync::Lazy<extendr_api::Strings> = once_cell::unsync::Lazy::new(||{
                Strings::from_values(#enum_levels_name_str)
            });
        }

        impl From<Rint> for #enum_name {
            fn from(value: Rint) -> Self {
                let value = value.inner();
                assert_ne!(value, 0, "zero index for factor is invalid");
                //TODO: missing handling of NA case
                #enum_levels_name[(value - 1) as usize]
            }
        }

        impl From<#enum_name> for Rint {
            fn from(value: #enum_name) -> Self {
                match value {
                    #(#enum_name::#field_names => Rint::new((#field_name_number + 1) as _)),*
                }
            }
        }

        impl From<#enum_name> for Robj {
            fn from(value: #enum_name) -> Self {
                let rint: Rint = value.into();
                let mut robj: Robj = rint.into();
                // TODO: consider using `single_threaded` here
                unsafe {
                    #enum_levels_name_strings.with(|strings_enum|{
                        let strings_enum = once_cell::unsync::Lazy::force(strings_enum);
                        libR_sys::Rf_setAttrib(robj.get_mut(), libR_sys::R_LevelsSymbol, strings_enum.get());
                    });
                    extendr_api::R_FactorSymbol.with(|factor_class| {
                        let factor_class = once_cell::unsync::Lazy::force(factor_class);
                        // a symbol is permanent, so no need to protect it
                        // printname is CHARSXP, and we need a STRSXP, hence `Rf_ScalarString`
                        // doesn't need protection, because it gets inserted into a protected `SEXP` immediately
                        libR_sys::Rf_setAttrib(robj.get_mut(), libR_sys::R_ClassSymbol, libR_sys::Rf_ScalarString(libR_sys::PRINTNAME(*factor_class)));
                    });
                }
                robj
            }
        }

        impl TryFrom<Robj> for #enum_name {
            type Error = extendr_api::Error;

            fn try_from(robj: Robj) -> Result<Self> {
                Self::try_from(&robj)
            }
        }

        impl TryFrom<&Robj> for #enum_name {
            type Error = extendr_api::Error;

            fn try_from(robj: &Robj) -> Result<Self> {
                if !robj.is_factor() {
                    return Err(Error::ExpectedFactor(robj.clone()));
                }

                let levels = robj.get_attrib(levels_symbol()).unwrap();
                let levels: Strings = levels.try_into()?;

                // same levels as enum?
                let levels_cmp_flag = #enum_levels_name_strings.with(|x|{
                    let target_levels = extendr_api::prelude::once_cell::unsync::Lazy::force(x);

                    //FIXME: propogate error instead of panic'ing.
                    if &levels == target_levels {
                        None
                    } else {
                        Some(Error::InvalidLevels(levels.into(), target_levels.into()))
                    }
                });
                if let Some(levels_err) = levels_cmp_flag {
                    return Err(levels_err);
                }

                use extendr_api::AsTypedSlice;
                let int_vector: &[Rint] = robj.as_typed_slice().unwrap();
                if int_vector.len() != 1 {
                    return Err(Error::ExpectedScalarFactor(robj.clone()))
                }

                let result: #enum_name = int_vector[0].into();

                Ok(result)
            }
        }
    ).into()
}
