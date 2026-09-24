use std::{collections::{BTreeMap, HashMap}};
// use anyhow::Ok;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// use crate::EngineRequest::{Cancel, Place};


#[derive(Debug,Deserialize,Clone)]
pub struct  OrderDetails{
    pub order_qnt:u64,
    pub price:u64,
    pub quantity: u64,
    pub order_type: OrderType,
    pub time: DateTime<Utc>,
    pub user_id: u64,
}

#[derive( Debug, Clone, Copy,Serialize)]
pub struct OrderStatus {
    pub total:u64,
    pub filled:u64,
    pub remaining:u64,
    pub order_meta:OrderMetadata
}

#[derive( Debug, Clone, Copy,Serialize,Deserialize)]
pub struct OrderStatus1 {
    pub match_id:u64,
    pub match_price:u64,
    pub match_qnt:u64,//this is the total qnt of the matched order
    pub total:u64,
    pub filled:u64,
    pub remaining:u64,
    pub order_meta:OrderMetadata
}


#[derive(PartialEq, Debug, Clone, Copy,Serialize,Deserialize)]
pub enum OrderType {
    BUY,
    SELL,
}

#[derive(Debug, Clone, Copy,Serialize,Deserialize)]
pub struct  OrderMetadata {
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
    ErrorProcessingOrder
}


#[derive(Debug,Serialize,Deserialize)]
pub struct CancelOrderStatus{
    pub order_id:u64,
    pub user_id:u64,
    pub qnt:u64,
    pub price:u64,
    pub order_type:OrderType
}

#[derive(Debug)]
pub enum EngineEvents{
    OrderStatus(Vec<OrderStatus1>),
    CanceledOrder(CancelOrderStatus)
}

#[derive(Debug,Serialize,Deserialize)]
pub struct CancelOrder{
    pub order_id:u64,
    pub user_id:u64
}

#[derive(Debug)]
pub enum EngineRequest{
    Place(OrderDetails),
    Cancel(CancelOrder)
}

#[derive(Debug,Clone)]
pub struct LimitOrderBook {
    //this is kind of global counter which is initialized to 0 at the start and is incremented for the next order 
    pub next_order_id:u64,    
    pub multiplier: u64,
    pub tick_size: u64,
//  So BUY/SELL is a BtreeMap so here-:
//  Keys=price number e.g. 100
//  Values=Pricelevel-->this stores the order info who placed the order for the Price=Key i.e Price=100
    pub buy: BTreeMap<u64, PriceLevel>,
    pub ask: BTreeMap<u64, PriceLevel>,
    pub orders_pool: Vec<OrderNode>,
    pub free_list: Vec<usize>,
    pub orders_map: HashMap<u64, OrderLocation>,
}

impl LimitOrderBook {
    pub fn new(tick_size: u64, multiplier: u64) -> Self {
        Self {
            next_order_id:0,
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

    pub fn cancel_order(&mut self, order_id: u64) -> Result<CancelOrderStatus, CustomError> {
        
        let loc = match self.orders_map.get(&order_id).copied() {
            Some(l) => l,
            None => return Err(CustomError::OrderNotFound),
        };

        let node = self.orders_pool[loc.index];
        let qty = node.order.order_metadata.quantity;
        let user_id=node.order.order_metadata.user_id;
        let price=loc.price;
        let order_type=loc.order_type;

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
        //here instead of sending Ok() we should 
        //create a separate cancelled order status struct and send it to the user
        //right now this function is little rigid like 
        //it does not provide the flexibility of removing specific number of
        //quantites from a specific order -->will add that it future 


        /*
         CancelOrderStatus{
            order_id
            user_id
            quantities
            price
            OrderType
        }
         */ 
        // */

        let cancel_o_s=CancelOrderStatus{
            order_id,
            user_id,
            qnt:qty,
            price,
            order_type
        };
        Ok(cancel_o_s)
    }

    //this fn returns the current id i.e. the number which is currently stored for the current order
    //and increments it for the next order when arrives
    pub fn get_next_id(&mut self)->u64{
        let id=self.next_order_id;
        self.next_order_id+=1;
        id
    }


    //this function is the core of the matching engine as it matches or stores the BUY/SELL orders
    pub fn execute_order(&mut self, price: u64, order_meta: &mut OrderMetadata) -> Result<Vec<OrderStatus1>, CustomError> {
        
        if price % self.tick_size != 0 {
            return Err(CustomError::InvalidPrice);
        }

        if order_meta.quantity == 0 {
            return Err(CustomError::InvalidQuantity);
        }

        //here we just copied the quantity to another var to avoid changing the original qnt directly
        //it is helpful as whether the order type is BUY/SELL we keep running the loop until
        //remaining qnt>0
        let mut remaining_qty = order_meta.quantity;

        let initial_order_meta=*order_meta;
                  
        let order_id=self.get_next_id();

        let mut events=Vec::<OrderStatus1>::new();

        match order_meta.order_type {
            OrderType::BUY => {

                while remaining_qty > 0 {
                    //the min_ask stores the price for the minimum selling price 
                    //as in the btreeMap for ASK the key=price so we fetch that price 
                    //and do computation according that
                    let min_ask = match self.ask.keys().next().copied() {
                        Some(p) => p,
                        None => break,
                    };

                    //this is the check
                    //we only proceed further if min_ask<=price
                    //as this is limit order matching engine 
                    if min_ask > price {
                        break;
                    }

                    //here we get the mut ref to the pricelevel which has min ask 
                    //so we can essentially call it the current PriceLevel
                    let level = self.ask.get_mut(&min_ask).unwrap();

                    //since orders are originally stored in the orders pool(type=Vector) 
                    //the price level is the linked list which stores the indexes of the
                    //orders stored in the order pool
                    //so here we fetch the first order at the current price level
                    let mut current_idx = level.head;

                    //this loop is to traverse the current pricelevel and match the orders
                    while let Some(idx) = current_idx {

                        if remaining_qty == 0 {
                            break;
                        }

                           //since originally the order pool stores the Orders we get the access to it using the
                        //index 
                        let order = self.orders_pool[idx].order;
                        let order_qty = order.order_metadata.quantity;

                        //here we check the current order qty and compare it with the remaining qnt
                        //if cur_order_qnt > remaining qnt it means it means cur order can consume the 
                        //the whole remaining qnt we can break the loop for current price level
                        //otherwise we just reduce the qnt the current order can produce 
                        if order_qty > remaining_qty {
                            
                            self.orders_pool[idx].order.order_metadata.quantity -= remaining_qty;
                            
                            level.total_quantity -= remaining_qty;
                            
                            let order_s=OrderStatus1{
                                match_id:order.order_metadata.user_id,
                                match_price:min_ask,
                                match_qnt:order.order_metadata.quantity,
                                total:initial_order_meta.quantity,
                                filled:remaining_qty,
                                remaining:0,
                                order_meta:initial_order_meta
                            };
                            
                            remaining_qty = 0;
                            
                            events.push(order_s);
                            break;
                        } else {
                            remaining_qty -= order_qty;
                            level.total_quantity -= order_qty;
                            
                            let order_s=OrderStatus1{
                                match_id:order.order_metadata.user_id,
                                match_price:min_ask,
                                match_qnt:order.order_metadata.quantity,
                                total:initial_order_meta.quantity,
                                filled:order_qty,
                                remaining:remaining_qty,
                                order_meta:initial_order_meta
                            };
                            
                            events.push(order_s);

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

                    //debugging-
                    // println!("{}",max_bid);
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

                            //sellers_orderstatus
                            let order_s=OrderStatus1{
                                match_id:order.order_metadata.user_id,
                                match_price:max_bid,
                                match_qnt:order.order_metadata.quantity,
                                total:initial_order_meta.quantity,
                                filled:remaining_qty,
                                remaining:0,
                                order_meta:initial_order_meta
                            };

                            //buyer_orderstatus
                            let order_b =OrderStatus1{
                                match_id:initial_order_meta.user_id,
                                match_price:max_bid,
                                match_qnt:remaining_qty,
                                total:order_qty,
                                filled:remaining_qty,
                                remaining:order_qty-remaining_qty,
                                order_meta:self.orders_pool[idx].order.order_metadata
                            };

                            remaining_qty = 0;

                            events.push(order_s);
                            events.push(order_b);

                            break;
                        } else {
                            remaining_qty -= order_qty;
                            level.total_quantity -= order_qty;

                            let order_s=OrderStatus1{
                                match_id:order.order_metadata.user_id,
                                match_price:max_bid,
                                match_qnt:order.order_metadata.quantity,
                                total:initial_order_meta.quantity,
                                filled:order_qty,
                                remaining:remaining_qty,
                                order_meta:initial_order_meta
                            };

                            let order_b =OrderStatus1{
                                match_id:initial_order_meta.user_id,
                                match_price:max_bid,
                                match_qnt:order.order_metadata.quantity,
                                total:order_qty,
                                filled:order_qty,
                                remaining:0,
                                order_meta:self.orders_pool[idx].order.order_metadata
                            };
                            
                            events.push(order_s);
                            events.push(order_b);

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

       
        Ok(events)
    }

    //this function inherently calls the execute_order() fn 
    //to match the orders
    pub fn placing_order(&mut self,engine_request:EngineRequest)->Result<EngineEvents,CustomError>{

        match engine_request{
            EngineRequest::Place(order_detail)=>{
                let mut order_meta=OrderMetadata::new(order_detail.quantity,
                    order_detail.order_type, 
                    order_detail.time, 
                    order_detail.user_id);

                self.execute_order(order_detail.price,&mut order_meta).map(
                    |od|EngineEvents::OrderStatus(od)
                )
            },

            EngineRequest::Cancel(cancel_order)=>{
                
                self.cancel_order(cancel_order.order_id).map(
                    |cancel_order_status|
                    EngineEvents::CanceledOrder(cancel_order_status))
                
            }
        }

        
        //so here after the orders get executed we can also just send the current limit order book state
        //or we can create another function which when called by backend we can get the current 
        //order book state and we can have separate websocket connection from backend to frontend which will
        //keep on updating as the order gets updated
    }


    //this fn just prints the order book
    //when called
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
