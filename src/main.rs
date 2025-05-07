mod ascii_art;

use ascii_art::DIGITS;
use clap::Parser;

type Time = (u64, u64, u64);

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    time: String,
    message: Option<String>,
}

fn parse_time(time: &str) -> Result<Time, String> {
    let parts: Vec<&str> = time.split(':').collect();

    let (h, m, s) = match parts.len() {
        1 => ("0", "0", parts[0]),
        2 => ("0", parts[0], parts[1]),
        3 => (parts[0], parts[1], parts[2]),
        _ => return Err(format!("Invalid time format: {time}")),
    };

    let h: u64 = h.parse().map_err(|_| format!("Invalid hours '{h}'"))?;
    let m: u64 = m.parse().map_err(|_| format!("Invalid minutes '{m}'"))?;
    let s: u64 = s.parse().map_err(|_| format!("Invalid seconds '{s}'"))?;

    Ok((h, m, s))
}

fn print_digit(digit: &[&str]) {
    for line in digit {
        println!("{}", line);
    }
}

fn main() {
    let args = Args::parse();
    println!("{args:?}");

    let (h, m, s) = parse_time(&args.time).unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        std::process::exit(1);
    });
    println!("{h}:{m}:{s}");

    print_digit(DIGITS[0]);
}
