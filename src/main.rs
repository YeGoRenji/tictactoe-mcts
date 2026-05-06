use std::io::{self, Write};

mod game_engine {
    use std::{fmt, io::{self}, num::ParseIntError};

use rand::seq::IteratorRandom;
    #[derive(Clone, Copy, PartialEq, Debug)]
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
    
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum BoardState {
        None,
        Win(Player),
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

    impl From<Cell> for Player {
        fn from(value: Cell) -> Self {
            match value {
                Cell::X => Player::X,
                Cell::O => Player::O,
                Cell::None => panic!("empty cell has no player"),
            }
        }
    }
    
    pub type Board = [Cell; 9];
    
    pub trait XOBoard {
        fn print(&self, indices: bool);
        fn play(&mut self, index: usize, player: Cell) -> Result<(), &str>;
        fn check_final(&self) -> BoardState;
        fn is_terminal(&self) -> bool;
        fn fmt_xo(&self, index: u8) -> char;
        fn play_random(&mut self, player: Cell);
        fn available_cells(&self) -> [bool; 9];
    }
    
    impl XOBoard for Board {
        fn print(&self, indices: bool) {
            if indices {
                println!("| {} | {} | {} |", self.fmt_xo(0), self.fmt_xo(1), self.fmt_xo(2));
                println!("-------------");
                println!("| {} | {} | {} |", self.fmt_xo(3), self.fmt_xo(4), self.fmt_xo(5));
                println!("-------------");
                println!("| {} | {} | {} |", self.fmt_xo(6), self.fmt_xo(7), self.fmt_xo(8))
            } else {
                println!("| {} | {} | {} |", self[0], self[1], self[2]);
                println!("-------------");
                println!("| {} | {} | {} |", self[3], self[4], self[5]);
                println!("-------------");
                println!("| {} | {} | {} |", self[6], self[7], self[8])
            }
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

        fn available_cells(&self) -> [bool; 9] {
            self.map(|cell| match cell {
                Cell::None => true,
                _ => false
            })
        }
        
        fn play_random(&mut self, player: Cell)
        {
            let mut rng = rand::thread_rng();
            let empty_cells = self.available_cells();
            let random_move = empty_cells
                .iter()
                .enumerate()
                .filter(|(_, value)| **value)
                .map(|i| i)
                .choose(&mut rng).unwrap();
            self.play(random_move.0, player);
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
                        return BoardState::Win(self[possible_win[0]].into());
                    }
                }
                
                if self.contains(&Cell::None) {
                    BoardState::None
                } else {
                    BoardState::Draw
                }
        }
        
        fn is_terminal(&self) -> bool {
            self.check_final() != BoardState::None
        }
    }
    
    pub fn read_index(buf: &mut String) -> Result<usize, ParseIntError> {
        buf.clear();
        io::stdin().read_line(buf).unwrap();
        buf.trim().parse::<usize>()
    }
}

mod mcts {
    use std::fmt;

    use crate::game_engine::*;

    const UCB_CNST: f32 = 2.0;

    struct Node {
        board: Board,
        player: Player,
        parent_index: i32,
        children_indices: [i32; 9],
        visits: i32,
        value: i32,
    }
    
    impl Node {
        pub fn new(parent_index: i32) -> Self {
            let board = [Cell::None; 9];
            let children_indices = [-1; 9];
            let player = Player::X;
            Self { board, player, parent_index, children_indices, visits: 0, value: 0}
        }

        pub fn dbg(&self) {
            println!("Node dbg ----");
            self.board.print(false);
            println!("visits = {}", self.visits);
            println!("value = {}", self.value);
            println!("END  dbg ----");
        }

        pub fn ucb_calc(&self, nodes: &[Node]) -> f32 {
            if self.parent_index == -1 {
                return 0.0;
            }
            if self.visits == 0 {
                return f32::INFINITY;
            }
            let exploit = self.value as f32 / self.visits as f32;
            let parent = &nodes[self.parent_index as usize];
            let explore = f32::sqrt(f32::ln(parent.visits as f32) / self.visits as f32);
            return exploit + UCB_CNST * explore;
        }

        pub fn is_leaf(&self) -> bool {
            if self.children_indices[0] == -1 {
                false
            } else {
                true
            }
        }

        pub fn is_terminal(&self) -> bool {
            self.board.check_final() != BoardState::None
        }

        pub fn rollout(&self, player: Player) -> i32 {
            let mut sim_board: Board = self.board;
            loop {
                if sim_board.is_terminal() {
                    return board_state_to_value(sim_board, player);
                }
                sim_board.play_random(player.into());
            }
        }
    }

    fn board_state_to_value(board: Board, player: Player) -> i32 {
        match board.check_final() {
            BoardState::None => panic!("Shouldn't value a non terminal state"),
            BoardState::Win(cell) => if cell == player { 1 } else { -1 },
            BoardState::Draw => 0
        }
    }

    pub fn mct_play(current_board: &Board, iterations: i32, player: Player) {
        let mut mct_nodes: [Node; 512] = core::array::from_fn(|_| Node::new(-1));
        let mut nodes_size = 1;

        for _ in 0..iterations {
            let current_node = mct_select(&mct_nodes);

            if current_node.visits == 0 {
                current_node.rollout(player);
            }
        }
    }

    fn mct_select(nodes: &[Node]) -> &Node {
        let mut curr = &nodes[0];
        while !curr.is_leaf() {
            let mut max_ucb = f32::NEG_INFINITY;
            let mut max_ucb_child = curr;
            for child_idx in curr.children_indices {
                if child_idx == -1 {
                    break; // no more child
                }
                let child = &nodes[child_idx as usize];
                if child.visits == 0 {
                    return child;
                }
                let ucb = child.ucb_calc(nodes);
                if ucb > max_ucb {
                    max_ucb = ucb;
                    max_ucb_child = child;
                }
            }
            curr = max_ucb_child;
        }
        return curr;
    }
}

use game_engine::*;
use mcts::*;

fn main() {
    // mct_play();

    let mut test: Board = [Cell::None; 9];

    let mut input = String::new();

    let mut current_player = Player::X;

    while test.check_final() == BoardState::None {
        test.print(true);
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

    test.print(false);
    match test.check_final() {
        BoardState::None => assert!(false),
        BoardState::Win(cell) => println!("Winner is {}", cell),
        BoardState::Draw => println!("It's a Draw!"),
    }
}
