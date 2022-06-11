use std::io;
use std::io::Write;

fn main() {
    let mut out = std::io::stdout();
    let board = Board::new();
    for turn in 0..=17 {
        board.print(turn, &mut out).unwrap();
        writeln!(&mut out).unwrap();
    }
}

struct Board {
    spots: u64,
    pieces: u64,
}

impl Board {
    fn new() -> Self {
        Self {
            spots: 0xFEDCBA9876543210,
            pieces: 0xFEDCBA9876543210,
        }
    }

    fn try_move(&mut self, turn: u32, mv: Move) -> Result<Valid, Invalid> {
        todo!()
    }

    /// Print the board given the turn
    ///
    /// The format looks like this
    /// . | x y z w
    /// --|--------
    /// s | . . . .
    /// t | . . . .
    /// u | . . . .
    /// v | . . . .
    ///
    /// The top-left corner shows the selected piece that the current player must place in a spot.
    /// Each piece is shown as the hexadecimal digit of its bit pattern.
    /// Each spot on the board is identified by a row and a column label.
    /// Row labels are s-v, and column labels are x-w.
    ///
    /// Empty spots are shown as dots.
    ///
    /// Note that in turn 0 (before the first move), there is no selected piece.
    /// Similarly, there is no selected piece in a draw (when the board is full).
    /// These states are also indicated by a dot in the top-left corner.
    ///
    /// A quarto is shown as a Q in the top-left corner.
    ///
    fn print(&self, turn: u32, writer: &mut dyn io::Write) -> Result<(), io::Error> {
        let mut grid = [[None; 4]; 4];
        for i in 1..turn {
            let rc = (self.spots >> (4 * (i - 1))) as usize;
            let c = rc & 0x3;
            let r = (rc >> 2) & 0x3;
            grid[r][c] = Some(Piece((self.pieces >> (4 * (i - 1))) as u32 & 0xF));
        }

        let row_headers = ['s', 't', 'u', 'v'];
        writeln!(
            writer,
            "{} | x y z w",
            option_piece_to_char(&self.selected_piece(turn))
        )?;
        writeln!(writer, "--|--------")?;
        for r in 0..4 {
            write!(writer, "{} |", row_headers[r])?;
            for c in 0..4 {
                write!(writer, " {}", option_piece_to_char(&grid[r][c]))?;
            }
            writeln!(writer)?;
        }
        Ok(())
    }

    fn selected_piece(&self, turn: u32) -> Option<Piece> {
        if turn > 0 && turn <= 16 {
            Some(Piece((self.pieces >> (4 * (turn - 1)) & 0xF) as u32))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Piece(u32);

fn piece_to_char(piece: &Piece) -> char {
    match piece.0 & 0xF {
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
struct Spot(u32);

enum Valid {}

enum Invalid {}

struct Move {
    spot: Option<SpotIndex>,
    piece: PieceIndex,
}

#[derive(Debug, Clone, Copy)]
struct PieceIndex(u32);

#[derive(Debug, Clone, Copy)]
struct SpotIndex(u32);
