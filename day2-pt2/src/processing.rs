use crate::structs::{ Game };

use tokio::task::JoinSet;

use std::sync::mpsc::{ Receiver };

use anyhow::{ Result };

pub async fn distribute_work( rx_line: Receiver<String>) -> Result<()> {
    let mut join_set = JoinSet::new();

    while let Ok( input_line ) = rx_line.recv() {
        join_set.spawn( Game::new( input_line ) );
    }

    let mut running_total:usize = 0;

    loop {
        match join_set.join_next().await {
            Some( inner ) => {
                match inner {
                    Ok( inner ) => {
                        if let Some( game ) = inner {
                            running_total += game.red_max() * game.blue_max() * game.green_max();
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
