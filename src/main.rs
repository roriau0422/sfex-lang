use clap::{Parser, Subcommand};
use sfex_lang::{Interpreter, Lexer, Parser as SFXParser};
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "sfex")]
#[command(author = "Temuujin <roriau@gmail.com>")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "SFX (Situation Framework eXchange) - Programming the way humans think", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run { file: PathBuf },
    Lex { file: PathBuf },
    New { name: String },
    Version,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => {
            if run_script(&file).is_err() {
                process::exit(1);
            }
        }
        Commands::Lex { file } => {
            if lex_script(&file).is_err() {
                process::exit(1);
            }
        }
        Commands::New { name } => {
            println!("Creating new SFX project: {}", name);
            println!("Project scaffolding coming soon!");
        }
        Commands::Version => {
            print_version_info();
        }
    }
}

fn run_script(path: &PathBuf) -> Result<(), ()> {
    println!("Running SFX script: {}", path.display());
    println!();

    let source = fs::read_to_string(path).map_err(|e| {
        eprintln!("Error reading file: {}", e);
    })?;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| {
        eprintln!("Lexer error: {:?}", e);
    })?;

    // for token in &tokens {
    //     println!("{:?}", token.token_type);
    // }

    let mut parser = SFXParser::new(tokens);
    let program = parser.parse().map_err(|e| {
        eprintln!("Parser error: {:?}", e);
    })?;

    let mut interpreter = Interpreter::new();
    interpreter.run(program).map_err(|e| {
        eprintln!("Runtime error: {:?}", e);
    })?;

    Ok(())
}

fn lex_script(path: &PathBuf) -> Result<(), ()> {
    println!("🔍 Tokenizing SFX script: {}", path.display());
    println!();

    let source = fs::read_to_string(path).map_err(|e| {
        eprintln!("Error reading file: {}", e);
    })?;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| {
        eprintln!("Lexer error: {:?}", e);
    })?;

    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Token Analysis                                              │");
    println!("├─────────────────────────────────────────────────────────────┤");
    println!("│ Total tokens: {:<45} │", tokens.len());
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    use sfex_lang::TokenType;

    for (i, token) in tokens.iter().enumerate() {
        let token_display = match &token.token_type {
            TokenType::Eof => "EOF".to_string(),
            TokenType::Newline => "NEWLINE".to_string(),
            TokenType::Indent => "INDENT".to_string(),
            TokenType::Dedent => "DEDENT".to_string(),
            TokenType::ErrorToken => "ERROR".to_string(),
            TokenType::Number(n) => format!("NUMBER({})", n),
            TokenType::String_(s) => format!("STRING(\"{}\")", s),
            TokenType::Identifier(id) => format!("ID({})", id),
            TokenType::Comment(c) => format!("COMMENT({})", c),
            TokenType::Story => "KEYWORD(Story)".to_string(),
            TokenType::Concept => "KEYWORD(Concept)".to_string(),
            TokenType::Situation => "KEYWORD(Situation)".to_string(),
            TokenType::Adjust => "KEYWORD(Adjust)".to_string(),
            TokenType::If => "KEYWORD(If)".to_string(),
            TokenType::Else => "KEYWORD(Else)".to_string(),
            TokenType::Repeat => "KEYWORD(Repeat)".to_string(),
            TokenType::For => "KEYWORD(For)".to_string(),
            TokenType::Return => "KEYWORD(Return)".to_string(),
            TokenType::Break => "KEYWORD(Break)".to_string(),
            TokenType::Continue => "KEYWORD(Continue)".to_string(),
            TokenType::To => "KEYWORD(To)".to_string(),
            TokenType::With => "KEYWORD(with)".to_string(),
            TokenType::Is => "OPERATOR(is)".to_string(),
            TokenType::Plus => "OPERATOR(+)".to_string(),
            TokenType::Minus => "OPERATOR(-)".to_string(),
            TokenType::Star => "OPERATOR(*)".to_string(),
            TokenType::Slash => "OPERATOR(/)".to_string(),
            TokenType::Equals => "OPERATOR(=)".to_string(),
            TokenType::Greater => "OPERATOR(>)".to_string(),
            TokenType::Less => "OPERATOR(<)".to_string(),
            TokenType::True_ => "LITERAL(True)".to_string(),
            TokenType::False_ => "LITERAL(False)".to_string(),
            _ => format!("{:?}", token.token_type),
        };

        println!(
            "{:4} │ Line {:3}, Col {:3} │ {}",
            i + 1,
            token.line,
            token.column,
            token_display
        );
    }

    println!();
    println!("Tokenization complete!");
    Ok(())
}

fn print_version_info() {
    println!(
        "SFX (Situation Framework eXchange) v{}",
        env!("CARGO_PKG_VERSION")
    );
    println!("Programming the way humans think");
    println!();
    println!("Build information:");
    println!("  - Target: {}", std::env::consts::ARCH);
    println!("  - OS: {}", std::env::consts::OS);
    println!();
}
