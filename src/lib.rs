use std::collections::{BTreeMap, HashMap};
use chrono::{DateTime, Utc};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum OrderType {
    BUY,
    SELL,
}

#[derive(Debug, Clone, Copy)]
pub struct OrderMetadata {
    pub quantity: u64,
    pub order_type: OrderType,
    pub time: DateTime<Utc>,
    pub user_id: u64,
}

impl OrderMetadata {
    pub fn new(quantity: u64, order_type: OrderType, time: DateTime<Utc>, user_id: u64) -> Self {
        OrderMetadata { quantity, order_type, time, user_id }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Order {
    pub order_id: u64,
    pub order_metadata: OrderMetadata,
}

impl Order {
    pub fn new(order_id: u64, order_metadata: OrderMetadata) -> Self {
        Order { order_id, order_metadata }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OrderNode {
    pub order: Order,
    pub prev: Option<usize>,
    pub next: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct PriceLevel {
    pub head: Option<usize>,
    pub tail: Option<usize>,
    pub total_quantity: u64,
}

impl PriceLevel {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            total_quantity: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OrderLocation {
    pub index: usize,
    pub price: u64,
    pub order_type: OrderType,
}

#[derive(Debug)]
pub enum CustomError {
    InvalidPrice,
    InvalidQuantity,
    InvalidOrderType,
    OrderNotFound,
    ErrorFetchingAskLevel,
    ErrorFetchingBuyLevel,
    ErrorFetchingVecDeque,
    NegativeTickSize,
}

#[derive(Debug)]
pub struct LimitOrderBook {
    pub multiplier: u64,
    pub tick_size: u64,
    pub buy: BTreeMap<u64, PriceLevel>,
    pub ask: BTreeMap<u64, PriceLevel>,
    pub orders_pool: Vec<OrderNode>,
    pub free_list: Vec<usize>,
    pub orders_map: HashMap<u64, OrderLocation>,
}

impl LimitOrderBook {
    pub fn new(tick_size: u64, multiplier: u64) -> Self {
        Self {
            tick_size,
            buy: BTreeMap::new(),
            ask: BTreeMap::new(),
            multiplier,
            orders_pool: Vec::with_capacity(10000),
            free_list: Vec::new(),
            orders_map: HashMap::new(),
        }
    }

    fn allocate_node(&mut self, order: Order, price: u64) -> usize {
        let index = if let Some(idx) = self.free_list.pop() {
            self.orders_pool[idx] = OrderNode { order, prev: None, next: None };
            idx
        } else {
            let idx = self.orders_pool.len();
            self.orders_pool.push(OrderNode { order, prev: None, next: None });
            idx
        };
        self.orders_map.insert(order.order_id, OrderLocation {
            index,
            price,
            order_type: order.order_metadata.order_type,
        });
        index
    }

    fn free_node(&mut self, index: usize) {
        let id = self.orders_pool[index].order.order_id;
        self.orders_map.remove(&id);
        self.free_list.push(index);
    }

    fn append_to_level(&mut self, price: u64, order_type: OrderType, order_id: u64, meta: OrderMetadata) {
        let order = Order::new(order_id, meta);
        let qty = meta.quantity;
        let index = self.allocate_node(order, price);

        let level = match order_type {
            OrderType::BUY => self.buy.entry(price).or_insert_with(PriceLevel::new),
            OrderType::SELL => self.ask.entry(price).or_insert_with(PriceLevel::new),
        };

        match level.tail {
            Some(tail_idx) => {
                self.orders_pool[tail_idx].next = Some(index);
                self.orders_pool[index].prev = Some(tail_idx);
                level.tail = Some(index);
            }
            None => {
                level.head = Some(index);
                level.tail = Some(index);
            }
        }
        level.total_quantity += qty;
    }

    pub fn cancel_order(&mut self, order_id: u64) -> Result<(), CustomError> {
        let loc = match self.orders_map.get(&order_id).copied() {
            Some(l) => l,
            None => return Err(CustomError::OrderNotFound),
        };

        let node = self.orders_pool[loc.index];
        let qty = node.order.order_metadata.quantity;

        // Unlink from Doubly Linked List
        if let Some(prev) = node.prev {
            self.orders_pool[prev].next = node.next;
        }
        if let Some(next) = node.next {
            self.orders_pool[next].prev = node.prev;
        }

        let levels = match loc.order_type {
            OrderType::BUY => &mut self.buy,
            OrderType::SELL => &mut self.ask,
        };

        if let Some(level) = levels.get_mut(&loc.price) {
            level.total_quantity -= qty;

            if level.head == Some(loc.index) {
                level.head = node.next;
            }
            if level.tail == Some(loc.index) {
                level.tail = node.prev;
            }

            if level.head.is_none() {
                levels.remove(&loc.price);
            }
        }

        self.free_node(loc.index);
        Ok(())
    }

    pub fn execute_order(&mut self, price: u64, order_id: u64, order_meta: &mut OrderMetadata) -> Result<String, CustomError> {
        if price % self.tick_size != 0 {
            return Err(CustomError::InvalidPrice);
        }
        if order_meta.quantity == 0 {
            return Err(CustomError::InvalidQuantity);
        }

        let mut remaining_qty = order_meta.quantity;

        match order_meta.order_type {
            OrderType::BUY => {
                while remaining_qty > 0 {
                    let min_ask = match self.ask.keys().next().copied() {
                        Some(p) => p,
                        None => break,
                    };

                    if min_ask > price {
                        break;
                    }

                    let level = self.ask.get_mut(&min_ask).unwrap();
                    let mut current_idx = level.head;

                    while let Some(idx) = current_idx {
                        if remaining_qty == 0 {
                            break;
                        }

                        let order = self.orders_pool[idx].order;
                        let order_qty = order.order_metadata.quantity;

                        if order_qty > remaining_qty {
                            self.orders_pool[idx].order.order_metadata.quantity -= remaining_qty;
                            level.total_quantity -= remaining_qty;
                            remaining_qty = 0;
                            break;
                        } else {
                            remaining_qty -= order_qty;
                            level.total_quantity -= order_qty;

                            current_idx = self.orders_pool[idx].next;
                            level.head = current_idx;
                            if let Some(next_idx) = current_idx {
                                self.orders_pool[next_idx].prev = None;
                            } else {
                                level.tail = None;
                            }

                            let id_to_free = self.orders_pool[idx].order.order_id;
                            self.orders_map.remove(&id_to_free);
                            self.free_list.push(idx);
                        }
                    }

                    if level.head.is_none() {
                        self.ask.remove(&min_ask);
                    }
                }

                if remaining_qty > 0 {
                    let mut new_meta = *order_meta;
                    new_meta.quantity = remaining_qty;
                    self.append_to_level(price, OrderType::BUY, order_id, new_meta);
                }
            }
            OrderType::SELL => {
                while remaining_qty > 0 {
                    let max_bid = match self.buy.keys().next_back().copied() {
                        Some(p) => p,
                        None => break,
                    };

                    if max_bid < price {
                        break;
                    }

                    let level = self.buy.get_mut(&max_bid).unwrap();
                    let mut current_idx = level.head;

                    while let Some(idx) = current_idx {
                        if remaining_qty == 0 {
                            break;
                        }

                        let order = self.orders_pool[idx].order;
                        let order_qty = order.order_metadata.quantity;

                        if order_qty > remaining_qty {
                            self.orders_pool[idx].order.order_metadata.quantity -= remaining_qty;
                            level.total_quantity -= remaining_qty;
                            remaining_qty = 0;
                            break;
                        } else {
                            remaining_qty -= order_qty;
                            level.total_quantity -= order_qty;

                            current_idx = self.orders_pool[idx].next;
                            level.head = current_idx;
                            if let Some(next_idx) = current_idx {
                                self.orders_pool[next_idx].prev = None;
                            } else {
                                level.tail = None;
                            }

                            let id_to_free = self.orders_pool[idx].order.order_id;
                            self.orders_map.remove(&id_to_free);
                            self.free_list.push(idx);
                        }
                    }

                    if level.head.is_none() {
                        self.buy.remove(&max_bid);
                    }
                }

                if remaining_qty > 0 {
                    let mut new_meta = *order_meta;
                    new_meta.quantity = remaining_qty;
                    self.append_to_level(price, OrderType::SELL, order_id, new_meta);
                }
            }
        }

        Ok("()".to_string())
    }

    pub fn print_summary(&self) {
        println!("\n================ ORDER BOOK ================");

        println!("\n------------- ASKS -------------");
        for (price, level) in self.ask.iter().rev() {
            println!("Price: {:<10} | Total Qty: {}", *price as f64 / (self.multiplier as f64), level.total_quantity);
        }

        println!("\n------------- BIDS -------------");
        for (price, level) in self.buy.iter().rev() {
            println!("Price: {:<10} | Total Qty: {}", *price as f64 / (self.multiplier as f64), level.total_quantity);
        }

        println!("============================================\n");
    }

    pub fn print_detailed(&self) {
        println!("\n================ ORDER BOOK (DETAILED) ================");

        println!("\n------------- ASKS -------------");
        for (price, level) in &self.ask {
            println!("Price: {} | Total Qty: {}", *price as f64 / (self.multiplier as f64), level.total_quantity);

            let mut current = level.head;
            while let Some(idx) = current {
                let order = &self.orders_pool[idx].order;
                println!(
                    "   OrderID: {} | Qty: {} | User: {} | Time: {} | Type: {:?}",
                    order.order_id,
                    order.order_metadata.quantity,
                    order.order_metadata.user_id,
                    order.order_metadata.time,
                    order.order_metadata.order_type
                );
                current = self.orders_pool[idx].next;
            }
        }

        println!("\n------------- BIDS -------------");
        for (price, level) in &self.buy {
            println!("Price: {} | Total Qty: {}", *price as f64 / (self.multiplier as f64), level.total_quantity);

            let mut current = level.head;
            while let Some(idx) = current {
                let order = &self.orders_pool[idx].order;
                println!(
                    "   OrderID: {} | Qty: {} | User: {} | Time: {} | Type: {:?}",
                    order.order_id,
                    order.order_metadata.quantity,
                    order.order_metadata.user_id,
                    order.order_metadata.time,
                    order.order_metadata.order_type
                );
                current = self.orders_pool[idx].next;
            }
        }

        println!("=======================================================\n");
    }
}
