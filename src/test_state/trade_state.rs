use bc_order_filters_gw::test_state::ORDER_FILTERS_STATE;
use bc_utils_lg::prelude::*;
use bc_utils_lg::test_state::prelude::*;

pub static TRADE_STATE: LazyLock<fn() -> TradeState<'static>> = LazyLock::new(|| {
    || TradeState {
        capital: Capital(TRADE.capital),
        orders: RefCell::new(MAP::from_iter([
            (
                "count_2",
                vec![ORDER_FILTERS_STATE()["count_2"].0.unwrap().order.clone()],
            ),
            (
                "wrap",
                vec![ORDER_FILTERS_STATE()["wrap"].0.unwrap().order.clone()],
            ),
        ])),
        orders_trigger: RefCell::new(MAP::from_iter([(
            "count_3",
            vec![{
                let v = ORDER_FILTERS_STATE()["count_3"].0.unwrap().clone();
                (v.order, v.trigger.unwrap())
            }],
        )])),
        ..Default::default() // positions: RefCell::new(MAP::from_iter([(1, POSITION.clone())])),
    }
});

pub static TRADE_STATE_EMPTY: LazyLock<fn() -> TradeState<'static>> =
    LazyLock::new(|| || TradeState::new(Capital(TRADE.capital)));
