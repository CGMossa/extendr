use extendr_api::{Attributes, GetSexp};
use extendr_engine::with_r;
use extendr_macros::list;
use libR_sys::{R_compute_identical, Rboolean_TRUE, Rf_PrintValue};

#[test]
fn test_what_is_returned_from_set_class() {
    with_r(|| {
        let mut a = list!(a = 42);
        let a_class = a.set_class(&["class"]).unwrap();

        unsafe {
            Rf_PrintValue(a.get());
            Rf_PrintValue(a_class.get());

            /* R_compute_identical:  C version of identical() function
            The third arg to R_compute_identical() consists of bitmapped flags for non-default options:
            currently the first 4 and the 6th default to TRUE, so the flag is set for FALSE values:
            1 = !NUM_EQ
            2 = !SINGLE_NA
            4 = !ATTR_AS_SET
            8 = !IGNORE_BYTECODE
            16 = !IGNORE_ENV
            32 = !IGNORE_SRCREF
            Default from R's default: 16 = (0 + 0 + 0 + 0 + 16 + 0)
            */
            assert!(R_compute_identical(a.get(), a_class.get(), 0) == Rboolean_TRUE);
            // assert!(R_compute_identical(a.get(), a_class.get(), 1) == Rboolean_TRUE);
            // assert!(R_compute_identical(a.get(), a_class.get(), 2) == Rboolean_TRUE);
            // assert!(R_compute_identical(a.get(), a_class.get(), 4) == Rboolean_TRUE);
            // assert!(R_compute_identical(a.get(), a_class.get(), 8) == Rboolean_TRUE);
            // // R default flag is 16
            // assert!(R_compute_identical(a.get(), a_class.get(), 16) == Rboolean_TRUE);
            // assert!(R_compute_identical(a.get(), a_class.get(), 32) == Rboolean_TRUE);
        }
    })
}
