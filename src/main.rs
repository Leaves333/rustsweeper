use console::*;
use k_board::{keyboard::Keyboard, keys::Keys};
use rand::seq::SliceRandom;
use std::{cmp::min, usize};

const BOARD_SIZE_X: u16 = 15;
const BOARD_SIZE_Y: u16 = 15;
const NUM_MINES: u16 = 15;

#[derive(Clone)]
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

    let mut coords: Vec<u16> = vec![0; 2];
    display(&board, coords[0], coords[1]);
    for key in Keyboard::new() {
        match key {
            Keys::Up | Keys::Char('k') => coords[1] = if coords[1] > 0 { coords[1] - 1 } else { 0 },
            Keys::Down | Keys::Char('j') => coords[1] = min(BOARD_SIZE_Y - 1, coords[1] + 1),
            Keys::Left | Keys::Char('h') => {
                coords[0] = if coords[0] > 0 { coords[0] - 1 } else { 0 }
            }
            Keys::Right | Keys::Char('l') => coords[0] = min(BOARD_SIZE_X - 1, coords[0] + 1),
            Keys::Escape | Keys::Char('q') => break,
            Keys::Enter | Keys::Char('d') => {
                board[coords[1] as usize][coords[0] as usize].status = Status::Cleared
            }
            Keys::Char('f') => {
                board[coords[1] as usize][coords[0] as usize].status = Status::Flagged
            }
            _ => {}
        }
        display(&board, coords[0], coords[1]);
        println!("{:?}", key);
    }
}

fn display(board: &Vec<Vec<Cell>>, x: u16, y: u16) {
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

            let target_char = match cell.status {
                Status::Unknown => '#',
                Status::Flagged => 'F',
                Status::Cleared => {
                    let i = i as i32;
                    let j = j as i32;
                    let mut adjacent_locations: Vec<i32> = Vec::new();
                    for dx in -1..1 as i32 {
                        for dy in -1..1 as i32 {
                            adjacent_locations.push((i + dy) * BOARD_SIZE_Y as i32 + j + dx);
                        }
                    }

                    let adjacent_mines = adjacent_locations
                        .iter()
                        .copied()
                        .filter(|x| *x >= 0 && *x < (BOARD_SIZE_X * BOARD_SIZE_Y) as i32)
                        .map(|x| x as u16)
                        .filter(|x| {
                            board[(x / BOARD_SIZE_Y) as usize][(x % BOARD_SIZE_Y) as usize].mine
                        })
                        .count();

                    if adjacent_mines == 0 {
                        '.'
                    } else {
                        format!("{}", adjacent_mines)
                            .chars()
                            .next()
                            .expect("expected char")
                    }
                }
            };
            let formatted_char = format!("{}", target_style.apply_to(&target_char));
            line_to_print += &formatted_char;
        }
        let _ = term.write_line(&line_to_print);
    }
}
