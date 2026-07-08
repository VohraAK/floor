use std::collections::HashMap;
use crate::book::OrderBook;
use shared::event_types::RejectionReport;
use shared::order::{Order, Trade};

pub struct Exchange {
    pub books: HashMap<String, OrderBook>   // map tickers to orderbooks

    // orders from traders will hit a service
    // this service routes the incoming orders to an exchange
    // the exchange matches orders based on their tickers
    // exchange will generate ExecutionReports based on the trades filled
}

impl Exchange {
    pub fn new() -> Self {
        Self {
            books: HashMap::new(),
        }
    }

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
                let trades = book.match_order(order);
                return Ok(trades);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::order::Side;
    use uuid::Uuid;

    fn make_order(side: Side, ticker: &str, price: u64, qty: u64) -> Order {
        Order {
            id: Uuid::new_v4(),
            trader_id: Uuid::new_v4(),
            side,
            ticker: ticker.into(),
            qty,
            timestamp: 0,
            limit_price: price,
        }
    }

    #[test]
    fn rejects_zero_qty() {
        let mut exchange = Exchange::with_tickers(vec!["AAPL".into()]);
        let order = make_order(Side::Buy, "AAPL", 100, 0);

        let result = exchange.process(order);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().reason, "Order cannot be empty!");
    }

    #[test]
    fn rejects_zero_price() {
        let mut exchange = Exchange::with_tickers(vec!["AAPL".into()]);
        let order = make_order(Side::Buy, "AAPL", 0, 10);

        let result = exchange.process(order);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().reason, "Price cannot be zero!");
    }

    #[test]
    fn rejects_unknown_ticker() {
        let mut exchange = Exchange::with_tickers(vec!["AAPL".into()]);
        let order = make_order(Side::Buy, "MSFT", 100, 10);

        let result = exchange.process(order);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().reason, "Ticker does not exist!");
    }

    #[test]
    fn routes_valid_order_to_correct_book() {
        let mut exchange = Exchange::with_tickers(vec!["AAPL".into(), "MSFT".into()]);

        let result = exchange.process(make_order(Side::Buy, "AAPL", 100, 10));

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0); // no cross, just rests
        assert_eq!(exchange.books.get("AAPL").unwrap().bids.len(), 1);
        assert_eq!(exchange.books.get("MSFT").unwrap().bids.len(), 0);
    }

    #[test]
    fn valid_crossing_orders_produce_trade() {
        let mut exchange = Exchange::with_tickers(vec!["AAPL".into()]);
        exchange.process(make_order(Side::Sell, "AAPL", 100, 10)).unwrap();

        let trades = exchange.process(make_order(Side::Buy, "AAPL", 100, 10)).unwrap();

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].qty, 10);
    }
}