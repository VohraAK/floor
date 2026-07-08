use std::{collections::{BTreeMap, VecDeque}, cmp::{Reverse}};
use crate::{order::{Order, Side, Trade}, utils::get_system_time};

type Level = VecDeque<Order>;

// OrderBook impl
pub struct OrderBook {
    pub bids: BTreeMap<Reverse<u64>, Level>,  // descending order of price
    pub asks: BTreeMap<u64, Level>,           // ascending order of price
    pub ltp: u64                              // last traded price of stock
}

impl OrderBook {
    // constructor
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            ltp: 0,
        }
    }

    // rest an order
    pub fn rest(&mut self, order: Order) {
        match order.side {
            Side::Buy => {self.rest_bid(order)},
            Side::Sell => {self.rest_ask(order)},
        }
    }

    // rest a bid
    pub fn rest_bid(&mut self, order: Order) {
        self.bids.entry(Reverse(order.limit_price)).or_insert_with(VecDeque::new).push_back(order);
    }

    // rest an ask
    pub fn rest_ask(&mut self, order: Order) {
        self.asks.entry(order.limit_price).or_insert_with(VecDeque::new).push_back(order);
    }

    // basic matching engine (top-level function called by Exchange)
    // TODO: messaging, events, scalable
    pub fn match_order(&mut self, mut order: Order) -> Vec<Trade> {
        let mut trades = Vec::<Trade>::new();

        match order.side {
            Side::Buy => {self.match_against_asks(&mut order, &mut trades)},
            Side::Sell => {self.match_against_bids(&mut order, &mut trades)},
        };
        
        // rest order if some quantity is remaining
        // also fires when the orderbook is empty
        if order.qty > 0 {self.rest(order);}

        trades
        
    }

    pub fn match_against_asks(&mut self, order: &mut Order, trades: &mut Vec<Trade>) {
        // match the order, and store the trade
        let mut remaining_qty: u64 = order.qty;

        while let Some(mut entry) = self.asks.first_entry() {
            let ask_price_level = *entry.key();
            
            if ask_price_level > order.limit_price {break;}
            
            // get the queue of asks
            let asks_queue = entry.get_mut();

            while let Some(mut resting_order) = asks_queue.pop_front() {
                let fill_qty: u64 = resting_order.qty.min(remaining_qty);

                resting_order.qty -= fill_qty;
                remaining_qty -= fill_qty;

                self.ltp = ask_price_level;

                let trade = Trade {
                    buy_order_id: order.id,
                    sell_order_id: resting_order.id,
                    ticker: order.ticker.clone(),
                    price: ask_price_level,
                    qty: fill_qty,
                    timestamp: get_system_time(),
                };
                
                trades.push(trade);

                if remaining_qty == 0 {
                    // order fully satisfied

                    if resting_order.qty > 0 {
                        // restore resting
                       asks_queue.push_front(resting_order); 
                    }
                    break;
                }
            }

            // inner orderop exits: either no qty remains, or price level is exhausted
            if remaining_qty == 0 {break;}

            // else, remove the exhausted price level
            let _ = entry.remove();

            // the loop will search the next price level
        }

        order.qty = remaining_qty;
        
    }
    
    pub fn match_against_bids(&mut self, order: &mut Order, trades: &mut Vec<Trade>) {
        // match the order, and store the trade
        let mut remaining_qty: u64 = order.qty;

        while let Some(mut entry) = self.bids.first_entry() {
            let bid_price_level = (*entry.key()).0;
            
            if bid_price_level < order.limit_price {break;}
            
            // get the queue of bids
            let bids_queue = entry.get_mut();

            while let Some(mut resting_order) = bids_queue.pop_front() {
                let fill_qty: u64 = resting_order.qty.min(remaining_qty);

                resting_order.qty -= fill_qty;
                remaining_qty -= fill_qty;

                self.ltp = bid_price_level;

                let trade = Trade {
                    buy_order_id: resting_order.id,
                    sell_order_id: order.id,
                    price: bid_price_level,
                    qty: fill_qty,
                    ticker: order.ticker.clone(),
                    timestamp: get_system_time(),
                };

                trades.push(trade);

                if remaining_qty == 0 {
                    // order fully satisfied

                    if resting_order.qty > 0 {
                        // restore resting
                       bids_queue.push_front(resting_order); 
                    }
                    break;
                }
            }

            // inner loop exits: either no qty remains, or price level is exhausted
            if remaining_qty == 0 {break;}

            // else, remove the exhausted price level
            let _ = entry.remove();

            // the loop will search the next price level
        }
        
        order.qty = remaining_qty;
    }
}