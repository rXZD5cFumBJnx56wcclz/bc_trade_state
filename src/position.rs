use crate::prelude::*;

pub trait PositionExt {
    fn upd_pnl(&mut self, last_price: f64);
    fn execute_is_reduce(&mut self, order: &Order) -> f64;
    fn execute(&mut self, last_price: f64, order: &Order);
}

impl PositionExt for Position {
    fn upd_pnl(&mut self, last_price: f64) {
        let percent = (last_price - self.avg_open_price) / self.avg_open_price
            * self.leverage
            * if self.side == "buy" { 1. } else { -1. };
        self.pnl_percent = percent;
        // qty == entry_qty * leverage = self.qty / self.leverage == real pnl
        self.pnl_qty = percent * self.qty / self.leverage;
    }
    fn execute_is_reduce(&mut self, order: &Order) -> f64 {
        self.qty -= order.qty;
        if self.qty < 0. {
            self.is_active = false;
            return self.qty * -1.;
        }
        0.
    }
    fn execute(&mut self, last_price: f64, order: &Order) {
        self.qty += order.qty;
        self.avg_open_price = (self.avg_open_price + last_price) / 2.;
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_state::position::POSITION;
    use bc_test_kit::prelude::*;

    #[test]
    fn upd_pnl_res_1() {
        let mut pos = POSITION.clone();
        let percent = (pos.avg_open_price - 1.9) / pos.avg_open_price * 10. * -1.;
        pos.upd_pnl(1.9);
        assert_eq_pr!(pos.pnl_percent, percent);
        assert_eq_pr!(pos.pnl_qty, percent / pos.leverage * pos.qty);
    }
}
