extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, ItemImpl};

use crate::{extendr_options::ExtendrOptions, wrappers};

/// Handle trait implementations.
///
/// Example:
/// ```ignore
/// use extendr_api::prelude::*;
/// #[derive(Debug)]
/// struct Person {
///     pub name: String,
/// }
/// #[extendr]
/// impl Person {
///     fn new() -> Self {
///         Self { name: "".to_string() }
///     }
///     fn set_name(&mut self, name: &str) {
///         self.name = name.to_string();
///     }
///     fn name(&self) -> &str {
///         self.name.as_str()
///     }
/// }
/// #[extendr]
/// fn aux_func() {
/// }
/// // Macro to generate exports
/// extendr_module! {
///     mod classes;
///     impl Person;
///     fn aux_func;
/// }
/// ```
#[doc(alias = "extendr-impl")]
pub fn extendr_impl(mut item_impl: ItemImpl, opts: &ExtendrOptions) -> syn::Result<TokenStream> {
    // Only `impl name { }` allowed
    if item_impl.defaultness.is_some() {
        return Err(syn::Error::new_spanned(
            item_impl,
            "default not allowed in #[extendr] impl",
        ));
    }

    if item_impl.unsafety.is_some() {
        return Err(syn::Error::new_spanned(
            item_impl,
            "unsafe not allowed in #[extendr] impl",
        ));
    }

    if item_impl.generics.const_params().count() != 0 {
        return Err(syn::Error::new_spanned(
            item_impl,
            "const params not allowed in #[extendr] impl",
        ));
    }

    if item_impl.generics.type_params().count() != 0 {
        return Err(syn::Error::new_spanned(
            item_impl,
            "type params not allowed in #[extendr] impl",
        ));
    }

    // if item_impl.generics.lifetimes().count() != 0 {
    //     return quote! { compile_error!("lifetime params not allowed in #[extendr] impl"); }.into();
    // }

    if item_impl.generics.where_clause.is_some() {
        return Err(syn::Error::new_spanned(
            item_impl,
            "where clause not allowed in #[extendr] impl",
        ));
    }

    let self_ty = item_impl.self_ty.as_ref();
    let self_ty_name = wrappers::type_name(self_ty);
    let prefix = format!("{}__", self_ty_name);
    let mut method_meta_names = Vec::new();
    let doc_string = wrappers::get_doc_string(&item_impl.attrs);

    // Generate wrappers for methods.
    // eg.
    // ```
    // #[no_mangle]
    // #[allow(non_snake_case)]
    // pub extern "C" fn wrap__Person__new() -> extendr_api::SEXP {
    //     unsafe {
    //         use extendr_api::FromRobj;
    //         extendr_api::Robj::from(<Person>::new()).get()
    //     }
    // }
    // ```
    let mut wrappers: Vec<ItemFn> = Vec::new();
    for impl_item in &mut item_impl.items {
        if let syn::ImplItem::Fn(ref mut method) = impl_item {
            method_meta_names.push(format_ident!(
                "{}{}__{}",
                wrappers::META_PREFIX,
                self_ty_name,
                method.sig.ident
            ));
            wrappers::make_function_wrappers(
                &opts,
                &mut wrappers,
                prefix.as_str(),
                &method.attrs,
                &mut method.sig,
                Some(self_ty),
            )?;
        }
    }

    let meta_name = format_ident!("{}{self_ty_name}", wrappers::META_PREFIX);

    let conversion_impls = if opts.use_try_from {
        quote!(
            // Output conversion function for this type.
            impl TryFrom<#self_ty> for Robj {
                type Error = Error;
                fn try_from(value: #self_ty) -> Result<Self> {
                    let mut res: Robj = ExternalPtr::new(value).try_into()?;
                    res.set_attrib(class_symbol(), #self_ty_name)?;
                    Ok(res)
                }
            }

            // Output conversion function for this type.
            impl TryFrom<&Robj> for &#self_ty {
                type Error = Error;
                fn try_from(robj: &Robj) -> Result<Self> {
                    let external_ptr: &ExternalPtr<#self_ty> = robj.try_into()?;
                    external_ptr.as_ref().ok_or_else(|| Error::ExpectedExternalNonNullPtr(robj.clone()))
                }
            }

            // Input conversion function for a mutable reference to this type.
            impl TryFrom<&mut Robj> for &mut #self_ty {
                type Error = Error;
                fn try_from(robj: &mut Robj) -> Result<Self> {
                    let external_ptr: &mut ExternalPtr<#self_ty> = robj.try_into()?;
                    external_ptr.as_mut().ok_or_else(|| Error::ExpectedExternalNonNullPtr(robj.clone()))
                }
            }
        )
    } else {
        quote!(
            // Input conversion function for this type.
            impl<'a> extendr_api::FromRobj<'a> for &#self_ty {
                fn from_robj(robj: &'a Robj) -> std::result::Result<Self, &'static str> {
                    use libR_sys::*;
                    unsafe {
                        let ptr = R_ExternalPtrAddr(robj.get()).cast::<#self_ty>();
                        // assume it is not C NULL
                        if ptr.is_null() {
                            Err("stored externalptr is invalid / NULL")
                        } else {
                        Ok(&*ptr)
                        }
                    }
                }
            }

            // Input conversion function for a mutable reference to this type.
            impl<'a> extendr_api::FromRobj<'a> for &mut #self_ty {
                fn from_robj(robj: &'a Robj) -> std::result::Result<Self, &'static str> {
                    use libR_sys::*;
                    unsafe {
                        //FIXME: it should be `get_mut` instead of `get`
                        // let ptr = R_ExternalPtrAddr(robj.get_mut()) as *mut #self_ty;
                        let ptr = R_ExternalPtrAddr(robj.get()).cast::<#self_ty>();
                        // assume it is not C NULL
                        if ptr.is_null() {
                            Err("stored externalptr is invalid / NULL")
                        } else {
                        Ok(&mut *ptr)
                        }
                    }
                }
            }

            // Output conversion function for this type.
            impl From<#self_ty> for Robj {
                fn from(value: #self_ty) -> Self {
                    let mut res: Robj = ExternalPtr::new(value).into();
                    res.set_attrib(class_symbol(), #self_ty_name).unwrap();
                    res
                }
            }
        )
    };

    let expanded = TokenStream::from(quote! {
        // The impl itself copied from the source.
        #item_impl

        // Function wrappers
        #( #wrappers )*

        #conversion_impls

        #[allow(non_snake_case)]
        fn #meta_name(impls: &mut Vec<extendr_api::metadata::Impl>) {
            let mut methods = Vec::new();
            #( #method_meta_names(&mut methods); )*
            impls.push(extendr_api::metadata::Impl {
                doc: #doc_string,
                name: #self_ty_name,
                methods,
            });
        }
    });

    //eprintln!("{}", expanded);
    Ok(expanded)
}

// This structure contains parameters parsed from the #[extendr_module] definition.
