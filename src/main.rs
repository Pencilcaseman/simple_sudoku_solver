use colored::Colorize;

const ROW_SEP: &str = "+-------+-------+-------+";

const BOARD_SEP: usize = 3;
const BOARD_LEN: usize = BOARD_SEP * BOARD_SEP;
const BOARD_SIZE: usize = BOARD_LEN * BOARD_LEN;

#[derive(Debug, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Default, Clone, Copy)]
enum Cell {
    #[default]
    Empty,
    Fixed(u8),
    Collapsed(u8),
    Superposition([bool; BOARD_LEN]),
}

impl Cell {
    fn count_superstates(&self) -> Option<usize> {
        match self {
            Cell::Superposition(s) => Some(s.iter().filter(|&&x| x).count()),
            _ => None,
        }
    }

    fn collapse(&self) -> Option<u8> {
        match self {
            Cell::Superposition(s) => {
                let mut count = 0;
                let mut value = 0;

                for (idx, val) in s.iter().enumerate() {
                    if *val {
                        count += 1;
                        value = idx + 1;
                    }
                }

                if count == 1 { Some(value as u8) } else { None }
            }
            _ => None,
        }
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cell::Empty => write!(f, " "),
            Cell::Fixed(n) => write!(f, "{}", n.to_string().green()),
            Cell::Collapsed(n) => write!(f, "{}", n.to_string().yellow()),
            Cell::Superposition(_) => write!(f, "{}", "+".red()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Sudoku {
    grid: [Cell; BOARD_SIZE],
}

impl std::default::Default for Sudoku {
    fn default() -> Self {
        Self {
            grid: [const { Cell::Empty }; BOARD_SIZE],
        }
    }
}

impl Sudoku {
    fn from_zero_grid(grid: &[[u8; BOARD_LEN]; BOARD_LEN]) -> Self {
        let mut sudoku = Sudoku::default();

        let mut idx = 0;

        for row in grid {
            for cell in row {
                sudoku.grid[idx] = if *cell == 0 {
                    Cell::Empty
                } else {
                    Cell::Fixed(*cell)
                };

                idx += 1;
            }
        }

        sudoku
    }

    fn coord_to_idx((row, col): (usize, usize)) -> usize {
        row * BOARD_LEN + col
    }

    fn idx_to_coord(idx: usize) -> (usize, usize) {
        let row = idx / BOARD_LEN;
        let col = idx % BOARD_LEN;
        (row, col)
    }

    fn row_idx(idx: usize) -> usize {
        let row = idx / BOARD_LEN;
        row * BOARD_LEN
    }

    fn col_idx(idx: usize) -> usize {
        idx % BOARD_LEN
    }

    fn subsection_idx(idx: usize) -> usize {
        let (mut row, mut col) = Sudoku::idx_to_coord(idx);
        row -= row % BOARD_SEP;
        col -= col % BOARD_SEP;
        row * BOARD_LEN + col
    }

    fn is_solved(&self) -> bool {
        self.grid
            .iter()
            .all(|cell| matches!(cell, Cell::Fixed(_) | Cell::Collapsed(_)))
    }

    fn initialize_superpositions(&mut self) {
        self.grid.iter_mut().for_each(|cell| match cell {
            Cell::Fixed(_) => (),
            Cell::Empty => *cell = Cell::Superposition([true; 9]),
            _ => unreachable!(),
        });
    }

    fn propagate(&mut self, idx: usize) {
        let (Cell::Fixed(n) | Cell::Collapsed(n)) = self.grid[idx] else {
            return;
        };

        // Nothing horizontally can be the same
        let row_idx = Sudoku::row_idx(idx);
        for col in row_idx..row_idx + BOARD_LEN {
            if let &mut Cell::Superposition(ref mut s) = &mut self.grid[col] {
                s[n as usize - 1] = false;
            }
        }

        // Nothing vertically can be the same
        let col_idx = Sudoku::col_idx(idx);
        for row in (col_idx..BOARD_SIZE).step_by(BOARD_LEN) {
            if let &mut Cell::Superposition(ref mut s) = &mut self.grid[row] {
                s[n as usize - 1] = false;
            }
        }

        // Nothing in the same subsection can be the same
        let subsection_idx = Sudoku::subsection_idx(idx);
        let (row, col) = Sudoku::idx_to_coord(subsection_idx);

        for row in row..row + BOARD_SEP {
            for col in col..col + BOARD_SEP {
                if let &mut Cell::Superposition(ref mut s) =
                    &mut self.grid[Sudoku::coord_to_idx((row, col))]
                {
                    s[n as usize - 1] = false;
                }
            }
        }
    }

    fn solve_pure_negative(&mut self, idx: usize) {
        // If no other cell in the same row/col/subsection can have a certain
        // value, this cell must have that value

        let Cell::Superposition(superposition) = self.grid[idx] else {
            return;
        };

        for (val_idx, _) in superposition.iter().enumerate().filter(|(_, val)| **val) {
            let mut num_alternatives = 0;

            // Nothing horizontally can be the same
            let row_idx = Sudoku::row_idx(idx);
            for col in row_idx..row_idx + BOARD_LEN {
                if let &mut Cell::Superposition(ref mut s) = &mut self.grid[col] {
                    if col != idx && s[val_idx] {
                        num_alternatives += 1;
                    }
                }
            }

            if num_alternatives == 0 {
                // Include the current cell
                self.grid[idx] = Cell::Collapsed(val_idx as u8 + 1);
                break;
            }

            // Nothing vertically can be the same
            let col_idx = Sudoku::col_idx(idx);
            num_alternatives = 0;
            for row in (col_idx..BOARD_SIZE).step_by(BOARD_LEN) {
                if let &mut Cell::Superposition(ref mut s) = &mut self.grid[row] {
                    if row != idx && s[val_idx] {
                        num_alternatives += 1;
                    }
                }
            }

            if num_alternatives == 0 {
                self.grid[idx] = Cell::Collapsed(val_idx as u8 + 1);
                break;
            }

            // Nothing in the same subsection can be the same
            let subsection_idx = Sudoku::subsection_idx(idx);
            let (row, col) = Sudoku::idx_to_coord(subsection_idx);
            num_alternatives = 0;

            for row in row..row + BOARD_SEP {
                for col in col..col + BOARD_SEP {
                    let tmp_idx = Sudoku::coord_to_idx((row, col));

                    if let &mut Cell::Superposition(ref mut s) = &mut self.grid[tmp_idx] {
                        if tmp_idx != idx && s[val_idx] {
                            num_alternatives += 1;
                        }
                    }
                }
            }

            if num_alternatives == 0 {
                self.grid[idx] = Cell::Collapsed(val_idx as u8 + 1);
                break;
            }
        }
    }

    fn solve(&mut self) {
        let mut iters_without_collapse = 0;

        while !self.is_solved() {
            for idx in 0..BOARD_SIZE {
                self.solve_pure_negative(idx);
                self.propagate(idx);
            }

            let mut collapsed = false;

            for idx in 0..BOARD_SIZE {
                if matches!(self.grid[idx], Cell::Superposition(_)) {
                    if let Some(value) = self.grid[idx].collapse() {
                        self.grid[idx] = Cell::Collapsed(value);
                        self.solve_pure_negative(idx);
                        self.propagate(idx);
                        collapsed = true;
                    }
                }

                if self.grid[idx].count_superstates().unwrap_or(1) == 0 {
                    return;
                }
            }

            if collapsed {
                iters_without_collapse = 0;
            } else {
                iters_without_collapse += 1;
            }

            if iters_without_collapse > 3 {
                break;
            }
        }

        if !self.is_solved() {
            // Backtrack
            let idx = self
                .grid
                .iter()
                .enumerate()
                .position(|(_, cell)| matches!(cell, Cell::Superposition(_)))
                .expect("No superstates found");

            let Cell::Superposition(s) = self.grid[idx] else {
                unreachable!()
            };

            for possible_val in s
                .iter()
                .enumerate()
                .filter_map(|(idx, val)| if *val { Some(idx + 1) } else { None })
            {
                let mut clone = *self;
                clone.grid[idx] = Cell::Collapsed(possible_val as u8);

                clone.solve();

                if clone.is_solved() {
                    *self = clone;
                    return;
                }
            }
        }
    }
}

impl std::fmt::Display for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, cell) in self.grid.iter().enumerate() {
            if idx % (BOARD_LEN * BOARD_SEP) == 0 {
                if idx > 0 {
                    writeln!(f, "|")?;
                }

                write!(f, "{}\n| ", ROW_SEP)?;
            } else if idx % BOARD_SEP == 0 {
                write!(f, "| ")?;

                if idx % BOARD_LEN == 0 {
                    write!(f, "\n| ")?;
                }
            }

            write!(f, "{} ", cell)?;
        }

        write!(f, "|\n{}", ROW_SEP)?;

        Ok(())
    }
}

// Easy grid
// const EXAMPLE_GRID: [[u8; 9]; 9] = [
//     [0, 0, 0, 2, 6, 0, 7, 0, 1],
//     [6, 8, 0, 0, 7, 0, 0, 9, 0],
//     [1, 9, 0, 0, 0, 4, 5, 0, 0],
//     [8, 2, 0, 1, 0, 0, 0, 4, 0],
//     [0, 0, 4, 6, 0, 2, 9, 0, 0],
//     [0, 5, 0, 0, 0, 3, 0, 2, 8],
//     [0, 0, 9, 3, 0, 0, 0, 7, 4],
//     [0, 4, 0, 0, 5, 0, 0, 3, 6],
//     [7, 0, 3, 0, 1, 8, 0, 0, 0],
// ];

// Izzy's grid
// const SAMPLE_GRID: [[u8; 9]; 9] = [
//     [4, 0, 7, 5, 0, 0, 0, 3, 0],
//     [0, 1, 0, 0, 7, 0, 0, 0, 0],
//     [0, 0, 0, 0, 0, 8, 0, 0, 0],
//     [1, 0, 0, 2, 0, 6, 4, 0, 0],
//     [0, 0, 0, 0, 0, 0, 0, 0, 7],
//     [0, 8, 0, 3, 0, 0, 0, 1, 9],
//     [0, 3, 0, 0, 5, 0, 0, 4, 8],
//     [2, 0, 0, 0, 1, 0, 0, 9, 5],
//     [0, 0, 0, 0, 0, 0, 0, 0, 6],
// ];

// Hard grid
const SAMPLE_GRID: [[u8; 9]; 9] = [
    [0, 2, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 6, 0, 0, 0, 0, 3],
    [0, 7, 4, 0, 8, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 3, 0, 0, 2],
    [0, 8, 0, 0, 4, 0, 0, 1, 0],
    [6, 0, 0, 5, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 1, 0, 7, 8, 0],
    [5, 0, 0, 0, 0, 9, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 4, 0],
];

fn main() {
    let target_time = std::time::Duration::from_millis(5000);
    let mut iters = 0;

    let start = std::time::Instant::now();

    while start.elapsed() < target_time {
        let mut sudoku = Sudoku::from_zero_grid(&SAMPLE_GRID);
        sudoku.initialize_superpositions();
        sudoku.solve();

        iters += 1;
    }

    println!("Elapsed: {:?}", start.elapsed());
    println!("Average: {:?}", start.elapsed() / iters);

    let mut sudoku = Sudoku::from_zero_grid(&SAMPLE_GRID);

    println!("{}", sudoku);

    sudoku.initialize_superpositions();
    sudoku.solve();

    println!("{}", sudoku);
}
