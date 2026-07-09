use bc_indicators::main_trait::*;
use bc_orders_collectors::main_trait::OrderCollector;
use bc_signals::ready::main_trait::*;
use bc_signals::train::main_trait::*;
use bc_utils_lg::structs::settings::*;
use bc_utils_lg::types::maps::{FUNCS_EXTRACT_ARGS_TYPE as FA, MAP};

use crate::buffer::ToBuff;
use crate::statistics::StatCollector;
use crate::trade_data::TradeData;

pub fn backtest<'a>(
    symbol: String,
    s: &'a SETTINGS,
    src: Vec<Vec<f64>>,
    w_max: usize,
    fa_indicators: &FA<SETTINGS_IND, Box<dyn Indicator>>,
    fa_signals_ready: &FA<SETTINGS_SIGNAL, Box<dyn SignalReady>>,
    fa_signals_train: &FA<SETTINGS_SIGNAL, Box<dyn SignalTrain>>,
    fa_orders_collectors: &FA<SETTINGS_ORDER_COLLECTOR, Box<dyn OrderCollector>>,
) -> StatCollector<'a> {
    let mut stat_collector = StatCollector::new(symbol.clone(), &s.trade);
    let mut buffer = src[..w_max].to_buff();
    let trade_data = TradeData::new(
        &mut buffer,
        s,
        &symbol,
        fa_indicators,
        fa_signals_ready,
        fa_signals_train,
        fa_orders_collectors,
    );

    for series in src.into_iter().skip(w_max) {
        buffer.update(series);
        trade_data.update(&mut buffer, Some(&mut stat_collector));
    }
    stat_collector
}

pub fn backtest_multi<'a>(
    s: &'a SETTINGS,
    src_symbols: MAP<String, Vec<Vec<f64>>>,
    w_max: usize,
    fa_indicators: &FA<SETTINGS_IND, Box<dyn Indicator>>,
    fa_signals_ready: &FA<SETTINGS_SIGNAL, Box<dyn SignalReady>>,
    fa_signals_train: &FA<SETTINGS_SIGNAL, Box<dyn SignalTrain>>,
    fa_orders_collectors: &FA<SETTINGS_ORDER_COLLECTOR, Box<dyn OrderCollector>>,
) -> MAP<String, StatCollector<'a>> {
    src_symbols
        .into_iter()
        .map(|(symbol, src)| {
            (
                symbol.clone(),
                backtest(
                    symbol,
                    s,
                    src,
                    w_max,
                    fa_indicators,
                    fa_signals_ready,
                    fa_signals_train,
                    fa_orders_collectors,
                ),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use bc_indicators_gw::gw::get_w_max;

    use super::*;
    use crate::prelude_tests::prelude::*;

    #[test]
    fn backtest_res_1() {
        // assert_eq_pr!(
        //     backtest(
        //         "".to_string(),
        //         &S,
        //         SRC.clone(),
        //         get_w_max(&S.indications, &FA_I()),
        //         &FA_I(),
        //         &FA_R(),
        //         &FA_T(),
        //         &FA_O()
        //     ).cells.last().unwrap().capital,
        //     StatCollector::new("".to_string(), &S.trade).cells.last().unwrap_or(&TradeCell::default()).capital
        // );
    }
}
