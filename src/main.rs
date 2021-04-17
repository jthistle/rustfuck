extern crate argparse;
use argparse::{ArgumentParser, Store, StoreFalse};

use std::{str, fs, iter, io};
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

type ReplaceVec = Vec::<(usize, usize, Token)>;

fn replace_in_ast(ast: &mut Ast, mut replacements: ReplaceVec) {
    replacements.reverse();
    for (start, end, token) in replacements {
        ast.drain(start..end);
        ast.insert(start, token);
    }
}

fn pass_collapse_duplicated(ast: &mut Ast) {
    let mut start: usize = 0;
    let mut count: usize = 0;
    let mut current = TokenType::Invalid;
    let mut replace = ReplaceVec::new();
    for (i, node) in ast.iter().enumerate() {
        if current == TokenType::Invalid {
            if node.tk == TokenType::Add
            || node.tk == TokenType::Sub
            || node.tk == TokenType::Left
            || node.tk == TokenType::Right {
                start = i;
                count = 1;
                current = node.tk;
            }
        } else {
            if node.tk == current {
                count += 1;
            } else {
                if count > 1 {
                    replace.push((start, i, Token::new(current, count as i32)));
                }
                current = TokenType::Invalid;
            }
        }
    }

    replace_in_ast(ast, replace);
}

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

fn optimize(ast: &mut Ast) {
    pass_collapse_duplicated(ast);
    pass_zero_cell(ast);
}

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

type CellMaxSize = u64;
type CellMaxSizeLower = u32;

fn execute(ast: &Ast, cell_size: u16, tape_size: usize) -> Result<(), &'static str> 
{
    if cell_size < 1 || cell_size > 32 {
        return Err("Invalid cell size");
    }

    if tape_size < 1 {
        return Err("Tape size must be greater than 0");
    } 

    let mut cells: Vec<CellMaxSize> = iter::repeat(0).take(1000).collect();
    let mut data_pointer = 0;
    let mut stdout = io::stdout();
    let mut stdin = io::stdin();
    let mut instruction_pointer = 0;

    let max = (2 as CellMaxSize).pow(cell_size as CellMaxSizeLower);

    loop {
        let token = &ast[instruction_pointer];
        match token.tk {
            TokenType::Add => {
                cells[data_pointer] = cells[data_pointer].wrapping_add(token.value as CellMaxSize);
                if cells[data_pointer] >= max {
                    cells[data_pointer] %= max;
                }
            },
            TokenType::Sub => {
                if cells[data_pointer] < (token.value as CellMaxSize) {
                    cells[data_pointer] = max - (token.value as CellMaxSize - cells[data_pointer]);
                } else {
                    cells[data_pointer] -= token.value as CellMaxSize;
                }
            },
            TokenType::Left => {
                if data_pointer < token.value as usize {
                    return Err("Data pointer moved out of bounds (too far left)!")
                }
                data_pointer -= token.value as usize;
            },
            TokenType::Right => {
                let new_pos = data_pointer + token.value as usize;
                if new_pos >= tape_size {
                    return Err("Data pointer moved out of bounds (too far right)!")
                } else if new_pos > cells.len() {
                    // Allocate more space for the tape, we need it
                    cells.extend(
                        iter::repeat::<CellMaxSize>(0).take(new_pos - cells.len() + 1000)
                    );
                }

                data_pointer += token.value as usize;
            },
            TokenType::LoopStart => {
                if cells[data_pointer] == 0 {
                    instruction_pointer = token.value as usize;
                }
            },
            TokenType::LoopEnd => {
                if cells[data_pointer] > 0 {
                    instruction_pointer = token.value as usize;
                }
            },
            TokenType::In => {
                let mut buf = [0]; 
                match stdin.read_exact(&mut buf) {
                    Ok(_) => {
                        cells[data_pointer] = buf[0] as CellMaxSize;
                    },
                    Err(x) => {
                        if x.kind() == io::ErrorKind::UnexpectedEof {
                            // Treat EOF as 0
                            cells[data_pointer] = 0;
                        } else {
                            return Err("Could not read from stdin")
                        }
                    }
                }
            },
            TokenType::Out => {
                let buf = [cells[data_pointer] as u8];
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
                cells[data_pointer] = token.value as CellMaxSize;
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
    let mut cell_size: u16 = 8;
    let mut tape_size: usize = 30000;

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
            .add_option(&["-s", "--cell-size"], Store, "Size of each cell in bits. Accepted values: 1, 2, 4, 8, 16, 32. Default 8.");
        ap.refer(&mut tape_size)
            .add_option(&["-t", "--tape-size"], Store, "Size of the data tape. Default 30000.");
        ap.parse_args_or_exit();
    }

    if filename != "" {
        raw = match fs::read_to_string(filename) {
            Ok(x) => x,
            Err(_) => return Err("Could not open file"),
        }
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

    execute(&ast, cell_size, tape_size)
}
