use std::error::Error;

use bc_utils_lg::{structs::trade::*, types::maps::MAP};

use crate::{core::*, utils::*};

pub trait StepState<'a, 'b> {
    fn step(&mut self, orders: MAP<&'a str, Option<&'b (Order, bool, Option<Trigger>)>>);
    fn execute(&mut self, src: &[f64], src_l: &[f64]) -> Result<(), Box<dyn Error>>;
    fn clear(&mut self);
}

impl<'a, 'b> StepState<'a, 'b> for TradeState<'a> {
    fn step(&mut self, orders: MAP<&'a str, Option<&'b (Order, bool, Option<Trigger>)>>) {
        for (key, order_wrap) in orders {
            if let Some((order, include_in_storage, trigger)) = order_wrap {
                if *include_in_storage {
                    self.orders_storage
                        .borrow_mut()
                        .insert(key, (order.clone(), trigger.clone().unwrap()));
                } else {
                    self.orders.borrow_mut().insert(key, order.clone());
                }
            }
        }
    }
    fn execute(&mut self, src: &[f64], src_l: &[f64]) -> Result<(), Box<dyn Error>> {
        self.orders.borrow_mut().extend(
            self.orders_storage
                .borrow_mut()
                .extract_if(|_, (_, t)| {
                    price_is_crossed_direction(t.price, t.direction, src, src_l, &t.trigger_by)
                })
                .map(|(k, v)| (k, v.0))
                .collect::<MAP<_, _>>(),
        );
        let mut modify_res = Ok(());
        for order in self.orders.borrow_mut().values_mut() {
            if is_executable(order, src, src_l) {
                self.positions
                    .borrow_mut()
                    .entry(order.position_idx)
                    .and_modify(|p| modify_res = modify(&mut self.capital, p, order, src))
                    .or_insert(insert(order, src, &mut self.capital)?);
                order.is_active = false;
            }
        }
        modify_res?;
        Ok(())
    }
    fn clear(&mut self) {
        self.orders.borrow_mut().retain(|_, v| v.is_active);
        self.orders_storage
            .borrow_mut()
            .retain(|_, v| v.0.is_active);
        self.positions.borrow_mut().retain(|_, v| v.is_active);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::prelude_tests::prelude::*;

    #[test]
    fn step_res_1() {
        let mut t = TradeState::new(100.);
        t.step(MAP::from_iter([(
            "order_creator_1",
            Some(&(
                Order {
                    type_: "market".to_string(),
                    ..Default::default()
                },
                false,
                None,
            )),
        )]));
        assert_eq_pr!(
            t,
            TradeState {
                capital: 100.,
                orders: RefCell::new(MAP::from_iter([(
                    "order_creator_1",
                    Order {
                        type_: "market".to_string(),
                        ..Default::default()
                    }
                )])),
                ..Default::default()
            }
        )
    }

    #[test]
    fn execute_res_1() {
        let mut t = TradeState {
            capital: 100.,
            orders: RefCell::new(MAP::from_iter([(
                "order_creator_1",
                Order {
                    type_: "market".to_string(),
                    qty: 10.,
                    commission: 0.001 * 10.,
                    position_idx: 1,
                    side: "buy".to_string(),
                    leverage: 2.,
                    ..Default::default()
                },
            )])),
            ..Default::default()
        };
        t.execute(&[2.; 5], &[1.9; 5]).unwrap();
        assert_eq_pr!(
            t,
            TradeState {
                capital: 100.0 - 10. - 10. * 0.001,
                positions: RefCell::new(MAP::from_iter([(
                    1,
                    Position {
                        side: "buy".to_string(),
                        leverage: 2.,
                        qty: 10.,
                        avg_open_price: 2.,
                        position_idx: 1,
                        ..Default::default()
                    }
                )])),
                orders: RefCell::new(MAP::from_iter([(
                    "order_creator_1",
                    Order {
                        type_: "market".to_string(),
                        qty: 10.,
                        commission: 0.001 * 10.,
                        position_idx: 1,
                        side: "buy".to_string(),
                        leverage: 2.,
                        is_active: false,
                        ..Default::default()
                    },
                )])),
                ..Default::default()
            }
        )
    }

    #[test]
    fn clear_res_1() {
        let mut t = TradeState {
            orders: RefCell::new(MAP::from_iter([(
                "1",
                Order {
                    is_active: false,
                    ..Default::default()
                },
            )])),
            orders_storage: RefCell::new(MAP::from_iter([(
                "1",
                (
                    Order {
                        is_active: false,
                        ..Default::default()
                    },
                    Default::default(),
                ),
            )])),
            positions: RefCell::new(MAP::from_iter([(
                1,
                Position {
                    is_active: false,
                    ..Default::default()
                },
            )])),
            ..Default::default()
        };
        t.clear();
        assert!(t.orders.borrow().is_empty());
        assert!(t.orders_storage.borrow().is_empty());
        assert!(t.positions.borrow().is_empty());
    }
}
