use anyhow::{ Result };
use tokio::sync::mpsc::{ UnboundedSender, UnboundedReceiver };

use tokio::task::JoinSet;

const NUMBER_TO_CHAR:[( &str, char);9] = [
    (   "one", '1'),
    (   "two", '2'),
    ( "three", '3'),
    (  "four", '4'),
    (  "five", '5'),
    (   "six", '6'),
    ( "seven", '7'),
    ( "eight", '8'),
    (  "nine", '9'),
];

async fn process_line( line:String ) -> Option<usize> {
    let mut tokens:Vec<(usize, char)> = Vec::new();

    for ( string, number) in NUMBER_TO_CHAR {
        let mut first_index = 0;
        while let Some( loc ) = line[first_index..].find( string ) {
            tokens.push( ( first_index + loc, number ) );
            first_index +=  loc + 1;
        }
    }

    for ( index, c) in line.chars().enumerate() {
        if c.is_digit(10) {
            tokens.push( ( index, c ) );
        }
    }

    tokens.sort_by( | a, b| a.0.partial_cmp( &b.0).unwrap() );

    let first_digit:char = tokens.first()?.1; 
    let final_digit:char = tokens.last()?.1; 
    
    let string:String = format!( "{}{}", first_digit, final_digit);


    Some(string.parse::<usize>().ok()?)
}

#[cfg(test)]
mod process_tests {
    use super::*;

    async fn test_process_line( input_output: Vec<(&str, usize)> ) {
        for io in input_output {
            assert_eq!( process_line( io.0.to_string() ).await, Some(io.1) );
        }
    }

    #[tokio::test]
    async fn test_process_line_text() {
        let input_output:Vec<(&str, usize)> = vec![
            ("one", 11),
            ("two", 22),
            ("three", 33),
            ("four", 44),
            ("five", 55),
            ("six", 66),
            ("seven", 77),
            ("eight", 88),
            ("nine", 99),
            ("oneight", 18),
            ("threeight", 38),
            ("fiveight", 58),
            ("nineight", 98),
            ("sevenine", 79),
            ("eightwo", 82),
            ("eighthree", 83),
        ];
        test_process_line( input_output ).await;
    }
    #[tokio::test]
    async fn test_process_line_digits() {
        let input_output:Vec<(&str, usize)> = vec![
            ("123", 13),
            ("456", 46),
            ("789", 79),
            ("1235", 15),
        ];
        test_process_line( input_output ).await;
    }
    #[tokio::test]
    async fn test_process_line_text_digits() {
        let input_output:Vec<(&str, usize)> = vec![
            ("one23", 13),
            ("4five6", 46),
            ("78nine", 79),
            ("one2twofive", 15),
            ("threeight7", 37),
        ];
        test_process_line( input_output ).await;
    }
    #[tokio::test]
    async fn test_process_line_no_digits() {
        assert_eq!( process_line( "azyx".to_string() ).await, None );
        assert_eq!( process_line( "-?a@$*(@".to_string() ).await, None );
    }
}

pub async fn chunker(
    mut rx: UnboundedReceiver<String>,
    tx: UnboundedSender<String>
) -> Result<()> {
    let mut current_chunk = String::new();

    while let Some( chunk ) = rx.recv().await {
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

#[cfg(test)]
mod chunker_tests {
    use super::*;
    use tokio::sync::mpsc::{ unbounded_channel };

    #[tokio::test]
    async fn test_long_line() {
        let input_output:Vec<(&str, Vec<&str>)> = vec![
            ("abc\n123\ncome-with-me\n", [ "abc", "123", "come-with-me"].to_vec() ),
            ("neah\nnee\nwee\n", [ "neah", "nee", "wee"].to_vec() ),
            ("neah\nnee\nwee\nfeh\nleh\njeh\ntee\n", [ "neah", "nee", "wee", "feh", "leh", "jeh", "tee"].to_vec() ),
            ("neah\nnee\nwee\nfeh\nleh\njeh\ntee", [ "neah", "nee", "wee", "feh", "leh", "jeh"].to_vec() ),
        ];


        for ( input, output) in input_output {
            let (tx,rx) = unbounded_channel::<String>();
            let (tx_2,mut rx_2) = unbounded_channel::<String>();
            let chunker_future = tokio::spawn(chunker(rx, tx_2));

            let send = tx.send( input.to_string() );
            assert!( send.is_ok() );

            for out in output {
                assert_eq!( rx_2.recv().await, Some( out.to_string() ) );
            }

            drop( tx );
            let chunker_result = chunker_future.await;
            assert!( chunker_result.is_ok() );
        }
    }
}

pub async fn distribute_work( mut rx: UnboundedReceiver<String>) -> Result<()> {

    let mut join_set: JoinSet<Option<usize>> = JoinSet::new();

    while let Some( line ) = rx.recv().await {
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
