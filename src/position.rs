use crate::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub symbol: String,
    pub side: String,
    pub qty: f64,
    pub leverage: f64,
    pub avg_open_price: f64,
    pub position_idx: usize,
    pub is_active: bool,
    pub pnl_qty_percent: f64,
    pub pnl_qty: f64,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            symbol: Default::default(),
            side: Default::default(),
            qty: Default::default(),
            leverage: 1.,
            avg_open_price: 0.,
            position_idx: Default::default(),
            is_active: true,
            pnl_qty: 0.,
            pnl_qty_percent: 0.,
        }
    }
}

impl Position {
    pub fn new(
        symbol: String,
        side: String,
        qty: f64,
        leverage: f64,
        avg_open_price: f64,
        position_idx: usize,
        is_active: bool,
    ) -> Self {
        Self {
            symbol,
            side,
            qty,
            leverage,
            avg_open_price,
            position_idx,
            is_active,
            pnl_qty: 0.,
            pnl_qty_percent: 0.,
        }
    }
}

impl Position {
    pub fn upd_pnl(&mut self, last_price: f64) {
        let percent = (last_price - self.avg_open_price) / self.avg_open_price
            * self.leverage
            * if self.side == "buy" { 1. } else { -1. };
        self.pnl_qty_percent = percent;
        // qty == entry_qty * leverage = self.qty / self.leverage == real pnl
        self.pnl_qty = percent * self.qty / self.leverage;
    }
    pub fn execute_is_reduce(&mut self, order: &Order) -> f64 {
        self.qty -= order.qty;
        if self.qty < 0. {
            self.is_active = false;
            return self.qty * -1.;
        }
        0.
    }
    pub fn execute(&mut self, last_price: f64, order: &Order) {
        self.qty += order.qty;
        self.avg_open_price = (self.avg_open_price + last_price) / 2.;
    }
}

#[cfg(test)]
mod tests {

    use bc_test_kit::prelude::*;

    #[test]
    fn upd_pnl_res_1() {
        let mut pos = POSITION.clone();
        let percent = (pos.avg_open_price - 1.9) / pos.avg_open_price * 10. * -1.;
        pos.upd_pnl(1.9);
        assert_eq_pr!(pos.pnl_qty_percent, percent);
        assert_eq_pr!(pos.pnl_qty, percent / pos.leverage * pos.qty);
    }
}
