use crate::prelude::*;

pub trait CapitalExt {
    fn execute_order_reduce(&mut self, order: &Order, remainder: f64);
    fn execute_order(&mut self, order: &Order);
}

impl CapitalExt for Capital {
    fn execute_order_reduce(&mut self, order: &Order, remainder: f64) {
        *self -= order.commission;
        *self += (order.qty - remainder) / order.leverage;
    }
    fn execute_order(&mut self, order: &Order) {
        *self -= order.commission;
        *self -= order.qty / order.leverage;
    }
}
