//! This provides an abstraction for R's `data.frame`-constructor in Rust.
//! For a given `struct` say `CustomRow`, one may implement or derive [`IntoDataFrameRow`],
//! thus being able to convert `Vec<CustomRow>` to an instance of `Dataframe<CustomRow>`,
//! see [`Dataframe`].
//!
//!
//! [`IntoDataFrameRow`]: ::extendr_macros::IntoDataFrameRow
use super::*;

/// A trait to convert a collection of `IntoDataFrameRow` into
/// [`Dataframe`]. Typical usage involves using the derive-macro [`IntoDataFrameRow`]
/// on a struct, which would generate `impl IntoDataframe<T> for Vec<T>`.
///
/// [`IntoDataFrameRow`]: ::extendr_macros::IntoDataFrameRow
pub trait IntoDataframe<T> {
    fn into_dataframe(self) -> Result<Dataframe<T>>;
}

#[derive(PartialEq, Clone)]
pub struct Dataframe<T> {
    pub(crate) robj: Robj,
    marker: std::marker::PhantomData<T>,
}

impl<T> std::convert::TryFrom<&Robj> for Dataframe<T> {
    type Error = Error;
    fn try_from(robj: &Robj) -> Result<Self> {
        // TODO: check type using derived trait.
        if !robj.is_frame() {
            return Err(Error::ExpectedDataframe(robj.clone()));
        }
        Ok(Dataframe {
            robj: robj.clone(),
            marker: std::marker::PhantomData,
        })
    }
}

impl<T> std::convert::TryFrom<Robj> for Dataframe<T> {
    type Error = Error;
    fn try_from(robj: Robj) -> Result<Self> {
        (&robj).try_into()
    }
}

impl<T> Dataframe<T> {
    /// Use `#[derive(IntoDataFrameRow)]` to use this.
    pub fn try_from_values<I: IntoDataframe<T>>(iter: I) -> Result<Self> {
        iter.into_dataframe()
    }
}

impl<T> std::fmt::Debug for Dataframe<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "dataframe!({})",
            self.as_list()
                .unwrap()
                .iter()
                .map(|(k, v)| if !k.is_empty() {
                    format!("{}={:?}", k, v)
                } else {
                    format!("{:?}", v)
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl<T> IntoRobj for Dataframe<T> {
    fn into_robj(self) -> Robj {
        self.robj
    }
}
