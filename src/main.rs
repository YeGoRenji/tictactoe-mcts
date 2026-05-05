use std::io::{self, Write};

mod game_engine {
    use std::{fmt, io::{self}, num::ParseIntError};
    #[derive(Clone, Copy, PartialEq)]
    pub enum Player {
        X = 1,
        O
    }
    
    impl fmt::Display for Player {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Player::X => write!(f, "X"),
                Player::O => write!(f, "O"),
            }
        }
    }
    
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum Cell {
        None,
        X,
        O
    }
    
    #[derive(PartialEq, Debug)]
    pub enum BoardState {
        None,
        Win(Cell),
        Draw
    }
    
    impl fmt::Display for Cell {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Cell::None => write!(f, "-"),
                Cell::X => write!(f, "X"),
                Cell::O => write!(f, "O"),
            }
        }
    }
    
    impl From<Player> for Cell {
        fn from(value: Player) -> Self {
            match value {
                Player::X => Cell::X,
                Player::O => Cell::O,
            }
        }
    }
    
    pub type Board = [Cell; 9];
    
    pub trait XOBoard {
        fn print(&self);
        fn play(&mut self, index: usize, player: Cell) -> Result<(), &str>;
        fn check_final(&self) -> BoardState;
        fn fmt_xo(&self, index: u8) -> char;
    }
    
    
    
    impl XOBoard for Board {
        fn print(&self) {
            println!("| {} | {} | {} |", self.fmt_xo(0), self.fmt_xo(1), self.fmt_xo(2));
            println!("-------------");
            println!("| {} | {} | {} |", self.fmt_xo(3), self.fmt_xo(4), self.fmt_xo(5));
            println!("-------------");
            println!("| {} | {} | {} |", self.fmt_xo(6), self.fmt_xo(7), self.fmt_xo(8))
        }
    
        fn fmt_xo(&self, index: u8) -> char
        {
            match self[index as usize] {
                Cell::None => index.to_string().chars().nth(0).unwrap(),
                Cell::X => 'X',
                Cell::O => 'O',
            }
        }
        
        fn play(&mut self, index: usize, player: Cell) -> Result<(), &str> {
            if index >= self.len() {
                return Err("Out of bound!");
            }
            if self[index] != Cell::None {
                return Err("Space is occupied!");
            }
            assert_eq!(self[index], Cell::None);
            self[index] = player;
            
            Ok(())
        }
    
        fn check_final(&self) -> BoardState {
            let possible_wins: [[usize; 3]; 8] = [
                [0, 1, 2],
                [3, 4, 5],
                [6, 7, 8],
                
                [0, 3, 6],
                [1, 4, 7],
                [2, 5, 8],
                
                [0, 4, 8],
                [2, 4, 6],
                ];
                
                // | 0 | 1 | 2 |
                // -------------
                // | 3 | 4 | 5 |
                // -------------
                // | 6 | 7 | 8 |
                for ref possible_win in possible_wins {
                    if self[possible_win[0]] != Cell::None 
                    && self[possible_win[0]] == self[possible_win[1]]
                    && self[possible_win[1]] == self[possible_win[2]] {
                        return BoardState::Win(self[possible_win[0]]);
                    }
                }
                
                if self.contains(&Cell::None) {
                    BoardState::None
                } else {
                    BoardState::Draw
                }
        }
    }
    
    pub fn read_index(buf: &mut String) -> Result<usize, ParseIntError> {
        buf.clear();
        io::stdin().read_line(buf).unwrap();
        buf.trim().parse::<usize>()
    }
}

mod mcts {
    use crate::game_engine::*;

    struct Node {
        board: Board,
        player: Player,
        parent_index: i32,
        children_indeces: [i32; 9],
        visits: i32,
        value: i32,
        ucb: f32
    }
    
    impl Node {
        pub fn new(parent_index: i32) -> Self {
            let board = [Cell::None; 9];
            let children_indeces = [-1; 9];
            let player = Player::X;
            Self { board, player, parent_index, children_indeces, visits: 0, value: 0, ucb: 0.0 }
        }
    }

}

use game_engine::*;
use mcts::*;

fn main() {
    let mut test: Board = [Cell::None; 9];

    let mut input = String::new();

    let mut current_player = Player::X;

    while test.check_final() == BoardState::None {
        test.print();
        print!("{}'s turn -> index: ", current_player);
        io::stdout().flush().unwrap();
        if let Ok(index) = read_index(&mut input) {
            if let Err(err) = test.play(index, Cell::from(current_player)) {
                println!("{}", err);
                continue;
            }
            current_player = if current_player == Player::X { Player::O } else { Player::X };
        } else {
            println!("Please type a number!");
        }
    }

    test.print();
    match test.check_final() {
        BoardState::None => assert!(false),
        BoardState::Win(cell) => println!("Winner is {}", cell),
        BoardState::Draw => println!("It's a Draw!"),
    }
}
