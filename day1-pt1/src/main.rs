use std::io::Read;
use std::fs::File;

use clap::Parser;

use anyhow::{ Result };

use tokio::join;
use tokio::task::JoinSet;

use std::sync::mpsc::{ channel, Sender, Receiver };

#[derive( Parser, Debug )]
#[command( version, about, long_about = None )]
struct Args {
    #[arg(short, long)]
    file_name: Option<String>,
}

async fn read_input( file_name: &str, tx: Sender<String> ) -> Result<()> {

    let mut file = File::open( file_name )?;

    let mut file_contents:Vec<u8> = Vec::new();

    file.read_to_end( &mut file_contents )?;

    let string_contents = String::from_utf8( file_contents )?;

    for line in string_contents.lines() {
        if line != "" {
            if let Err( err ) = tx.send( line.to_string() ) {
                eprintln!( "Error in sending {err}!" );
            }
        }
    }

    Ok( () )
}

async fn distribute_work( rx: Receiver<String> ) -> Result<()> {

    let mut join_set: JoinSet<Option<usize>> = JoinSet::new();

    while let Ok( line ) = rx.recv() {
        println!( "Got it {line}" );
        join_set.spawn( process_line( line ) );
    }

    let mut running_total:usize = 0;

    loop {
        match join_set.join_next().await {
            Some( inner ) => {
                match inner {
                    Ok( inner ) => {
                        running_total += inner.unwrap_or(0);
                    },
                    Err( err ) => {
                        eprintln!( "Error processing line {err}!" );
                    }
                }
            },
            None => {
                println!( "That's all she wrote..." );
                break;
            }
        }
    }

    println!( "Result: {running_total}");

    Ok( () )
}

async fn process_line( line:String ) -> Option<usize> {
    let mut digits:Vec<char> = Vec::new();

    for c in line.chars() {
        if c.is_digit(10) {
            digits.push( c );
        }
    }

    let first_digit:char = *digits.first()?; 
    let final_digit:char = *digits.last()?; 
    
    let string:String = format!( "{}{}", first_digit, final_digit);

    Some(string.parse::<usize>().ok()?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!( "Game Calculator..." );

    let (tx_line,rx_line) = channel::<String>(); 

    let file_name = args.file_name.unwrap_or("input".to_string());

    let input_future = read_input(&file_name, tx_line);
    let distribute_work_future = distribute_work( rx_line);

    join!( input_future, distribute_work_future ).0?;

    Ok( () )
}
