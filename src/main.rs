use console::*;

fn main() {
    let term = Term::stdout();
    let x = 490;
    let red = Style::new().red();
    let text = format!("x is {}", red.apply_to(x));
    let _ = term.write_line(&text);
}
