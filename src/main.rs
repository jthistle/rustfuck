use std::{str, fs, iter, io};
use io::{Write, Read};

fn interpret(raw: &String) -> Result<(), &'static str> {
    let mut chars = raw.chars().enumerate();
    let mut cells: Vec<u8> = iter::repeat(0).take(30000).collect();
    let mut data_pointer = 0;
    let mut loop_stack: Vec<usize> = Vec::new();
    let mut stdout = io::stdout();
    let mut stdin = io::stdin();

    'mainloop: loop {
        let mut c = chars.next();
        match c {
            Some((_, '+')) => {
                if cells[data_pointer] == 255 {
                    cells[data_pointer] = 0;
                } else {
                    cells[data_pointer] += 1;
                }
            },
            Some((_, '-')) => {
                if cells[data_pointer] == 0 {
                    cells[data_pointer] = 255;
                } else {
                    cells[data_pointer] -= 1;
                }
            },
            Some((_, '>')) => {
                data_pointer += 1;

                if data_pointer >= 30000 {
                    break Ok(())
                }
            },
            Some((_, '<')) => {
                if data_pointer == 0 {
                    break Err("Cannot decrement data pointer - is already as far left as possible")
                }

                data_pointer -= 1;
            },
            Some((_, '.')) => {
                match stdout.write(&cells[data_pointer..data_pointer+1]) {
                    Ok(_) => {},
                    Err(_) => break Err("Could not write to stdout")
                }

                match stdout.flush() {
                    Ok(_) => {},
                    Err(_) => break Err("Could not flush stdout")
                }
            },
            Some((_, ',')) => {
                match stdin.read(&mut cells[data_pointer..data_pointer+1]) {
                    Ok(_) => {},
                    Err(_) => break Err("Could not read from stdin")
                }
            },
            Some((i, '[')) => {
                if cells[data_pointer] == 0 {
                    let mut depth: usize = 1;
                    loop {
                        c = chars.next();
                        match c {
                            Some((_, '[')) => depth += 1,
                            Some((_, ']')) => {
                                depth -= 1;
                                if depth == 0 { break }
                            },
                            None => break 'mainloop Err("Unmatched ["),
                            _ => {},
                        }
                    };
                } else {
                    loop_stack.push(i);
                }
            },
            Some((_, ']')) => {
                if cells[data_pointer] != 0 {
                    match loop_stack.last() {
                        Some(new_pos) => {
                            chars = raw.chars().enumerate();
                            chars.nth(*new_pos).unwrap();
                        },
                        None => break Err("Unmatched ]"),
                    }
                } else {
                    loop_stack.pop();
                }
            }
            None => {
                if loop_stack.len() > 0 {
                    break Err("Unmatched ]")
                } else {
                    break Ok(())
                }
            },
            _ => {},
        }
    }
}


fn main() -> Result<(), &'static str> {
    let mut args = std::env::args();

    if args.len() == 1 {
        return Err("Need filename");
    }

    let filename = args.nth(1).unwrap();

    let raw = fs::read_to_string(filename)
        .expect("Could not open file");

    match interpret(&raw) {
        Ok(_) => {
            println!("");
            Ok(())
        },
        err => err,
    }
}
