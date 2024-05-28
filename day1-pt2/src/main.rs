mod processor;
mod read;

use clap::Parser;

use anyhow::{ Result };

use tokio::join;

use tokio::sync::mpsc::{ unbounded_channel };

#[derive( Parser, Debug )]
#[command( version, about, long_about = None )]
struct Args {
    #[arg(short, long)]
    file_name: Option<String>,
}


#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!( "Game Calculator..." );

    let (tx_line,rx_line) = unbounded_channel::<String>(); 
    let (tx_line2,rx_line2) = unbounded_channel::<String>(); 

    let file_name = args.file_name.unwrap_or("input".to_string());

    let input_future = read::read_input(&file_name, tx_line);
    let chunker_future = tokio::spawn(processor::chunker(rx_line, tx_line2));
    let distribute_work_future = tokio::spawn(processor::distribute_work( rx_line2));

    join!( input_future, chunker_future, distribute_work_future ).0?;

    Ok( () )
}
