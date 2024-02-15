use extendr_api::prelude::*;

#[extendr]
#[derive(Debug, Copy, Clone)]
enum Model {
    A,
    B,
    C,
}

#[extendr(use_try_from = true)]
fn tst_enum_wrapper(_x: Robj, enum_fct: Model) {
    match enum_fct {
        a => {
            rprintln!("Successfully processed `enum_fct` with value {a:?}");
        }
    };
}

#[extendr(use_try_from = true)]
fn my_enum(e: &str) -> Model {
    match e.to_uppercase().as_str() {
        "A" => Model::A,
        "B" => Model::B,
        "C" => Model::C,
        _ => unimplemented!(),
    }
}

extendr_module! {
    mod enum_as_factor;
    fn tst_enum_wrapper;
    fn my_enum;
}
