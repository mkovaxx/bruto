use std::{
    io::{self, Write},
    num::Wrapping,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::io::stdin();
    let mut output = std::io::stdout();

    let mut engine: Box<dyn Engine> = Box::new(Rando::new());

    let mut history = History::new();
    let mut turn = 0;
    let mut human_turn_parity = 0;
    loop {
        let mv = if turn & 1 == human_turn_parity {
            // ask human for next move
            write!(output, "human> ")?;
            output.flush()?;
            let mut input_line = String::new();
            let _count = input.read_line(&mut input_line)?;
            if input_line == "exit\n" {
                break;
            }
            if input_line == "swap\n" {
                human_turn_parity ^= 1;
                continue;
            }
            match parse_move(&input_line) {
                Ok(mv) => mv,
                Err(err) => {
                    writeln!(output, "ERROR: {:?}", err)?;
                    continue;
                }
            }
        } else {
            // ask engine for next move
            write!(output, "engine> ")?;
            let mv = engine.play(&history, turn);
            // print move
            writeln!(output, "{}", print_move(&mv))?;
            mv
        };
        let result = history.try_move(turn, &mv);
        match result {
            Ok(_) => {
                // print board after move
                turn += 1;
                let position = history.get_position(turn);
                position.print(&mut output).unwrap();
                if position.is_quarto() {
                    break;
                }
            }
            Err(_) => {
                writeln!(output, "ERROR: illegal move")?;
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct Move {
    spot: Option<Spot>,
    piece: Option<Piece>,
}

#[derive(Debug)]
enum ParseError {
    InputTooShort,
    InvalidRow,
    InvalidColumn,
    RowColumnMismatch,
    InvalidProp,
    PropsMismatch,
}

fn parse_move(text: &str) -> Result<Move, ParseError> {
    if text.len() < 6 {
        return Err(ParseError::InputTooShort);
    }
    let chars: Vec<char> = text.chars().take(6).collect();
    let spot = parse_spot(chars[0], chars[1])?;
    let piece = parse_piece(chars[2..6].try_into().unwrap())?;
    Ok(Move { spot, piece })
}

fn parse_spot(r: char, c: char) -> Result<Option<Spot>, ParseError> {
    let row = match r {
        'a' => Some(0),
        'b' => Some(1),
        'c' => Some(2),
        'd' => Some(3),
        '.' => None,
        _ => return Err(ParseError::InvalidRow),
    };
    let col = match c {
        '1' => Some(0),
        '2' => Some(1),
        '3' => Some(2),
        '4' => Some(3),
        '.' => None,
        _ => return Err(ParseError::InvalidColumn),
    };
    match (row, col) {
        (Some(row), Some(col)) => Ok(Some(Spot::from_row_col(row, col))),
        (None, None) => Ok(None),
        _ => Err(ParseError::RowColumnMismatch),
    }
}

fn parse_piece(p: &[char; 4]) -> Result<Option<Piece>, ParseError> {
    match [
        parse_prop(p[0])?,
        parse_prop(p[1])?,
        parse_prop(p[2])?,
        parse_prop(p[3])?,
    ] {
        [Some(p0), Some(p1), Some(p2), Some(p3)] => {
            Ok(Some(Piece(p0 | (p1 << 1) | (p2 << 2) | (p3 << 3))))
        }
        [None, None, None, None] => Ok(None),
        _ => Err(ParseError::PropsMismatch),
    }
}

fn parse_prop(p: char) -> Result<Option<i8>, ParseError> {
    match p {
        'o' => Ok(Some(0)),
        'x' => Ok(Some(1)),
        '.' => Ok(None),
        _ => Err(ParseError::InvalidProp),
    }
}

fn print_move(mv: &Move) -> String {
    option_spot_to_chars(&mv.spot)
        .iter()
        .chain(option_piece_to_chars(&mv.piece).iter())
        .collect()
}

trait Engine {
    fn play(&mut self, history: &History, turn: i8) -> Move;
}

struct Rando {
    pcg: Pcg,
}

impl Rando {
    fn new() -> Self {
        Self { pcg: Pcg::new() }
    }
}

impl Engine for Rando {
    fn play(&mut self, history: &History, turn: i8) -> Move {
        let mut response = Move {
            spot: None,
            piece: None,
        };
        if turn >= 1 && turn <= 16 {
            // pick a spot
            let spot_random_source = self.pcg.rand_16_fact();
            let free_spot_count = 17 - turn as u64;
            let spot_index = (spot_random_source % free_spot_count) as i8;
            response.spot = Some(history.get_raw_spot(turn - 1 + spot_index));
        }
        if turn >= 0 && turn <= 15 {
            // pick a piece
            let piece_random_source = self.pcg.rand_16_fact();
            let free_piece_count = 16 - turn as u64;
            let piece_index = (piece_random_source % free_piece_count) as i8;
            response.piece = Some(history.get_raw_piece(turn + piece_index));
        }
        response
    }
}

struct Bruto {
    pcg: Pcg,
    nodes: Vec<Node>,
    temperature_factor: f32,
    playout_batch_size: u32,
}

struct Node {
    value: u32,
    count: u32,
    child_count: usize,
    first_child: usize,
    history: History,
}

impl Engine for Bruto {
    fn play(&mut self, history: &History, turn: i8) -> Move {
        todo!()
    }
}

impl Bruto {
    fn new() -> Self {
        Self {
            pcg: Pcg::new(),
            nodes: vec![],
            temperature_factor: 1.4,
            playout_batch_size: 1000,
        }
    }

    fn expand(&mut self, n: usize, turn: i8) -> [u32; 2] {
        let counters = if self.nodes[n].child_count > 0 {
            let node = &self.nodes[n];
            // pick child node to traverse into
            let ln_n = f32::ln(node.count as f32);
            let mut best_value = 0.0;
            let mut best_index = node.first_child;
            for k in node.first_child..(node.first_child + node.child_count) {
                let child = &self.nodes[k];
                let value = child.value as f32 / child.count as f32
                    + self.temperature_factor * f32::sqrt(ln_n / child.count as f32);
                if value > best_value {
                    best_value = value;
                    best_index = k;
                }
            }
            self.expand(best_index, turn + 1)
        } else {
            // add new children for all legal moves
            let first_child = self.nodes.len();
            if turn >= 1 {
                for piece_index in turn..16 {
                    for spot_index in (turn - 1)..16 {
                        let mut descendant = self.nodes[n].history.clone();
                        descendant.swap_pieces(turn, piece_index);
                        descendant.swap_spots(turn - 1, spot_index);
                        self.nodes.push(Node {
                            value: 0,
                            count: 0,
                            child_count: 0,
                            first_child: 0,
                            history: descendant,
                        });
                    }
                }
            } else {
                for piece_index in 0..16 {
                    let mut descendant = self.nodes[n].history.clone();
                    descendant.swap_pieces(0, piece_index);
                    self.nodes.push(Node {
                        value: 0,
                        count: 0,
                        child_count: 0,
                        first_child: 0,
                        history: descendant,
                    });
                }
            }
            let child_count = self.nodes.len() - first_child;

            self.nodes[n].first_child = first_child;
            self.nodes[n].child_count = child_count;

            // do playouts from the first child
            let mut counters = [0; 2];
            for _i in 0..self.playout_batch_size {
                let result = random_playout(
                    &mut self.nodes[first_child].history,
                    turn + 1,
                    self.pcg.rand_16_fact(),
                    self.pcg.rand_16_fact(),
                );
                match result {
                    Some(final_turn) => {
                        counters[final_turn as usize & 1] += 2;
                    }
                    None => {
                        counters[0] += 1;
                        counters[1] += 1;
                    }
                }
            }

            counters
        };

        self.nodes[n].value += counters[turn as usize & 1];
        self.nodes[n].count += self.playout_batch_size;

        counters
    }
}

struct Position {
    // TODO(mkovaxx): use a more redundant encoding that does less work per move to detect a quarto
    board_pieces: u64,
    board_mask: u64,
    selected_piece: Option<Piece>,
}

impl Position {
    fn new() -> Self {
        Self {
            selected_piece: None,
            board_pieces: 0,
            board_mask: 0,
        }
    }

    fn get_piece(&self, spot: Spot) -> Option<Piece> {
        if (self.board_mask >> (4 * spot.0)) & 0xF != 0 {
            Some(Piece((self.board_pieces >> (4 * spot.0)) as i8 & 0xF))
        } else {
            None
        }
    }

    fn place_piece(&mut self, spot: Spot, piece: Piece) {
        self.board_mask |= (0xF as u64) << (4 * spot.0);
        self.board_pieces |= (piece.0 as u64) << (4 * spot.0);
    }

    fn get_chosen_piece(&self) -> Option<Piece> {
        self.selected_piece
    }

    fn choose_piece(&mut self, piece: Option<Piece>) {
        self.selected_piece = piece;
    }

    fn is_quarto(&self) -> bool {
        let group_masks = [
            // rows
            0x0000_0000_0000_FFFF,
            0x0000_0000_FFFF_0000,
            0x0000_FFFF_0000_0000,
            0xFFFF_0000_0000_0000,
            // columns
            0x000F_000F_000F_000F,
            0x00F0_00F0_00F0_00F0,
            0x0F00_0F00_0F00_0F00,
            0xF000_F000_F000_F000,
            // diagonals
            0xF000_0F00_00F0_000F,
            0x000F_00F0_0F00_F000,
        ];

        let attrib_masks = [
            0x1111_1111_1111_1111,
            0x2222_2222_2222_2222,
            0x4444_4444_4444_4444,
            0x8888_8888_8888_8888,
        ];

        for group_mask in group_masks {
            if self.board_mask & group_mask != group_mask {
                continue;
            }
            let slice = self.board_pieces & group_mask;
            let not_slice = !self.board_pieces & group_mask;
            for attrib_mask in attrib_masks {
                if slice & attrib_mask == 0 || not_slice & attrib_mask == 0 {
                    return true;
                }
            }
        }

        return false;
    }

    /// Print the position
    ///
    /// The format looks like this
    /// [ o | 1 2 3 4 ][ x | 1 2 3 4 ][ o | 1 2 3 4 ][ x | 1 2 3 4 ]
    /// [ --|-------- ][ --|-------- ][ --|-------- ][ --|-------- ]
    /// [ a | . . o . ][ a | . . o . ][ a | . . x . ][ a | . . x . ]
    /// [ b | . . . o ][ b | . . . x ][ b | . . . x ][ b | . . . x ]
    /// [ c | . . . . ][ c | . . . . ][ c | . . . . ][ c | . . . . ]
    /// [ d | . x . . ][ d | . o . . ][ d | . o . . ][ d | . x . . ]
    ///
    /// The position is shown as slices laid out side-by-side, one for each property.
    /// The top-left corners are about the selected piece that must be played.
    /// Each spot on the board is identified by a row and a column label.
    /// Row labels are a..d, and column labels are 1..4.
    ///
    /// Empty spots are shown as dots.
    ///
    /// Note that in turn 0 (before the first move), there is no selected piece.
    /// Similarly, there is no selected piece in a draw (when the board is full).
    /// These states are also indicated by a dot in the top-left corners.
    ///
    /// A quarto is shown as a * in the top-left corners.
    ///
    fn print(&self, writer: &mut dyn io::Write) -> Result<(), io::Error> {
        let row_headers = ['a', 'b', 'c', 'd'];

        let top_left = if self.is_quarto() {
            ['*'; 4]
        } else {
            option_piece_to_chars(&self.get_chosen_piece())
        };
        for p in 0..4 {
            write!(writer, "[ {} | 1 2 3 4 ]", top_left[p])?;
        }
        writeln!(writer)?;

        for _p in 0..4 {
            write!(writer, "[ --|-------- ]")?;
        }
        writeln!(writer)?;

        for r in 0..4 {
            let mut row = [['.'; 4]; 4];
            for c in 0..4 {
                let spot = Spot::from_row_col(r as i8, c as i8);
                row[c] = option_piece_to_chars(&self.get_piece(spot));
            }

            for p in 0..4 {
                write!(writer, "[ {} |", row_headers[r])?;
                for c in 0..4 {
                    write!(writer, " {}", row[c][p])?;
                }
                write!(writer, " ]")?;
            }

            writeln!(writer)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct History {
    pieces_permut: [i8; 16],
    spots_permut: [i8; 16],
}

impl History {
    fn new() -> Self {
        Self {
            pieces_permut: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            spots_permut: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        }
    }

    fn try_move(&mut self, turn: i8, mv: &Move) -> Result<(), ()> {
        let mut temp = self.clone();

        if let Some(spot) = mv.spot {
            if !(turn >= 1 && turn <= 16) {
                return Err(());
            }
            if let Some(spot_index) = (turn - 1..16).find(|i| self.get_raw_spot(*i) == spot) {
                temp.swap_spots(turn - 1, spot_index as i8);
            } else {
                return Err(());
            }
        } else {
            if turn >= 1 && turn <= 16 {
                return Err(());
            }
        }

        if let Some(piece) = mv.piece {
            if !(turn >= 0 && turn <= 15) {
                return Err(());
            }
            if let Some(piece_index) = (turn..16).find(|i| self.get_raw_piece(*i) == piece) {
                temp.swap_pieces(turn, piece_index as i8);
            } else {
                return Err(());
            }
        } else {
            if turn >= 0 && turn <= 15 && !temp.get_position(turn + 1).is_quarto() {
                return Err(());
            }
        }

        *self = temp;
        Ok(())
    }

    fn get_piece(&self, turn: i8) -> Option<Piece> {
        if turn >= 0 && turn <= 15 {
            Some(Piece(self.pieces_permut[turn as usize]))
        } else {
            None
        }
    }

    fn get_raw_piece(&self, index: i8) -> Piece {
        Piece(self.pieces_permut[index as usize])
    }

    fn swap_pieces(&mut self, index_0: i8, index_1: i8) {
        let piece_0 = self.pieces_permut[index_0 as usize];
        let piece_1 = self.pieces_permut[index_1 as usize];
        self.pieces_permut[index_0 as usize] = piece_1;
        self.pieces_permut[index_1 as usize] = piece_0;
    }

    fn get_spot(&self, turn: i8) -> Option<Spot> {
        if turn >= 1 && turn <= 16 {
            Some(Spot(self.spots_permut[turn as usize - 1]))
        } else {
            None
        }
    }

    fn get_raw_spot(&self, index: i8) -> Spot {
        Spot(self.spots_permut[index as usize])
    }

    fn swap_spots(&mut self, index_0: i8, index_1: i8) {
        let spot_0 = self.spots_permut[index_0 as usize];
        let spot_1 = self.spots_permut[index_1 as usize];
        self.spots_permut[index_0 as usize] = spot_1;
        self.spots_permut[index_1 as usize] = spot_0;
    }

    fn get_position(&self, turn: i8) -> Position {
        let mut pos = Position::new();

        for i in 0..turn {
            if let (Some(piece), Some(spot)) = (self.get_piece(i - 1), self.get_spot(i)) {
                pos.place_piece(spot, piece);
            }
            pos.choose_piece(self.get_piece(i));
        }

        pos
    }
}

fn random_playout(
    history: &mut History,
    turn: i8,
    mut piece_random_source: u64,
    mut spot_random_source: u64,
) -> Option<i8> {
    for i in turn..=17 {
        // TODO(mkovaxx): optimize away unpacking the history by updating the position with just the next move
        let position = history.get_position(i);

        /*
        position.print(&mut std::io::stdout()).unwrap();
        println!();
        */

        if position.is_quarto() {
            return Some(i);
        }
        if i <= 15 {
            // pick and commit piece
            let free_piece_count = 16 - i as u64;
            let piece_index = (piece_random_source % free_piece_count) as i8;
            piece_random_source /= free_piece_count;
            history.swap_pieces(i, i + piece_index);
        }
        if i >= 1 && i <= 16 {
            // pick and commit spot
            let free_spot_count = 17 - i as u64;
            let spot_index = (spot_random_source % free_spot_count) as i8;
            spot_random_source /= free_spot_count;
            history.swap_spots(i - 1, i - 1 + spot_index);
        }
    }

    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Piece(i8);

fn piece_to_chars(piece: &Piece) -> [char; 4] {
    let mut chars = ['.'; 4];
    const SYMBOLS: [char; 2] = ['o', 'x'];
    for p in 0..4 {
        chars[p] = SYMBOLS[(piece.0 >> p) as usize & 1];
    }
    chars
}

fn option_piece_to_chars(option_piece: &Option<Piece>) -> [char; 4] {
    match option_piece {
        Some(piece) => piece_to_chars(piece),
        None => ['.'; 4],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Spot(i8);

impl Spot {
    fn from_row_col(row: i8, col: i8) -> Spot {
        Spot(col | (row << 2))
    }
}

fn spot_to_chars(spot: &Spot) -> [char; 2] {
    const ROW_SYMBOLS: [char; 4] = ['a', 'b', 'c', 'd'];
    const COL_SYMBOLS: [char; 4] = ['1', '2', '3', '4'];
    [
        ROW_SYMBOLS[(spot.0 >> 2) as usize & 3],
        COL_SYMBOLS[spot.0 as usize & 3],
    ]
}

fn option_spot_to_chars(option_spot: &Option<Spot>) -> [char; 2] {
    match option_spot {
        Some(spot) => spot_to_chars(spot),
        None => ['.'; 2],
    }
}

/// The initial/default state to initialize the Pcg struct with
const INIT_STATE: u64 = 0x853c_49e6_748f_ea9b;

/// The initial/default incrementing value to initialize the Pcg struct with
const INIT_INC: u64 = 0xda3e_39cb_94b9_5bdb;

/// The value to multiply the state with when a random number is generated in order to
/// alter the random number generator's state
const INCREMENTOR: u64 = 6_364_136_223_846_793_005;

/// 16!
const SIXTEEN_FACTORIAL: u64 = 20_922_789_888_000;

/// Largest k such that k * 16! <= 2^64
const LARGEST_MULTIPLE: u64 = 881657;

/// Taken from the PCG crate, version 4.1.0
struct Pcg {
    state: u64,
    inc: u64,
}

impl Pcg {
    fn new() -> Pcg {
        Pcg {
            state: INIT_STATE,
            inc: INIT_INC,
        }
    }

    fn next_u64(&mut self) -> u64 {
        let old_state = self.state;
        self.state = (Wrapping(old_state) * Wrapping(INCREMENTOR) + Wrapping(self.inc)).0;
        let xor_shifted = (old_state >> 18) ^ old_state >> 27;

        // need to cast to i64 to allow the `-` operator (casting between integers of
        // the same size is a no-op)
        let rot = (old_state >> 59) as i64;
        (xor_shifted >> rot as u64) | (xor_shifted << ((-rot) & 31))
    }

    /// Generate a uniform random number from [0, 16!)
    fn rand_16_fact(&mut self) -> u64 {
        loop {
            let n = self.next_u64();
            if n < LARGEST_MULTIPLE * SIXTEEN_FACTORIAL {
                return n % SIXTEEN_FACTORIAL;
            }
        }
    }
}
