use std::sync::Arc;

use axum::{Json, body, extract::{State, WebSocketUpgrade, ws::{WebSocket,Message}}, response::{IntoResponse, Response}};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use order_book::{ CancelOrder, EngineRequest, OrderDetails, OrderStatus, OrderStatus1, OrderType};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{TradeResponse, UserState,};
 

#[derive(Debug,Serialize,Deserialize)]
pub struct UserPayload{
    pub order_qnt:u64,
    pub price:u64,
    pub quantity: u64,
    pub order_type: OrderType,
    pub user_id: u64,
}



#[derive(Debug,Serialize,Deserialize)]
pub enum UserRequest{
    Place(UserPayload),
    Cancel(CancelOrder)
}

pub async fn place_order(ws:WebSocketUpgrade,State(state):State<Arc<UserState>>,
// Json(payload):Json<UserPayload>,
)->impl IntoResponse{
    println!("d");
    //have to update the userpaylod:add utc time::now for timestap
   ws.on_upgrade(move|socket|{handle_socket(socket,state)})     

}

async fn handle_socket(
    mut socket: WebSocket,
    state: Arc<UserState>,
){
    let (out_tx,mut out_recv)=mpsc::channel::<TradeResponse>(1000);
    let (mut sender, mut receiver) = socket.split();
    
    tokio::spawn(async move{
        while let Some(m)=out_recv.recv().await{

            match m{
                TradeResponse::OrderStatus(order_status)=>{
                    if let Ok(serialized) = serde_json::to_string(&order_status){
                        sender.send(Message::Text(serialized.into())).await;
                    }
                },
                TradeResponse::CancelledOrder(cancel_order)=>{
                    if let Ok(serialized) = serde_json::to_string(&cancel_order){
                        sender.send(Message::Text(serialized.into())).await;
                    }
                },
            }
        }
    });


    while let Some(msg)=receiver.next().await{
    match msg{

        Ok(m)=>{
            if let Ok(p_to_string)=m.to_text(){
                if let Ok(request)=serde_json::from_str::<UserRequest>(p_to_string){
                    
                    match request{
                        UserRequest::Place(payload)=>{
                            let order_type=payload.order_type;
                            let time=Utc::now();
                            let order_details=OrderDetails{
                            order_qnt:payload.order_qnt,
                            price:payload.price,
                            quantity:payload.quantity,
                            order_type:order_type,
                            time,  
                            user_id:payload.user_id 
                            };
                            let new_txn=state.txn.clone();
                            {
                            let mut map=state.map.lock().unwrap();
                            map.insert(payload.user_id, out_tx.clone());
                            }
                            new_txn.send(EngineRequest::Place((order_details))).await;
                        },
                        UserRequest::Cancel(cancel_order)=>{
                            let new_txn=state.txn.clone();
                            {
                            let mut map=state.map.lock().unwrap();
                            map.insert(cancel_order.user_id, out_tx.clone());
                            }
                            new_txn.send(EngineRequest::Cancel(cancel_order)).await;
                        
                        },
                        _=>{

                        }
                    }
                    
                };

            }

        },
        Err(_)=>{

        }
    
    }
    }

}