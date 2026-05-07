use std::io::{self, Write};

mod game_engine {
    use std::{fmt, io::{self}, num::ParseIntError};

    use rand::seq::IteratorRandom;
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum Player {
        X = 1,
        O
    }

    pub trait PlayerTrait {
        fn next(&mut self);
        fn get_next(&self) -> Player;
    }

    impl PlayerTrait for Player {
        fn next(&mut self) {
            *self = match *self {
                Player::X => Player::O,
                Player::O => Player::X,
            }
        }

        fn get_next(&self) -> Player {
            match *self {
                Player::X => Player::O,
                Player::O => Player::X,
            }
        }
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
        fn play_random(&mut self, player: Cell) -> Result<(), &str>;
        fn check_final(&self) -> BoardState;
        fn is_terminal(&self) -> bool;
        fn fmt_xo(&self, index: u8) -> char;
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
        
        fn play_random(&mut self, player: Cell) -> Result<(), &str>
        {
            let mut rng = rand::thread_rng();
            let empty_cells = self.available_cells();
            let random_move = empty_cells
                .iter()
                .enumerate()
                .filter(|(_, value)| **value)
                .map(|(i,_)| i)
                .choose(&mut rng).unwrap();
            
            self.play(random_move, player)
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

    #[derive(Clone, Copy)]
    struct Node {
        board: Board,
        player: Player,
        parent_index: i32,
        children_indices: [i32; 9],
        action: usize,
        visits: i32,
        value: i32,
    }
    
    impl Node {
        pub fn new(parent_index: i32) -> Self {
            let board = [Cell::None; 9];
            let children_indices = [-1; 9];
            let player = Player::X;
            Self { board, player, parent_index, children_indices, visits: 0, value: 0, action: 0}
        }

        pub fn new_from_board(parent_index: i32, board: &Board, player: Player, action: usize) -> Self {
            let children_indices = [-1; 9];
            Self { board: *board, player, parent_index, children_indices, action, visits: 0, value: 0}
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
                if let Err(err)  = sim_board.play_random(player.into()) {
                    panic!("{}", err)
                }
            }
        }

        pub fn backpropagate(&mut self, self_idx: i32, player: Player, mct_nodes: &mut [Node], value: i32) {
            let mut current_idx = self_idx;
            while current_idx != -1 {
                let current = &mut mct_nodes[current_idx as usize];
                current.value += current.get_value_multiplier(player) * value;
                current_idx = current.parent_index;
            }
        }

        fn get_value_multiplier(&self, player: Player) -> i32 {
            if self.player == player {
                1
            } else {
                -1
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

    pub fn mct_play(current_board: &mut Board, iterations: i32, player: Player) {
        let mut mct_nodes: [Node; 512] = [Node::new_from_board(-1, current_board, player.get_next(), 0); 512];
        let mut nodes_size: usize = 1;
        
        for _ in 0..iterations {
            println!("here0");
            let (mut current_node, mut idx) = mct_select(&mct_nodes);
            println!("here1");
            
            if current_node.visits != 0 {
                mct_nodes = mct_expand(&current_node, idx, mct_nodes, &mut nodes_size);
                idx = current_node.children_indices[0];
                current_node = mct_nodes[idx as usize];
            }
            let value = current_node.rollout(player);
            println!("here2");
            current_node.backpropagate(idx, player, &mut mct_nodes, value);
            println!("here3");
        }
        
        let mct_move: usize = mct_best_next_move(&mct_nodes);
        current_board.play(mct_move, player.into()).unwrap();
    }

    fn mct_best_next_move(nodes: &[Node]) -> usize
    {
        let mut max_ucb = f32::NEG_INFINITY;
        let mut max_ucb_idx = 0;
        let node = &nodes[0];
        for idx in node.children_indices {
            let ucb = nodes[idx as usize].ucb_calc(nodes);
            if ucb > max_ucb {
                max_ucb = ucb;
                max_ucb_idx = idx;
            }
        }

        return nodes[max_ucb_idx as usize].action;
    }

    fn mct_select(nodes: &[Node]) -> (Node, i32) {
        let mut curr = &nodes[0];
        let mut max_ucb_child_idx = 0;
        while !curr.is_leaf() {
            let mut max_ucb = f32::NEG_INFINITY;
            let mut max_ucb_child = curr;
            for child_idx in curr.children_indices {
                if child_idx == -1 {
                    break; // no more child
                }
                max_ucb_child_idx = child_idx;
                let child = &nodes[child_idx as usize];
                if child.visits == 0 {
                    return (*child, max_ucb_child_idx);
                }
                let ucb = child.ucb_calc(nodes);
                if ucb > max_ucb {
                    max_ucb = ucb;
                    max_ucb_child = child;
                    max_ucb_child_idx = child_idx;
                }
            }
            curr = max_ucb_child;
            println!("AA");
        }
        return (*curr, max_ucb_child_idx);
    }

    fn mct_expand(node_to_expand: &Node, node_index: i32, mut nodes: [Node; 512], nodes_size: &mut usize) -> [Node; 512] {
        if node_to_expand.is_terminal() {
            return nodes;
        }
        let mut node_board = node_to_expand.board;
        let mut node_player = node_to_expand.player;
        let available_cells= node_board.available_cells();
        let children_nodes: Vec<Node> = available_cells
            .iter()
            .enumerate()
            .filter(|(_, is_empty)| **is_empty)
            .map(|(cell_idx, _)| {
                node_player.next();
                node_board.play(cell_idx, node_player.into()).unwrap();
                Node::new_from_board(node_index, &node_board, node_player, cell_idx)
            })
            .collect();

        for (idx, node) in children_nodes.iter().enumerate() {
            nodes[*nodes_size + idx] = *node;
        }
        *nodes_size = *nodes_size + 9;
        return nodes;
    }
}

use game_engine::*;
use mcts::*;

fn main() {
    // mct_play();

    let mut board: Board = [Cell::None; 9];

    let mut input = String::new();

    let mut current_player = Player::X;

    while board.check_final() == BoardState::None {
        board.print(true);

        if current_player == Player::O {
            print!("{}'s turn -> index: ", current_player);
            io::stdout().flush().unwrap();
            if let Ok(index) = read_index(&mut input) {
                if let Err(err) = board.play(index, Cell::from(current_player)) {
                    println!("{}", err);
                    continue;
                }
            } else {
                println!("Please type a number!");
            }
        } else {
            mct_play(&mut board, 2, Player::X);
        }

        current_player.next();
    }

    board.print(false);
    match board.check_final() {
        BoardState::None => assert!(false),
        BoardState::Win(cell) => println!("Winner is {}", cell),
        BoardState::Draw => println!("It's a Draw!"),
    }
}
