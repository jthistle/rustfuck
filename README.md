# Rustfuck

An optimizing interpreter for brainfuck, written in Rust.

## Build & install

Requires:

- `git`
- `cargo`

From source:

```
git clone https://github.com/jthistle/rustfuck.git && cd rustfuck
cargo build --release
```

Then to install to `/usr/local/bin/`:

```
sudo cp ./target/release/rustfuck /usr/local/bin/
```

## Usage

### Example

Run a file:

`rustfuck ./hello_world.b`

### Full options

```
Usage:
  rustfuck [OPTIONS] [FILENAME]

Run brainfuck code.

Positional arguments:
  filename              File containing brainfuck code

Optional arguments:
  -h,--help             Show this help message and exit
  -r,--raw RAW          Raw brainfuck code to run
  --no-optimize         Don't optimize code
  -s,--cell-size CELL_SIZE
                        Size of each cell in bits. Accepted values: 1, 2, 4, 8,
                        16, 32. Default 8.
```

## Design

Rustfuck is a simple but powerful interpreter. It works as follows:

1. Brainfuck code is parsed into an intermediate representation (an abstract syntax tree of a sort, but there
   aren't really any branches in it which makes it a pretty rubbish tree). At this stage, the points to which
   each loop token jumps are calculated.
2. A couple of optimization passes are made over the tree:
   1. The first pass collapses duplicated symbols into a single token
      in the tree, i.e. `-------` gets collapsed into an instruction to `-7` from the current cell. This applies to `-`, `+`, `>`, and `<`.
      This is a really simple optimization, but can save loads of time in loops.
   2. The second pass translates any occurences of `[-]` into a single instruction to set the current cell's value to `0`.
      The same applies to `[+]`.
3. Finally, the syntax tree is executed. Each token in the tree is taken in turn and executed sequentially, and loop jumps are carried out
   when needed.

There are other optimizations that could be made, but none have so much of an effect as the two previously mentioned.
   

