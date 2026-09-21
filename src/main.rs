#![allow(unused)]

use std::{collections::HashMap, net::TcpListener, sync::{Arc, Mutex}};

use anyhow::Result;
use axum::{Router, routing::{post,get}};
use order_book::{CancelOrderStatus, EngineEvents, EngineRequest, LimitOrderBook, OrderDetails, OrderStatus, OrderStatus1};
use tokio::sync::{ mpsc::{self, Sender}};

use crate::apis::order_placing::{UserRequest, place_order};

const PORT:usize=3000;

const TICK_SIZE:u64=5;

const MULTIPLIER:u64=5;

#[derive(Debug,Clone)]
pub struct UserState{
    txn:Sender<EngineRequest>,
    map:Arc<Mutex<HashMap<u64,Sender<TradeResponse>>>>
}

#[derive(Debug)]
pub enum TradeResponse{
    OrderStatus(OrderStatus1),
    CancelledOrder(CancelOrderStatus)
}

mod test_fn;

mod apis;

type MapType=Arc<Mutex<HashMap<u64,Sender<TradeResponse>>>>;

#[tokio::main]
async fn main()->Result<()>{

    let mut map:MapType=Arc::new(Mutex::new(HashMap::new()));
    let (txn,mut recv)=mpsc::channel::<EngineRequest>(100000);
    
    let map_clone=map.clone();

    let state=Arc::new(UserState{
        txn:txn.clone(),
        map
    });
    
    let recv_handle=tokio::spawn(async move{

        let mut lob:LimitOrderBook=LimitOrderBook::new(5, 10);
        println!("here");
        while let Some(orders)=recv.recv().await{
            if let Ok(events)=lob.placing_order(orders){
                    // println!("{:?}",events);

                    match events{
                        EngineEvents::OrderStatus(status_vec)=>{
                            for e in status_vec{
                                //     here we will compute for the events 
                                //     since orderstatus contains the matcher id and user id the corresponding price
                                //     and the filled qnt
                                //     So we have to fetch their current details from 
                                    
                                    let user_id=e.order_meta.user_id;
                            
            
                                    let out_txn={
                                        let map=map_clone.lock().unwrap();
                                        map.get(&user_id).unwrap().clone()
                                    };
            
                                    let os=e;
            
                                    out_txn.send(TradeResponse::OrderStatus(os)).await;
                                }
                        },
                        EngineEvents::CanceledOrder(cancel_order_status)=>{
                            //todo
                            let user_id=cancel_order_status.user_id;
                            let out_txn={
                                let map=map_clone.lock().unwrap();
                                map.get(&user_id).unwrap().clone()
                            };
    
                            // let os=e;
    
                            out_txn.send(TradeResponse::CancelledOrder(cancel_order_status)).await;

                        }
                    }

                   
                // let user_id=events.order_meta.user_id;
                

                // let out_txn={
                //     let map=map_clone.lock().unwrap();
                //     map.get(&user_id).unwrap().clone()
                // };

                // out_txn.send(events).await;

            };
            // println!("{:?}",events);
        }

    });

    let app=Router::new()
    .route("/placeOrder", get(place_order))
    .with_state(state);

    

    let listener=tokio::net::TcpListener::bind(format!("0.0.0.0:{}",PORT)).await?;
    axum::serve(listener,app).await?;

    Ok(())
}