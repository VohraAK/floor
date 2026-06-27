use std::{cmp::Reverse, collections::VecDeque};

use crate::types::{Order, OrderBook, Side};

// a basic price matching function:
// given an incoming LO (bid or ask), match it with LO if it exists, otherwise store it in the orderbook
pub fn match_order(mut lo: Order, order_book: &mut OrderBook) {

    // the bid matches against crossing ask quantities until:
    // a) the bid order is satisfied completely
    // b) no more crossing asks remain for the bid to be satisfied
    // 
    // 1) the bid checks the first price level for crossing (ascending BTreeMap)
    // 2) if the bid crosses against the price level, then the asks in that price level are fulfilled
    // 3) if the bid is unsatisfied in the current price level (all asks are exhausted), then it checks the next price level in the book
    // 4) the loop continues until the bid is satisfied, OR no more crossing asks remain
    // 
    // the outer loop iterates on crossing price levels, the inner loop executes the orders
    // if the bid quantity still remains after both loops, then rest the order
    // the first time a price level (best_ask_price) is > than bid price, there are no crossing asks left.

    let mut remaining_qty: u64 = lo.qty;
    
    match lo.side {
        Side::Buy => {
            // rest the bid is no asks available
            if order_book.asks.is_empty() {
                order_book.bids.entry(Reverse(lo.limit_price)).or_insert_with(VecDeque::new).push_back(lo);

                return;     // TODO: emit ExecutionStatus::Resting event
                
            }

            while let Some(mut entry) = order_book.asks.first_entry() {
                let ask_price_level = *entry.key();
                
                if ask_price_level > lo.limit_price {break;}
                
                // get the queue of asks
                let asks_queue = entry.get_mut();

                while let Some(mut resting_order) = asks_queue.pop_front() {
                    let fill_qty: u64 = resting_order.qty.min(remaining_qty);

                    resting_order.qty -= fill_qty;
                    remaining_qty -= fill_qty;

                    order_book.ltp = ask_price_level;
                    // TODO: emit trade event!

                    if remaining_qty == 0 {
                        // order fully satisfied

                        if resting_order.qty > 0 {
                            // restore resting
                           asks_queue.push_front(resting_order); 
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

            if remaining_qty > 0 {
                // rest remainder in bids...
                lo.qty = remaining_qty;
                order_book.bids.entry(Reverse(lo.limit_price)).or_insert_with(VecDeque::new).push_back(lo);
            }
            
        }
        Side::Sell => {
            // rest the ask if no bids available
            if order_book.bids.is_empty() {
                order_book.asks.entry(lo.limit_price).or_insert_with(VecDeque::new).push_back(lo);
                return;     // TODO: emit ExecutionStatus::Resting event
            }

            while let Some(mut entry) = order_book.bids.first_entry() {
                let bid_price_level = (*entry.key()).0;
                
                if bid_price_level < lo.limit_price {break;}
                
                // get the queue of bids
                let bids_queue = entry.get_mut();

                while let Some(mut resting_order) = bids_queue.pop_front() {
                    let fill_qty: u64 = resting_order.qty.min(remaining_qty);

                    resting_order.qty -= fill_qty;
                    remaining_qty -= fill_qty;

                    order_book.ltp = bid_price_level;
                    // TODO: emit trade event!

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

            if remaining_qty > 0 {
                // rest remainder in bids...
                lo.qty = remaining_qty;
                order_book.asks.entry(lo.limit_price).or_insert_with(VecDeque::new).push_back(lo);
            }
            
        }
    }
}