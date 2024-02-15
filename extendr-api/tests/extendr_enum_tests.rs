use extendr_api::prelude::*;

#[derive(Debug, Clone, Copy)]
#[extendr(use_try_from = true)]
enum Model {
    MeanModel,
    LinearModel,
    BinomialModel,
    PoissonModel,
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
// #[extendr]
fn my_enum(e: &str) -> Model {
    match e.to_uppercase().as_str() {
        "A" => Model::MeanModel,
        "B" => Model::BinomialModel,
        "C" => Model::LinearModel,
        _ => unimplemented!(),
    }
}
