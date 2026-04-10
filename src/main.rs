#![allow(unused)]
use std::{ collections::{BTreeMap, VecDeque, btree_map::Entry}, thread::{current, sleep}, time::{SystemTime, UNIX_EPOCH}};

use chrono::{DateTime, Utc};

const COUNTER:u64=0;
const DEFAULT_LEVEL_CAPACITY: usize = 32;

#[derive(PartialEq,Debug,Clone, Copy)]
enum OrderType{
    BUY,
    SELL
}

#[derive(Debug,Clone, Copy)]
struct OrderMetadata{
    quantity:u64,
    order_type:OrderType,
    time:DateTime<Utc>,
    user_id:u64
}
impl OrderMetadata{
    fn new( quantity:u64,order_type:OrderType,
        time:DateTime<Utc>,user_id:u64)->Self{
            OrderMetadata {quantity, order_type, time, user_id }
    }
}

#[derive(Debug,Clone, Copy)]
struct Order{
    order_id:u64,
    order_metadata:OrderMetadata
}

impl Order {
    fn new(order_id:u64,order_metadata:OrderMetadata)->Self{

            Order{
                order_id,
                order_metadata
            }

    }
}

#[derive(Clone,Debug)]
struct PriceLevel{
    vec_level:VecDeque<Order>,
    total_quantity:u64
}

impl PriceLevel {
    fn new(total_quantity:u64,capacity: usize) -> Self {
        Self {
            vec_level: VecDeque::with_capacity(capacity),
            total_quantity,
        }
    }
}


#[derive(Debug)]
struct LimitOrderBook{
    tick_size:u8,
    buy:BTreeMap<u64,PriceLevel>,//Btremap<price,pricelevel queue=> this represents the
    //  total number of orders at the particular price>
    ask:BTreeMap<u64,PriceLevel>
}

impl LimitOrderBook {
    fn new(tick_size:u8)->Self{

        let ask: BTreeMap<u64, PriceLevel>=BTreeMap::new();
        let buy: BTreeMap<u64, PriceLevel>=BTreeMap::new();

        Self { tick_size, buy, ask }
    }

    fn execute_order(&mut self,price:u64,order_id:u64,order_meta:&mut OrderMetadata)->Result<String,CustomError>{
        if price<=0 {
            return Err(CustomError::InvalidPrice)
        }

        if order_meta.order_type!=OrderType::BUY&& order_meta.order_type!=OrderType::SELL {
            return Err(CustomError::InvalidOrderType)
        }

        if order_meta.quantity<=0 {
            return Err(CustomError::InvalidQuantity)
        }
        let user_quant=order_meta.quantity;
        match order_meta.order_type{
            OrderType::BUY=>{
                    let mut remaining_qauntity=user_quant;
            
                
                loop{

                    if self.ask.is_empty(){
                        break;
                    }

                    let Some(mut ask)= self.ask.first_entry() else{
                        println!("dsad");
                        return Err(CustomError::ErrorFetchingAskLevel);
                    };
                    let mut minimum_ask=*ask.key();
                    let seller_price_level=ask.get_mut();
                    
                    let mut ask_total_quantity=seller_price_level.total_quantity;
                    
                    let mut removed_asks_order: VecDeque<Order>=VecDeque::with_capacity(DEFAULT_LEVEL_CAPACITY);//storing all the aks orders that
                    //are used to fill the buy order
    
    
                    if minimum_ask>price{
                        break;
                    }

                    if ask_total_quantity>remaining_qauntity{
                        seller_price_level.total_quantity-=remaining_qauntity;
                        let mut counted_quantity:u64=remaining_qauntity;

                        let seller_queue=&mut seller_price_level.vec_level;
                        //looping through the vecdeque at current price to get the orders that fill the buy price 
                        loop {
                            let Some(front)=seller_queue.front_mut()else{
                                return Err(CustomError::ErrorFetchingVecDeque);
                            };

                            let mut current_order_quant=front.order_metadata.quantity;
                            
                            if current_order_quant>counted_quantity{
                                current_order_quant-=counted_quantity;
                            
                                front.order_metadata.quantity=current_order_quant;
                            
                                removed_asks_order.push_back(Order { 
                                    order_id: front.order_id,
                                    order_metadata:OrderMetadata { quantity:counted_quantity,
                                        ..front.order_metadata} 
                                });
                                break;

                            }else if current_order_quant==counted_quantity{

                                if let Some(remove_order)=seller_queue.pop_front(){
                                    removed_asks_order.push_back(remove_order);
                                };
                                break;
                            }else{
                                //here we have to remove the  the front order from level and store in removed asks vec
                                if let Some(remove_order)=seller_queue.pop_front(){
                                    counted_quantity-=remove_order.order_metadata.quantity;
                                    removed_asks_order.push_back(remove_order);
                                };
                            }
                            

                        }
                        ask_total_quantity-=remaining_qauntity;
                        remaining_qauntity=0;
                        break;
                    }else if ask_total_quantity==remaining_qauntity{
                        seller_price_level.total_quantity-=0;

                        ask_total_quantity=0;
                        remaining_qauntity=0;
                        // let Some( price_lvl)=&mut self.ask.pop_first()else{
                        //     return Err(CustomError::ErrorFetchingAskLevel);
                        // };
                        let mut price_lvl=ask.remove();
                        removed_asks_order.append( &mut price_lvl.vec_level);
                        break;
                    }else{//if buy_qunt>ask_total_qunt 
                        //write the logic to change the variables and move  to the next level
                        seller_price_level.total_quantity-=0;
                        
                        let mut price_lvl=ask.remove();
                        removed_asks_order.append(&mut price_lvl.vec_level);
                        

                        remaining_qauntity-=ask_total_quantity;
                    }


                }

                if remaining_qauntity>0{
                            
                    let new_order_meta=OrderMetadata{quantity:remaining_qauntity,..*order_meta};
                    
                    let new_order=Order::new(order_id,new_order_meta);

                    match self.buy.entry(price) {
                        Entry::Occupied(mut entry) => {
                            // Operation if key ALREADY EXISTS
                            let level=entry.get_mut();
                            level.total_quantity+=remaining_qauntity;
                            level.vec_level.push_back(new_order);
                        }
                        Entry::Vacant(entry) => {
                            // Operation if key is MISSING
                            // println!("Creating new entry");
                            let mut new_level=PriceLevel::new(remaining_qauntity, DEFAULT_LEVEL_CAPACITY);
                            new_level.vec_level.push_back(new_order);
                            entry.insert(new_level);
                        }
                    }
                  

                        
                        
                }

            },
            OrderType::SELL=>{

                    let mut remaining_qauntity=user_quant;
            
                
                loop{

                    if self.buy.is_empty(){
                        break;
                    }

                    let Some(mut buy)= self.buy.last_entry() else{
                        return Err(CustomError::ErrorFetchingbuyLevel);
                    };
                    let mut max_buy=*buy.key();
                    let buy_price_level=buy.get_mut();
                    
                    let mut buy_total_quantity=buy_price_level.total_quantity;
                    
                    let mut removed_buys_order: VecDeque<Order>=VecDeque::with_capacity(DEFAULT_LEVEL_CAPACITY);//storing all the aks orders that
                    //are used to fill the buy order
                    if max_buy<price{
                        break;
                    }

                    if buy_total_quantity>remaining_qauntity{
                        
                        buy_price_level.total_quantity-=remaining_qauntity;
                        let mut counted_quantity:u64=remaining_qauntity;

                        let buyer_queue=&mut buy_price_level.vec_level;
                        //looping through the vecdeque at current price to get the orders that fill the buy price 
                        loop {
                            let Some(front)=buyer_queue.front_mut()else{
                                return Err(CustomError::ErrorFetchingVecDeque);
                            };

                            let mut current_order_quant=front.order_metadata.quantity;
                            
                            if current_order_quant>counted_quantity{
                                current_order_quant-=counted_quantity;
                            
                                front.order_metadata.quantity=current_order_quant;
                                
                                removed_buys_order.push_back(Order { 
                                    order_id: front.order_id,
                                    order_metadata:OrderMetadata { quantity:counted_quantity,
                                        ..front.order_metadata} 
                                });
                                break;

                            }else if current_order_quant==counted_quantity{

                                if let Some(remove_order)=buyer_queue.pop_front(){
                                    removed_buys_order.push_back(remove_order);
                                };
                                break;
                            }else{
                                //here we have to remove the  the front order from level and store in removed buys vec
                                if let Some(remove_order)=buyer_queue.pop_front(){
                                    counted_quantity-=remove_order.order_metadata.quantity;
                                    removed_buys_order.push_back(remove_order);
                                };
                            }
                            

                        }
                        buy_total_quantity-=remaining_qauntity;
                        remaining_qauntity=0;
                        break;
                    }else if buy_total_quantity==remaining_qauntity{
                        buy_price_level.total_quantity-=remaining_qauntity;

                        buy_total_quantity=0;
                        remaining_qauntity=0;
                        // let Some(mut price_lvl)=self.buy.pop_last()else{
                        //     return Err(CustomError::ErrorFetchingbuyLevel);
                        // };
                        let mut price_lvl=buy.remove();
                        removed_buys_order.append(&mut price_lvl.vec_level);
                        break;
                    }else{//if ask_qunt>buy_total_qunt 
                        //write the logic to change the variables and move  to the next level
                        buy_price_level.total_quantity-=0;
                    
                        // let Some( price_lvl)=&mut self.buy.pop_last()else{
                        //     return Err(CustomError::ErrorFetchingbuyLevel);
                        // };
                        let mut price_lvl=buy.remove();
                        removed_buys_order.append(&mut price_lvl.vec_level);
                        

                        remaining_qauntity-=buy_total_quantity;
                        // buy_total_quantity=0;
                    }


                }

                if remaining_qauntity>0{
                    
                            
                    let new_order_meta=OrderMetadata{quantity:remaining_qauntity,..*order_meta};
                    
                    let new_order=Order::new(order_id,new_order_meta);

                    match self.ask.entry(price) {
                        Entry::Occupied(mut entry) => {
                            // Operation if key ALREADY EXISTS
                            let level=entry.get_mut();
                            level.total_quantity+=remaining_qauntity;
                            level.vec_level.push_back(new_order);
                        }
                        Entry::Vacant(entry) => {
                            // Operation if key is MISSING
                            // println!("Creating new entry");
                            let mut new_level=PriceLevel::new(remaining_qauntity, DEFAULT_LEVEL_CAPACITY);
                            new_level.vec_level.push_back(new_order);
                            entry.insert(new_level);
                        }
                    }
                    // let level=self.ask.entry(price);
                    // // .or_insert_with(||PriceLevel::new(remaining_qauntity, DEFAULT_LEVEL_CAPACITY));
                    // level.total_quantity=remaining_qauntity;

                    // level.vec_level.push_back(new_order);

                        
                        
                }
               

            
            }
        }

        Ok("()".into())

    }   

    pub fn print_summary(&self) {
        println!("\n================ ORDER BOOK ================");

        println!("\n------------- ASKS -------------");
        for (price, level) in self.ask.iter().rev() {
            println!("Price: {:<10} | Total Qty: {}", price, level.total_quantity);
        }

        println!("\n------------- BIDS -------------");
        for (price, level) in self.buy.iter().rev() {
            println!("Price: {:<10} | Total Qty: {}", price, level.total_quantity);
        }

        println!("============================================\n");
    }

    pub fn print_detailed(&self) {
        println!("\n================ ORDER BOOK (DETAILED) ================");

        println!("\n------------- ASKS -------------");
        for (price, level) in &self.ask {
            println!("Price: {} | Total Qty: {}", price, level.total_quantity);

            for order in &level.vec_level {
                println!(
                    "   OrderID: {} | Qty: {} | User: {} | Time: {} | Type: {:?}",
                    order.order_id,
                    order.order_metadata.quantity,
                    order.order_metadata.user_id,
                    order.order_metadata.time,
                    order.order_metadata.order_type
                );
            }
        }

        println!("\n------------- BIDS -------------");
        for (price, level) in &self.buy {
            println!("Price: {} | Total Qty: {}", price, level.total_quantity);

            for order in &level.vec_level {
                println!(
                    "   OrderID: {} | Qty: {} | User: {} | Time: {} | Type: {:?}",
                    order.order_id,
                    order.order_metadata.quantity,
                    order.order_metadata.user_id,
                    order.order_metadata.time,
                    order.order_metadata.order_type
                );
            }
        }

        println!("=======================================================\n");
    }
}

fn main() {

    let mut order_book = LimitOrderBook::new(5);

    // BUY 1
    let mut order_meta = OrderMetadata {
        quantity: 5,
        order_type: OrderType::BUY,
        time: Utc::now(),
        user_id: 2313123
    };

    order_book.execute_order(100, 312311213, &mut order_meta).unwrap();


    // BUY 2
    let mut order_meta = OrderMetadata {
        quantity: 9,
        order_type: OrderType::BUY,
        time: Utc::now(),
        user_id: 213131
    };

    match order_book.execute_order(100, 2222, &mut order_meta){
        Ok(_)=>{println!("inserted");},
        Err(_)=>{println!("error")}
    };


    // SELL 1
    let mut order_meta = OrderMetadata {
        quantity: 7,
        order_type: OrderType::SELL,
        time: Utc::now(),
        user_id: 88888
    };

    order_book.execute_order(105, 4444, &mut order_meta).unwrap();


    // SELL 2
    let mut order_meta = OrderMetadata {
        quantity: 3,
        order_type: OrderType::SELL,
        time: Utc::now(),
        user_id: 99999
    };

    order_book.execute_order(104, 5555, &mut order_meta).unwrap();


    order_book.print_summary();
    // order_book.print_detailed();
    // selling
    println!("status before");

    let mut order_meta = OrderMetadata {
        quantity: 15,
        order_type: OrderType::SELL,
        time: Utc::now(),
        user_id: 77777
    };
    
    match order_book.execute_order(100, 6666, &mut order_meta) {
        Ok(_) => {
            println!("after selling 6-100");
            order_book.print_summary();
        },
        Err(e) => {
            println!("{:?}", e);
        }
    };
    // buying
    let mut order_meta = OrderMetadata {
        quantity: 8,
        order_type: OrderType::BUY,
        time: Utc::now(),
        user_id: 55555
    };
    
    match order_book.execute_order(104, 7777, &mut order_meta) {
        Ok(_) => {
            println!("after buying 8-104");
            order_book.print_summary();
        },
        Err(e) => {
            println!("{:?}", e);
        }
    };
    // order_book.print_summary();
    order_book.print_detailed();
}


#[derive(Debug)]
enum CustomError{
    InvalidPrice,
    InvalidQuantity,
    InvalidOrderType,
    ErrorFetchingAskLevel,
    ErrorFetchingbuyLevel,
    ErrorFetchingVecDeque
}

