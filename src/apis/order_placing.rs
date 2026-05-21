use std::sync::Arc;

use axum::{Json, body, extract::{State, WebSocketUpgrade, ws::{WebSocket,Message}}, response::{IntoResponse, Response}};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use order_book::{OrderDetails, OrderStatus, OrderType};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::UserState;
 

#[derive(Debug,Deserialize)]
pub struct UserPayload{
    pub order_qnt:u64,
    pub price:u64,
    pub quantity: u64,
    pub order_type: OrderType,
    pub user_id: u64,
}

pub async fn place_order(ws:WebSocketUpgrade,State(state):State<Arc<UserState>>,
// Json(payload):Json<UserPayload>,
)->impl IntoResponse{
    println!("dada");
    //have to update the userpaylod:add utc time::now for timestap
   ws.on_upgrade(move|socket|{handle_socket(socket,state)})     

}

async fn handle_socket(
    mut socket: WebSocket,
    state: Arc<UserState>,
){

    let (out_tx,mut out_recv)=mpsc::channel::<OrderStatus>(1000);
    let (mut sender, mut receiver) = socket.split();
    tokio::spawn(async move{
        while let Some(m)=out_recv.recv().await{
            if let Ok(serialized)=serde_json::to_string::<OrderStatus>(&m){
                
                sender.send(Message::Text(serialized.into())).await;
            }
        }
    });


    while let Some(msg)=receiver.next().await{
    match msg{

        Ok(m)=>{
            if let Ok(p_to_string)=m.to_text(){
                if let Ok(payload)=serde_json::from_str::<UserPayload>(p_to_string){

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
                    new_txn.send(order_details).await;
                };
            }

        },
        Err(_)=>{

        }
    
    }
    }

}