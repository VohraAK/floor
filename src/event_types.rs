use crate::order::{Side, Trade};
use uuid::Uuid;

// client submits an order to the server
pub struct SubmitOrder {
    pub side: Side,
    pub ticker: String,
    pub limit_price: u64,
    pub qty: u64
}

#[derive(Debug)]
pub struct RejectionReport {
    pub order_id: Uuid,
    pub ticker: String,
    pub reason: String,     // free text    
}

// Messages
// wrapping the messages in client and server enums
pub enum ClientMessage {
    SubmitOrder(SubmitOrder),
    // we can add more here
}

pub enum ServerMessage {
    Trade(Trade),
    RejectionReport(RejectionReport)
}