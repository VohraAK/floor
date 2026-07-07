use uuid::Uuid;
use floor::event_types::RejectionReport;

fn main()
{
    let reject = RejectionReport {
        order_id: Uuid::new_v4(),
        ticker: "SPCX".into(),
        reason: "Ticker does not exist!".into(),
    };

    eprintln!("ERROR: {reject:?}");

}