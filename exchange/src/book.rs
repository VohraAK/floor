use std::{collections::{BTreeMap, VecDeque}, cmp::{Reverse}};
use shared::order::{Order, Side, Trade};
use crate::utils::get_system_time;

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

    // highest resting bid price
    pub fn best_bid(&self) -> Option<u64> {
        self.bids.first_key_value().map(|(p, _)| p.0)
    }

    // lowest resting ask price
    pub fn best_ask(&self) -> Option<u64> {
        self.asks.first_key_value().map(|(p, _)| *p)
    }

    // ask - bid at the top of book
    pub fn spread(&self) -> Option<u64> {
        self.best_ask()?.checked_sub(self.best_bid()?)
    }

    // midpoint between best bid and best ask
    pub fn mid_price(&self) -> Option<f64> {
        Some((self.best_bid()? + self.best_ask()?) as f64 / 2.0)
    }

    // total resting qty at the best bid price level
    pub fn best_bid_qty(&self) -> Option<u64> {
        self.bids.first_key_value().map(|(_, level)| level.iter().map(|o| o.qty).sum())
    }

    // total resting qty at the best ask price level
    pub fn best_ask_qty(&self) -> Option<u64> {
        self.asks.first_key_value().map(|(_, level)| level.iter().map(|o| o.qty).sum())
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

            // pop each resting order in a loop...
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
            
            // 
            // remove the price level only once its queue is fully drained
            if asks_queue.is_empty() {
                let _ = entry.remove();
            }

            // inner orderop exits: either no qty remains, or price level is exhausted
            if remaining_qty == 0 {break;}

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

            // remove the price level only once its queue is fully drained
            if bids_queue.is_empty() {
                let _ = entry.remove();
            }

            // inner loop exits: either no qty remains, or price level is exhausted
            if remaining_qty == 0 {
                break;
            }

            // the loop will search the next price level
        }

        order.qty = remaining_qty;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_order(side: Side, price: u64, qty: u64) -> Order {
        Order {
            id: Uuid::new_v4(),
            trader_id: Uuid::new_v4(),
            side,
            ticker: "TEST".into(),
            qty,
            timestamp: 0,
            limit_price: price,
        }
    }

    #[test]
    fn no_cross_rests_order() {
        let mut book = OrderBook::new();
        let bid = make_order(Side::Buy, 100, 10);

        let trades = book.match_order(bid);

        assert!(trades.is_empty());
        assert_eq!(book.ltp, 0);
        assert_eq!(book.bids.get(&Reverse(100)).unwrap().len(), 1);
    }

    #[test]
    fn exact_full_match() {
        let mut book = OrderBook::new();
        book.rest(make_order(Side::Sell, 100, 10));

        let trades = book.match_order(make_order(Side::Buy, 100, 10));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].qty, 10);
        assert_eq!(trades[0].price, 100);
        assert_eq!(book.ltp, 100);
        assert!(book.asks.is_empty());
        assert!(book.bids.is_empty());
    }

    #[test]
    fn partial_fill_incoming_bigger_rests_remainder() {
        let mut book = OrderBook::new();
        book.rest(make_order(Side::Sell, 100, 5));

        let trades = book.match_order(make_order(Side::Buy, 100, 8));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].qty, 5);
        assert!(book.asks.is_empty());

        let resting_bid = book.bids.get(&Reverse(100)).unwrap();
        assert_eq!(resting_bid.len(), 1);
        assert_eq!(resting_bid[0].qty, 3);
    }

    #[test]
    fn partial_fill_resting_bigger_keeps_resting_order_at_front() {
        let mut book = OrderBook::new();
        book.rest(make_order(Side::Sell, 100, 10));

        let trades = book.match_order(make_order(Side::Buy, 100, 4));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].qty, 4);
        assert!(book.bids.is_empty());

        let resting_ask = book.asks.get(&100).unwrap();
        assert_eq!(resting_ask.len(), 1);
        assert_eq!(resting_ask[0].qty, 6);
    }

    #[test]
    fn price_time_priority_same_level_fifo() {
        let mut book = OrderBook::new();
        let first = make_order(Side::Sell, 100, 5);
        let first_id = first.id;
        book.rest(first);
        book.rest(make_order(Side::Sell, 100, 5));

        let trades = book.match_order(make_order(Side::Buy, 100, 5));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].sell_order_id, first_id);
        assert_eq!(book.asks.get(&100).unwrap().len(), 1);
    }

    #[test]
    fn multi_level_sweep() {
        let mut book = OrderBook::new();
        book.rest(make_order(Side::Sell, 100, 5));
        book.rest(make_order(Side::Sell, 101, 5));

        let trades = book.match_order(make_order(Side::Buy, 101, 10));

        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[1].price, 101);
        assert!(book.asks.is_empty());
        assert!(book.bids.is_empty());
        assert_eq!(book.ltp, 101);
    }

    #[test]
    fn no_cross_when_price_does_not_reach() {
        let mut book = OrderBook::new();
        book.rest(make_order(Side::Sell, 105, 10));

        let trades = book.match_order(make_order(Side::Buy, 100, 10));

        assert!(trades.is_empty());
        assert_eq!(book.ltp, 0);
        assert_eq!(book.asks.get(&105).unwrap().len(), 1);
        assert_eq!(book.bids.get(&Reverse(100)).unwrap().len(), 1);
    }

    #[test]
    fn ltp_uses_resting_order_price_not_incoming_price() {
        let mut book = OrderBook::new();
        book.rest(make_order(Side::Sell, 95, 10));

        // incoming bid crosses at a higher limit_price than the resting ask
        book.match_order(make_order(Side::Buy, 100, 10));

        assert_eq!(book.ltp, 95);
    }
}