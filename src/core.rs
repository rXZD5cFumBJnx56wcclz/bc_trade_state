use crate::prelude::*;

pub(crate) fn is_to_execute(order: &Order, src: &[f64], src_l: &[f64]) -> bool {
    order.is_active
        && (order.price.is_none()
            || price_is_crossed(order.price.unwrap(), src, src_l, &order.type_price_cross))
}

pub(crate) fn is_executable(capital: Capital, order: &Order) -> Result<(), ErrorTrade> {
    if !order.is_reduce {
        if capital - (order.qty / order.leverage + order.commission) < 0. {
            return Err(ErrorTrade::NotEnoughCapital(
                capital,
                order.qty,
                order.commission,
            ));
        }
    }
    Ok(())
}

pub(crate) fn modify(capital: &mut Capital, p: &mut Position, order: &Order, last_price: f64) {
    if order.is_reduce {
        capital.execute_order_reduce(order, p.execute_is_reduce(order));
    } else {
        p.execute(last_price, order);
        capital.execute_order(order);
    }
    p.upd_pnl(last_price);
}

pub(crate) fn insert(order: &Order, last_price: f64, capital: &mut Capital) -> Position {
    capital.execute_order(order);
    Position::new(
        order.symbol.to_string(),
        order.side.to_string(),
        order.qty,
        order.leverage,
        last_price,
        order.position_idx,
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use bc_test_kit::prelude::*;

    #[test]
    fn is_to_execute_res_1() {
        assert_eq_pr!(
            is_to_execute(
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
    fn is_to_execute_res_2() {
        assert_eq_pr!(
            is_to_execute(
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
        let last_price = src[4];
        let mut t = TradeState {
            capital: Capital(100.),
            positions: positions.clone(),
            ..Default::default()
        };
        modify(
            &mut t.capital,
            t.positions.borrow_mut().get_mut(&1).unwrap(),
            &mut order,
            last_price,
        );
        assert_eq_pr!(
            t,
            TradeState {
                capital: Capital(100. - order.qty / order.leverage - order.commission),
                positions: {
                    let positions_clone = positions.clone();
                    let mut positions_mut = positions_clone.borrow_mut();
                    let bind = positions_mut.get_mut(&1).unwrap();
                    bind.qty += order.qty;
                    bind.avg_open_price = (bind.avg_open_price + last_price) / 2.;
                    bind.upd_pnl(last_price);
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
                1.9,
                &mut Capital(100.),
            ),
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
    fn is_executable_res_1() {
        assert!(
            is_executable(
                Capital(4.5),
                &Order {
                    type_: "market".to_string(),
                    side: "sell".to_string(),
                    qty: 9.,
                    commission: 9. * 0.001,
                    type_price_cross: "last".to_string(),
                    leverage: 2.,
                    ..Default::default()
                },
            )
            .is_err()
        )
    }
}
