use bc_utils_lg::structs::{settings::SETTINGS_TRADE, trade::*};

use crate::utils_cell::*;
use bc_orders_collectors_gw::gw::OrdersCollectorsGateway;

pub trait StepCell {
    fn step(
        &mut self,
        // 0time 1open 2high 3low 4close 5volume 6turnover 7price_index 8price_mark
        src: &[f64],
        src_l: &[f64],
        orders: Vec<Order>,
        settings: &SETTINGS_TRADE,
        order_collector_gw: &OrdersCollectorsGateway,
    );
    fn clear(&mut self);
}

impl StepCell for TradeCell {
    fn step(
        &mut self,
        src: &[f64],
        src_l: &[f64],
        orders: Vec<Order>,
        settings: &SETTINGS_TRADE,
        order_collector_gw: &OrdersCollectorsGateway,
    ) {
        self.src = src.to_vec();
        self.src_l = src_l.to_vec();
        for mut order in orders {
            // recursive iteration is not required, as no such orders will be created
            self.trigger_orders.borrow_mut().extend(
                order
                    .sl
                    .drain(0..order.sl.len())
                    .map(|v| (v.order_link_id.clone(), v)),
            );
            self.trigger_orders.borrow_mut().extend(
                order
                    .tp
                    .drain(0..order.tp.len())
                    .map(|v| (v.order_link_id.clone(), v)),
            );
            if order.is_trigger() {
                self.trigger_orders
                    .borrow_mut()
                    .insert(order.order_link_id.clone(), order);
            } else if order.is_limit() {
                self.limit_orders
                    .borrow_mut()
                    .insert(order.order_link_id.clone(), order);
            } else {
                self.market_orders
                    .borrow_mut()
                    .insert(order.order_link_id.clone(), order);
            }
        }
        for market_order in self.market_orders.clone().borrow().values() {
            modify_positions(settings, self, market_order);
        }
        for trigger_order in self.trigger_orders.clone().borrow().values() {
            modify_positions_or_not(settings, self, trigger_order);
        }
        for limit_order in self.limit_orders.clone().borrow().values() {
            modify_positions_or_not(settings, self, limit_order);
        }
        order_collector_gw.collect_orders(&self);
    }
    fn clear(&mut self) {
        self.market_orders.borrow_mut().clear();
        self.trigger_orders.borrow_mut().retain(|_, v| v.is_active);
        self.limit_orders.borrow_mut().retain(|_, v| v.is_active);
        self.positions.borrow_mut().retain(|_, v| v.is_active);
    }
}

pub trait IsActive {
    fn is_active(&self) -> bool;
}

impl IsActive for Position {
    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl IsActive for Order {
    fn is_active(&self) -> bool {
        self.is_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bc_orders_collectors_gw::gw::{OrdersCollectors, OrdersCollectorsGateway};

    use crate::prelude_tests::prelude::*;

    const SRC_EL_L3_: [f64; 9] = [1.91; 9];
    const SRC_EL_L2_: [f64; 9] = [1.9; 9];
    const SRC_EL_L1_: [f64; 9] = [2.02; 9];
    const SRC_EL_L_: [f64; 9] = [2.124; 9];
    const SRC_EL_: [f64; 9] = [1.8; 9];
    static SRC_: LazyLock<Vec<Vec<f64>>> = LazyLock::new(|| {
        vec![
            SRC_EL_L3_.to_vec(),
            SRC_EL_L2_.to_vec(),
            SRC_EL_L1_.to_vec(),
            SRC_EL_L_.to_vec(),
            SRC_EL_.to_vec(),
        ]
    });

    #[test]
    fn trade_cell_step_res_1() {
        let mut cell = TradeCell::new(S.trade.capital, SRC_EL_L2_.to_vec(), SRC_EL_L3_.to_vec());
        let order_collectors = OrdersCollectors::new(&S.trade.order_collectors, &FA_O());
        let orders_collectors_gw = OrdersCollectorsGateway::new(&order_collectors);
        let (qty_market, commission_market) =
            qty_and_commission(&S.trade, cell.capital, &SIGNAL, "market", 0., 0.);
        let triggers = vec![
            Order::new(
                "symbol".to_string(),
                "buy".to_string(),
                *SIGNAL,
                0.,
                0.3,
                S.trade.leverage,
                Default::default(),
                "market".to_string(),
                Default::default(),
                Default::default(),
                Some("last".to_string()),
                Some(SRC_[2][1]),
                Some(1),
                true,
                "sdlfsdkl2312".to_string(),
                "1".to_string(),
                true,
            ),
            Order::new(
                "symbol".to_string(),
                "buy".to_string(),
                *SIGNAL,
                0.,
                0.8,
                S.trade.leverage,
                Default::default(),
                "market".to_string(),
                Default::default(),
                Default::default(),
                Some("last".to_string()),
                Some(SRC_[3][1]),
                Some(1),
                true,
                "sdlfsdkl22".to_string(),
                "1".to_string(),
                true,
            ),
            Order::new(
                "symbol".to_string(),
                "buy".to_string(),
                *SIGNAL,
                0.,
                1.,
                S.trade.leverage,
                Default::default(),
                "market".to_string(),
                Default::default(),
                Default::default(),
                Some("last".to_string()),
                Some(SRC_[4][1]),
                Some(2),
                true,
                "wesdlfsdkl2312".to_string(),
                "1".to_string(),
                true,
            ),
        ];
        cell.step(
            &SRC_[1],
            &SRC_[0],
            vec![Order::new(
                "symbol".to_string(),
                "buy".to_string(),
                *SIGNAL,
                qty_market,
                0.,
                S.trade.leverage,
                Some(SRC_[1][1]),
                "market".to_string(),
                triggers[..2].to_vec(),
                triggers[1..].to_vec(),
                Default::default(),
                Default::default(),
                Default::default(),
                false,
                "sdlfsdkl2312".to_string(),
                "1".to_string(),
                true,
            )],
            &S.trade,
            &orders_collectors_gw,
        );
        cell.clear();
        let mut res = TradeCell::new(
            S.trade.capital - commission_market - qty_market,
            SRC_EL_L2_.to_vec(),
            SRC_EL_L3_.to_vec(),
        );
        res.push_position(Position::new(
            "symbol".to_string(),
            "buy".to_string(),
            qty_market,
            S.trade.leverage,
            SRC_[1][1],
            "1".to_string(),
            true,
        ));
        res.push_triggers_orders(triggers.clone());
        assert_eq_pr!(cell, res);
        cell.step(
            &SRC_[2],
            &SRC_[1],
            Default::default(),
            &S.trade,
            &orders_collectors_gw,
        );
        cell.clear();
        res.positions
            .borrow_mut()
            .entry("1".to_string())
            .and_modify(|p| {
                let qty_sub = p.qty * 0.3;
                p.qty -= qty_sub;
                res.capital -= qty_sub * S.trade.commission_market * S.trade.leverage;
                res.capital += qty_sub;
                res.capital += qty_pnl(
                    S.trade.leverage,
                    qty_sub,
                    p.avg_open_price,
                    SRC_[2][4],
                    &p.position_idx,
                );
            });
        res.trigger_orders
            .borrow_mut()
            .remove(triggers[0].order_link_id.as_str());
        res.src = SRC_[2].to_vec();
        res.src_l = SRC_[1].to_vec();
        assert_eq_pr!(cell, res);
        cell.step(
            &SRC_[3],
            &SRC_[2],
            Default::default(),
            &S.trade,
            &orders_collectors_gw,
        );
        cell.clear();
        res.trigger_orders
            .borrow_mut()
            .remove(triggers[1].order_link_id.as_str());
        res.positions
            .borrow_mut()
            .entry("1".to_string())
            .and_modify(|p| {
                let qty_sub = p.qty * 0.8;
                p.qty -= qty_sub;
                res.capital -= qty_sub * S.trade.commission_market * S.trade.leverage;
                res.capital += qty_sub;
                res.capital += qty_pnl(
                    S.trade.leverage,
                    qty_sub,
                    p.avg_open_price,
                    SRC_[3][4],
                    &p.position_idx,
                );
            });
        res.src = SRC_[3].to_vec();
        res.src_l = SRC_[2].to_vec();
        assert_eq_pr!(cell, res);
        cell.step(
            &SRC_[4],
            &SRC_[3],
            Default::default(),
            &S.trade,
            &orders_collectors_gw,
        );
        cell.clear();
        res.trigger_orders
            .borrow_mut()
            .remove(triggers[2].order_link_id.as_str());
        res.positions
            .borrow_mut()
            .entry("1".to_string())
            .and_modify(|p| {
                let qty_sub = p.qty * 1.;
                p.qty -= qty_sub;
                res.capital -= qty_sub * S.trade.commission_market * S.trade.leverage;
                res.capital += qty_sub;
                res.capital += qty_pnl(
                    S.trade.leverage,
                    qty_sub,
                    p.avg_open_price,
                    SRC_[4][4],
                    &p.position_idx,
                );
            });
        res.positions.borrow_mut().remove("1");
        res.src = SRC_[4].to_vec();
        res.src_l = SRC_[3].to_vec();
        assert_eq_pr!(cell, res);
    }
}
