use std::fmt::Display;
use std::fmt;

use clap::Parser;

use anyhow::{ Context, Result, anyhow };

#[derive( Parser, Debug )]
#[command( version, about, long_about = None )]
pub struct Args {
    #[arg(short, long)]
    file_name: Option<String>,
}

impl Args {
    pub fn file_name( &self ) -> Option<&str> {
        match &self.file_name {
            Some( file_name ) => Some(&file_name),
            None => None
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

pub struct Game {
    id: usize,
    sets: Vec<Set>,
    red_max: usize,
    green_max: usize,
    blue_max: usize,
}

impl Game {
    pub async fn new( line: String ) -> Option<Game> {
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

        let game = Game {
            id: game_id,
            sets: set,
            red_max,
            green_max,
            blue_max,
        };

        Some( game )
    }
    #[cfg(test)]
    fn id( &self ) -> usize {
        self.id
    }
    pub fn red_max( &self ) -> usize {
        self.red_max
    }
    pub fn green_max( &self ) -> usize {
        self.green_max
    }
    pub fn blue_max( &self ) -> usize {
        self.blue_max
    }
}

#[cfg(test)]
mod game_tests {
    use super::*;

    #[tokio::test]
    async fn test_valid_line() {
        let input_output:Vec< (&str, (usize, usize, usize, usize) ) > = vec![
            (
              "Game 1: 1 red, 2 green, 3 blue ",
                ( 1, 1, 2, 3)
            ),
            (
              "Game 2: 3 red, 8 green, 7 blue ",
                ( 2, 3, 8, 7)
            ),
            (
              "Game 1010: 1337 red, 420 green, 69 blue ",
                ( 1010, 1337, 420, 69)
            ),
        ];
        
        for io in input_output {
            match Game::new( io.0.to_string() ).await {
                Some( game ) => {
                    assert_eq!( io.1.0, game.id() );
                    assert_eq!( io.1.1, game.red_max() );
                    assert_eq!( io.1.2, game.green_max() );
                    assert_eq!( io.1.3, game.blue_max() );
                },
                None => continue,
            }
        }
    }

    #[tokio::test]
    async fn test_valid_sum() {
        let input_output:Vec< (Vec<&str>, usize ) > = vec![
            (
                vec![
                  "Game 1: 1 red, 2 blue, 3 red",
                  "Game 2: 13 red, 14 blue, 15 red",
                  "Game 3: 1 red, 2 blue, 3 red",
                  "Game 4: 1 red, 2 blue, 3 red",
                  "Game 5: 1 red, 2 blue, 3 red",
                ],
                0
            ),
            (
                vec![
                  "Game 1: 13 red, 14 blue, 15 red",
                  "Game 2: 1 red, 1 blue, 1 red",
                  "Game 3: 13 red, 14 blue, 15 red",
                  "Game 4: 13 red, 14 blue, 15 red",
                  "Game 5: 13 red, 14 blue, 15 red",
                ],
                0
            ),
        ];
        
        for io in input_output {
            let mut sum = 0;
            for io in io.0 {
                match Game::new( io.to_string() ).await {
                    Some( game ) => {
                        sum += game.red_max() * game.blue_max() * game.green_max();
                    },
                    None => continue,
                }
            }

            assert_eq!( io.1, sum );
        }
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
