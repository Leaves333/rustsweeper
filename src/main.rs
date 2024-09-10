use console::*;
use k_board::{keyboard::Keyboard, keys::Keys};
use rand::seq::SliceRandom;
use std::{char, cmp::min, env, usize};

const BOARD_SIZE_X: u16 = 21;
const BOARD_SIZE_Y: u16 = 11;
const NUM_MINES: u16 = 20;

const INSTRUCTION_TEXT: &str = "arrow keys / hjkl: move cursor      d / enter: dig     f: flag";

#[derive(Clone, PartialEq)]
enum Status {
    Cleared,
    Flagged,
    Unknown,
}

#[derive(PartialEq)]
enum GameProgress {
    Lose,
    Win,
    InProgress,
}

#[derive(PartialEq)]
enum ClearResult {
    Ok,
    Mine,
    Win,
}

#[derive(Clone)]
struct Cell {
    mine: bool,
    status: Status,
}

fn main() {
    // randomly decide which cells should be mines
    let total_cells = BOARD_SIZE_X * BOARD_SIZE_Y;
    let mut randomized_locations: Vec<u16> = (0..total_cells).into_iter().collect();
    randomized_locations.shuffle(&mut rand::thread_rng());
    let mine_locations = &randomized_locations[0..NUM_MINES as usize];

    // generate a 2d vector of cells
    let mut board: Vec<Vec<Cell>> = vec![
        vec![
            Cell {
                mine: false,
                status: Status::Unknown
            };
            BOARD_SIZE_X as usize
        ];
        BOARD_SIZE_Y as usize
    ];
    for x in mine_locations {
        board[(x / BOARD_SIZE_X) as usize][(x % BOARD_SIZE_X) as usize].mine = true;
    }

    // calculate adjacent mines for each cell
    let mut board_char: Vec<Vec<char>> =
        vec![vec!['.'; BOARD_SIZE_X as usize]; BOARD_SIZE_Y as usize];
    board_char = calculate_adjacencies(&board, board_char);

    // hide the cursor at the start of game
    let _ = Term::stdout().hide_cursor();

    // initial display of game state
    let mut coords: Vec<u16> = vec![0; 2];
    display(
        &board,
        &board_char,
        coords[0],
        coords[1],
        GameProgress::InProgress,
    );

    // main game loop: repeatedly grab keyboard input and update display
    let mut first_clear = true;
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
                // prevent losing on the first click
                if first_clear && board[coords[1] as usize][coords[0] as usize].mine {
                    // remove the mine from the cursor
                    board[coords[1] as usize][coords[0] as usize].mine = false;

                    // move the mine to the next random spot that isn't a mine
                    for i in NUM_MINES..(BOARD_SIZE_X * BOARD_SIZE_Y) {
                        let loc = randomized_locations[i as usize];
                        let cell = &mut board[(loc / BOARD_SIZE_X) as usize]
                            [(loc % BOARD_SIZE_X) as usize];
                        if !cell.mine {
                            cell.mine = true;
                            break;
                        }
                    }

                    // recalculate mine adjacencies
                    board_char = calculate_adjacencies(&board, board_char);
                }

                if first_clear {
                    first_clear = false;
                }

                match clear(&mut board, &board_char, coords[0], coords[1]) {
                    ClearResult::Ok => {}
                    ClearResult::Mine => {
                        display(
                            &board,
                            &board_char,
                            coords[0],
                            coords[1],
                            GameProgress::Lose,
                        );
                        break;
                    }
                    ClearResult::Win => {
                        display(&board, &board_char, coords[0], coords[1], GameProgress::Win);
                        break;
                    }
                }
            }

            // flag selected cell
            Keys::Char('f') => flag(&mut board, coords[0], coords[1]),

            // match remaining keys, do nothing
            _ => {}
        }
        display(
            &board,
            &board_char,
            coords[0],
            coords[1],
            GameProgress::InProgress,
        );
    }

    // show cursor at the end of game
    let _ = Term::stdout().show_cursor();
}

fn calculate_adjacencies(board: &Vec<Vec<Cell>>, mut board_char: Vec<Vec<char>>) -> Vec<Vec<char>> {
    for i in 0..BOARD_SIZE_Y {
        for j in 0..BOARD_SIZE_X {
            // grab cells adjacent to current one
            let mut adjacent_locations: Vec<(i32, i32)> = Vec::new();
            for dx in -1..=1 as i32 {
                for dy in -1..=1 as i32 {
                    adjacent_locations.push((j as i32 + dx, i as i32 + dy));
                }
            }

            // filter for number of mines
            let adjacent_mines = adjacent_locations
                .iter()
                .copied()
                .filter(|x| {
                    x.0 >= 0 && x.0 < BOARD_SIZE_X as i32 && x.1 >= 0 && x.1 < BOARD_SIZE_Y as i32
                })
                .filter(|x| board[x.1 as usize][x.0 as usize].mine)
                .count();

            // convert found value to char
            if adjacent_mines != 0 {
                board_char[i as usize][j as usize] = format!("{}", adjacent_mines)
                    .chars()
                    .next()
                    .expect("expected char");
            } else {
                board_char[i as usize][j as usize] = '.';
            }
        }
    }
    board_char
}

fn flag(board: &mut Vec<Vec<Cell>>, x: u16, y: u16) {
    let cell = &mut board[y as usize][x as usize];
    match cell.status {
        Status::Flagged => cell.status = Status::Unknown,
        Status::Unknown => cell.status = Status::Flagged,
        Status::Cleared => {}
    };
}

fn clear(board: &mut Vec<Vec<Cell>>, board_char: &Vec<Vec<char>>, x: u16, y: u16) -> ClearResult {
    // check that cell is still unknown
    if board[y as usize][x as usize].status != Status::Unknown {
        return ClearResult::Ok;
    }

    // return error if you hit a mine
    if board[y as usize][x as usize].mine {
        return ClearResult::Mine;
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
                if new_x >= 0
                    && new_x < BOARD_SIZE_X as i32
                    && new_y >= 0
                    && new_y < BOARD_SIZE_Y as i32
                {
                    stack.push((new_x as usize, new_y as usize));
                }
            }
        }
    }

    // check if we've won
    for row in board {
        for cell in row {
            if !cell.mine && cell.status != Status::Cleared {
                return ClearResult::Ok;
            }
        }
    }

    ClearResult::Win
}

fn display(
    board: &Vec<Vec<Cell>>,
    board_char: &Vec<Vec<char>>,
    x: u16,
    y: u16,
    game_progress: GameProgress,
) {
    // initalize terminal and check for ok terminal size
    let term = Term::stdout();
    let _ = term.clear_screen();
    match check_terminal_size(&term) {
        Ok(_) => {}
        Err(_) => return,
    }

    let (term_y, term_x) = term.size();

    // top padding and title
    for _ in 0..(((term_y - BOARD_SIZE_Y) / 2) - 2) {
        let _ = term.write_line("");
    }
    let title_string = &pad_str("minesweeper", term_x as usize, Alignment::Center, None);
    let _ = term.write_line(&title_string);
    let _ = term.write_line("");

    // display the board
    for i in 0..BOARD_SIZE_Y {
        let mut line_to_print: String = "".to_string();
        for j in 0..BOARD_SIZE_X {
            let cell = &board[i as usize][j as usize];

            let mut target_style = match cell.status {
                Status::Unknown => Style::new().bold().white(),
                Status::Flagged => Style::new().bold().red(),
                Status::Cleared => match board_char[i as usize][j as usize] {
                    '.' => Style::new().white(),
                    '1' => Style::new().blue(),
                    '2' => Style::new().green(),
                    '3' => Style::new().red(),
                    '4' => Style::new().magenta(),
                    _ => Style::new().yellow(),
                },
            };
            if i == y && j == x {
                target_style = target_style.reverse();
            }

            let mut target_char = match cell.status {
                Status::Unknown => '#',
                Status::Flagged => 'F',
                Status::Cleared => board_char[i as usize][j as usize],
            };

            if game_progress == GameProgress::Lose && cell.mine {
                target_style = target_style.bold().red();
                target_char = 'x';
            }

            let formatted_char = format!("{}", target_style.apply_to(&target_char));
            line_to_print += &formatted_char;
        }

        let padded_str = pad_str(&line_to_print, term_x as usize, Alignment::Center, None);
        let _ = term.write_line(&padded_str);
    }

    // bottom padding text
    for _ in 0..(((term_y - BOARD_SIZE_Y) / 2) - 2) {
        let _ = term.write_line("");
    }

    match game_progress {
        GameProgress::Lose => {
            let _ = term.write_line(&pad_str(
                "oops you hit the mine",
                term_x as usize,
                Alignment::Center,
                None,
            ));
        }
        GameProgress::Win => {
            let _ = term.write_line(&pad_str(
                "hooray you're a winner!!!",
                term_x as usize,
                Alignment::Center,
                None,
            ));
        }
        GameProgress::InProgress => {
            let _ = term.write_line(&pad_str(
                INSTRUCTION_TEXT,
                term_x as usize,
                Alignment::Center,
                None,
            ));
        }
    }
}

fn check_terminal_size(term: &Term) -> Result<String, String> {
    let (term_y, term_x) = term.size();

    const VERTICAL_PADDING: u16 = 7;
    const HORIZONTAL_PADDING: u16 = 2;
    let required_x = std::cmp::max(BOARD_SIZE_X, measure_text_width(INSTRUCTION_TEXT) as u16)
        + HORIZONTAL_PADDING;
    let required_y = BOARD_SIZE_Y + VERTICAL_PADDING;

    if term_y < required_y || term_x < required_x {
        // terminal is too small, print btop-esque info

        // blank padding on top
        let vertical_blank_lines = (term_y - 5) / 2;
        for _ in 0..vertical_blank_lines {
            let _ = term.write_line("");
        }

        // current terminal size info
        let _ = term.write_line(&pad_str(
            &format!("{}", style("terminal size too small:").bold()),
            term_x as usize,
            Alignment::Center,
            None,
        ));

        let width_text = format!("{}", {
            let term_x_string = format!("{}", term_x);
            if term_x < required_x {
                style(term_x_string).red().bold()
            } else {
                style(term_x_string).green()
            }
        });
        let height_text = format!("{}", {
            let term_y_string = format!("{}", term_y);
            if term_y < required_y {
                style(term_y_string).red().bold()
            } else {
                style(term_y_string).green()
            }
        });
        let _ = term.write_line(&pad_str(
            &("width = ".to_string() + &width_text + " height = " + &height_text),
            term_x as usize,
            Alignment::Center,
            None,
        ));

        // blank line for padding
        let _ = term.write_line("");

        // expected terminal size info
        let _ = term.write_line(&pad_str(
            &format!("{}", style("required terminal size:").bold()),
            term_x as usize,
            Alignment::Center,
            None,
        ));
        let _ = term.write_line(&pad_str(
            &("width = ".to_string()
                + &format!("{}", required_x)
                + " height = "
                + &format!("{}", required_y)),
            term_x as usize,
            Alignment::Center,
            None,
        ));

        // return error back to the main function
        return Err("terminal too small".to_string());
    }

    Ok("ok terminal size".to_string())
}
