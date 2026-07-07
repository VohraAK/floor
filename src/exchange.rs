use std::collections::HashMap;
use uuid::Uuid;

use crate::book::OrderBook;
use crate::event_types::{RejectionReport};
use crate::order::{Order, Trade};

pub struct Exchange {
    pub books: HashMap<String, OrderBook>   // map tickers to orderbooks

    // orders from traders will hit a service
    // this service routes the incoming orders to an exchange
    // the exchange matches orders based on their tickers
    // exchange will generate ExecutionReports based on the trades filled
}

impl Exchange {
    // pub fn new() -> Self {
    //     Self {
    //         books: HashMap::new(),
    //     }
    // }

    pub fn with_tickers(tickers: Vec<String>) -> Self {
        let books = tickers.into_iter().map(|t| (t, OrderBook::new())).collect();
        Self {books}
    }
    
    pub fn process(&mut self, order: Order) -> Result<Vec<Trade>, RejectionReport> {

        // validate everything
        // 1) ticker existing
        // 2) empty orders
        // 3) zero limit_price

        if order.qty == 0 {
            let reject = RejectionReport {
                order_id: order.id,
                ticker: order.ticker.clone(),
                reason: "Order cannot be empty!".into(),
            };

            eprintln!("ERROR: {reject:?}");

            return Err(reject)
        }

        if order.limit_price == 0 {
            let reject = RejectionReport {
                order_id: order.id,
                ticker: order.ticker.clone(),
                reason: "Price cannot be zero!".into(),
            };
            
            eprintln!("ERROR: {reject:?}");

            return Err(reject)
        }

        match self.books.get_mut(&order.ticker) {
            None => {
                let reject = RejectionReport {
                    order_id: order.id,
                    ticker: order.ticker.clone(),
                    reason: "Ticker does not exist!".into(),
                };
                
                eprintln!("ERROR: {reject:?}");

                return Err(reject)
            }
            
            Some(book) => {
                
            }

            
        }
        
    }
}