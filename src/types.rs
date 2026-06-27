use std::{cmp::Reverse, collections::{BTreeMap, HashMap, VecDeque}};

use uuid::Uuid;

// Buy or Sell-side LO
#[derive(Debug, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

// Status of LO in the exchange
#[derive(Debug, PartialEq)]
pub enum ExecutionStatus {
    Fill,
    PartialFill,
    Resting,
}

// Limit Order structure
#[derive(Debug, PartialEq)]
pub struct Order {
    pub id: Uuid,
    pub side: Side,
    pub ticker: String,
    pub qty: u64,
    pub timestamp: u64,
    pub limit_price: u64    // in "cents"
}

// Trade event structure
pub struct Trade {
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub ticker: String,
    pub price: u64,     // acts as an LTP event for this ticker
    pub qty: u64,
    pub timestamp: u64
}

pub struct OrderBook {
    pub bids: BTreeMap<Reverse<u64>, VecDeque<Order>>,  // descending order of price
    pub asks: BTreeMap<u64, VecDeque<Order>>,           // ascending order of price
    pub ltp: u64                                        // last traded price of stock
}

pub struct Exchange {
    pub books: HashMap<String, OrderBook>   // map tickers to orderbooks
}

// message enums for client and server
// client submits an order to the server
pub struct SubmitOrder {
    pub side: Side,
    pub ticker: String,
    pub limit_price: u64,
    pub qty: u64
}

// server sends multiple types of messages to the clients
// 1. Report (unicast)
pub struct ExecutionReport {
    pub ticker: String,
    pub status: ExecutionStatus,
    pub filled_qty: u64,
    pub remaining_qty: u64,
    pub fill_price: u64
}

// 2. LTP (broadcast)
// pub struct Ltp {
//     pub ticker: String,
//     pub price: u64,
//     pub qty: u64
// }

// wrapping the messages in client and server enums
pub enum ClientMessage {
    SubmitOrder(SubmitOrder),
    // we can add more here
}

pub enum ServerMessage {
    Trade(Trade),
    ExecutionReport(ExecutionReport),
}

// struct for engine event emissions
pub enum EngineEvent {
    Trade(Trade),
    ExecutionReport(ExecutionReport),
}