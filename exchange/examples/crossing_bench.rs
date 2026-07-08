use std::time::Instant;
use rand::Rng;
use uuid::Uuid;
use shared::order::{Order, Side};
use exchange::book::OrderBook;

fn make_order(side: Side, price: u64, qty: u64) -> Order {
    Order {
        id: Uuid::new_v4(),
        trader_id: Uuid::new_v4(),
        side,
        ticker: "BENCH".into(),
        qty,
        timestamp: 0,
        limit_price: price,
    }
}

fn main() {

    let total_orders_vec = vec![1000, 100_000, 1_000_000, 10_000_000];
    
    // const TOTAL_ORDERS: u64 = 1_000_000;
    const PRICE_LOW: u64 = 95;
    const PRICE_HIGH: u64 = 105; // narrow band around 100 -> frequent crosses

    let mut book = OrderBook::new();
    let mut trade_count = 0;
    let mut rng = rand::thread_rng();

    let start = Instant::now();

    for total_orders in total_orders_vec {
        println!("Processing {total_orders} orders...");
        let snapshot_interval = (total_orders / 10).max(1);

        for i in 0..total_orders {
            let side = if rng.gen_bool(0.5) { Side::Buy } else { Side::Sell };
            let price = rng.gen_range(PRICE_LOW..=PRICE_HIGH);
            let qty = rng.gen_range(1..=20);

            let trades = book.match_order(make_order(side, price, qty));
            trade_count += trades.len();

            if (i + 1) % snapshot_interval == 0 {
                println!(
                    "  [{}/{total_orders}] ltp={} best_bid={:?} best_ask={:?} spread={:?} mid={:?} bid_qty={:?} ask_qty={:?}",
                    i + 1,
                    book.ltp,
                    book.best_bid(),
                    book.best_ask(),
                    book.spread(),
                    book.mid_price(),
                    book.best_bid_qty(),
                    book.best_ask_qty(),
                );
            }
        }

        let elapsed = start.elapsed();
        
        println!("orders processed: {total_orders}");
        println!("trades produced:  {trade_count}");
        println!("trade-to-order ratio:  {too_ratio:.4}", too_ratio = (trade_count as f64 / total_orders as f64));
        println!("total time:       {elapsed:?}");
        println!("avg per order:    {:?}", elapsed / total_orders as u32);
        println!("----------------------------------------");
    
    }
}
