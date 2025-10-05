use colored::*;
use rand::Rng;
use std::fmt::{Debug, Display, Formatter};

// A board on which the next thing to do is to play.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct PlayableBoard(Board);

impl PlayableBoard {
    /// Returns an initial board, with a single random tile.
    pub fn init() -> PlayableBoard {
        let mut board = Board::EMPTY;
        board.add_random();
        PlayableBoard(board)
    }

    pub fn apply(&self, action: Action) -> Option<RandableBoard> {
        match self.0.apply(action) {
            Some(board) => Some(RandableBoard(board)),
            None => None,
        }
    }

    pub fn has_at_least_tile(&self, i: u8) -> bool {
        self.0.cells.iter().flatten().any(|tile| *tile >= i)
    }
}

impl Display for PlayableBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A board on which the next thing to do is to radomly place a tile.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct RandableBoard(Board);

impl RandableBoard {
    pub fn with_random_tile(&self) -> PlayableBoard {
        let mut board = self.0;
        board.add_random();
        PlayableBoard(board)
    }

    /// Given a board for which an action has already been applied, returns the list of possible successors as a result of placing a random tile (2 or 4) on an empty cell.
    ///
    /// ```rust
    /// let init = Board::init();
    /// let current = init.apply(Action::Left).expect("oups");
    /// for (proba, succ_board) in current.random_successors() {
    ///   println!("May get the following board with probability {proba}:\n{succ_board}");
    /// }
    /// ```
    pub fn successors(&self) -> impl Iterator<Item = (f32, PlayableBoard)> + '_ {
        self.0
            .random_successors()
            .map(|(proba, board)| (proba, PlayableBoard(board)))
    }

    pub fn evaluate(&self) -> f32 {
        crate::eval::eval(&self.0)
    }
}

impl Display for RandableBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Size of board
pub const N: usize = 4;

// A board is an NxN matrix where each entry represents a tile.
//
// A tile is encoded by an 8-bits unsigned int where:
//
//  - 0 represent the empty tile
//  - n > 0 represents the tile `2^n`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board {
    pub cells: [[u8; N]; N],
}

impl Board {
    /// The completly empty board. This is not the initial board which can be built with the `PlayableBoard::init` method.
    const EMPTY: Board = Board { cells: [[0; N]; N] };

    /// Returns the board resuting from the action, or None if the action is not applicable.
    pub fn apply(&self, action: Action) -> Option<Board> {
        let mut next = self.clone();
        // we only know how to push left, so this method:
        // - applies some symmetries to build a board where we can push left
        // - push left
        // - unapply the symmetries to get in the normal configuration
        match action {
            Action::Left => {
                next.push_left();
            }
            Action::Up => {
                next.transpose();
                next.push_left();
                next.transpose();
            }
            Action::Down => {
                next.transpose();
                next.swap_lr();
                next.push_left();
                next.swap_lr();
                next.transpose();
            }
            Action::Right => {
                next.swap_lr();
                next.push_left();
                next.swap_lr();
            }
        }
        if *self != next {
            // the board has changed meaning the action is applicatble, return the resulting board
            Some(next)
        } else {
            // Nothing changed, the action is not applicable
            None
        }
    }

    /// Places a random tile (2 or 4) on an emtpy cell of the board
    pub fn add_random(&mut self) {
        // compute the nuber of empty cells
        let n = self.num_empty();

        // decide which empty of the cell to update in [0,n)
        let picked = rand::rng().random_range(0..n);

        // get a mutable reference of cell
        let picked = self
            .cells
            .iter_mut()
            .map(|row| row.iter_mut())
            .flatten()
            .filter(|cell| **cell == 0)
            .nth(picked)
            .unwrap();

        // decide which value to put in the cell (2^1 = 2 with probability 0.9, 2^2 = 4 with probability 0.1)
        let value = if rand::rng().random_bool(0.9) { 1 } else { 2 };

        // update the board by setting the value to the selected empty cell
        *picked = value;
    }

    /// Counts the number of empty tiles on the board
    pub fn num_empty(&self) -> usize {
        self.cells
            .iter()
            .flatten()
            .filter(|&&cell| cell == 0)
            .count()
    }

    /// Given a board for which an action has already been applied, returns the list of possible successors as a result of placing a random tile (2 or 4) on an empty cell.
    ///
    /// ```rust
    /// let init = Board::init();
    /// let current = init.apply(Action::Left).expect("oups");
    /// for (proba, succ_board) in current.random_successors() {
    ///   println!("May get the following board with probability {proba}:\n{succ_board}");
    /// }
    /// ```
    pub fn random_successors(&self) -> impl Iterator<Item = (f32, Board)> + '_ {
        let n = self.num_empty() as f32;

        let empty_cells = self.cells.iter().enumerate().flat_map(|(i, row)| {
            row.iter()
                .enumerate()
                .filter_map(move |(j, &cell)| if cell == 0 { Some((i, j)) } else { None })
        });

        empty_cells.flat_map(move |(i, j)| {
            [(1, 0.9), (2, 0.1)]
                .into_iter()
                .map(move |(new_value, proba)| {
                    let mut next = self.clone();
                    next.cells[i][j] = new_value;
                    (proba / n, next)
                })
        })
    }

    /// Switches the matrix left/right
    fn swap_lr(&mut self) {
        for row in &mut self.cells {
            let mut i = 0;
            let mut j = N - 1;
            while i < j {
                row.swap(i, j);
                i += 1;
                j -= 1;
            }
        }
    }

    /// Transposes the matrix, inverting lines and columns
    fn transpose(&mut self) {
        for i in 0..N {
            for j in 0..i {
                let tmp = self.cells[i][j];
                self.cells[i][j] = self.cells[j][i];
                self.cells[j][i] = tmp;
            }
        }
    }

    /// Build an equivalent board where the lines an columns have been transposed
    pub fn transposed(&self) -> Board {
        let mut transposed = self.clone();
        transposed.transpose();
        transposed
    }

    /// Applies the action of playing *Left*
    fn push_left(&mut self) {
        // apply the mush left method on each line
        for row in &mut self.cells {
            push_left(row);
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", format!("╔═{}╗", "═".repeat(8 * N)).bold())?;
        for row in &self.cells {
            write!(f, "{}", "║ ".bold())?;
            for &cell in row {
                if cell != 0 {
                    let value = 2u32.pow(cell as u32);
                    let formatted = format!("{:^7}", value);
                    let colored = match value {
                        2 => formatted.black().on_truecolor(238, 228, 218), // #eee4da
                        4 => formatted.black().on_truecolor(237, 224, 200), // #ede0c8
                        8 => formatted.black().on_truecolor(242, 177, 121), // #f2b179
                        16 => formatted.black().on_truecolor(245, 149, 99), // #f59563
                        32 => formatted.black().on_truecolor(246, 124, 95), // #f67c5f
                        64 => formatted.black().on_truecolor(246, 94, 59),  // #f65e3b
                        128 => formatted.black().on_truecolor(237, 207, 114), // #edcf72
                        256 => formatted.black().on_truecolor(237, 204, 97), // #edcc61
                        512 => formatted.black().on_truecolor(237, 200, 80), // #edc850
                        1024 => formatted.black().on_truecolor(237, 197, 63), // #edc53f
                        2048 => formatted.black().on_truecolor(237, 194, 46), // 2048 -> #edc22e
                        _ => formatted.bold().black().on_truecolor(237, 194, 46), // 4096+ -> #edc22e + bold
                    };
                    write!(f, "{} ", colored)?;
                } else {
                    let formatted = format!("   .   ");
                    let colored = formatted.black().on_truecolor(205, 193, 180); // #cdc1b4
                    write!(f, "{} ", colored)?;
                }
            }
            writeln!(f, "{} ", "║".bold())?;
        }
        writeln!(f, "{}", format!("╚═{}╝", "═".repeat(8 * N)).bold())?;
        Ok(())
    }
}

/// The set of possible actions to apply on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
}

/// An iterable list of all possible actions.
pub const ALL_ACTIONS: [Action; 4] = [Action::Up, Action::Down, Action::Left, Action::Right];

/// Applies the action of playing "left", on a single Row
fn push_left(row: &mut [u8; N]) {
    let mut write_index = 0; // Position to write next non-zero tile
    let mut read_index = 0; // Reading index

    // Move non-zero tiles forward and merge adjacent ones
    while read_index < N {
        if row[read_index] == 0 {
            read_index += 1;
            continue;
        }

        let value = row[read_index];
        read_index += 1;

        // Merge with the next non-zero value if it matches
        if read_index < N {
            while read_index < N && row[read_index] == 0 {
                read_index += 1; // Skip empty cell
            }
            if read_index < N && row[read_index] == value {
                row[write_index] = value + 1;
                read_index += 1; // Skip merged cell
            } else {
                row[write_index] = value;
            }
        } else {
            row[write_index] = value;
        }

        write_index += 1;
    }

    row[write_index..].fill(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_left() {
        fn check(row: [u8; N], expected: [u8; N]) {
            let mut pushed = row;
            push_left(&mut pushed);
            assert_eq!(pushed, expected);
        }
        check([0, 0, 0, 0], [0, 0, 0, 0]);
        check([0, 1, 0, 0], [1, 0, 0, 0]);
        check([0, 0, 1, 0], [1, 0, 0, 0]);
        check([0, 0, 0, 1], [1, 0, 0, 0]);
        check([0, 0, 0, 0], [0, 0, 0, 0]);
        check([1, 1, 0, 1], [2, 1, 0, 0]);
        check([0, 0, 1, 1], [2, 0, 0, 0]);
        check([0, 1, 0, 1], [2, 0, 0, 0]);
        check([1, 2, 0, 1], [1, 2, 1, 0]);
    }

    #[test]
    fn test_actions() {
        let board = Board {
            cells: [[1, 2, 1, 0], [4, 1, 0, 0], [3, 0, 0, 0], [0, 0, 0, 0]],
        };
        let target = Board {
            cells: [[0, 0, 0, 0], [1, 0, 0, 0], [4, 2, 0, 0], [3, 1, 1, 0]],
        };
        assert_eq!(board.apply(Action::Down), Some(target));
    }
}
