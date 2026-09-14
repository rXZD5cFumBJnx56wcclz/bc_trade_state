use crate::prelude::*;

#[derive(Debug, ThisError)]
pub enum ErrorTrade {
    #[error("not enough capital: {0}\nto execute:\nqty order:{1}\nqty commission: {2}")]
    NotEnoughCapital(Capital, f64, f64),
}
