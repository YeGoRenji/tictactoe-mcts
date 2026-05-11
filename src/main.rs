use std::{io::{self, Write}, time::Instant};

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
    use crate::game_engine::*;
    use crate::debug::*;

    const UCB_CNST: f32 = 1.0;
    const NODES_N: usize = 10000;

    #[derive(Clone, Copy, Debug)]
    pub struct Node {
        pub(crate) board: Board,
        pub(crate) player: Player,
        pub(crate) parent_index: i32,
        pub(crate) children_indices: [i32; 9],
        pub(crate) action: usize,
        pub(crate) visits: i32,
        pub(crate) value: i32,
    }
    
    impl Node {

        pub fn new_from_board(parent_index: i32, board: &Board, player: Player, action: usize) -> Self {
            let children_indices = [-1; 9];
            Self { board: *board, player, parent_index, children_indices, action, visits: 0, value: 0}
        }

        pub fn dbg(&self) {
            println!("Node dbg ----");
            self.board.print(false);
            println!("visits = {}", self.visits);
            println!("value = {}", self.value);
            println!("who_played = {}", self.player);
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
                true
            } else {
                false
            }
        }

        pub fn is_terminal(&self) -> bool {
            self.board.check_final() != BoardState::None
        }

        pub fn rollout(&self) -> (i32, Player) {
            let mut sim_board: Board = self.board;
            let mut current_player: Player = self.player;
            loop {
                if sim_board.is_terminal() {
                    // sim_board.print(false);
                    let val = board_state_to_value(sim_board, current_player);
                    return (val, current_player);
                }
                current_player.next();
                if let Err(err) = sim_board.play_random(current_player.into()) {
                    panic!("{}", err)
                }
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
            BoardState::Win(who_won) => if who_won == player { 1 } else { 
                panic!("The last player can't lose");
            },
            BoardState::Draw => 0
        }
    }

    pub fn mct_play(current_board: &mut Board, iterations: i32, bot: Player) {
        let mut mct_nodes: [Node; NODES_N] = [Node::new_from_board(-1, current_board, bot.get_next(), 0); NODES_N];
        let mut nodes_size: usize = 1;
        
        for _ in 0..iterations {
            // println!("Iteration {} ================", it);
            let mut curr_idx= mct_select(&mct_nodes);
            if mct_nodes[curr_idx as usize].visits != 0 {
                mct_nodes = mct_expand(curr_idx, mct_nodes, &mut nodes_size);
                if !mct_nodes[curr_idx as usize].is_leaf() {
                    curr_idx = mct_nodes[curr_idx as usize].children_indices[0];
                }
            }
            let (value, sim_last_player) = mct_nodes[curr_idx as usize].rollout();
            assert!(value == 0 || value == 1);
            backpropagate(curr_idx, &mut mct_nodes, value, sim_last_player);
        }
        
        export_tree_dot(&mct_nodes, nodes_size, 9, "./tree.dot");
        
        let mct_move: usize = mct_best_next_move(&mct_nodes);
        println!("best move is {}", mct_move);
        current_board.play(mct_move, bot.into()).unwrap();
    }

    // value is -1 if bot lost, +1 if bot won, 0 if its draw
    fn backpropagate(self_idx: i32, mct_nodes: &mut [Node], value: i32, sim_last_player: Player) {
        let mut current_idx = self_idx;
        while current_idx != -1 {
            let current = &mut mct_nodes[current_idx as usize];
            current.value += current.get_value_multiplier(sim_last_player) * value;
            current.visits += 1;
            current_idx = current.parent_index;
        }
    }

    fn mct_best_next_move(nodes: &[Node]) -> usize
    {
        let mut max_visits = i32::MIN;
        let mut max_visits_idx = 0;
        let node = &nodes[0];
        for idx in node.children_indices {
            if idx == -1 {
                break;
            }
            let visits = nodes[idx as usize].visits;
            // nodes[idx as usize].dbg();
            if visits > max_visits {
                max_visits = visits;
                max_visits_idx = idx;
            }
        }

        return nodes[max_visits_idx as usize].action;
    }

    fn mct_select(nodes: &[Node]) -> i32 {
        let mut curr = &nodes[0];
        let mut max_ucb_child_idx = 0;
        while !curr.is_leaf() {
            let mut max_ucb = f32::NEG_INFINITY;
            let mut max_ucb_child = curr;
            for child_idx in curr.children_indices {
                if child_idx == -1 {
                    break; // no more child
                }
                let child = &nodes[child_idx as usize];
                if child.visits == 0 { // Has infinite UCB
                    return child_idx;
                }
                let ucb = child.ucb_calc(nodes);
                if ucb > max_ucb {
                    max_ucb = ucb;
                    max_ucb_child = child;
                    max_ucb_child_idx = child_idx;
                }
            }
            curr = max_ucb_child;
        }
        return max_ucb_child_idx;
    }

    fn mct_expand(node_index: i32, mut nodes: [Node; NODES_N], nodes_size: &mut usize) -> [Node; NODES_N] {
        if nodes[node_index as usize].is_terminal() {
            return nodes;
        }
        // println!("Expanding {node_index} ============");
        let node_board = nodes[node_index as usize].board;
        let node_player = nodes[node_index as usize].player;
        let next_player = node_player.get_next();
        let available_cells= node_board.available_cells();
        let children_nodes: Vec<Node> = available_cells
            .iter()
            .enumerate()
            .filter(|(_, is_empty)| **is_empty)
            .map(|(cell_idx, _)| {
                let mut next_board = node_board;
                next_board.play(cell_idx, next_player.into()).unwrap();
                Node::new_from_board(node_index, &next_board, next_player, cell_idx)
            })
            .collect();
        for (idx, node) in children_nodes.iter().enumerate() {
            nodes[*nodes_size + idx] = *node;
            nodes[node_index as usize].children_indices[idx] = (*nodes_size + idx) as i32;
        }
        *nodes_size = *nodes_size + children_nodes.len();
        return nodes;
    }
}

mod debug {
    use crate::game_engine::*;
    use crate::mcts::*;
    use std::fs::File;
    use std::io::Write;

    
    fn board_to_string(board: &Board) -> String {
        format!("|{}|{}|{}|\\n|{}|{}|{}|\\n|{}|{}|{}|",
                    board[0], board[1], board[2],
                    board[3], board[4], board[5],
                    board[6], board[7], board[8])
    }

    pub fn export_tree_dot(nodes: &[Node], size: usize, max_id: usize, path: &str) {
        let mut file = File::create(path).unwrap();

        writeln!(file, "digraph MCTS {{").unwrap();
        writeln!(file, "    node [shape=box];").unwrap();

        for (i, node) in nodes.iter().enumerate() {
            if i > size {
                break;
            }

            if i > max_id {
                break;
            }
            let board_str = board_to_string(&node.board);

            let label = format!(
                "id={}\\na={}\\nV={}\\nN={}\\n{}",
                i,
                node.action,
                node.value,
                node.visits,
                board_str
            );

            writeln!(
                file,
                r#"    {} [label="{}"];"#,
                i,
                label
            ).unwrap();

            for &child in &node.children_indices {
                
                if child == -1 {
                    break;
                }
                if child as usize > max_id {
                    break;
                }

                if child >= 0 {
                    writeln!(file, "    {} -> {};", i, child).unwrap();
                }
            }
        }

        writeln!(file, "}}").unwrap();
    }
}

use game_engine::*;
use mcts::*;

fn main() {
    let mut board: Board = [Cell::None; 9];

    let mut input = String::new();

    let mut current_player = Player::X;

    let user_is = Player::O;
    let bot_is = user_is.get_next();

    while board.check_final() == BoardState::None {
        board.print(true);

        if user_is == current_player {
            print!("{}'s turn -> index: ", current_player);
            io::stdout().flush().unwrap();
            if let Ok(index) = read_index(&mut input) {
                if let Err(err) = board.play(index, Cell::from(current_player)) {
                    println!("{}", err);
                    continue;
                }
                current_player.next();
            } else {
                println!("Please type a number!");
            }
        } else {
            let start = Instant::now(); // Start the timer
    
            mct_play(&mut board, 1000, bot_is);
            
            let duration = start.elapsed(); // Calculate time passed
            
            println!("Time elapsed in my_slow_function() is: {:?}", duration);
            
            current_player.next();
        }
    }

    board.print(false);
    match board.check_final() {
        BoardState::None => assert!(false),
        BoardState::Win(cell) => println!("Winner is {}", cell),
        BoardState::Draw => println!("It's a Draw!"),
    }
}
