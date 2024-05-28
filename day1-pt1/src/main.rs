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

async fn read_input( file_name: &str, tx: Sender<String>) -> Result<()> {

    let mut file = File::open( file_name )?;

    const BUF_SIZE:usize = 16;

    let mut buffer:[u8; BUF_SIZE] = [0;BUF_SIZE];
    let mut current_chunk = String::new();

    loop {
        if let Ok( read ) = file.read( &mut buffer ) {
            let string = String::from_utf8(buffer[0..read].to_vec()).unwrap();
            tx.send( string );

            if read < BUF_SIZE {
                break;
            }
        } else {
            break;
        }
    }


    Ok( () )
}

async fn chunker( rx: Receiver<String>, tx: Sender<String> ) -> Result<()> {

    let mut current_chunk = String::new();

    while let Ok( chunk ) = rx.recv() {
        current_chunk.push_str( &chunk );

        while let Some( (first,second ) ) = current_chunk.split_once( "\n" ) {
            if let Err( err ) = tx.send( first.to_string() ) {
                eprintln!( "Error in sending {err}!" );
            }
            current_chunk = second.to_string();
        }
    }
    Ok( () )
}

async fn distribute_work( rx: Receiver<String>) -> Result<()> {

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
    let (tx_line2,rx_line2) = channel::<String>(); 

    let file_name = args.file_name.unwrap_or("input".to_string());

    let input_future = read_input(&file_name, tx_line);
    let chunker_future = tokio::spawn(chunker(rx_line, tx_line2));
    let distribute_work_future = tokio::spawn(distribute_work( rx_line2));

    join!( input_future, chunker_future, distribute_work_future ).0?;

    Ok( () )
}
