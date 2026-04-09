#![allow(unused)]
use std::{ collections::{BTreeMap, VecDeque}, thread::{current, sleep}, time::{SystemTime, UNIX_EPOCH}};

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

    fn execute_order(&mut self,price:u64,order_id:u64,order_meta:OrderMetadata)->Result<String,CustomError>{
        if price<=0 {
            return Err(CustomError::InvalidPrice)
        }

        let order_type=order_meta.order_type;
        if order_type!=OrderType::BUY||order_type!=OrderType::SELL {
            return Err(CustomError::InvalidOrderType)
        }

        if order_meta.quantity<=0 {
            return Err(CustomError::InvalidQuantity)
        }
        let buyer_quantity=order_meta.quantity;
        match order_type{
            OrderType::BUY=>{
                if !self.ask.is_empty(){
                
                let Some(ask)=&mut self.ask.first_entry() else{
                    return Err(CustomError::ErrorFetchingAskLevel);
                };
                    let mut minimum_ask=*ask.key();
                    let seller_price_level=ask.get_mut();
                    
                    let mut ask_total_quantity=seller_price_level.total_quantity;
                    
                    let mut remaining_qauntity=buyer_quantity;
                    let mut removed_asks_order: VecDeque<Order>=VecDeque::with_capacity(DEFAULT_LEVEL_CAPACITY);//storing all the aks orders that
                    //are used to fill the buy order


                    while  minimum_ask<=price{

                        if ask_total_quantity>remaining_qauntity{
                            
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

                            ask_total_quantity=0;
                            remaining_qauntity=0;
                            let Some(mut price_lvl)=self.ask.pop_first()else{
                                return Err(CustomError::ErrorFetchingAskLevel);
                            };
                            removed_asks_order.append(&mut price_lvl.1.vec_level);
                            break;
                        }else{//if buy_qunt>ask_total_qunt 
                            //write the logic to change the variables and move  to the next level
                        
                            // let Some( price_lvl)=&mut self.ask.pop_first()else{
                            //     return Err(CustomError::ErrorFetchingAskLevel);
                            // };
                            // removed_asks_order.append(&mut price_lvl.1.vec_level);
                            

                            remaining_qauntity-=ask_total_quantity;
                            ask_total_quantity=0;
                        }


                    }

                    if remaining_qauntity>0{
                        if self.buy.contains_key(&price){

                            if let Some(current_price_level)=self.buy.get_mut(&price){
                                current_price_level.total_quantity+=order_meta.quantity;
     
                                let new_order=Order::new(order_id,order_meta);
     
                                current_price_level.vec_level.push_back(new_order);
     
                            };
                         }else{
                             let new_order=Order::new(order_id,order_meta);
     
                             let level=self.buy.entry(price)
                             .or_insert_with(||PriceLevel::new(order_meta.quantity, DEFAULT_LEVEL_CAPACITY));
                         
                         }
                    }
                }else{
                    // if sellers queue is empty
                    if self.buy.contains_key(&price){

                       if let Some(current_price_level)=self.buy.get_mut(&price){
                           current_price_level.total_quantity+=order_meta.quantity;

                           let new_order=Order::new(order_id,order_meta);

                           current_price_level.vec_level.push_back(new_order);

                       };
                    }else{
                        let new_order=Order::new(order_id,order_meta);

                        let level=self.buy.entry(price)
                        .or_insert_with(||PriceLevel::new(order_meta.quantity, DEFAULT_LEVEL_CAPACITY));
                    
                    }

                };

            },
            OrderType::SELL=>{

            }
        }

        Ok("()".into())

    }   

}

fn main() {


    let mut map = BTreeMap::from([(1, "a"), (2, "b"), (3, "c")]);

    // Manipulate the smallest key (1)
    if let Some(mut entry) = map.first_entry() {
        *entry.get_mut() = "new_first";
    }
    
    // Manipulate the largest key (3)
    if let Some(mut entry) = map.last_entry() {
        entry.remove(); // Removes key 3 from the map
    }


    println!("Hello, world!");
}

enum CustomError{
    InvalidPrice,
    InvalidQuantity,
    InvalidOrderType,
    ErrorFetchingAskLevel,
    ErrorFetchingVecDeque
}

