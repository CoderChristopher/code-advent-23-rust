mod structs;
mod processing;

use std::io::Read;

use clap::Parser;

use anyhow::{ Result };

use tokio::join;

use std::sync::mpsc::{ channel, Sender };


async fn read_input(input_file:&str, tx_line: Sender<String>) -> Result<()> {
    let mut file = std::fs::File::open( input_file )?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    for line in contents.lines() {
        if line != "" {
            tx_line.send( line.to_string() )?;
        }
    }
    Ok( () )
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = structs::Args::parse();

    println!( "Game Calculator..." );

    let (tx_line,rx_line) = channel::<String>(); 

    let file_name = args.file_name().unwrap_or("input");

    let input_future = read_input(&file_name, tx_line);
    let distribute_work_future = processing::distribute_work( rx_line);

    join!( input_future, distribute_work_future ).0?;

    Ok( () )
}
