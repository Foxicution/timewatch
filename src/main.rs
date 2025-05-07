mod ascii_art;

use ascii_art::DIGITS;

pub fn print_digit(digit: &[&str]) {
    for line in digit {
        println!("{}", line);
    }
}

fn main() {
    print_digit(DIGITS[0]);
}
