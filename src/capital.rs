use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

use crate::prelude::*;

#[derive(Debug, Clone, Default, PartialEq, Copy)]
pub struct Capital(pub f64);

impl Display for Capital {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<f64> for Capital {
    type Output = f64;
    fn add(self, rhs: f64) -> Self::Output {
        self.0 + rhs
    }
}

impl Sub<f64> for Capital {
    type Output = f64;
    fn sub(self, rhs: f64) -> Self::Output {
        self.0 - rhs
    }
}

impl Mul<f64> for Capital {
    type Output = f64;
    fn mul(self, rhs: f64) -> Self::Output {
        self.0 * rhs
    }
}

impl MulAssign<f64> for Capital {
    fn mul_assign(&mut self, rhs: f64) {
        self.0 *= rhs;
    }
}

impl SubAssign<f64> for Capital {
    fn sub_assign(&mut self, rhs: f64) {
        self.0 -= rhs;
    }
}

impl AddAssign<f64> for Capital {
    fn add_assign(&mut self, rhs: f64) {
        self.0 += rhs;
    }
}

impl Capital {
    pub fn execute_order_reduce(&mut self, order: &Order, remainder: f64) {
        *self -= order.commission;
        *self += (order.qty - remainder) / order.leverage;
    }
    pub fn execute_order(&mut self, order: &Order) {
        *self -= order.commission;
        *self -= order.qty / order.leverage;
    }
}
