use uuid::Uuid;

// Buy or Sell-side LO
#[derive(Debug, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

// Limit Order structure
#[derive(Debug, PartialEq)]
pub struct Order {
    pub id: Uuid,
    pub trader_id: Uuid,
    pub side: Side,
    pub ticker: String,
    pub qty: u64,
    pub timestamp: u64,
    pub limit_price: u64    // in "cents"
}

#[derive(Debug)]
// Trade event structure
pub struct Trade {
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub ticker: String,
    pub price: u64,     // acts as an LTP event for this ticker
    pub qty: u64,
    pub timestamp: u64
}