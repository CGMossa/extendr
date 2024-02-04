use super::*;
use std::any::Any;
use std::fmt::Debug;

/// Wrapper for creating R objects containing any Rust object.
///
/// ```
/// use extendr_api::prelude::*;
/// test! {
///     let extptr = ExternalPtr::new(1);
///     assert_eq!(*extptr, 1);
///     let robj : Robj = extptr.into();
///     let extptr2 : ExternalPtr<i32> = robj.try_into().unwrap();
///     assert_eq!(*extptr2, 1);
/// }
/// ```
///
#[derive(PartialEq, Clone)]
pub struct ExternalPtr<T: Debug + 'static> {
    /// This is the contained Robj.
    pub(crate) robj: Robj,

    /// This is a zero-length object that holds the type of the object.
    marker: std::marker::PhantomData<T>,
}

impl<T: Debug + 'static> robj::GetSexp for ExternalPtr<T> {
    unsafe fn get(&self) -> SEXP {
        self.robj.get()
    }

    unsafe fn get_mut(&mut self) -> SEXP {
        self.robj.get_mut()
    }

    /// Get a reference to a Robj for this type.
    fn as_robj(&self) -> &Robj {
        &self.robj
    }

    /// Get a mutable reference to a Robj for this type.
    fn as_robj_mut(&mut self) -> &mut Robj {
        &mut self.robj
    }
}

/// len() and is_empty()
impl<T: Debug + 'static> Length for ExternalPtr<T> {}

/// rtype() and rany()
impl<T: Debug + 'static> Types for ExternalPtr<T> {}

/// as_*()
impl<T: Debug + 'static> Conversions for ExternalPtr<T> {}

/// find_var() etc.
impl<T: Debug + 'static> Rinternals for ExternalPtr<T> {}

/// as_typed_slice_raw() etc.
impl<T: Debug + 'static> Slices for ExternalPtr<T> {}

/// dollar() etc.
impl<T: Debug + 'static> Operators for ExternalPtr<T> {}

impl<T: Debug + 'static> Deref for ExternalPtr<T> {
    type Target = T;

    /// This allows us to treat the Robj as if it is the type T.
    fn deref(&self) -> &Self::Target {
        self.addr()
    }
}

impl<T: Debug + 'static> DerefMut for ExternalPtr<T> {
    /// This allows us to treat the Robj as if it is the mutable type T.
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.addr_mut()
    }
}

impl<T: Any + Debug> ExternalPtr<T> {
    /// Construct an external pointer object from any type T.
    /// In this case, the R object owns the data and will drop the Rust object
    /// when the last reference is removed via register_c_finalizer.
    ///
    /// An ExternalPtr behaves like a Box except that the information is
    /// tracked by a R object.
    pub fn new(val: T) -> Self {
        single_threaded(|| unsafe {
            // This allocates some memory for our object and moves the object into it.
            let boxed = Box::new(val);

            // This constructs an external pointer to our boxed data.
            // into_raw() converts the box to a malloced pointer.
            let robj = {
                let p = Box::into_raw(boxed);
                let prot = R_NilValue;
                let type_name: Robj = std::any::type_name::<T>().into();
                Robj::from_sexp({
                    R_MakeExternalPtr(p as *mut std::os::raw::c_void, type_name.get(), prot)
                })
            };

            unsafe extern "C" fn finalizer<T>(x: SEXP) {
                unsafe {
                    let ptr = R_ExternalPtrAddr(x) as *mut T;

                    // Free the `tag`, which is the type-name
                    R_SetExternalPtrTag(x, R_NilValue);

                    // Convert the pointer to a box and drop it implictly.
                    // This frees up the memory we have used and calls the "T::drop" method if there is one.
                    drop(Box::from_raw(ptr));

                    // Now set the pointer in ExternalPtr to C `NULL`
                    R_ClearExternalPtr(x);
                }
            }

            // Tell R about our finalizer
            // Use R_RegisterCFinalizerEx() and set onexit to 1 (TRUE) to invoke the
            // finalizer on a shutdown of the R session as well.
            R_RegisterCFinalizerEx(robj.get(), Some(finalizer::<T>), Rboolean::TRUE);

            // Return an object in a wrapper.
            Self {
                robj,
                marker: std::marker::PhantomData,
            }
        })
    }

    // TODO: make a constructor for references?

    /// Get the "tag" of an external pointer. This is the type name in the common case.
    pub fn tag(&self) -> Robj {
        unsafe { Robj::from_sexp(R_ExternalPtrTag(self.robj.get())) }
    }

    /// Get the "protected" field of an external pointer. This is NULL in the common case.
    pub fn protected(&self) -> Robj {
        unsafe { Robj::from_sexp(R_ExternalPtrProtected(self.robj.get())) }
    }

    /// Get the "address" field of an external pointer.
    /// Normally, we will use Deref to do this.
    pub fn addr<'a>(&self) -> &'a T {
        unsafe {
            let ptr = R_ExternalPtrAddr(self.robj.get()) as *const T;
            ptr.as_ref().unwrap()
        }
    }

    /// Get the "address" field of an external pointer as a mutable reference.
    /// Normally, we will use DerefMut to do this.
    pub fn addr_mut(&mut self) -> &mut T {
        unsafe {
            let ptr = R_ExternalPtrAddr(self.robj.get_mut()) as *mut T;
            ptr.as_mut().unwrap()
        }
    }
}

impl<T: Any + Debug> TryFrom<&Robj> for ExternalPtr<T> {
    type Error = Error;

    fn try_from(robj: &Robj) -> Result<Self> {
        let clone = robj.clone();
        if clone.rtype() != Rtype::ExternalPtr {
            return Err(Error::ExpectedExternalPtr(clone));
        }

        // NOTE: omitting type checking because it is unnecessary and inaccurate.

        let res = ExternalPtr::<T> {
            robj: clone,
            marker: std::marker::PhantomData,
        };

        Ok(res)
    }
}

impl<T: Any + Debug> TryFrom<Robj> for ExternalPtr<T> {
    type Error = Error;

    fn try_from(robj: Robj) -> Result<Self> {
        <ExternalPtr<T>>::try_from(&robj)
    }
}

impl<T: Any + Debug> From<ExternalPtr<T>> for Robj {
    fn from(val: ExternalPtr<T>) -> Self {
        val.robj
    }
}

impl<T: Debug + 'static> std::fmt::Debug for ExternalPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (&**self as &T).fmt(f)
    }
}
