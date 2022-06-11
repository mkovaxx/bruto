use std::io;
use std::io::Write;

fn main() {
    let mut out = std::io::stdout();
    let board = History::new();
    for turn in 0..=17 {
        let position = board.get_position(turn).unwrap();
        position.print(&mut out).unwrap();
        writeln!(&mut out).unwrap();
    }
}

struct Position {
    board_pieces: u64,
    board_mask: u16,
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

    fn get_piece(&self, row: u32, col: u32) -> Option<Piece> {
        let row_col = col | (row << 2);
        if (self.board_mask >> row_col) & 1 != 0 {
            Some(Piece((self.board_pieces >> (4 * row_col)) as u32))
        } else {
            None
        }
    }

    fn place_piece(&mut self, row: u32, col: u32, piece: Piece) {
        let row_col = col | (row << 2);
        self.board_mask |= 1 << row_col;
        self.board_pieces |= (piece.0 as u64) << (4 * row_col);
    }

    fn get_selected_piece(&self) -> Option<Piece> {
        self.selected_piece
    }

    fn select_piece(&mut self, piece: Piece) {
        self.selected_piece = Some(piece);
    }

    /// Print the position
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
    fn print(&self, writer: &mut dyn io::Write) -> Result<(), io::Error> {
        let row_headers = ['s', 't', 'u', 'v'];
        writeln!(
            writer,
            "{} | x y z w",
            option_piece_to_char(&self.get_selected_piece())
        )?;
        writeln!(writer, "--|--------")?;
        for row in 0..4 {
            write!(writer, "{} |", row_headers[row])?;
            for col in 0..4 {
                write!(
                    writer,
                    " {}",
                    option_piece_to_char(&self.get_piece(row as u32, col as u32))
                )?;
            }
            writeln!(writer)?;
        }
        Ok(())
    }
}

struct History {
    spots_permut: u64,
    pieces_permut: u64,
}

impl History {
    fn new() -> Self {
        Self {
            spots_permut: 0xFEDCBA9876543210,
            pieces_permut: 0xFEDCBA9876543210,
        }
    }

    fn try_move(&mut self, turn: u32, mv: Move) -> Result<Valid, Invalid> {
        todo!()
    }

    fn get_position(&self, turn: u32) -> Option<Position> {
        if turn > 17 {
            return None;
        }

        let mut pos = Position::new();

        if turn > 0 {
            for i in 0..(turn - 1) {
                let row_col = (self.spots_permut >> (4 * i)) as u32 & 0xF;
                let row = row_col >> 2;
                let col = row_col & 0x3;
                let piece = Piece((self.pieces_permut >> (4 * i)) as u32 & 0xF);
                pos.place_piece(row, col, piece);
            }
            if turn < 17 {
                pos.select_piece(Piece((self.pieces_permut >> (4 * (turn - 1))) as u32 & 0xF));
            }
        }

        Some(pos)
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
