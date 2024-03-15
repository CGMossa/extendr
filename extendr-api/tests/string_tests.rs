#[allow(unused_imports)]
use extendr_api::{r, GetSexp, Robj};
use extendr_engine::with_r;
use extendr_macros::R;
#[allow(unused_imports)]
use libR_sys::{R_BlankScalarString, R_NilValue, Rf_PrintValue, Rf_xlength, STRING_ELT, TYPEOF};
#[allow(unused_imports)]
use libR_sys::{R_BlankString, R_NaString, CHARSXP, STRSXP};

/// This is a "test" to print out all the assumptions on strings we have,
/// from empty, NA, and multi-byte UTF8 characters
#[test]
fn test_byte_length() {
    // assuming that for a CHARSXP, the function Rf_xlength
    with_r(|| unsafe {
        // what is a Scalar NA string?

        let na_character_ = R!(r#"NA_character_"#).unwrap();
        assert_eq!(TYPEOF(na_character_.get()), STRSXP);
        let string_with_na = R!(r#"c(NA_character_)"#).unwrap();
        assert_eq!(TYPEOF(string_with_na.get()), STRSXP);
        assert_ne!(
            string_with_na.get(),
            R_NaString,
            "a character vector with single NA is not NA"
        );
        assert_ne!(
            string_with_na.get(),
            R_BlankScalarString,
            "c(NA_character_) is not BlankScalarString"
        );
        assert_eq!(TYPEOF(STRING_ELT(string_with_na.get(), 0)), CHARSXP);
        assert_eq!(
            STRING_ELT(string_with_na.get(), 0),
            R_NaString,
            "the CHARSXP is NA"
        );

        let r_str_vec = R!(r#"c(NA_character_, "", "1", "123", "✅")"#).unwrap();
        Rf_PrintValue(r_str_vec.get());

        // let r_str_vec_ref = &r_str_vec;
        // Rf_PrintValue(R!("length({{r_str_vec_ref}})").unwrap().get());

        dbg!(
            TYPEOF(r_str_vec.get()),
            Rf_xlength(r_str_vec.get()),
            // Rf_length(r_str_vec.get())
        );

        for i in 0..Rf_xlength(r_str_vec.get()) {
            dbg!(i);
            let elt = STRING_ELT(r_str_vec.get(), i);
            // print!("elt =");
            Rf_PrintValue(elt);
            dbg!(
                TYPEOF(elt),
                Rf_xlength(elt),
                libR_sys::Rf_length(elt),
                elt == libR_sys::R_NaString,
                elt == libR_sys::R_BlankString,
                elt == libR_sys::R_BlankScalarString,
                // same as `Rf_xlength` anyways
                // R_nchar(
                //     elt,
                //     nchar_type_Bytes,
                //     libR_sys::Rboolean::FALSE,
                //     libR_sys::Rboolean::TRUE,
                //     ptr::null()
                // )
            );
        }
    });
}
