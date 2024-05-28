use anyhow::{ Result };

use std::io::Read;
use std::fs::File;

use tokio::sync::mpsc::{ UnboundedSender };

const BUF_SIZE:usize = 16;

pub async fn read_input( file_name: &str, tx: UnboundedSender<String>) -> Result<()> {

    let mut file = File::open( file_name )?;

    let mut buffer:[u8; BUF_SIZE] = [0;BUF_SIZE];

    loop {
        if let Ok( read ) = file.read( &mut buffer ) {
            let string = String::from_utf8(buffer[0..read].to_vec()).unwrap();
            tx.send( string )?;

            if read < BUF_SIZE {
                break;
            }
        } else {
            break;
        }
    }


    Ok( () )
}

#[cfg(test)]
mod file_tests {
    use super::*;
    use tokio::sync::mpsc::{ unbounded_channel };

    #[tokio::test]
    async fn test_read_file() {
        const FILE_NAME:&str = "tests/input";
        let output:Vec<String> = vec![
            "abc\n".to_string(),
            "def\n".to_string(),
            "ghi\n".to_string(),
            "jkl\n".to_string(),
            "mno\n".to_string(),
            "pqr\n".to_string(),
            "stu\n".to_string(),
            "vwx\n".to_string(),
            "yz\n".to_string(),
            "\n".to_string(),
        ];

        let output_stringified = output.iter().fold( "".to_string(), | a, b| format!( "{}{}", a, b) );

        let (tx,mut rx) = unbounded_channel::<String>();

        let read_result = read_input( FILE_NAME, tx).await;

        let mut current_index = 0;

        while let Some( read_line ) = rx.recv().await {
            let expected_result;
            if current_index + BUF_SIZE > output_stringified.len() {
                expected_result = &output_stringified[current_index..];
            } else {
                expected_result = &output_stringified[current_index..(current_index+BUF_SIZE)];
            }

            assert_eq!( read_line, expected_result );

            current_index += BUF_SIZE;
            if current_index > output_stringified.len() {
                break;
            }
        }

        assert!( read_result.is_ok() );

    }
}
