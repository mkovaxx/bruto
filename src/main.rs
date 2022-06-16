use std::{
    io::{self, Write},
    num::Wrapping,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::io::stdin();
    let mut output = std::io::stdout();

    let mut engine = Rando::new();

    let mut pcg = Pcg::new();

    let mut noms = [0, 0];
    let mut den = 0;

    const N: u32 = 1_000_000;

    for _i in 0..N {
        let mut history = History::new();
        let result = random_playout(&mut history, 0, pcg.rand_16_fact(), pcg.rand_16_fact());

        //println!("result: {:?}", result);

        /*
        if result == None {
            for turn in 0..=17 {
                let pos = history.get_position(turn);
                pos.print(&mut output).unwrap();
                println!();
            }
        }
        */
        if let Some(turn) = result {
            noms[turn as usize & 1] += 1;
        }
    }

    println!("win:  {}", noms[0] as f64 / N as f64);
    println!("lose: {}", noms[1] as f64 / N as f64);
    println!("draw: {}", (N - noms[0] - noms[1]) as f64 / N as f64);

    /*
    let mut position = Position::new();
    loop {
        write!(output, "> ")?;
        output.flush()?;
        let mut input_line = String::new();
        let _count = input.read_line(&mut input_line)?;
        if input_line.starts_with("exit") {
            break;
        }
        match parse_move(&input_line) {
            Ok(mv) => {
                writeln!(output, "OK: {:?}", mv)?;
                if let (Some(piece), Some(spot)) = (position.get_chosen_piece(), mv.spot) {
                    position.place_piece(spot, piece);
                }
                position.choose_piece(mv.piece);
                position.print(&mut output)?;
            }
            Err(err) => {
                writeln!(output, "ERROR: {:?}", err)?;
            }
        }
    }
    */

    Ok(())
}

#[derive(Debug)]
struct Move {
    spot: Option<Spot>,
    piece: Option<Piece>,
}

#[derive(Debug)]
enum ParseError {
    TooShort,
    InvalidRow,
    InvalidColumn,
    RowColumnMismatch,
    InvalidPiece,
}

fn parse_move(text: &str) -> Result<Move, ParseError> {
    if text.len() < 3 {
        return Err(ParseError::TooShort);
    }
    let chars: Vec<char> = text.chars().take(3).collect();
    let spot = parse_spot(chars[0], chars[1])?;
    let piece = parse_piece(chars[2])?;
    Ok(Move { spot, piece })
}

fn parse_spot(r: char, c: char) -> Result<Option<Spot>, ParseError> {
    let row = match r {
        's' => Some(0),
        't' => Some(1),
        'u' => Some(2),
        'v' => Some(3),
        '.' => None,
        _ => return Err(ParseError::InvalidRow),
    };
    let col = match c {
        'w' => Some(0),
        'x' => Some(1),
        'y' => Some(2),
        'z' => Some(3),
        '.' => None,
        _ => return Err(ParseError::InvalidColumn),
    };
    match (row, col) {
        (Some(row), Some(col)) => Ok(Some(Spot::from_row_col(row, col))),
        (None, None) => Ok(None),
        _ => Err(ParseError::RowColumnMismatch),
    }
}

fn parse_piece(p: char) -> Result<Option<Piece>, ParseError> {
    match p {
        '0' => Ok(Some(Piece(0x0))),
        '1' => Ok(Some(Piece(0x1))),
        '2' => Ok(Some(Piece(0x2))),
        '3' => Ok(Some(Piece(0x3))),
        '4' => Ok(Some(Piece(0x4))),
        '5' => Ok(Some(Piece(0x5))),
        '6' => Ok(Some(Piece(0x6))),
        '7' => Ok(Some(Piece(0x7))),
        '8' => Ok(Some(Piece(0x8))),
        '9' => Ok(Some(Piece(0x9))),
        'A' => Ok(Some(Piece(0xA))),
        'B' => Ok(Some(Piece(0xB))),
        'C' => Ok(Some(Piece(0xC))),
        'D' => Ok(Some(Piece(0xD))),
        'E' => Ok(Some(Piece(0xE))),
        'F' => Ok(Some(Piece(0xF))),
        '.' => Ok(None),
        _ => Err(ParseError::InvalidPiece),
    }
}

trait Player {
    fn start(&mut self) -> Move;
    fn op_move(&mut self, mv: &Move) -> Move;
}

struct Rando {
    history: History,
    position: Position,
}

impl Rando {
    fn new() -> Self {
        Self {
            history: History::new(),
            position: Position::new(),
        }
    }
}

impl Player for Rando {
    fn start(&mut self) -> Move {
        todo!()
    }

    fn op_move(&mut self, mv: &Move) -> Move {
        todo!()
    }
}

struct Position {
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
            Some(Piece((self.board_pieces >> (4 * spot.0)) as u8 & 0xF))
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
    /// . | w x y z
    /// --|--------
    /// s | . . . .
    /// t | . . . .
    /// u | . . . .
    /// v | . . . .
    ///
    /// The top-left corner shows the selected piece that the current player must place in a spot.
    /// Each piece is shown as the hexadecimal digit of its bit pattern.
    /// Each spot on the board is identified by a row and a column label.
    /// Row labels are s-v, and column labels are w-x.
    ///
    /// Empty spots are shown as dots.
    ///
    /// Note that in turn 0 (before the first move), there is no selected piece.
    /// Similarly, there is no selected piece in a draw (when the board is full).
    /// These states are also indicated by a dot in the top-left corner.
    ///
    /// A quarto is shown as a * in the top-left corner.
    ///
    fn print(&self, writer: &mut dyn io::Write) -> Result<(), io::Error> {
        let row_headers = ['s', 't', 'u', 'v'];
        let top_left = if self.is_quarto() {
            '*'
        } else {
            option_piece_to_char(&self.get_chosen_piece())
        };
        writeln!(writer, "{} | w x y z", top_left,)?;
        writeln!(writer, "--|--------")?;
        for row in 0..4 {
            write!(writer, "{} |", row_headers[row])?;
            for col in 0..4 {
                let spot = Spot::from_row_col(row as u8, col as u8);
                write!(writer, " {}", option_piece_to_char(&self.get_piece(spot)))?;
            }
            writeln!(writer)?;
        }
        Ok(())
    }
}

struct History {
    pieces_permut: [u8; 16],
    spots_permut: [u8; 16],
}

impl History {
    fn new() -> Self {
        Self {
            pieces_permut: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            spots_permut: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        }
    }

    fn try_move(&mut self, turn: u8, mv: Move) -> Result<Valid, Invalid> {
        todo!()
    }

    fn get_piece(&self, turn: u8) -> Option<Piece> {
        if turn >= 1 && turn <= 16 {
            Some(Piece(self.pieces_permut[turn as usize - 1]))
        } else {
            None
        }
    }

    fn swap_pieces(&mut self, index_0: u8, index_1: u8) {
        let piece_0 = self.pieces_permut[index_0 as usize];
        let piece_1 = self.pieces_permut[index_1 as usize];
        self.pieces_permut[index_0 as usize] = piece_1;
        self.pieces_permut[index_1 as usize] = piece_0;
    }

    fn get_spot(&self, turn: u8) -> Option<Spot> {
        if turn >= 2 && turn <= 17 {
            Some(Spot(self.spots_permut[turn as usize - 2]))
        } else {
            None
        }
    }

    fn swap_spots(&mut self, index_0: u8, index_1: u8) {
        let spot_0 = self.spots_permut[index_0 as usize];
        let spot_1 = self.spots_permut[index_1 as usize];
        self.spots_permut[index_0 as usize] = spot_1;
        self.spots_permut[index_1 as usize] = spot_0;
    }

    fn get_position(&self, turn: u8) -> Position {
        let mut pos = Position::new();

        for i in 0..=turn {
            if let (Some(piece), Some(spot)) = (pos.get_chosen_piece(), self.get_spot(i)) {
                pos.place_piece(spot, piece);
            }
            pos.choose_piece(self.get_piece(i));
        }

        pos
    }
}

fn random_playout(
    history: &mut History,
    turn: u8,
    mut piece_random_source: u64,
    mut spot_random_source: u64,
) -> Option<u8> {
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
            let piece_index = (piece_random_source % free_piece_count) as u8;
            piece_random_source /= free_piece_count;
            history.swap_pieces(i, i + piece_index);
        }
        if i >= 1 && i <= 16 {
            // pick and commit spot
            let free_spot_count = 17 - i as u64;
            let spot_index = (spot_random_source % free_spot_count) as u8;
            spot_random_source /= free_spot_count;
            history.swap_spots(i - 1, i - 1 + spot_index);
        }
    }

    None
}

#[derive(Debug, Clone, Copy)]
struct Piece(u8);

fn piece_to_char(piece: &Piece) -> char {
    match piece.0 {
        0x0 => '0',
        0x1 => '1',
        0x2 => '2',
        0x3 => '3',
        0x4 => '4',
        0x5 => '5',
        0x6 => '6',
        0x7 => '7',
        0x8 => '8',
        0x9 => '9',
        0xA => 'A',
        0xB => 'B',
        0xC => 'C',
        0xD => 'D',
        0xE => 'E',
        0xF => 'F',
        _ => unreachable!(),
    }
}

fn option_piece_to_char(option_piece: &Option<Piece>) -> char {
    match option_piece {
        Some(piece) => piece_to_char(piece),
        None => '.',
    }
}

#[derive(Debug, Clone, Copy)]
struct Spot(u8);

impl Spot {
    fn from_row_col(row: u8, col: u8) -> Spot {
        Spot(col | (row << 2))
    }
}

enum Valid {}

enum Invalid {}

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
