//! Conversions to [`Robj`]

use super::*;

macro_rules! impl_try_from_scalar_integer {
    ($t:ty) => {
        impl TryFrom<&Robj> for $t {
            type Error = Error;

            /// Convert a numeric object to an integer value.
            fn try_from(robj: &Robj) -> Result<Self> {
                // Check if the value is a scalar
                match robj.len() {
                    0 => return Err(Error::ExpectedNonZeroLength(robj.clone())),
                    1 => {}
                    _ => return Err(Error::ExpectedScalar(robj.clone())),
                };

                // Check if the value is not a missing value
                if robj.is_na() {
                    return Err(Error::MustNotBeNA(robj.clone()));
                }

                // If the conversion is int-to-int, check the limits. This
                // needs to be done by `TryFrom` because the conversion by `as`
                // is problematic when converting a negative value to unsigned
                // integer types (e.g. `-1i32 as u8` becomes 255).
                if let Some(v) = robj.as_integer() {
                    if let Ok(v) = Self::try_from(v) {
                        return Ok(v);
                    } else {
                        return Err(Error::OutOfLimits(robj.clone()));
                    }
                }

                // If the conversion is float-to-int, check if the value is
                // integer-like (i.e., an integer, or a float representing a
                // whole number). This needs to be down with `as`, as no
                // `TryFrom` is implemented for float types. `FloatToInt` trait
                // might eventually become available in future, though.
                if let Some(v) = robj.as_real() {
                    let result = v as Self;
                    if (result as f64 - v).abs() < f64::EPSILON {
                        return Ok(result);
                    } else {
                        return Err(Error::ExpectedWholeNumber(robj.clone()));
                    }
                }

                Err(Error::ExpectedNumeric(robj.clone()))
            }
        }
    };
}

macro_rules! impl_try_from_scalar_real {
    ($t:ty) => {
        impl TryFrom<&Robj> for $t {
            type Error = Error;

            /// Convert a numeric object to a real value.
            fn try_from(robj: &Robj) -> Result<Self> {
                // Check if the value is a scalar
                match robj.len() {
                    0 => return Err(Error::ExpectedNonZeroLength(robj.clone())),
                    1 => {}
                    _ => return Err(Error::ExpectedScalar(robj.clone())),
                };

                // Check if the value is not a missing value
                if robj.is_na() {
                    return Err(Error::MustNotBeNA(robj.clone()));
                }

                // <Robj>::as_xxx() methods can work only when the underlying
                // `SEXP` is the corresponding type, so we cannot use `as_real()`
                // directly on `INTSXP`.
                if let Some(v) = robj.as_real() {
                    return Ok(v as Self);
                }
                if let Some(v) = robj.as_integer() {
                    return Ok(v as Self);
                }

                Err(Error::ExpectedNumeric(robj.clone()))
            }
        }
    };
}

impl_try_from_scalar_integer!(u8);
impl_try_from_scalar_integer!(u16);
impl_try_from_scalar_integer!(u32);
impl_try_from_scalar_integer!(u64);
impl_try_from_scalar_integer!(usize);
impl_try_from_scalar_integer!(i8);
impl_try_from_scalar_integer!(i16);
impl_try_from_scalar_integer!(i32);
impl_try_from_scalar_integer!(i64);
impl_try_from_scalar_integer!(isize);
impl_try_from_scalar_real!(f32);
impl_try_from_scalar_real!(f64);

impl TryFrom<&Robj> for bool {
    type Error = Error;

    /// Convert an `LGLSXP` object into a boolean.
    /// NAs are not allowed.
    fn try_from(robj: &Robj) -> Result<Self> {
        if robj.is_na() {
            Err(Error::MustNotBeNA(robj.clone()))
        } else {
            Ok(<Rbool>::try_from(robj)?.is_true())
        }
    }
}

impl TryFrom<&Robj> for &str {
    type Error = Error;

    /// Convert a scalar `STRSXP` object into a string slice.
    /// NAs are not allowed.
    fn try_from(robj: &Robj) -> Result<Self> {
        if robj.is_na() {
            return Err(Error::MustNotBeNA(robj.clone()));
        }
        match robj.len() {
            0 => Err(Error::ExpectedNonZeroLength(robj.clone())),
            1 => {
                if let Some(s) = robj.as_str() {
                    Ok(s)
                } else {
                    Err(Error::ExpectedString(robj.clone()))
                }
            }
            _ => Err(Error::ExpectedScalar(robj.clone())),
        }
    }
}

impl TryFrom<&Robj> for String {
    type Error = Error;

    /// Convert an scalar `STRSXP` object into a `String`.
    /// Note: Unless you plan to store the result, use a string slice instead.
    /// NAs are not allowed.
    fn try_from(robj: &Robj) -> Result<Self> {
        <&str>::try_from(robj).map(|s| s.to_string())
    }
}

impl<T> TryFrom<&Robj> for Vec<T>
where
    T: Clone,
    Robj: for<'a> AsTypedSlice<'a, T>,
{
    type Error = Error;

    /// Convert an `INTSXP` object into a vector of `i32` (integer).
    /// Note: Unless you plan to store the result, use a slice instead.
    /// Use `value.is_na()` to detect `NA` values.
    fn try_from(robj: &Robj) -> Result<Self> {
        let v = robj.try_into_typed_slice()?;
        // TODO: check NAs
        Ok(Vec::from(v))
    }
}

impl TryFrom<&Robj> for Vec<String> {
    type Error = Error;

    /// Convert a `STRSXP` object into a vector of `String`s.
    /// Note: Unless you plan to store the result, use a slice instead.
    fn try_from(robj: &Robj) -> Result<Self> {
        if let Some(iter) = robj.as_str_iter() {
            // check for `NA`'s in the string vector
            if iter.clone().any(|s| s.is_na()) {
                Err(Error::MustNotBeNA(robj.clone()))
            } else {
                Ok(iter.map(|s| s.to_string()).collect::<Vec<String>>())
            }
        } else {
            Err(Error::ExpectedString(robj.clone()))
        }
    }
}

// region: `RobjRef` / `RobjMut`

impl<T> TryFrom<&Robj> for RobjRef<'_, [T]>
where
    Robj: for<'a> AsTypedSlice<'a, T>,
{
    type Error = Error;

    fn try_from(robj: &Robj) -> Result<Self> {
        robj.try_into_typed_slice().map(Self)
    }
}

impl<T> TryFrom<&mut Robj> for RobjMut<'_, [T]>
where
    Robj: for<'a> AsTypedSlice<'a, T>,
{
    type Error = Error;

    fn try_from(robj: &mut Robj) -> Result<Self> {
        robj.try_into_typed_slice_mut().map(Self)
    }
}

impl<T> TryFrom<&Robj> for RobjRef<'_, T>
where
    Robj: for<'a> AsTypedSlice<'a, T>,
{
    type Error = Error;

    fn try_from(robj: &Robj) -> Result<Self> {
        let slice = robj.try_into_typed_slice()?;
        match robj.len() {
            0 => Err(Error::ExpectedNonZeroLength(robj.clone())),
            1 => Ok(Self(slice.first().unwrap())),
            _ => Err(Error::ExpectedScalar(robj.clone())),
        }
    }
}

impl<T> TryFrom<&mut Robj> for RobjMut<'_, T>
where
    Robj: for<'a> AsTypedSlice<'a, T>,
{
    type Error = Error;

    fn try_from(robj: &mut Robj) -> Result<Self> {
        let slice = robj.try_into_typed_slice_mut()?;
        match robj.len() {
            0 => Err(Error::ExpectedNonZeroLength(robj.clone())),
            1 => Ok(Self(slice.first_mut().unwrap())),
            _ => Err(Error::ExpectedScalar(robj.clone())),
        }
    }
}

// endregion

impl<T> TryFrom<&Robj> for &[T]
where
    Robj: for<'a> AsTypedSlice<'a, T>,
{
    type Error = Error;

    /// Use `value.is_na()` to detect `NA` values.
    fn try_from(robj: &Robj) -> Result<Self> {
        robj.try_into_typed_slice()
    }
}

// NOTE: Cannot support `Box<[T]>` as that may cause a double-free!

impl TryFrom<&Robj> for Rcplx {
    type Error = Error;

    fn try_from(robj: &Robj) -> Result<Self> {
        // Check if the value is a scalar
        match robj.len() {
            0 => return Err(Error::ExpectedNonZeroLength(robj.clone())),
            1 => {}
            _ => return Err(Error::ExpectedScalar(robj.clone())),
        };

        // Check if the value is not a missing value.
        if robj.is_na() {
            return Ok(Rcplx::na());
        }

        // This should always work, `NA` is handled above.
        if let Some(v) = robj.as_real() {
            return Ok(Rcplx::from(v));
        }

        // Any integer (32 bit) can be represented as `f64`,
        // this always works.
        if let Some(v) = robj.as_integer() {
            return Ok(Rcplx::from(v as f64));
        }

        // Complex slices return their first element.
        let s = robj.try_into_typed_slice()?;
        Ok(s[0])
    }
}

// Convert `TryFrom<&Robj>` into `TryFrom<Robj>`. Sadly, we are unable to make a blanket
// conversion using `GetSexp` with the current version of Rust.
#[macro_export]
macro_rules! impl_try_from_robj_ref {
    ($($type : ty)*) => {
        $(
            impl TryFrom<Robj> for $type {
                type Error = Error;

                fn try_from(robj: Robj) -> Result<Self> {
                    <$type>::try_from(&robj)
                }
            }

            impl TryFrom<&Robj> for Option<$type> {
                type Error = Error;

                fn try_from(robj: &Robj) -> Result<Self> {
                    if robj.is_null() || robj.is_na() {
                        Ok(None)
                    } else {
                        Ok(Some(<$type>::try_from(robj)?))
                    }
                }
            }

            impl TryFrom<Robj> for Option<$type> {
                type Error = Error;

                fn try_from(robj: Robj) -> Result<Self> {
                    <Option::<$type>>::try_from(&robj)
                }
            }
        )*
    }
}
pub(crate) use impl_try_from_robj_ref;

impl_try_from_robj_ref!(
    u8 u16 u32 u64 usize
    i8 i16 i32 i64 isize
    bool
    Rint Rfloat Rbool Rcplx
    f32 f64
    HashMap::<String, Robj> HashMap::<&str, Robj>
    Vec::<String>
    Vec::<Rint> Vec::<Rfloat> Vec::<Rbool> Vec::<Rcplx> Vec::<u8> Vec::<i32> Vec::<f64>
    &[Rint] &[Rfloat] &[Rbool] &[Rcplx] &[u8] &[i32] &[f64]
    //TODO: RobjRef<'_, str>
    //TODO: RobjRef<'_, [str]>
    RobjRef<'_, Rstr> RobjRef<'_, Rint> RobjRef<'_, Rfloat> RobjRef<'_, Rbool> RobjRef<'_, Rcplx> RobjRef<'_, u8> RobjRef<'_, i32> RobjRef<'_, f64>
    RobjRef<'_, [Rstr]> RobjRef<'_, [Rint]> RobjRef<'_, [Rfloat]> RobjRef<'_, [Rbool]> RobjRef<'_, [Rcplx]> RobjRef<'_, [u8]> RobjRef<'_, [i32]> RobjRef<'_, [f64]>
    &str String
);

// region: mutable

// Convert `TryFrom<&mut Robj>` into `TryFrom<Robj>`.
// TODO: Try with GATS now?
// Sadly, we are unable to make a blanket
// conversion using `GetSexp` with the current version of Rust.

// TODO: It might be possible to unify `impl_try_from_robj_ref` and `impl_try_from_robj_mut`,
// by first making a macro that takes in a list of tokens, then making one that
// specifically expects owned, ref, and ref mut together with type.
//
/*
macro_rules! impl_try_from_on {
    // Base case: no more tokens to process
    () => {};

    // Handle Vec<T>
    (Vec<$t:ty>  $(, $($rest:tt)*)?) => {
        println!("Vec<{}>", stringify!($t));
        $(impl_try_from_on!($($rest)*);)?
    };

    // Handle slice references &[T]
    (&[$t:ty]  $(, $($rest:tt)*)?) => {
        println!("&[{}]", stringify!($t));
        $(impl_try_from_on!($($rest)*);)?
    };

    // Handle mutable slice references &mut [T]
    (&mut [$t:ty]  $(, $($rest:tt)*)?) => {
        println!("&mut [{}]", stringify!($t));
        $(impl_try_from_on!($($rest)*);)?
    };
    // Handle Option<T>
    (Option<$t:ty>  $(, $($rest:tt)*)?) => {
        println!("Option<{}>", stringify!($t));
        $(impl_try_from_on!($($rest)*);)?
    };

    // Handle Option<&T>
    (Option<&$t:ty>  $(, $($rest:tt)*)?) => {
        println!("Option<&{}>", stringify!($t));
        $(impl_try_from_on!($($rest)*);)?
    };
    // Handle Option<&mut T>
    (Option<&mut $t:ty> $(, $($rest:tt)*)?) => {
        println!("Option<&mut {}>", stringify!($t));
        $(impl_try_from_on!($($rest)*);)?
    };

    // Handle references &T
    (&$t:ty  $(, $($rest:tt)*)?) => {
        println!("&{}", stringify!($t));
        $(impl_try_from_on!($($rest)*);)?
    };

    // Handle mutable references &mut T
    (&mut $t:ty  $(, $($rest:tt)*)?) => {
        println!("&mut {}", stringify!($t));
        $(impl_try_from_on!($($rest)*);)?
    };

    // Handle owned types
    ($t:ty  $(, $($rest:tt)*)?) => {
        println!("Owned {}", stringify!($t));
        $(impl_try_from_on!($($rest)*);)?
    };
}
 */
//
#[macro_export]
macro_rules! impl_try_from_robj_mut {
    ($($type : ty)*) => {
        $(
            impl TryFrom<Robj> for $type {
                type Error = Error;

                fn try_from(mut robj: Robj) -> Result<Self> {
                    <$type>::try_from(&mut robj)
                }
            }

            impl TryFrom<&mut Robj> for Option<$type> {
                type Error = Error;

                fn try_from(robj: &mut Robj) -> Result<Self> {
                    if robj.is_null() || robj.is_na() {
                        Ok(None)
                    } else {
                        Ok(Some(<$type>::try_from(robj)?))
                    }
                }
            }

            impl TryFrom<Robj> for Option<$type> {
                type Error = Error;

                fn try_from(mut robj: Robj) -> Result<Self> {
                    <Option::<$type>>::try_from(&mut robj)
                }
            }
        )*
    }
}
pub(crate) use impl_try_from_robj_mut;

impl_try_from_robj_mut!(
    &mut [Rint] &mut [Rfloat] &mut [Rbool] &mut [Rcplx] &mut [u8] &mut [i32] &mut [f64]
    RobjMut<'_, Rint> RobjMut<'_, Rfloat> RobjMut<'_, Rbool> RobjMut<'_, Rcplx> RobjMut<'_, u8> RobjMut<'_, i32> RobjMut<'_, f64>
    RobjMut<'_, [Rint]> RobjMut<'_, [Rfloat]> RobjMut<'_, [Rbool]> RobjMut<'_, [Rcplx]> RobjMut<'_, [u8]> RobjMut<'_, [i32]> RobjMut<'_, [f64]>
);

impl<T> TryFrom<&mut Robj> for &mut [T]
where
    Robj: for<'a> AsTypedSlice<'a, T>,
{
    type Error = Error;

    /// Convert an `INTSXP` object into a mutable slice of `i32` (integer).
    /// Use `value.is_na()` to detect `NA` values.
    fn try_from(robj: &mut Robj) -> Result<Self> {
        robj.try_into_typed_slice_mut()
    }
}

// endregion

impl TryFrom<&Robj> for HashMap<String, Robj> {
    type Error = Error;
    fn try_from(robj: &Robj) -> Result<Self> {
        Ok(robj
            .as_list()
            .map(|l| l.iter())
            .ok_or_else(|| Error::ExpectedList(robj.clone()))?
            .map(|(k, v)| (k.to_string(), v))
            .collect::<HashMap<String, Robj>>())
    }
}

impl TryFrom<&Robj> for HashMap<&str, Robj> {
    type Error = Error;
    fn try_from(robj: &Robj) -> Result<Self> {
        Ok(robj
            .as_list()
            .map(|l| l.iter())
            .ok_or_else(|| Error::ExpectedList(robj.clone()))?
            .collect::<HashMap<&str, Robj>>())
    }
}
