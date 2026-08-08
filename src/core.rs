use std::error::Error;

use crate::prelude::*;
use crate::utils::{pnl, price_is_crossed};

pub(crate) fn is_executable(order: &Order, src: &[f64], src_l: &[f64]) -> bool {
    order.is_active
        && (order.price.is_none()
            || price_is_crossed(order.price.unwrap(), src, src_l, &order.type_price_cross))
}

pub(crate) fn modify(
    capital: &mut f64,
    p: &mut Position,
    order: &mut Order,
    src: &[f64],
) -> Result<(), Box<dyn Error>> {
    *capital -= order.commission;
    order.is_active = false;
    if order.is_reduce {
        let bind = p.qty - order.qty;
        let sub = bind
            + if bind <= 0. {
                order.qty
            } else {
                order.qty - bind
            };
        let (_, pnl) = pnl(sub, p.avg_open_price, src[4], p.leverage, &p.side);
        *capital += sub;
        *capital += pnl;
        if bind <= 0. {
            p.is_active = false;
        }
    } else {
        if p.qty < *capital {
            p.qty += order.qty;
            *capital -= order.qty;
            p.avg_open_price = (p.avg_open_price + src[4]) / 2.
        } else {
            return Err("not enough capital to execute".into());
        }
    }
    Ok(())
}

pub(crate) fn insert(
    order: &Order,
    src: &[f64],
    capital: &mut f64,
) -> Result<Position, Box<dyn Error>> {
    if order.qty + order.commission <= *capital {
        *capital -= order.qty + order.commission;
        Ok(Position {
            symbol: order.symbol.to_string(),
            side: order.side.to_string(),
            qty: order.qty,
            leverage: order.leverage,
            avg_open_price: src[4],
            position_idx: order.position_idx,
            is_active: true,
        })
    } else {
        Err("not enough capital to execute".into())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::prelude_tests::prelude::*;

    #[test]
    fn is_executable_res_1() {
        assert_eq_pr!(
            is_executable(
                &Order {
                    price: Some(1.9),
                    type_: "limit".to_string(),
                    ..Default::default()
                },
                &[1.91; 5],
                &[1.89; 5],
            ),
            true
        )
    }

    #[test]
    fn is_executable_res_2() {
        assert_eq_pr!(
            is_executable(
                &Order {
                    type_: "market".to_string(),
                    ..Default::default()
                },
                &[1.91; 5],
                &[1.89; 5],
            ),
            true
        )
    }

    #[test]
    fn modify_res_1() {
        let positions = RefCell::new(MAP::from_iter([(
            1,
            Position {
                side: "sell".to_string(),
                qty: 1.,
                leverage: 2.,
                avg_open_price: 1.87,
                position_idx: 2,
                ..Default::default()
            },
        )]));
        let mut order = Order {
            type_: "market".to_string(),
            side: "sell".to_string(),
            qty: 9.,
            commission: 9. * 0.001,
            type_price_cross: "last".to_string(),
            leverage: 2.,
            ..Default::default()
        };
        let src = vec![1.89; 5];
        let _src_l = vec![1.88; 5];
        let mut t = TradeState {
            capital: 100.,
            positions: positions.clone(),
            ..Default::default()
        };
        modify(
            &mut t.capital,
            t.positions.borrow_mut().get_mut(&1).unwrap(),
            &mut order,
            &src,
        )
        .unwrap();
        assert_eq_pr!(order.is_active, false);
        assert_eq_pr!(
            t,
            TradeState {
                capital: 100. - order.qty - order.commission,
                positions: {
                    let positions_clone = positions.clone();
                    let mut positions_mut = positions_clone.borrow_mut();
                    let bind = positions_mut.get_mut(&1).unwrap();
                    bind.qty += order.qty;
                    bind.avg_open_price = (bind.avg_open_price + src[4]) / 2.;
                    drop(positions_mut);
                    positions_clone
                },
                ..Default::default()
            }
        );
    }

    #[test]
    fn insert_res_1() {
        assert_eq_pr!(
            insert(
                &Order {
                    type_: "market".to_string(),
                    side: "sell".to_string(),
                    qty: 9.,
                    commission: 9. * 0.001,
                    type_price_cross: "last".to_string(),
                    leverage: 2.,
                    ..Default::default()
                },
                &[1.9; 5],
                &mut 100.,
            )
            .unwrap(),
            Position {
                avg_open_price: 1.9,
                side: "sell".to_string(),
                qty: 9.,
                leverage: 2.,
                ..Default::default()
            }
        )
    }

    #[test]
    fn insert_res_2() {
        assert!(
            insert(
                &Order {
                    type_: "market".to_string(),
                    side: "sell".to_string(),
                    qty: 9.,
                    commission: 9. * 0.001,
                    type_price_cross: "last".to_string(),
                    leverage: 2.,
                    ..Default::default()
                },
                &[1.9; 5],
                &mut 9.,
            )
            .is_err()
        )
    }
}
