use crate::prelude::*;

pub trait TradeStateExt<'a> {
    fn step(&mut self, orders: &MAP<&'a str, (Option<&OrderWrap>, bool)>);
    fn triggers_to_orders(&mut self, src: &[f64], src_l: &[f64]);
    fn execute(
        &mut self,
        src: &[f64],
        src_l: &[f64],
    ) -> Result<MAP<&'a str, Vec<Order>>, ErrorTrade>;
    fn clear(&mut self);
}

impl<'a> TradeStateExt<'a> for TradeState<'a> {
    fn step(&mut self, orders: &MAP<&'a str, (Option<&OrderWrap>, bool)>) {
        for (key, (order_wrap, use_in_trade)) in orders {
            if let Some(order_wrap) = order_wrap
                && *use_in_trade
            {
                if order_wrap.is_trigger {
                    let trigger = || order_wrap.trigger.as_ref().unwrap().clone();
                    self.orders_trigger
                        .borrow_mut()
                        .entry(key)
                        .and_modify(|v| v.push((order_wrap.order.clone(), trigger())))
                        .or_insert_with(|| vec![(order_wrap.order.clone(), trigger())]);
                } else {
                    self.orders
                        .borrow_mut()
                        .entry(key)
                        .and_modify(|v| v.push(order_wrap.order.clone()))
                        .or_insert_with(|| vec![order_wrap.order.clone()]);
                }
            }
        }
    }
    fn triggers_to_orders(&mut self, src: &[f64], src_l: &[f64]) {
        for (k, order) in self.orders_trigger.borrow_mut().iter_mut() {
            self.orders.borrow_mut().entry(*k).and_modify(|v| {
                v.extend(
                    order
                        .extract_if(.., |(_, t)| {
                            price_is_crossed_direction(
                                t.price,
                                t.direction,
                                src,
                                src_l,
                                &t.trigger_by,
                            )
                        })
                        .map(|v| v.0),
                );
            });
        }
    }
    fn execute(
        &mut self,
        src: &[f64],
        src_l: &[f64],
    ) -> Result<MAP<&'a str, Vec<Order>>, ErrorTrade> {
        let mut res = MAP::default();
        for (id, orders) in self.orders.borrow_mut().iter_mut() {
            res.insert(
                *id,
                orders
                    .extract_if(.., |order| is_to_execute(order, src, src_l))
                    .map(|mut order| {
                        is_executable(self.capital, &order)?;
                        let last_price = src[4];
                        self.positions
                            .borrow_mut()
                            .entry(order.position_idx)
                            .and_modify(|p| modify(&mut self.capital, p, &order, last_price))
                            .or_insert(insert(&order, last_price, &mut self.capital));
                        order.is_active = false;
                        order.price = Some(last_price);
                        Ok(order)
                    })
                    .collect::<Result<Vec<Order>, ErrorTrade>>()?,
            );
        }
        Ok(res)
    }
    fn clear(&mut self) {
        for orders in self.orders.borrow_mut().values_mut() {
            orders.retain(|v| v.is_active);
        }
        for orders in self.orders_trigger.borrow_mut().values_mut() {
            orders.retain(|v| v.0.is_active);
        }
        self.positions.borrow_mut().retain(|_, v| v.is_active);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_state::trade_state::*;
    use bc_order_filters_gw::test_state::*;
    use bc_test_kit::prelude::*;

    #[test]
    fn step_res_1() {
        let mut t = TRADE_STATE_EMPTY();
        t.step(&ORDER_FILTERS_STATE());
        assert_eq_pr!(&t, &TRADE_STATE());
        assert_eq_pr!(t.orders_trigger.borrow()["count_3"].len(), 1);
    }

    #[test]
    fn triggers_to_orders_res_1() {
        let mut t = TRADE_STATE_EMPTY();
        t.step(&ORDER_FILTERS_STATE());
        t.triggers_to_orders(&SRC_EL, &SRC_EL1);
        assert_eq_pr!(&t, &TRADE_STATE());
        assert_eq_pr!(t.orders_trigger.borrow()["count_3"].len(), 1);
    }

    #[test]
    fn execute_res_1() {
        let mut t = TRADE_STATE();
        t.triggers_to_orders(&SRC_EL, &SRC_EL1);
        t.execute(&SRC_EL, &SRC_EL1).unwrap();
        assert!(!t.positions.borrow().is_empty());
        assert_eq_pr!(t.orders_trigger.borrow()["count_3"].len(), 1);
    }

    #[test]
    fn clear_res_1() {
        let mut t = TRADE_STATE();
        t.step(&ORDER_FILTERS_STATE());
        t.triggers_to_orders(&SRC_EL, &SRC_EL1);
        t.execute(&SRC_EL, &SRC_EL1).unwrap();
        assert!(!t.positions.borrow().is_empty());
        assert!(!t.orders.borrow().is_empty());
        assert!(!t.orders_trigger.borrow().is_empty());
        for v in t.positions.borrow_mut().values_mut() {
            v.is_active = false;
        }
        for v in t.orders.borrow_mut().values_mut() {
            for v in v.iter_mut() {
                v.is_active = false;
            }
        }
        for v in t.orders_trigger.borrow_mut().values_mut() {
            for (v, _) in v.iter_mut() {
                v.is_active = false;
            }
        }
        t.clear();
        assert!(t.positions.borrow().is_empty());
        for orders in t.orders.borrow().values() {
            assert!(orders.is_empty());
        }
        for orders in t.orders_trigger.borrow().values() {
            assert!(orders.is_empty());
        }
    }
}
