extern crate argparse;
use argparse::{ArgumentParser, Store, StoreFalse, StoreTrue};

mod cell_size;
use cell_size::{CellSize};

use std::{str, fs, io};
use io::{Write, Read};

#[derive(Debug, PartialEq, Clone, Copy)]
enum TokenType {
    Invalid,
    End,
    Add,
    Sub,
    Left,
    Right,
    Out,
    In,
    LoopStart,
    LoopEnd,
    Set,
    Move,
}

#[derive(Debug)]
struct Token {
    pub tk: TokenType,
    pub value: i32,
}

impl Token {
    fn new(tk: TokenType, value: i32) -> Token {
        Token {
            tk,
            value,
        }
    }
}

type Ast = Vec<Token>;

trait Dumpable {
    fn dump(&self) -> String;
}

impl Dumpable for Ast {
    fn dump(&self) -> String {
        let mut out = String::new();
        let mut depth = 0;
        let mut line = String::new();
        for token in self.iter() {
            let mut end_line = false;

            let mut part: String = match token.tk {
                TokenType::Add => {
                    if token.value == 1 {
                        "+".to_string()
                    } else {
                        format!("+{}", token.value)
                    }
                },
                TokenType::Sub => {
                    if token.value == 1 {
                        "-".to_string()
                    } else {
                        format!("-{}", token.value)
                    }
                },
                TokenType::Left => {
                    if token.value == 1 {
                        "<".to_string()
                    } else {
                        format!("<{}", token.value)
                    }
                },
                TokenType::Right => {
                    if token.value == 1 {
                        ">".to_string()
                    } else {
                        format!(">{}", token.value)
                    }
                },
                TokenType::In => {
                    ",".to_string()
                },
                TokenType::Out => {
                    ".".to_string()
                },
                TokenType::LoopStart => {
                    end_line = true;
                    "[".to_string()
                },
                TokenType::LoopEnd => {
                    end_line = true;
                    "]".to_string()
                },
                TokenType::Set => {
                    format!("S{}", token.value)
                },
                TokenType::Move => {
                    format!("M{}", token.value)
                },
                TokenType::Invalid => {
                    "INVALID".to_string()
                },
                TokenType::End => {
                    ":".to_string()
                },
            };

            part.push(' ');

            if ! end_line {
                line.push_str(&part);
            }

            if line.len() >= 80 || end_line {
                out.push_str(
                    &format!("{}{}\n",
                        "  ".repeat(depth),
                        line
                    )
                );
                line.clear();
            }

            if token.tk == TokenType::LoopEnd {
                depth -= 1;
            }

            if end_line {
                out.push_str(
                    &format!("{}{}\n",
                        "  ".repeat(depth),
                        part
                    )
                );
            }

            if token.tk == TokenType::LoopStart {
                depth += 1;
            }
        }

        out
    }
}

/// Parses raw text into an intermediate representation.
fn parse(raw: &String) -> Result<Ast, &'static str> {
    let mut ast = Ast::new();
    let mut chars = raw.chars();

    let res = loop {
        let c = chars.next();
        match c {
            Some('+') => {
                ast.push(
                    Token::new(TokenType::Add, 1)
                );
            },
            Some('-') => {
                ast.push(
                    Token::new(TokenType::Sub, 1)
                );
            },
            Some('>') => {
                ast.push(
                    Token::new(TokenType::Right, 1)
                );
            },
            Some('<') => {
                ast.push(
                    Token::new(TokenType::Left, 1)
                );
            },
            Some('.') => {
                ast.push(
                    Token::new(TokenType::Out, 0)
                );
            },
            Some(',') => {
                ast.push(
                    Token::new(TokenType::In, 0)
                );
            },
            Some('[') => {
                ast.push(
                    Token::new(TokenType::LoopStart, -1)
                );
            },
            Some(']') => {
                ast.push(
                    Token::new(TokenType::LoopEnd, -1)
                );
            },
            None => {
                break Ok(())
            },
            _ => {},
        }
    };

    ast.push(
        Token::new(TokenType::End, 0)
    );

    match res {
        Ok(_) => Ok(ast),
        Err(x) => Err(x),
    }
}

/// A vector of replacements to be made with form:
/// `(begin, end, replacement)`
/// where `begin` is included and `end` is excluded.
type ReplaceVec = Vec::<(usize, usize, Token)>;

/// Given a sorted vector of replacements to be made, replace the given
/// ranges of tokens with a single other token in the AST.
fn replace_in_ast(ast: &mut Ast, mut replacements: ReplaceVec) {
    replacements.reverse();
    for (start, end, token) in replacements {
        ast.drain(start..end);
        ast.insert(start, token);
    }
}

/// Collapses duplicated tokens into a single token.
///
/// e.g. `------`, which is represented as six `TokenType::Sub` with value `1`,
/// is replaced by a single `TokenType::Sub` with value `6`. This applies to `-`, `+`, `>`, and `<`.
fn pass_collapse_duplicated(ast: &mut Ast) {
    let mut start: usize = 0;
    let mut count: usize = 0;
    let mut current = TokenType::Invalid;
    let mut replace = ReplaceVec::new();
    for (i, node) in ast.iter().enumerate() {
        if node.tk == current {
            count += 1;
        } else {
            if count > 1 {
                replace.push((start, i, Token::new(current, count as i32)));
            }

            if node.tk == TokenType::Add
            || node.tk == TokenType::Sub
            || node.tk == TokenType::Left
            || node.tk == TokenType::Right {
                start = i;
                count = 1;
                current = node.tk;
            } else {
                count = 0;
                current = TokenType::Invalid;
            }
        }
    }

    replace_in_ast(ast, replace);
}

/// Replaces 'zeroing' instructions with a single token to reduce time spent in loops.
///
/// This replaces `[-]` and `[+]` (and all variants of these which have an odd number of inner symbols)
/// with a single token of `TokenType::Set` and value `0`.
///
/// This pass must be run after Collapse Duplicated.
fn pass_zero_cell(ast: &mut Ast) {
    let mut replace = ReplaceVec::new();
    let mut progress = 0;
    for (i, node) in ast.iter().enumerate() {
        if progress == 0 && node.tk == TokenType::LoopStart {
            progress += 1;
        } else if progress == 1 && (node.tk == TokenType::Sub || node.tk == TokenType::Add) && node.value % 2 == 1 {
            progress += 1;
        } else if progress == 2 && node.tk == TokenType::LoopEnd {
            replace.push((i - 2, i + 1, Token::new(
                TokenType::Set, 0
            )));
            progress = 0;
        } else {
            progress = 0;
        }
    }

    replace_in_ast(ast, replace);
}

/// Replaces idiomatic moves of the form `[->+<]` with a single `TokenType::Move` token.
/// This also accepts any number of left/right tokens, e.g. `[->>>+<<<]` can also be optimized.
/// Left/right tokens can also be put in the opposite order, e.g. `[-<<<+>>>]`.1
///
/// Note that a 'move' adds the value of the src cell to the destination - it doesn't replace it.
/// The src cell has its value set to 0 afterwards.
///
/// This pass must be run after Collapse Duplicated.
fn pass_move_value(ast: &mut Ast) {
    let mut replace = ReplaceVec::new();
    let mut next_direction = TokenType::Invalid;
    let mut move_count = -1;
    let mut progress = 0;

    for (i, node) in ast.iter().enumerate() {
        if progress == 0 && node.tk == TokenType::LoopStart {
            progress += 1;
        } else if progress == 1 && node.tk == TokenType::Sub && node.value == 1 {
            progress += 1;
        } else if progress == 2 && (node.tk == TokenType::Left || node.tk == TokenType::Right) {
            if node.tk == TokenType::Left {
                next_direction = TokenType::Right;
            } else {
                next_direction = TokenType::Left;
            }
            move_count = node.value;
            progress += 1;
        } else if progress == 3 && node.tk == TokenType::Add && node.value == 1 {
            progress += 1;
        } else if progress == 4 && node.tk == next_direction && node.value == move_count {
            progress += 1;
        } else if progress == 5 && node.tk == TokenType::LoopEnd {
            replace.push((i - 5, i + 1, Token::new(
                TokenType::Move, move_count * (if next_direction == TokenType::Right { -1 } else { 1 })
            )));
            progress = 0;
        } else {
            progress = 0;
        }
    }

    replace_in_ast(ast, replace);
}

/// Runs optimizer passes on the AST.
fn optimize(ast: &mut Ast) {
    pass_collapse_duplicated(ast);
    pass_zero_cell(ast);
    pass_move_value(ast);
}

/// Caches loop jump endpoints to reduce time spent searching during
/// execution.
fn link_loops(ast: &mut Ast) -> Result<(), &'static str> {
    let mut loop_stack: Vec<usize> = Vec::new();

    for i in 0..ast.len() {
        match ast[i].tk {
            TokenType::LoopStart => {
                loop_stack.push(i);
            },
            TokenType::LoopEnd => {
                let jmp = match loop_stack.pop() {
                    Some(x) => x,
                    None => return Err("Unmatched ]")
                };

                ast[i].value = jmp as i32;
                ast[jmp].value = i as i32;
            }
            _ => {},
        }
    };

    if loop_stack.len() > 0 {
        Err("Unmatched [")
    } else {
        Ok(())
    }
}

/// Runs the AST.
fn execute<T>(ast: &Ast, tape_size: usize) -> Result<(), &'static str>
where T: CellSize + Clone + Copy
{
    if tape_size < 1 {
        return Err("Tape size must be greater than 0");
    }

    let mut cells: Vec<T> = T::get_zeroes(1000).collect();
    let mut data_pointer = 0;
    let mut stdout = io::stdout();
    let mut stdin = io::stdin();
    let mut instruction_pointer = 0;

    loop {
        let token = &ast[instruction_pointer];
        match token.tk {
            TokenType::Add => {
                cells[data_pointer].add_to_cell(T::from_tk_value(token.value));
            },
            TokenType::Sub => {
                cells[data_pointer].sub_from_cell(T::from_tk_value(token.value));
            },
            TokenType::Left => {
                if data_pointer < token.value as usize {
                    return Err("Data pointer moved out of bounds (too far left)")
                }
                data_pointer -= token.value as usize;
            },
            TokenType::Right => {
                let new_pos = data_pointer + token.value as usize;
                if new_pos >= tape_size {
                    return Err("Data pointer moved out of bounds (too far right)")
                } else if new_pos > cells.len() {
                    // Allocate more space for the tape, we need it
                    cells.extend(
                        T::get_zeroes(new_pos - cells.len() + 1000)
                    );
                }

                data_pointer += token.value as usize;
            },
            TokenType::LoopStart => {
                if cells[data_pointer].is_zero() {
                    instruction_pointer = token.value as usize;
                }
            },
            TokenType::LoopEnd => {
                if cells[data_pointer].is_nonzero() {
                    instruction_pointer = token.value as usize;
                }
            },
            TokenType::In => {
                let mut buf = [0];
                match stdin.read_exact(&mut buf) {
                    Ok(_) => {
                        cells[data_pointer] = T::from_stdout(buf[0]);
                    },
                    Err(x) => {
                        if x.kind() == io::ErrorKind::UnexpectedEof {
                            // Treat EOF as 0
                            cells[data_pointer] = T::from_tk_value(0);
                        } else {
                            return Err("Could not read from stdin")
                        }
                    }
                }
            },
            TokenType::Out => {
                let buf = [cells[data_pointer].to_stdin()];
                match stdout.write(&buf) {
                    Ok(_) => {},
                    Err(_) => return Err("Could not write to stdout")
                }

                match stdout.flush() {
                    Ok(_) => {},
                    Err(_) => return Err("Could not flush stdout")
                }
            },
            TokenType::Set => {
                cells[data_pointer] = T::from_tk_value(token.value);
            },
            TokenType::Move => {
                if cells[data_pointer].is_nonzero() {
                    let dest = data_pointer as i32 + token.value;
                    if dest < 0 {
                        return Err("Data pointer moved out of bounds (too far left)")
                    }

                    let dest = dest as usize;
                    if dest >= tape_size {
                        return Err("Data pointer moved out of bounds (too far right)")
                    } else if dest > cells.len() {
                        // Allocate more space for the tape, we need it
                        // TODO this is duplicated code, refactor this in future
                        cells.extend(
                            T::get_zeroes(dest - cells.len() + 1000)
                        );
                    }

                    let val = cells[data_pointer];
                    cells[dest].add_to_cell(val);
                    cells[data_pointer] = T::from_tk_value(0);
                }
            },
            TokenType::End => return Ok(()),
            _ => {},
        }

        instruction_pointer += 1;
    }
}

fn main() -> Result<(), &'static str> {
    let mut filename = String::new();
    let mut raw = String::new();
    let mut do_optimize = true;
    let mut cell_size: u8 = 8;
    let mut tape_size: usize = 30000;
    let mut dump = false;

    {  // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Run brainfuck code.");
        ap.refer(&mut filename)
            .add_argument("filename", Store, "File containing brainfuck code");
        ap.refer(&mut raw)
            .add_option(&["-r", "--raw"], Store, "Raw brainfuck code to run");
        ap.refer(&mut do_optimize)
            .add_option(&["--no-optimize"], StoreFalse, "Don't optimize code");
        ap.refer(&mut cell_size)
            .add_option(&["-s", "--cell-size"], Store, "Size of each cell in bits. Accepted values: 8, 16, 32, 64. Default 8.");
        ap.refer(&mut tape_size)
            .add_option(&["-t", "--tape-size"], Store, "Size of the data tape. Default 30000.");
        ap.refer(&mut dump)
            .add_option(&["--dump"], StoreTrue, "Dump the AST and exit without executing the code.");
        ap.parse_args_or_exit();
    }

    if filename != "" {
        raw = match fs::read_to_string(filename) {
            Ok(x) => x,
            Err(_) => return Err("Could not open file"),
        }
    } else if raw == "" {
        return Err("Please provide a filename. Use flag  --help  for usage help.")
    }

    let mut ast = match parse(&raw) {
        Ok(ast) => ast,
        Err(err) => return Err(err),
    };

    if do_optimize {
        optimize(&mut ast);
    }

    match link_loops(&mut ast) {
        Ok(_) => {},
        Err(err) => return Err(err),
    };


    if dump {
        println!("{}", ast.dump());
        return Ok(());
    }

    match cell_size {
        8 => execute::<u8>(&ast, tape_size),
        16 => execute::<u16>(&ast, tape_size),
        32 => execute::<u32>(&ast, tape_size),
        64 => execute::<u64>(&ast, tape_size),
        _ => Err("Unsupported cell size")
    }

}
