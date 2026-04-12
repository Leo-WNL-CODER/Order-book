#![allow(unused)]

use chrono::Utc;
use order_book::{CustomError, LimitOrderBook, OrderMetadata, OrderType};

fn main() {

    // tick_size = 5 (0.5)
    // multiplier = 10 (1 decimal precision)
    let mut order_book = LimitOrderBook::new(5, 10);

    // BUY 1 (100.0)
    let mut order_meta = OrderMetadata {
        quantity: 5,
        order_type: OrderType::BUY,
        time: Utc::now(),
        user_id: 2313123
    };

    order_book.execute_order(1000, 312311213, &mut order_meta).unwrap();


    // BUY 2 (100.5)
    let mut order_meta = OrderMetadata {
        quantity: 9,
        order_type: OrderType::BUY,
        time: Utc::now(),
        user_id: 213131
    };

    match order_book.execute_order(1005, 2222, &mut order_meta) {
        Ok(_) => { println!("inserted"); },
        Err(_) => { println!("error") }
    };


    // SELL 1 (105.0)
    let mut order_meta = OrderMetadata {
        quantity: 7,
        order_type: OrderType::SELL,
        time: Utc::now(),
        user_id: 88888
    };

    order_book.execute_order(1050, 4444, &mut order_meta).unwrap();


    // SELL 2 (104.5)
    let mut order_meta = OrderMetadata {
        quantity: 3,
        order_type: OrderType::SELL,
        time: Utc::now(),
        user_id: 99999
    };

    order_book.execute_order(1045, 5555, &mut order_meta).unwrap();


    order_book.print_summary();

    println!("status before");

    // SELL (100.5)
    let mut order_meta = OrderMetadata {
        quantity: 15,
        order_type: OrderType::SELL,
        time: Utc::now(),
        user_id: 77777
    };

    match order_book.execute_order(1005, 6666, &mut order_meta) {
        Ok(_) => {
            println!("after selling 15 @ 100.5");
            order_book.print_summary();
        },
        Err(e) => {
            println!("{:?}", e);
        }
    };


    // BUY (104.5)
    let mut order_meta = OrderMetadata {
        quantity: 8,
        order_type: OrderType::BUY,
        time: Utc::now(),
        user_id: 55555
    };

    match order_book.execute_order(1045, 7777, &mut order_meta) {
        Ok(_) => {
            println!("after buying 8 @ 104.5");
            order_book.print_summary();
        },
        Err(e) => {
            println!("{:?}", e);
        }
    };

    order_book.print_detailed();
}