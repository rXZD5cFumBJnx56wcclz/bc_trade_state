#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    pub symbol: String,
    pub side: String,
    pub qty: f64,
    pub commission: f64,
    pub leverage: f64,
    pub price: Option<f64>,
    pub type_: String,
    pub is_reduce: bool,
    pub type_price_cross: String,
    pub order_link_id: String,
    pub position_idx: usize,
    pub is_active: bool,
}

impl Default for Order {
    fn default() -> Self {
        Self {
            symbol: Default::default(),
            side: Default::default(),
            qty: Default::default(),
            commission: Default::default(),
            leverage: 1.,
            price: Default::default(),
            type_: Default::default(),
            is_reduce: Default::default(),
            type_price_cross: "last".to_string(),
            order_link_id: Default::default(),
            position_idx: Default::default(),
            is_active: true,
        }
    }
}

impl Order {
    pub fn new(
        symbol: String,
        side: String,
        qty: f64,
        commission: f64,
        leverage: f64,
        price: Option<f64>,
        type_: String,
        is_reduce: bool,
        type_price_cross: String,
        order_link_id: String,
        position_idx: usize,
        is_active: bool,
    ) -> Self {
        Self {
            symbol,
            side,
            qty,
            commission,
            leverage,
            price,
            type_,
            is_reduce,
            type_price_cross,
            order_link_id,
            position_idx,
            is_active,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Trigger {
    pub price: f64,
    pub trigger_by: String,
    pub direction: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct OrderWrap {
    pub order: Order,
    pub is_trigger: bool,
    pub trigger: Option<Trigger>,
}
