use crate::structs::{ Game, Parameters};

use tokio::task::JoinSet;

use std::sync::mpsc::{ Receiver };
use std::sync::Arc;

use anyhow::{ Result };

pub async fn distribute_work( parameters: Arc<Parameters>, rx_line: Receiver<String>) -> Result<()> {
    let mut join_set = JoinSet::new();

    while let Ok( input_line ) = rx_line.recv() {
        join_set.spawn( Game::new( parameters.clone(), input_line ) );
    }

    let mut running_total:usize = 0;

    loop {
        match join_set.join_next().await {
            Some( inner ) => {
                match inner {
                    Ok( inner ) => {
                        if let Some( game ) = inner {
                            running_total += game.id();
                        }
                    },
                    Err( err ) => {
                        eprintln!( "Something went wrong {err:?}" );
                    }
                }
            }
            None => {
                break;
            }
        }
    }

    println!( "Total: {running_total}" );

    Ok( () )
}
