use bc_test_kit::prelude::*;
use bc_utils_lg::prelude::*;

pub static POSITION: LazyLock<Position> = LazyLock::new(|| Position {
    symbol: "".to_string(),
    side: "buy".to_string(),
    qty: 1010.,
    leverage: 10.,
    avg_open_price: OPEN_LAST,
    position_idx: 1,
    is_active: true,
    pnl_qty: 0.,
    pnl_percent: 0.,
});
