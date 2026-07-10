use std::borrow::Borrow;
use std::cell::RefCell;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr;

use crate::{statistics::StatCollector, trade::StepCell, utils_cell::orders_create};
use bc_indicators::main_trait::Indicator;
use bc_orders_collectors::main_trait::OrderCollector;
use bc_orders_collectors_gw::gw::{OrdersCollectors, OrdersCollectorsGateway};
use bc_signals::ready::main_trait::SignalReady;
use bc_signals::train::main_trait::SignalTrain;
use bc_utils_lg::types::maps::MAP;
use bc_utils_lg::{
    structs::{
        settings::{SETTINGS, SETTINGS_IND, SETTINGS_ORDER_COLLECTOR, SETTINGS_SIGNAL},
        trade::TradeCell,
    },
    types::maps::FUNCS_EXTRACT_ARGS_TYPE as FA,
};

use crate::buffer::Buffer;
use bc_indicators_gw::gw::{Indicators, IndicatorsGateway};
use bc_signals_gw::gw_ready::{SignalsReady, SignalsReadyGateway};
use bc_signals_gw::gw_train::{SignalsTrain, SignalsTrainGateway};

#[derive(Default)]
pub struct GWValues<'a> {
    pub indicators: RefCell<Indicators<'a>>,
    pub signals_ready: RefCell<SignalsReady<'a>>,
    pub signals_train: RefCell<SignalsTrain<'a>>,
    pub orders_collectors: OrdersCollectors,
}

impl<'a> GWValues<'a> {
    pub fn new(
        s: &'a SETTINGS,
        fa_indicators: &FA<SETTINGS_IND, Box<dyn Indicator>>,
        fa_signals_ready: &FA<SETTINGS_SIGNAL, Box<dyn SignalReady>>,
        fa_signals_train: &FA<SETTINGS_SIGNAL, Box<dyn SignalTrain>>,
        fa_orders_collectors: &FA<SETTINGS_ORDER_COLLECTOR, Box<dyn OrderCollector>>,
        // transposed
        src: &[Vec<f64>],
    ) -> Self {
        let bind = Indicators::new(&s.indications, fa_indicators, src);
        Self {
            signals_ready: RefCell::new(SignalsReady::new(
                &s.signals_ready,
                &s.indications,
                fa_signals_ready,
                src,
                &bind.indicators_without_bf,
            )),
            signals_train: RefCell::new(SignalsTrain::new(
                &s.signals_train,
                &s.indications,
                fa_signals_train,
                src,
                &bind.indicators_without_bf,
            )),
            indicators: RefCell::new(bind),
            orders_collectors: OrdersCollectors::new(
                &s.trade.order_collectors,
                fa_orders_collectors,
            ),
        }
    }
}

pub struct TradeData<'a, 'b> {
    pub gw_values: GWValues<'a>,
    pub indicators_gateway: IndicatorsGateway<'a>,
    pub signals_ready_gateway: SignalsReadyGateway<'a>,
    pub signals_train_gateway: SignalsTrainGateway<'a>,
    pub orders_collectors_gateway: OrdersCollectorsGateway,
    pub cell: RefCell<TradeCell>,
    pub symbol: &'b str,
    pub s: &'a SETTINGS,
    _pin: PhantomPinned,
}

impl<'a, 'b> TradeData<'a, 'b> {
    pub fn new(
        src: &mut Buffer,
        s: &'a SETTINGS,
        symbol: &'b str,
        fa_indicators: &FA<SETTINGS_IND, Box<dyn Indicator>>,
        fa_signals_ready: &FA<SETTINGS_SIGNAL, Box<dyn SignalReady>>,
        fa_signals_train: &FA<SETTINGS_SIGNAL, Box<dyn SignalTrain>>,
        fa_orders_collectors: &FA<SETTINGS_ORDER_COLLECTOR, Box<dyn OrderCollector>>,
    ) -> Pin<Box<Self>> {
        let mut res = Box::pin(Self {
            gw_values: {
                src.transpose_set();
                let bind = GWValues::new(
                    s,
                    fa_indicators,
                    fa_signals_ready,
                    fa_signals_train,
                    fa_orders_collectors,
                    src,
                );
                src.transpose_set();
                bind
            },
            cell: RefCell::new(TradeCell::new(
                s.trade.capital,
                src[src.len() - 1].to_vec(),
                src[src.len() - 2].to_vec(),
            )),
            symbol: symbol,
            s: s,
            indicators_gateway: IndicatorsGateway::new(std::ptr::null(), &s.indications),
            signals_ready_gateway: SignalsReadyGateway::new(
                ptr::null(),
                ptr::null(),
                &s.signals_ready,
                &s.indications,
            ),
            signals_train_gateway: SignalsTrainGateway::new(
                ptr::null(),
                ptr::null(),
                &s.signals_train,
                &s.indications,
            ),
            orders_collectors_gateway: OrdersCollectorsGateway::new(ptr::null()),
            _pin: PhantomPinned,
        });
        let outer_mut = unsafe { Pin::as_mut(&mut res).get_unchecked_mut() };
        outer_mut.indicators_gateway.indicators = &*outer_mut.gw_values.indicators.borrow();
        outer_mut.signals_ready_gateway.indicators = &*outer_mut.gw_values.indicators.borrow();
        outer_mut.signals_train_gateway.indicators = &*outer_mut.gw_values.indicators.borrow();
        outer_mut.signals_ready_gateway.signals_ready =
            &*outer_mut.gw_values.signals_ready.borrow();
        outer_mut.signals_train_gateway.signals_train =
            &*outer_mut.gw_values.signals_train.borrow();
        outer_mut.orders_collectors_gateway.orders_collectors =
            &outer_mut.gw_values.orders_collectors;
        res
    }
    pub fn update(
        self: &Pin<Box<Self>>,
        buffer: &mut Buffer,
        stat_collector: Option<&mut StatCollector<'a>>,
    ) {
        buffer.transpose_set();
        let indications = self.indicators_gateway.indications_series(&buffer);
        let signals_ready = self
            .signals_ready_gateway
            .signals_series(&indications, &buffer);
        buffer.transpose_set();
        let orders = orders_create(
            &self.s.trade,
            self.cell.borrow().borrow(),
            self.symbol,
            &indications,
            &signals_ready,
            buffer.as_slice(),
        );
        dbg!(&orders);
        self.as_ref().get_ref().cell.borrow_mut().step(
            buffer.as_slice().last().unwrap(),
            &buffer[buffer.len() - 2],
            orders,
            &self.as_ref().get_ref().s.trade,
            &self.as_ref().get_ref().orders_collectors_gateway,
        );
        if let Some(st) = stat_collector {
            // fix train signals
            st.push(
                self.cell.borrow().clone(),
                indications,
                signals_ready,
                Default::default(),
            );
        }
        self.as_ref().get_ref().cell.borrow_mut().clear();
    }
    pub fn update_bf(
        self: &Pin<Box<Self>>,
        buffer: &[Vec<f64>],
        fa_indicators: &FA<SETTINGS_IND, Box<dyn Indicator>>,
        fa_signals_ready: &FA<SETTINGS_SIGNAL, Box<dyn SignalReady>>,
        fa_signals_train: &FA<SETTINGS_SIGNAL, Box<dyn SignalTrain>>,
    ) {
        self.as_ref().gw_values.indicators.borrow_mut().update_bf(
            buffer,
            &(&*self.borrow().s).indications,
            &fa_indicators,
        );
        self.gw_values.signals_ready.borrow_mut().update_bf(
            buffer,
            &self.s,
            &fa_signals_ready,
            &self.gw_values.indicators.borrow().indicators_without_bf,
        );
        self.gw_values.signals_train.borrow_mut().update_bf(
            buffer,
            &self.s,
            &fa_signals_train,
            &self.gw_values.indicators.borrow().indicators_without_bf,
        );
    }
}

pub struct AfterTradeData<'a> {
    pub indicators_values: Indicators<'a>,
    pub indicators_columns: Indicators<'a>,
    pub indicators_gateway_values: IndicatorsGateway<'a>,
    pub indicators_gateway_columns: IndicatorsGateway<'a>,
    _pin: PhantomPinned,
}

impl<'a> AfterTradeData<'a> {
    pub fn new(
        s: &'a SETTINGS,
        src: &[Vec<f64>],
        fa: &FA<SETTINGS_IND, Box<dyn Indicator>>,
    ) -> Pin<Box<Self>> {
        let mut res = Box::pin(Self {
            indicators_values: Indicators::new(&s.indications_stat_values, fa, src),
            indicators_columns: Indicators::new(&s.indications_stat_columns, fa, src),
            indicators_gateway_values: IndicatorsGateway {
                indicators: ptr::null(),
                settings: &s.indications_stat_values,
            },
            indicators_gateway_columns: IndicatorsGateway {
                indicators: ptr::null(),
                settings: &s.indications_stat_columns,
            },
            _pin: PhantomPinned,
        });
        let outer_pin = unsafe { res.as_mut().get_unchecked_mut() };
        outer_pin.indicators_gateway_values.indicators = &outer_pin.indicators_values;
        outer_pin.indicators_gateway_columns.indicators = &outer_pin.indicators_columns;
        res
    }
    pub fn to_stat_values(
        &self,
        data: &[Vec<f64>],
    ) -> MAP<&'a str, f64> {
        self.indicators_gateway_values.indications_series(data)
    }
    pub fn to_stat_columns(
        &self,
        data: &[Vec<f64>],
    ) -> MAP<&'a str, Vec<f64>> {
        self.indicators_gateway_columns.indications_vec(data)
    }
    pub fn to_all(
        s: &'a SETTINGS,
        src: &[Vec<f64>],
        fa: &FA<SETTINGS_IND, Box<dyn Indicator>>,
    ) -> (MAP<&'a str, f64>, MAP<&'a str, Vec<f64>>) {
        let bind = AfterTradeData::new(s, src, fa);
        (bind.to_stat_values(src), bind.to_stat_columns(src))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Borrow;
    use std::pin::Pin;

    use crate::buffer::ToBuff;
    use bc_indicators::prelude::BF_INDICATOR;
    use bc_pack_indicators::FUNCS_EXTRACT_ARGS as FA_I;
    use bc_pack_signals_ready::FUNCS_EXTRACT_ARGS as FA_R;
    use bc_pack_signals_train::FUNCS_EXTRACT_ARGS as FA_T;
    use bc_signals::def_impl::BF_SIGNALS;

    use crate::prelude_tests::prelude::*;

    static TD: LazyLock<fn() -> Pin<Box<TradeData<'static, 'static>>>> = LazyLock::new(|| {
        || {
            TradeData::new(
                &mut SRC.to_buff(),
                &S,
                "",
                &FA_I(),
                &FA_R(),
                &FA_T(),
                &FA_O(),
            )
        }
    });

    #[test]
    fn update_res_1() {
        let td = TD();
        let res = TD();
        td.update(&mut SRC.to_buff(), None);
        let indications = res.indicators_gateway.indications_series(&SRC_TRANSPOSE);
        let orders = orders_create(
            &S.trade,
            res.cell.borrow().borrow(),
            res.symbol,
            &indications,
            &res.signals_ready_gateway
                .signals_series(&indications, &SRC_TRANSPOSE),
            &SRC,
        );
        res.as_ref().get_ref().cell.borrow_mut().step(
            SRC.last().unwrap(),
            &SRC[SRC.len() - 2],
            orders,
            &res.as_ref().get_ref().s.trade,
            &res.as_ref().get_ref().orders_collectors_gateway,
        );
        let td_ref = td.as_ref().get_ref();
        let res_ref = res.as_ref().get_ref();
        assert_eq_pr!(
            unsafe { &*td_ref.indicators_gateway.indicators }
                .indicators
                .values()
                .map(|(v1, _)| v1)
                .collect::<Vec<&BF_INDICATOR>>(),
            unsafe { &*res_ref.indicators_gateway.indicators }
                .indicators
                .values()
                .map(|(v1, _)| v1)
                .collect::<Vec<&BF_INDICATOR>>()
        );
        assert_eq_pr!(
            unsafe { &*td_ref.signals_ready_gateway.signals_ready }
                .signals_ready
                .values()
                .map(|(v1, _)| v1)
                .collect::<Vec<&BF_SIGNALS>>(),
            unsafe { &*res_ref.signals_ready_gateway.signals_ready }
                .signals_ready
                .values()
                .map(|(v1, _)| v1)
                .collect::<Vec<&BF_SIGNALS>>()
        );
        assert_eq_pr!(
            unsafe { &*td_ref.signals_train_gateway.signals_train }
                .signals_train
                .values()
                .map(|(v1, _)| v1)
                .collect::<Vec<&BF_SIGNALS>>(),
            unsafe { &*res_ref.signals_train_gateway.signals_train }
                .signals_train
                .values()
                .map(|(v1, _)| v1)
                .collect::<Vec<&BF_SIGNALS>>()
        );
    }

    #[test]
    fn update_bf_res_1() {
        let td = TD();
        let res = TD();
        // fix fa
        td.update_bf(&SRC_TRANSPOSE, &FA_I(), &FA_R(), &FA_T());
        res.as_ref().gw_values.indicators.borrow_mut().update_bf(
            &SRC_TRANSPOSE,
            &(&*res.borrow().s).indications,
            &FA_I(),
        );
        res.gw_values.signals_ready.borrow_mut().update_bf(
            &SRC_TRANSPOSE,
            &res.s,
            &FA_R(),
            &res.gw_values.indicators.borrow().indicators_without_bf,
        );
        res.gw_values.signals_train.borrow_mut().update_bf(
            &SRC_TRANSPOSE,
            &res.s,
            &FA_T(),
            &res.gw_values.indicators.borrow().indicators_without_bf,
        );
        let td_ref = td.as_ref().get_ref();
        let res_ref = res.as_ref().get_ref();
        assert_eq_pr!(
            unsafe { &*td_ref.indicators_gateway.indicators }
                .indicators
                .values()
                .map(|(v1, _)| v1)
                .collect::<Vec<&BF_INDICATOR>>(),
            unsafe { &*res_ref.indicators_gateway.indicators }
                .indicators
                .values()
                .map(|(v1, _)| v1)
                .collect::<Vec<&BF_INDICATOR>>()
        );
        assert_eq_pr!(
            unsafe { &*td_ref.signals_ready_gateway.signals_ready }
                .signals_ready
                .values()
                .map(|(v1, _)| v1)
                .collect::<Vec<&BF_SIGNALS>>(),
            unsafe { &*res_ref.signals_ready_gateway.signals_ready }
                .signals_ready
                .values()
                .map(|(v1, _)| v1)
                .collect::<Vec<&BF_SIGNALS>>()
        );
        assert_eq_pr!(
            unsafe { &*td_ref.signals_train_gateway.signals_train }
                .signals_train
                .values()
                .map(|(v1, _)| v1)
                .collect::<Vec<&BF_SIGNALS>>(),
            unsafe { &*res_ref.signals_train_gateway.signals_train }
                .signals_train
                .values()
                .map(|(v1, _)| v1)
                .collect::<Vec<&BF_SIGNALS>>()
        );
    }
}
