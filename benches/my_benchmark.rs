use std::hint::black_box;
use chrono::Utc;
use criterion::{criterion_group, criterion_main, Criterion};
use order_book::{LimitOrderBook, OrderMetadata, OrderType};


// Benchmark 1 — Initialize OrderBook
fn benchmark_initialize_book(c: &mut Criterion) {
    c.bench_function("initializing order book", |b| {
        b.iter(|| {
            LimitOrderBook::new(5, 10);
        });
    });
}


// Benchmark 2 — OrderBook Initialization (with black_box)
fn benchmark_orderbook_init(c: &mut Criterion) {
    c.bench_function("orderbook init", |b| {
        b.iter(|| {
            black_box(LimitOrderBook::new(5, 10));
        })
    });
}


// Benchmark 3 — Insert Orders (No Match)
fn benchmark_insert_orders(c: &mut Criterion) {
    c.bench_function("insert orders", |b| {
        b.iter(|| {
            let mut order_book = LimitOrderBook::new(5, 10);

            let mut order_meta = OrderMetadata {
                quantity: 5,
                order_type: OrderType::BUY,
                time: Utc::now(),
                user_id: 2313123
            };

            order_book.execute_order(1000, 312311213, &mut order_meta).unwrap();
        })
    });
}


// Benchmark 4 — Full Matching Scenario
fn benchmark_execute_matching(c: &mut Criterion) {
    c.bench_function("execute matching", |b| {
        b.iter(|| {

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

            order_book.execute_order(1005, 2222, &mut order_meta).unwrap();


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


            // Match SELL (100.5)
            let mut order_meta = OrderMetadata {
                quantity: 15,
                order_type: OrderType::SELL,
                time: Utc::now(),
                user_id: 77777
            };

            order_book.execute_order(1005, 6666, &mut order_meta).unwrap();


            // Match BUY (104.5)
            let mut order_meta = OrderMetadata {
                quantity: 8,
                order_type: OrderType::BUY,
                time: Utc::now(),
                user_id: 55555
            };

            order_book.execute_order(1045, 7777, &mut order_meta).unwrap();

            black_box(order_book);
        })
    });
}


criterion_group!(
    benches,
    benchmark_initialize_book,
    benchmark_orderbook_init,
    benchmark_insert_orders,
    benchmark_execute_matching
);

criterion_main!(benches);