#![allow(unused)]

use std::{collections::HashMap, net::TcpListener, sync::{Arc, Mutex}};

use anyhow::Result;
use axum::{Router, routing::{post,get}};
use order_book::{LimitOrderBook, OrderDetails, OrderStatus};
use tokio::sync::{ mpsc::{self, Sender}};

use crate::apis::order_placing::place_order;

const PORT:usize=3000;

const TICK_SIZE:u64=5;

const MULTIPLIER:u64=5;

#[derive(Debug,Clone)]
pub struct UserState{
    txn:Sender<OrderDetails>,
    map:Arc<Mutex<HashMap<u64,Sender<OrderStatus>>>>
}
mod test_fn;

mod apis;

type MapType=Arc<Mutex<HashMap<u64,Sender<OrderStatus>>>>;

#[tokio::main]
async fn main()->Result<()>{

    let mut map:MapType=Arc::new(Mutex::new(HashMap::new()));
    let (txn,mut recv)=mpsc::channel::<OrderDetails>(100000);
    
    let map_clone=map.clone();

    let state=Arc::new(UserState{
        txn:txn.clone(),
        map
    });
    
    let recv_handle=tokio::spawn(async move{

        let mut lob:LimitOrderBook=LimitOrderBook::new(5, 10);
        println!("here");
        while let Some(orders)=recv.recv().await{
            println!("H");
            if let Ok(response)=lob.placing_order(orders){
                let user_id=response.order_meta.user_id;
                

                let out_txn={
                    let map=map_clone.lock().unwrap();
                    map.get(&user_id).unwrap().clone()
                };

                out_txn.send(response).await;

            };
            // println!("{:?}",response);
        }

    });

    let app=Router::new()
    .route("/placeOrder", get(place_order))
    .with_state(state);

    

    let listener=tokio::net::TcpListener::bind(format!("0.0.0.0:{}",PORT)).await?;
    axum::serve(listener,app).await?;

    Ok(())
}