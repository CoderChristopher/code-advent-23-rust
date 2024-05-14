use std::fmt::Display;
use std::fmt;
use std::io::Read;

use clap::Parser;

use anyhow::{ Context, Result, anyhow };

use tokio::join;
use tokio::task::JoinSet;

use std::sync::mpsc::{ channel, Sender, Receiver };
use std::sync::Arc;

#[derive( Parser, Debug )]
#[command( version, about, long_about = None )]
struct Args {
    #[arg(short, long)]
    file_name: Option<String>,
    target_red: usize,
    target_green: usize,
    target_blue: usize,
}

struct Parameters {
    target_red: usize,
    target_green: usize,
    target_blue: usize
}

impl Parameters {
    fn new( cli_args: &Args ) -> Parameters {
        Parameters {
            target_red: cli_args.target_red,
            target_green: cli_args.target_green,
            target_blue: cli_args.target_blue,
        }
    }
}

#[derive( Debug, PartialEq )]
enum CubeColor {
    Red,
    Green,
    Blue
}

impl CubeColor {
    fn new( text: &str ) -> Result<CubeColor> {
        match text {
            "red" => Ok( CubeColor::Red ),
            "green" => Ok( CubeColor::Green ),
            "blue" => Ok( CubeColor::Blue ),
            error => Err( anyhow!( "Undefined color {}", error ) )
        }
    }
}

impl Display for CubeColor {
    fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> Result<(), fmt::Error> {
        match self {
            CubeColor::Red =>   write!( f, "Red" ),
            CubeColor::Green => write!( f, "Green" ),
            CubeColor::Blue =>  write!( f, "Blue" ),
        }
    }
}

struct Cubes {
    count: usize,
    color: CubeColor
}

impl Cubes {
    fn new( cubes_text: &str) -> Result<Vec<Cubes>> {
        let cubes = cubes_text.split(",");

        let mut parsed_cubes = Vec::new();

        for cube in cubes {
            let mut elements = cube.trim().split( " " );

            let cube = Cubes {
                count: elements.next().context( "No cube count!" )?.parse()?,
                color: CubeColor::new( elements.next().context( "No cube color!" )? )?
            };

            parsed_cubes.push( cube );
        }

        Ok( parsed_cubes )
    }
}

impl Display for Cubes {
    fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> Result<(), fmt::Error> {
        write!( f, "{} {}", self.count, self.color )
    }
}

struct Set {
    cubes: Vec<Cubes>
}

impl Set {
    fn new( set_text: &str) -> Result<Vec<Set>> {

        let sets = set_text.split( ";" );
        let mut parsed_sets = Vec::new();

        for set in sets {
            let set = Set {
                cubes: Cubes::new( set )?
            };
            parsed_sets.push( set )
        }

        Ok( parsed_sets )
    }
    fn cube_color_count( &self, color:CubeColor ) -> usize {
        let mut color_count = 0;
        for cube in &self.cubes {
            if cube.color == color {
                color_count += cube.count;
            }
        }
        color_count
    }
}

impl Display for Set {
    fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> Result<(), fmt::Error> {
        for cube in &self.cubes {
            write!( f, "{}, ", cube )?;
        }
        write!( f, "; " )
    }
}

struct Game {
    id: usize,
    sets: Vec<Set>,
    red_max: usize,
    green_max: usize,
    blue_max: usize,
}

impl Game {
    async fn new( parameters: Arc<Parameters>, line: String ) -> Option<Game> {

        let mut colon_split = line.split(":");
        
        let game_id:usize = colon_split
            .next().context("Unable to grab next in ':' sequence").ok()?
            .split(" ")
            .last().context("Unable to grab last in ' ' sequence").ok()?
            .parse().ok()?;

        let games_text = colon_split.next().context( "No game text!" ).ok()?;

        let set = Set::new( games_text ).ok()?;

        let red_max   = set.iter().map( | this_set | { this_set.cube_color_count( CubeColor::Red) } ).max().unwrap_or( 0 );
        let green_max = set.iter().map( | this_set | { this_set.cube_color_count( CubeColor::Green) } ).max().unwrap_or( 0 );
        let blue_max  = set.iter().map( | this_set | { this_set.cube_color_count( CubeColor::Blue) } ).max().unwrap_or( 0 );

        if red_max > parameters.target_red || green_max > parameters.target_green || blue_max > parameters.target_blue {
            return None;
        }

        let game = Game {
            id: game_id,
            sets: set,
            red_max,
            green_max,
            blue_max,
        };

        Some( game )
    }
}

impl Display for Game {
    fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> Result<(), fmt::Error> {
        write!( f, "Game {}: ", self.id )?;
        for set in &self.sets {
            write!( f, "{}", set )?;
        }
        write!( f, "\n\tHigh Red:{},\n\tHigh Green:{},\n\tHigh Blue:{}", self.red_max, self.green_max, self.blue_max )?;
        write!( f, "\n" )
    }
}

async fn distribute_work( parameters: Arc<Parameters>, rx_line: Receiver<String>) -> Result<()> {

    let mut join_set = JoinSet::new();

    while let Ok( input_line ) = rx_line.recv() {
        //println!( "Loading work: {input_line}" );
        join_set.spawn( Game::new( parameters.clone(), input_line ) );
    }

    let mut running_total:usize = 0;

    loop {
        match join_set.join_next().await {
            Some( inner ) => {
                match inner {
                    Ok( inner ) => {
                        if let Some( game ) = inner {
                            println!( "{}", game.id );
                            running_total += game.id;
                        }
                    },
                    Err( err ) => {
                        eprintln!( "Something went wrong {err:?}" );
                    }
                }
            }
            None => {
                println!( "That's all she wrote." );
                break;
            }
        }
    }

    println!( "Total: {running_total}" );

    Ok( () )
}

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
    let args = Args::parse();

    println!( "Game Calculator..." );

    let (tx_line,rx_line) = channel::<String>(); 

    let parameters = Arc::new(Parameters::new( &args ));

    let file_name = args.file_name.unwrap_or("input".to_string());

    let input_future = read_input(&file_name, tx_line);
    let distribute_work_future = distribute_work( parameters.clone(), rx_line);

    join!( input_future, distribute_work_future ).0?;

    Ok( () )
}
