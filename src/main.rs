use console::*;
use dialoguer::*;
use k_board::{keyboard::Keyboard, keys::Keys};

fn main() {
    //let term = Term::stdout();
    //let x = 490;
    //let red = Style::new().red();
    //let text = format!("x is {}", red.apply_to(x));
    //let _ = term.clear_screen();
    //let _ = term.write_line(&text);
    //
    //let sample_str: String = Input::new()
    //    .with_prompt("sample text")
    //    .interact_text()
    //    .unwrap();
    //let green = Style::new().blue();
    //let text = format!("sample_str is {}", green.apply_to(sample_str));
    //let _ = term.write_line(&text);

    let mut coords = vec![0; 2];
    display(coords[0], coords[1]);
    for key in Keyboard::new() {
        match key {
            Keys::Up | Keys::Char('k') => coords[1] -= 1,
            Keys::Down | Keys::Char('j') => coords[1] += 1,
            Keys::Left | Keys::Char('h') => coords[0] -= 1,
            Keys::Right | Keys::Char('l') => coords[0] += 1,
            Keys::Enter | Keys::Char('q') => break,
            _ => {}
        }
        display(coords[0], coords[1]);
        println!("{:?}", key);
    }
}

fn display(x: i16, y: i16) {
    let term = Term::stdout();
    let _ = term.clear_screen();
    let _ = term.write_line("messing around with k_board:");
    let _ = term.write_line("");
    for i in 0..10 {
        let mut string_to_print: String = "".to_string();
        for j in 0..10 {
            string_to_print += if i == y && j == x { "#" } else { "." }
        }
        let _ = term.write_line(&string_to_print);
    }
}
