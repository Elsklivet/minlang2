mod lexer;
mod ast;
mod parser;
mod program;
mod vm;

use clap::{Parser};
use std::fs;
use std::path::Path;
use std::process::exit;

use crate::vm::Vm;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source file path
    source_path: String,
    /// Whether to stop at the lexing phase
    #[arg(short, long)]
    lex: bool,
    /// Whether to stop at the parsing phase
    #[arg(short, long)]
    parse: bool,
    /// Whether to run the program in a VM
    #[arg(short, long)]
    run: bool,
    /// Whether to verbosely print information from the ending phase
    #[arg(short, long)]
    verbose: bool,
    /// Show the table after execution completes
    #[arg(short, long)]
    show_registers: bool,
}

fn main() {
    let args = Args::parse();

    let source_path = args.source_path;

    // Parse path into system path
    let path = Path::new(&source_path);

    let source_contents = fs::read_to_string(path).expect(format!("Unable to open file with path '{}'", source_path).as_str());

    // Lex the source file
    let mut lexr = lexer::Lexer::new(source_contents);

    let lex_result = lexr.lex();

    let token_stream = if let Ok(strm) = lex_result {
        strm
    } else {
        panic!("Lexer error: {}", lex_result.err().unwrap());
    };

    // Bail if only lexing
    if args.lex {
        if args.verbose {
            println!("{:?}", token_stream.tokens);
        }
        exit(0);
    }

    // Parse the program
    let mut parsr = parser::Parser::new(token_stream.tokens);

    let parse_result = parsr.parse_program();

    let prog = if let Ok(prg) = parse_result {
        prg
    } else {
        panic!("Parser error: {}", parse_result.err().unwrap());
    };

    if args.parse {
        if args.verbose {
            println!("{:?}", prog);
        }
    }

    if args.run {
        Vm::new(prog).run(args.show_registers);
    }
}
