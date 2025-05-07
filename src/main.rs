mod ascii_art;

use std::{
    io::{self, Stdout, Write, stdout},
    process::exit,
    time::{Duration, Instant},
};

use clap::Parser;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, poll, read},
    execute,
    style::{Print, Stylize},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode, size,
    },
};

use crate::ascii_art::{COLON, DIGITS, SYMBOL_DIMENSIONS};

type Time = (u64, u64, u64);

#[derive(Debug)]
enum Layout {
    Horizontal((u16, u16)),
    Vertical((u16, u16)),
    Impossible([(u16, u16); 2]),
}

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

fn digits(mut n: u64) -> Vec<u8> {
    let mut buf = [0u8; 20];
    let mut i = 20;
    loop {
        i -= 1;
        buf[i] = (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    if i > 18 { buf[18..].to_vec() } else { buf[i..].to_vec() }
}

fn choose_layout(
    term_dim: &(u16, u16),
    h_digits: &Vec<u8>,
    m_digits: &Vec<u8>,
    s_digits: &Vec<u8>,
    msg_lines: usize,
) -> Layout {
    let mut hl_dim = (0, 0); // hl - horizontal layout
    let mut vl_dim = (0, 0); // vl - vertical layout

    if h_digits.as_slice() != [0, 0] {
        hl_dim.0 += h_digits.len() * SYMBOL_DIMENSIONS.0 + h_digits.len() - 1 + SYMBOL_DIMENSIONS.0;
        vl_dim.0 += h_digits.len() * SYMBOL_DIMENSIONS.0 + h_digits.len() - 1;
        vl_dim.1 += SYMBOL_DIMENSIONS.1 + 1;
    };
    if m_digits.as_slice() != [0, 0] {
        hl_dim.0 += m_digits.len() * SYMBOL_DIMENSIONS.0 + m_digits.len() - 1 + SYMBOL_DIMENSIONS.0;
        vl_dim.0 += h_digits.len() * SYMBOL_DIMENSIONS.0 + h_digits.len() - 1;
        vl_dim.1 += SYMBOL_DIMENSIONS.1 + 1;
    };

    hl_dim.0 += s_digits.len() * SYMBOL_DIMENSIONS.0 + s_digits.len() - 1;
    vl_dim.0 += h_digits.len() * SYMBOL_DIMENSIONS.0 + h_digits.len() - 1;
    vl_dim.1 += SYMBOL_DIMENSIONS.1;

    hl_dim.1 += SYMBOL_DIMENSIONS.1;

    if msg_lines > 1 {
        hl_dim.1 += 1 + msg_lines;
        vl_dim.1 += 1 + msg_lines;
    }

    let hl_dim = (hl_dim.0 as u16, hl_dim.1 as u16);
    let vl_dim = (vl_dim.0 as u16, vl_dim.1 as u16);

    if term_dim.0 >= hl_dim.0 && term_dim.1 >= hl_dim.1 {
        Layout::Horizontal(hl_dim)
    } else if term_dim.0 >= vl_dim.0 && term_dim.1 >= vl_dim.1 {
        Layout::Vertical(vl_dim)
    } else {
        Layout::Impossible([hl_dim, vl_dim])
    }
}

fn center(line: &str, width: u16) -> String {
    let line_len = line.chars().count();

    if (width as usize) <= line_len {
        line.to_string()
    } else {
        let left_pad = (width as usize - line_len) / 2;
        format!("{left_pad}{line}")
    }
}

fn draw(
    seconds: u64,
    message: &Option<String>,
    term_dim: &(u16, u16),
    full_redraw: bool,
    stdout: &mut Stdout,
) -> io::Result<()> {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;

    let h_digits = digits(h);
    let m_digits = digits(m);
    let s_digits = digits(s);

    let message = match message {
        Some(text) => textwrap::wrap(text, term_dim.0 as usize),
        None => vec![],
    };

    let layout = choose_layout(term_dim, &h_digits, &m_digits, &s_digits, message.len());

    let h_digits: Vec<String> = match h_digits.as_slice() {
        [0, 0] => vec![],
        digits => (0..SYMBOL_DIMENSIONS.1)
            .map(|line_idx| {
                digits.iter().map(|d| DIGITS[*d as usize][line_idx]).collect::<Vec<_>>().join(" ")
            })
            .collect(),
    };
    let m_digits: Vec<String> = match m_digits.as_slice() {
        [0, 0] => vec![],
        digits => (0..SYMBOL_DIMENSIONS.1)
            .map(|line_idx| {
                digits.iter().map(|d| DIGITS[*d as usize][line_idx]).collect::<Vec<_>>().join(" ")
            })
            .collect(),
    };
    let s_digits: Vec<String> = (0..SYMBOL_DIMENSIONS.1)
        .map(|line_idx| {
            s_digits.iter().map(|d| DIGITS[*d as usize][line_idx]).collect::<Vec<_>>().join(" ")
        })
        .collect();

    if full_redraw {
        execute!(stdout, Clear(ClearType::All))?;
    }

    match layout {
        Layout::Horizontal(hl_size) => {
            let top_pad = (term_dim.1 - hl_size.1) / 2;

            for line_idx in 0..SYMBOL_DIMENSIONS.1 {
                let h_line = if !h_digits.is_empty() { h_digits[line_idx].as_str() } else { "" };
                let c1 = if !h_digits.is_empty() { COLON[line_idx] } else { "" };
                let m_line = if !m_digits.is_empty() { m_digits[line_idx].as_str() } else { "" };
                let c2 = if !m_digits.is_empty() { COLON[line_idx] } else { "" };
                let s_line = if !s_digits.is_empty() { s_digits[line_idx].as_str() } else { "" };
                let line = format!("{h_line}{c1}{m_line}{c2}{s_line}");
                let left_pad = (term_dim.0 - line.chars().count() as u16) / 2;

                execute!(
                    stdout,
                    MoveTo(left_pad, top_pad + line_idx as u16),
                    Clear(ClearType::CurrentLine),
                    Print(line)
                )?;
            }
            if !message.is_empty() && full_redraw {
                for line in message {
                    let left_pad = (term_dim.0 - line.chars().count() as u16) / 2;
                    execute!(
                        stdout,
                        MoveTo(left_pad, top_pad + SYMBOL_DIMENSIONS.1 as u16 + 1),
                        Clear(ClearType::CurrentLine),
                        Print(line.green())
                    )?;
                }
            }
        }
        Layout::Vertical(vl_size) => {
            let mut top_pad = (term_dim.1 - vl_size.1) / 2;

            for line in &h_digits {
                let left_pad = (term_dim.0 - line.chars().count() as u16) / 2;
                top_pad += 1;
                execute!(
                    stdout,
                    MoveTo(left_pad, top_pad),
                    Clear(ClearType::CurrentLine),
                    Print(line)
                )?;
            }

            if !h_digits.is_empty() {
                top_pad += 1;
            }

            for line in &m_digits {
                let left_pad = (term_dim.0 - line.chars().count() as u16) / 2;
                top_pad += 1;
                execute!(
                    stdout,
                    MoveTo(left_pad, top_pad),
                    Clear(ClearType::CurrentLine),
                    Print(line)
                )?;
            }

            if !m_digits.is_empty() {
                top_pad += 1;
            }

            for line in &s_digits {
                let left_pad = (term_dim.0 - line.chars().count() as u16) / 2;
                top_pad += 1;
                execute!(
                    stdout,
                    MoveTo(left_pad, top_pad),
                    Clear(ClearType::CurrentLine),
                    Print(line)
                )?;
            }

            if !message.is_empty() && full_redraw {
                top_pad += 1;

                for line in message {
                    let left_pad = (term_dim.0 - line.chars().count() as u16) / 2;
                    top_pad += 1;
                    execute!(
                        stdout,
                        MoveTo(left_pad, top_pad),
                        Clear(ClearType::CurrentLine),
                        Print(line.green())
                    )?;
                }
            }
        }
        Layout::Impossible([hl_size, vl_size]) => {
            let message = format!(
                "Error: terminal too small. Current: {}x{} Horizontal: {}x{} Vertical: {}x{}",
                term_dim.0, term_dim.1, hl_size.0, hl_size.1, vl_size.0, vl_size.1
            );
            execute!(stdout, Clear(ClearType::All), MoveTo(0, 0), Print(message))?;
        }
    };

    stdout.flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let (h, m, s) = parse_time(&args.time).unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        exit(1);
    });

    let wait_secs = h * 60 * 60 + m * 60 + s;
    let start_time = Instant::now();

    let mut stdout = stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide)?;

    let mut term_dim = size()?;
    let mut full_redraw = true;

    loop {
        if poll(Duration::from_millis(100))? {
            match read()? {
                Event::Key(KeyEvent { code, modifiers, kind: KeyEventKind::Press, .. }) => {
                    if code == KeyCode::Esc
                        || code == KeyCode::Char('q')
                        || (code == KeyCode::Char('c') && modifiers == KeyModifiers::CONTROL)
                    {
                        break;
                    }
                }
                Event::Resize(w, h) => {
                    term_dim = (w, h);
                    full_redraw = true;
                }
                _ => {}
            }
        }

        let elapsed_secs = start_time.elapsed().as_secs();
        let display_secs = wait_secs.checked_sub(elapsed_secs);

        match display_secs {
            Some(0) | None => break,
            Some(secs) => draw(secs, &args.message, &term_dim, full_redraw, &mut stdout)?,
        };
    }

    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
