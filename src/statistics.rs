use std::cell::Ref;
use std::ops::Deref;

use bc_utils::other::transpose;
use bc_utils_lg::structs::trade::{Position, TradeCell};
use bc_utils_lg::types::maps::MAP;
use bc_utils_lg::{structs::settings::SETTINGS_TRADE, types::maps::MAP_LINK};
use num_traits::Float;

use crate::structs::IsActive;
use crate::utils_cell::{price_is_real_time, qty_pnl};

#[derive(Debug)]
pub struct StatCollector<'a> {
    pub symbol: String,
    pub cells: Vec<TradeCell>,
    s: &'a SETTINGS_TRADE,
}

impl<'a> StatCollector<'a> {
    pub fn new(
        symbol: String,
        s: &'a SETTINGS_TRADE,
    ) -> Self {
        Self { symbol, cells: Vec::new(), s }
    }
    pub fn push(
        &mut self,
        cell: TradeCell,
    ) {
        self.cells.push(cell);
    }
}

impl<'a> IntoIterator for &'a StatCollector<'a> {
    type Item = &'a TradeCell;
    type IntoIter = std::slice::Iter<'a, TradeCell>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.cells).into_iter()
    }
}

impl StatCollector<'_> {
    pub fn to_all(values: &[Vec<f64>]) -> Vec<f64> {
        let first = values.first().unwrap();
        (0..first.len())
            .map(|i| {
                if values.iter().all(|v| v[i].is_normal()) {
                    first[i]
                } else {
                    f64::NAN
                }
            })
            .collect()
    }
    pub fn to_any(values: &[Vec<f64>]) -> Vec<f64> {
        (0..values.first().unwrap().len())
            .map(|i| {
                let bind = values.iter().map(|v| v[i]).find(|v| v.is_normal());
                if let Some(el) = bind {
                    el
                } else {
                    f64::NAN
                }
            })
            .collect()
    }
    pub fn to_some<T>(
        &self,
        func: fn(&TradeCell) -> Ref<MAP<String, T>>,
        include_inactive: bool,
    ) -> Vec<f64>
    where
        T: IsActive,
    {
        let f = |v: Ref<MAP<String, T>>| {
            if include_inactive {
                v.values().any(|v| v.is_active())
            } else {
                !v.is_empty()
            }
        };
        self.cells
            .iter()
            .map(|c| {
                if f(func(c)) {
                    // stat used open prices
                    c.src[1]
                } else {
                    f64::NAN
                }
            })
            .collect()
    }
    pub fn to_capital(&self) -> Vec<f64> {
        self.into_iter().map(|c| c.capital).collect()
    }
    pub fn to_pnl(&self) -> Vec<f64> {
        self.into_iter()
            .map(|c| {
                let positions = c.positions.borrow();
                if positions.is_empty() {
                    f64::NAN
                } else {
                    let position = positions.values().next().unwrap();
                    qty_pnl(
                        self.s.leverage,
                        position.qty,
                        position.avg_open_price,
                        price_is_real_time(self.s.work_in_real_time, &c.src),
                        &position.position_idx,
                    )
                }
            })
            .collect()
    }
    pub fn to_entry(&self) -> Vec<f64> {
        StatCollector::to_all(&[
            StatCollector::to_any(&[
                self.to_some(|c| c.market_orders.borrow(), true),
                self.into_iter()
                    .map(|c| {
                        if c.limit_orders
                            .borrow()
                            .values()
                            .any(|v| v.is_active == false)
                        {
                            c.src[1]
                        } else {
                            f64::NAN
                        }
                    })
                    .collect(),
            ]),
            self.to_some(|c| c.positions.borrow(), true),
        ])
    }
    pub fn to_exit(&self) -> Vec<f64> {
        self.into_iter()
            .map(|c| {
                if c.positions.borrow().values().any(|v| !v.is_active) {
                    c.src[1]
                } else {
                    f64::NAN
                }
            })
            .collect()
    }
    pub fn to_market_orders(&self) -> Vec<f64> {
        self.to_some(|c| c.market_orders.borrow(), true)
    }
    pub fn to_limit_orders(&self) -> Vec<f64> {
        self.to_some(|c| c.limit_orders.borrow(), true)
    }
    pub fn to_entry_and_exit(&self) -> Vec<f64> {
        self.to_entry()
            .into_iter()
            .zip(self.to_exit().into_iter())
            .map(|(v1, v2)| {
                let bind = [v1, v2].into_iter().find(|v| v.is_normal());
                if let Some(v) = bind {
                    v
                } else {
                    f64::NAN
                }
            })
            .collect()
    }
    pub fn to_positions_entry_exit(&self) -> Vec<f64> {
        StatCollector::to_all(&[
            self.to_some(|v| v.positions.borrow(), false),
            self.to_entry_and_exit(),
        ])
    }
    pub fn to_value_positions(
        &self,
        func: fn(&Position) -> f64,
    ) -> Vec<f64> {
        StatCollector::to_all(&[
            self.into_iter()
                .map(|c| {
                    if !c.positions.borrow().is_empty() {
                        func(&c.positions.borrow().values().next().unwrap())
                    } else {
                        f64::NAN
                    }
                })
                .collect(),
            self.to_entry_and_exit(),
        ])
    }
    pub fn to_data(&self) -> StatData {
        StatData(vec![
            MAP_LINK::from_iter([
                (
                    "time".to_string(),
                    (0..self.cells.len())
                        .map(|v| v as f64)
                        .collect::<Vec<f64>>(),
                ),
                (
                    "open".to_string(),
                    self.into_iter().map(|v| v.src[1]).collect(),
                ),
                (
                    "high".to_string(),
                    self.into_iter().map(|v| v.src[2]).collect(),
                ),
                (
                    "low".to_string(),
                    self.into_iter().map(|v| v.src[3]).collect(),
                ),
                (
                    "close".to_string(),
                    self.into_iter().map(|v| v.src[4]).collect(),
                ),
                (
                    "volume".to_string(),
                    self.into_iter().map(|v| v.src[5]).collect(),
                ),
                (
                    "turnover".to_string(),
                    self.into_iter().map(|v| v.src[6]).collect(),
                ),
                ("capital".to_string(), self.to_capital()),
                ("entry".to_string(), self.to_entry()),
                ("exit".to_string(), self.to_exit()),
                (
                    "pnl".to_string(),
                    StatCollector::to_all(&[self.to_pnl(), self.to_exit()]),
                ),
                ("qty".to_string(), self.to_value_positions(|v| v.qty)),
            ]),
            {
                let mut bind = transpose(
                    self.to_positions_entry_exit()
                        .into_iter()
                        .del_nan(1)
                        .map(|(time, pos)| vec![time as f64, pos])
                        .collect::<Vec<Vec<f64>>>(),
                );
                if !bind.is_empty() {
                    MAP_LINK::from_iter([
                        ("time".to_string(), bind.remove(0)),
                        ("positions_entry_exit".to_string(), bind.remove(0)),
                    ])
                } else {
                    Default::default()
                }
            },
        ])
    }
}

#[derive(PartialEq, Debug, Default)]
pub struct StatData(pub Vec<MAP_LINK<String, Vec<f64>>>);

impl StatData {
    pub fn to_vec(&self) -> Vec<Vec<Vec<f64>>> {
        self.0
            .iter()
            .map(|v| v.iter().map(|v| v.1.clone()).collect::<Vec<Vec<f64>>>())
            .collect::<Vec<Vec<Vec<f64>>>>()
    }
}

impl Deref for StatData {
    type Target = Vec<MAP_LINK<String, Vec<f64>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait NumsExt<T> {
    fn del_nan(
        self,
        sep: usize,
    ) -> impl Iterator<Item = (usize, T)>;
}

impl<T: Float + Default, V: Iterator<Item = T>> NumsExt<T> for V {
    fn del_nan(
        self,
        sep: usize,
    ) -> impl Iterator<Item = (usize, T)> {
        self.enumerate()
            .scan(0usize, move |num, el| {
                if el.1.is_normal() {
                    Some((el.0, el.1))
                } else {
                    *num += 1;
                    if *num <= sep {
                        Some((el.0, el.1))
                    } else {
                        *num = 0;
                        Some((Default::default(), T::nan()))
                    }
                }
            })
            .filter(|v| v.1.is_normal())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    use bc_utils::nums::nz_coll;
    use bc_utils_lg::types::maps::MAP_LINK;

    use crate::prelude_tests::prelude::*;

    static ST: LazyLock<fn() -> StatCollector<'static>> = LazyLock::new(|| {
        || {
            let mut bind = StatCollector::new("".to_string(), &S.trade);
            bind.cells.extend_from_slice(&[
                TradeCell {
                    capital: 100.,
                    src: SRC_EL1.to_vec(),
                    src_l: SRC_EL2.to_vec(),
                    trigger_orders: RefCell::new(MAP::from_iter([(
                        "1".to_string(),
                        Order::new(
                            "".to_string(),
                            "buy".to_string(),
                            *SIGNAL,
                            S.trade.capital * S.trade.percent_of_capital,
                            0.,
                            S.trade.leverage,
                            Some(3.),
                            "limit".to_string(),
                            Default::default(),
                            Default::default(),
                            None,
                            None,
                            None,
                            false,
                            "".to_string(),
                            "1".to_string(),
                            true,
                        ),
                    )])),
                    positions: RefCell::new(MAP::from_iter([(
                        "1".to_string(),
                        Position::new(
                            "".to_string(),
                            "buy".to_string(),
                            S.trade.capital * S.trade.percent_of_capital,
                            S.trade.leverage,
                            1.7,
                            "1".to_string(),
                            true,
                        ),
                    )])),
                    market_orders: RefCell::new(MAP::from_iter([(
                        "1".to_string(),
                        Order::new(
                            "".to_string(),
                            "".to_string(),
                            *SIGNAL,
                            S.trade.capital * S.trade.percent_of_capital,
                            0.,
                            S.trade.leverage,
                            None,
                            "market".to_string(),
                            Default::default(),
                            Default::default(),
                            None,
                            None,
                            None,
                            false,
                            "1".to_string(),
                            "1".to_string(),
                            true,
                        ),
                    )])),
                    ..Default::default()
                },
                TradeCell {
                    capital: 100.,
                    src: SRC_EL.to_vec(),
                    src_l: SRC_EL1.to_vec(),
                    positions: RefCell::new(MAP::from_iter([(
                        "1".to_string(),
                        Position::new(
                            "".to_string(),
                            "buy".to_string(),
                            S.trade.capital * S.trade.percent_of_capital,
                            S.trade.leverage,
                            1.7,
                            "1".to_string(),
                            false,
                        ),
                    )])),
                    market_orders: RefCell::new(MAP::from_iter([(
                        "1".to_string(),
                        Order::new(
                            "".to_string(),
                            "".to_string(),
                            *SIGNAL,
                            S.trade.capital * S.trade.percent_of_capital,
                            0.,
                            S.trade.leverage,
                            None,
                            "market".to_string(),
                            Default::default(),
                            Default::default(),
                            None,
                            None,
                            None,
                            true,
                            "1".to_string(),
                            "1".to_string(),
                            true,
                        ),
                    )])),
                    ..Default::default()
                },
            ]);
            bind
        }
    });

    #[test]
    fn to_all_res_1() {
        assert_eq_pr!(
            vec![1., 0.,],
            nz_coll::<Vec<f64>, _, _>(
                &StatCollector::to_all(&[vec![1., f64::NAN,], vec![1., 2.,]]),
                0.
            )
        )
    }

    #[test]
    fn to_any_res_1() {
        assert_eq_pr!(
            vec![1., 2.,],
            StatCollector::to_any(&[vec![1., f64::NAN,], vec![1., 2.,]])
        )
    }

    #[test]
    fn to_some_res_1() {
        assert_eq_pr!(
            vec![SRC_EL1[1], 0.0],
            nz_coll::<Vec<_>, _, _>(&ST().to_some(|v| v.trigger_orders.borrow(), true), 0.)
        )
    }

    #[test]
    fn to_capital_res_1() {
        assert_eq_pr!(vec![100., 100.,], ST().to_capital())
    }

    #[test]
    fn to_pnl_res_1() {
        assert_eq_pr!(
            vec![
                qty_pnl(
                    S.trade.leverage,
                    S.trade.capital * S.trade.percent_of_capital,
                    1.7,
                    SRC_EL1[1],
                    "1"
                ),
                qty_pnl(
                    S.trade.leverage,
                    S.trade.capital * S.trade.percent_of_capital,
                    1.7,
                    SRC_EL[1],
                    "1"
                )
            ],
            ST().to_pnl()
        )
    }

    #[test]
    fn to_entry_res_1() {
        assert_eq_pr!(
            vec![SRC_EL1[1], 0.],
            nz_coll::<Vec<f64>, _, _>(&ST().to_entry(), 0.)
        )
    }

    #[test]
    fn to_exit_res_1() {
        assert_eq_pr!(
            vec![0., SRC_EL[1]],
            nz_coll::<Vec<f64>, _, _>(&ST().to_exit(), 0.)
        )
    }

    #[test]
    fn to_market_orders_res_1() {
        assert_eq_pr!(
            ST().to_some(|c| c.market_orders.borrow(), true),
            ST().to_market_orders()
        )
    }

    #[test]
    fn to_limit_orders_res_1() {
        assert_eq_pr!(
            nz_coll::<Vec<_>, _, _>(&ST().to_some(|c| c.limit_orders.borrow(), true), 0.),
            nz_coll::<Vec<_>, _, _>(&ST().to_limit_orders(), 0.,),
        )
    }

    #[test]
    fn to_entry_and_exit_res_1() {
        assert_eq_pr!(vec![SRC_EL1[1], SRC_EL[1],], ST().to_entry_and_exit())
    }

    #[test]
    fn to_positions_entry_exit_res_1() {
        assert_eq_pr!(vec![SRC_EL1[1], SRC_EL[1]], ST().to_positions_entry_exit(),)
    }

    #[test]
    fn to_value_positions_res_1() {
        assert_eq_pr!(
            vec![S.trade.capital * S.trade.percent_of_capital; 2],
            ST().to_value_positions(|v| v.qty)
        )
    }

    #[test]
    fn to_data_res_1() {
        assert_eq_pr!(
            StatData(vec![
                MAP_LINK::from_iter([
                    ("time".to_string(), vec![0., 1.,]),
                    ("open".to_string(), OPEN[48..].to_vec()),
                    ("high".to_string(), HIGH[48..].to_vec()),
                    ("low".to_string(), LOW[48..].to_vec()),
                    ("close".to_string(), CLOSE[48..].to_vec()),
                    ("volume".to_string(), VOLUME[48..].to_vec()),
                    ("turnover".to_string(), TURNOVER[48..].to_vec()),
                    ("capital".to_string(), ST().to_capital()),
                    ("entry".to_string(), vec![SRC_EL1[1], 0.,]),
                    ("exit".to_string(), vec![0., SRC_EL[1],]),
                    (
                        "pnl".to_string(),
                        nz_coll::<Vec<_>, _, _>(
                            &StatCollector::to_all(&[ST().to_pnl(), ST().to_exit(),]),
                            0.
                        ),
                    ),
                    ("qty".to_string(), ST().to_value_positions(|v| v.qty)),
                ]),
                MAP_LINK::from_iter([
                    ("time".to_string(), vec![0., 1.,]),
                    (
                        "positions_entry_exit".to_string(),
                        vec![SRC_EL1[1], SRC_EL[1]]
                    )
                ])
            ]),
            {
                let mut bind = ST().to_data();
                bind.0[0]["entry"] = nz_coll::<Vec<_>, _, _>(&bind.0[0]["entry"], 0.);
                bind.0[0]["exit"] = nz_coll::<Vec<_>, _, _>(&bind.0[0]["exit"], 0.);
                bind.0[0]["pnl"] = nz_coll::<Vec<_>, _, _>(&bind.0[0]["pnl"], 0.);
                bind
            }
        )
    }

    #[test]
    fn del_nan_res_1() {
        assert_eq_pr!(
            vec![(0usize, 1.,)],
            vec![1., f64::NAN]
                .into_iter()
                .del_nan(0)
                .collect::<Vec<(usize, f64)>>()
        )
    }
}
