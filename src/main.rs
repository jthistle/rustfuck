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
    let mut loop_stack: Vec<usize> = Vec::new();
    let mut i = 0;

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
                loop_stack.push(i as usize);
                ast.push(
                    Token::new(TokenType::LoopStart, -1)
                );
            },
            Some(']') => {
                let jmp = match loop_stack.pop() {
                    Some(x) => x,
                    None => break Err("Unmatched ]")
                };
                
                ast.push(
                    Token::new(TokenType::LoopEnd, jmp as i32)
                );

                ast[jmp].value = i;
            }
            None => {
                if loop_stack.len() > 0 {
                    break Err("Unmatched ]")
                } else {
                    break Ok(())
                }
            },
            _ => {
                i -= 1;
            },
        }
        i += 1;
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
        let diff = (end - start - 1) as i32;
        ast.drain(start..end);
        ast.insert(start, token);

        // Update jump points
        for node in ast.iter_mut() {
            if node.tk == TokenType::LoopStart || node.tk == TokenType::LoopEnd {
                if node.value as usize > start {
                    node.value -= diff;
                }
            }
        }
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

fn optimize(ast: &mut Ast) {
    pass_collapse_duplicated(ast);
}

fn execute(ast: &Ast) -> Result<(), &'static str> {
    let mut cells: Vec<u8> = iter::repeat(0).take(30000).collect();
    let mut data_pointer = 0;
    let mut stdout = io::stdout();
    let mut stdin = io::stdin();
    let mut instruction_pointer = 0;

    loop {
        let token = &ast[instruction_pointer];
        match token.tk {
            TokenType::Add => {
                cells[data_pointer] = cells[data_pointer].wrapping_add(token.value as u8);
            },
            TokenType::Sub => {
                cells[data_pointer] = cells[data_pointer].wrapping_sub(token.value as u8);
            },
            TokenType::Left => {
                if data_pointer < token.value as usize {
                    return Err("Data pointer moved out of bounds!")
                }
                data_pointer -= token.value as usize;
            },
            TokenType::Right => {
                if data_pointer + token.value as usize >= 30000 {
                    return Err("Data pointer moved out of bounds!")
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
                match stdin.read(&mut cells[data_pointer..data_pointer+1]) {
                    Ok(_) => {},
                    Err(_) => return Err("Could not read from stdin")
                }
            },
            TokenType::Out => {
                match stdout.write(&cells[data_pointer..data_pointer+1]) {
                    Ok(_) => {},
                    Err(_) => return Err("Could not write to stdout")
                }

                match stdout.flush() {
                    Ok(_) => {},
                    Err(_) => return Err("Could not flush stdout")
                }
            },
            TokenType::End => return Ok(()),
            _ => {},
        }

        instruction_pointer += 1;
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

    let mut ast = match parse(&raw) {
        Ok(ast) => ast,
        Err(err) => return Err(err),
    };

    // println!("initial: {:?}\n", ast);

    optimize(&mut ast);
    
    // println!("optimized: {:?}", ast);

    execute(&ast)
}
