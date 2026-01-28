//! Benchmarks for Order Processing Performance
//! Phase 4: Performance testing for latency requirements

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;
use std::collections::HashMap;

// Simplified order struct for benchmarking
#[derive(Debug, Clone)]
struct Order {
    id: Uuid,
    symbol: String,
    quantity: Decimal,
    price: Decimal,
}

#[derive(Debug, Clone)]
struct Position {
    quantity: Decimal,
    avg_price: Decimal,
}

impl Position {
    fn apply_fill(&mut self, qty: Decimal, price: Decimal) {
        if self.quantity == dec!(0) {
            self.avg_price = price;
            self.quantity = qty;
        } else {
            let total_cost = (self.quantity * self.avg_price) + (qty * price);
            self.quantity = self.quantity + qty;
            if self.quantity != dec!(0) {
                self.avg_price = total_cost / self.quantity;
            }
        }
    }
}

fn create_order(symbol: &str, quantity: Decimal, price: Decimal) -> Order {
    Order {
        id: Uuid::new_v4(),
        symbol: symbol.to_string(),
        quantity,
        price,
    }
}

fn benchmark_order_creation(c: &mut Criterion) {
    c.bench_function("order_creation", |b| {
        b.iter(|| {
            black_box(create_order("BTC-USD", dec!(1.5), dec!(50000.00)))
        })
    });
}

fn benchmark_position_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("position_update");

    for size in [1, 10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut pos = Position {
                    quantity: dec!(0),
                    avg_price: dec!(0),
                };

                for i in 0..size {
                    let price = dec!(100) + Decimal::from(i);
                    pos.apply_fill(dec!(1), price);
                }

                black_box(pos)
            })
        });
    }

    group.finish();
}

fn benchmark_order_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("order_lookup");

    for size in [100, 1000, 10000].iter() {
        let mut orders: HashMap<Uuid, Order> = HashMap::new();
        let mut lookup_ids: Vec<Uuid> = Vec::new();

        for _ in 0..*size {
            let order = create_order("BTC-USD", dec!(1.0), dec!(50000.00));
            lookup_ids.push(order.id);
            orders.insert(order.id, order);
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let id = lookup_ids[0];
            b.iter(|| {
                black_box(orders.get(&id))
            })
        });
    }

    group.finish();
}

fn benchmark_weighted_average(c: &mut Criterion) {
    c.bench_function("weighted_average_1000_fills", |b| {
        b.iter(|| {
            let mut pos = Position {
                quantity: dec!(0),
                avg_price: dec!(0),
            };

            for i in 0..1000 {
                let price = dec!(100) + Decimal::from(i % 100);
                let qty = dec!(1) + Decimal::from(i % 10);
                pos.apply_fill(qty, price);
            }

            black_box(pos)
        })
    });
}

criterion_group!(
    benches,
    benchmark_order_creation,
    benchmark_position_update,
    benchmark_order_lookup,
    benchmark_weighted_average,
);

criterion_main!(benches);