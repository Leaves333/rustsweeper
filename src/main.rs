use console::*;
use k_board::{keyboard::Keyboard, keys::Keys};
use rand::seq::SliceRandom;
use std::{char, cmp::min, usize};

const BOARD_SIZE_X: u16 = 15;
const BOARD_SIZE_Y: u16 = 15;
const NUM_MINES: u16 = 50;

#[derive(Clone, PartialEq)]
enum Status {
    Cleared,
    Flagged,
    Unknown,
}

#[derive(Clone)]
struct Cell {
    mine: bool,
    status: Status,
}

fn main() {
    // randomly decide which cells should be mines
    let total_cells = BOARD_SIZE_X * BOARD_SIZE_Y;
    let mut all_locations: Vec<u16> = (0..total_cells).into_iter().collect();
    all_locations.shuffle(&mut rand::thread_rng());
    let mine_locations = &all_locations[0..NUM_MINES as usize];

    // generate a 2d vector of cells
    let mut board: Vec<Vec<Cell>> = vec![
        vec![
            Cell {
                mine: false,
                status: Status::Unknown
            };
            BOARD_SIZE_Y as usize
        ];
        BOARD_SIZE_X as usize
    ];
    for x in mine_locations {
        board[(x / BOARD_SIZE_X) as usize][(x % BOARD_SIZE_Y) as usize].mine = true;
    }

    // calculate adjacent mines for each cell
    let mut board_char: Vec<Vec<char>> =
        vec![vec!['.'; BOARD_SIZE_X as usize]; BOARD_SIZE_Y as usize];
    for i in 0..BOARD_SIZE_Y {
        for j in 0..BOARD_SIZE_X {
            let mut adjacent_locations: Vec<i32> = Vec::new();
            for dx in -1..=1 as i32 {
                for dy in -1..=1 as i32 {
                    adjacent_locations
                        .push(((i as i32 + dy) * BOARD_SIZE_Y as i32) + j as i32 + dx);
                }
            }

            let adjacent_mines = adjacent_locations
                .iter()
                .copied()
                .filter(|x| *x >= 0 && *x < (BOARD_SIZE_X * BOARD_SIZE_Y) as i32)
                .map(|x| x as u16)
                .filter(|x| board[(x / BOARD_SIZE_Y) as usize][(x % BOARD_SIZE_Y) as usize].mine)
                .count();

            if adjacent_mines != 0 {
                board_char[i as usize][j as usize] = format!("{}", adjacent_mines)
                    .chars()
                    .next()
                    .expect("expected char");
            }
        }
    }

    // initial display of game state
    let mut coords: Vec<u16> = vec![0; 2];
    display(&board, &board_char, coords[0], coords[1]);

    // main game loop: repeatedly grab keyboard input and update display
    for key in Keyboard::new() {
        match key {
            // directional controls
            Keys::Up | Keys::Char('k') => coords[1] = if coords[1] > 0 { coords[1] - 1 } else { 0 },
            Keys::Down | Keys::Char('j') => coords[1] = min(BOARD_SIZE_Y - 1, coords[1] + 1),
            Keys::Left | Keys::Char('h') => {
                coords[0] = if coords[0] > 0 { coords[0] - 1 } else { 0 }
            }
            Keys::Right | Keys::Char('l') => coords[0] = min(BOARD_SIZE_X - 1, coords[0] + 1),

            // quit game
            Keys::Escape | Keys::Char('q') => break,

            // clear selected cell
            Keys::Enter | Keys::Char('d') => {
                match clear(&mut board, &board_char, coords[0], coords[1]) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }

            // flag selected cell
            Keys::Char('f') => flag(&mut board, coords[0], coords[1]),

            // match remaining keys, do nothing
            _ => {}
        }
        display(&board, &board_char, coords[0], coords[1]);
        println!("{:?}", key);
    }
}

fn flag(board: &mut Vec<Vec<Cell>>, x: u16, y: u16) {
    let cell = &mut board[y as usize][x as usize];
    match cell.status {
        Status::Flagged => cell.status = Status::Unknown,
        Status::Unknown => cell.status = Status::Flagged,
        Status::Cleared => {}
    };
}

fn clear(
    board: &mut Vec<Vec<Cell>>,
    board_char: &Vec<Vec<char>>,
    x: u16,
    y: u16,
) -> Result<String, String> {
    // return error if you hit a mine
    if board[y as usize][x as usize].mine {
        return Err("oops you hit a mine".to_string());
    }

    // dfs to recursively clear cells
    let mut stack: Vec<(usize, usize)> = Vec::new();
    stack.push((x.into(), y.into()));

    while stack.len() > 0 {
        let top = stack.pop().unwrap();
        if board[top.1][top.0].status == Status::Cleared {
            continue;
        }
        board[top.1][top.0].status = Status::Cleared;

        if board_char[top.1][top.0] == '.' {
            const CHANGES: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
            for (dx, dy) in CHANGES {
                let new_x = top.0 as i32 + dx;
                let new_y = top.1 as i32 + dy;
                if new_x > 0
                    && new_x < BOARD_SIZE_X as i32
                    && new_y > 0
                    && new_y < BOARD_SIZE_Y as i32
                {
                    stack.push((new_x as usize, new_y as usize));
                }
            }
        }
    }

    Ok("successfully cleared cells".to_string())
}

fn display(board: &Vec<Vec<Cell>>, board_char: &Vec<Vec<char>>, x: u16, y: u16) {
    let term = Term::stdout();
    let _ = term.clear_screen();
    let _ = term.write_line("messing around with k_board:");
    let _ = term.write_line("");

    for i in 0..BOARD_SIZE_Y {
        let mut line_to_print: String = "".to_string();
        for j in 0..BOARD_SIZE_X {
            let cell = &board[i as usize][j as usize];

            let mut target_style = match cell.status {
                Status::Unknown => Style::new().bold().white(),
                Status::Flagged => Style::new().bold().red(),
                Status::Cleared => Style::new().white(),
            };
            if i == y && j == x {
                target_style = target_style.reverse();
            }

            let mut target_char = match cell.status {
                Status::Unknown => '#',
                Status::Flagged => 'F',
                Status::Cleared => board_char[i as usize][j as usize],
            };
            if cell.mine {
                target_char = 'x';
            }

            let formatted_char = format!("{}", target_style.apply_to(&target_char));
            line_to_print += &formatted_char;
        }
        let _ = term.write_line(&line_to_print);
    }
}
