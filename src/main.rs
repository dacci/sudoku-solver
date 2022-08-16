use clap::Parser;
use log::debug;
use std::collections::BTreeSet;
use std::fmt;
use std::io;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

#[derive(Parser)]
#[clap(version, about)]
struct Args {
    /// Path to the file that contains values of cells.
    path: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();

    let board = solve(open(&args.path)?)?;
    println!("{}", board);

    Ok(())
}

#[derive(Debug, Clone)]
enum Cell {
    NotSolved(BTreeSet<u8>),
    Solved(u8),
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotSolved(_) => f.write_str(" "),
            Self::Solved(n) => write!(f, "{n}"),
        }
    }
}

#[derive(Debug, Clone)]
struct Board(Vec<Cell>);

impl From<Vec<Cell>> for Board {
    fn from(v: Vec<Cell>) -> Self {
        Self(v)
    }
}

impl std::ops::Deref for Board {
    type Target = [Cell];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl std::ops::DerefMut for Board {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut_slice()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r in 0..9 {
            for c in 0..9 {
                write!(f, "{}", self[r * 9 + c])?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl From<u8> for Cell {
    fn from(b: u8) -> Self {
        match b {
            0 => Self::NotSolved((1..=9).collect()),
            1..=9 => Self::Solved(b),
            _ => panic!(),
        }
    }
}

fn open(path: impl AsRef<std::path::Path>) -> io::Result<Board> {
    use io::Read;

    let board = std::fs::File::open(path)?
        .bytes()
        .filter_map(Result::ok)
        .filter_map(|b| match b {
            0x30..=0x39 => Some(b - 0x30),
            _ => None,
        })
        .map(Cell::from)
        .collect::<Vec<_>>();

    match board.len() {
        81 => Ok(board.into()),
        _ => Err(io::ErrorKind::InvalidData.into()),
    }
}

fn solve(init: Board) -> Result<Board, String> {
    let level = Level::new();

    debug!(target: "solve", "[{level}] trying elimination");
    match eliminate(init) {
        Ok((board, true)) => Ok(board),
        Ok((board, false)) => {
            debug!(target: "solve", "[{level}] trying depth first search");
            match depth_first_search(board) {
                Ok(board) => Ok(board),
                Err(msg) => {
                    debug!(target: "solve", "[{level}] depth first search failed: {msg}");
                    Err(msg)
                }
            }
        }
        Err(msg) => {
            debug!(target: "solve", "[{level}] elimination failed: {msg}");
            Err(msg)
        }
    }
}

fn eliminate(mut board: Board) -> Result<(Board, bool), String> {
    let level = Level::new();

    loop {
        let mut solved = true;
        for i in 0..81 {
            if let Cell::Solved(answer) = board[i] {
                let (row, col) = (i / 9, i % 9);
                for i in (row * 9)..((row + 1) * 9) {
                    if let Cell::NotSolved(ref mut hints) = board[i] {
                        hints.remove(&answer);
                    }
                }
                for i in (col..81).step_by(9) {
                    if let Cell::NotSolved(ref mut hints) = board[i] {
                        hints.remove(&answer);
                    }
                }

                let (row, col) = (row / 3 * 3, col / 3 * 3);
                for row in row..row + 3 {
                    for col in col..col + 3 {
                        if let Cell::NotSolved(ref mut hints) = board[row * 9 + col] {
                            hints.remove(&answer);
                        }
                    }
                }
            } else {
                solved = false;
            }
        }
        if solved {
            debug!(target: "eliminate", "[{level}] all cells solved");
            return Ok((board, true));
        }

        let mut changed = false;
        for i in 0..81 {
            if let Cell::NotSolved(ref hints) = board[i] {
                if hints.is_empty() {
                    return Err(format!("unsolvable cell at {i}"));
                }

                if hints.len() != 1 {
                    continue;
                }

                let hint = *hints.iter().next().unwrap();

                let (row, col) = (i / 9, i % 9);
                for i in (row * 9)..((row + 1) * 9) {
                    if matches!(board[i], Cell::Solved(n) if n == hint) {
                        return Err(format!("duplicate cell at {i}"));
                    }
                }
                for i in (col..81).step_by(9) {
                    if matches!(board[i], Cell::Solved(n) if n == hint) {
                        return Err(format!("duplicate cell at {i}"));
                    }
                }

                let (row, col) = (row / 3 * 3, col / 3 * 3);
                for row in row..row + 3 {
                    for col in col..col + 3 {
                        if matches!(board[row * 9 + col], Cell::Solved(n) if n == hint) {
                            return Err(format!("duplicate cell at ({col}, {row})"));
                        }
                    }
                }

                board[i] = Cell::Solved(hint);
                changed = true;
            }
        }
        if !changed {
            debug!(target: "eliminate", "[{level}] no cell could be solved");
            return Ok((board, false));
        }
    }
}

fn depth_first_search(init: Board) -> Result<Board, String> {
    let level = Level::new();

    let (index, hints) = {
        let mut index = 0;
        let mut hits: Option<&BTreeSet<u8>> = None;
        for (i, c) in (0..).zip(init.iter()) {
            if let Cell::NotSolved(h) = c {
                if hits.is_none() || h.len() < hits.unwrap().len() {
                    index = i;
                    hits = Some(h);
                }
            }
        }

        match hits {
            Some(hints) => (index, hints.clone()),
            None => {
                debug!(target: "depth_first_search", "[{level}] already solved");
                return Ok(init);
            }
        }
    };

    for hint in hints {
        let mut board = init.clone();

        debug!(target: "depth_first_search", "[{level}] assuming cell({index}) = {hint}");
        board[index] = Cell::Solved(hint);

        match solve(board) {
            Ok(board) => return Ok(board),
            Err(msg) => debug!(target: "depth_first_search", "[{level}] cloud not solve: {msg}"),
        }
    }

    Err("all assumptions contradicted".to_owned())
}

static LEVEL: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct Level(usize);

impl Level {
    fn new() -> Self {
        Self(LEVEL.fetch_add(1, Ordering::SeqCst))
    }
}

impl Drop for Level {
    fn drop(&mut self) {
        LEVEL.fetch_sub(1, Ordering::SeqCst);
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
