use clap::{Parser, Subcommand};
use sfex_lang::stdlib::web;
use sfex_lang::{Interpreter, Lexer, Parser as SFXParser, project};
use std::fs;
use std::path::{Path, PathBuf};
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
    Run {
        file: PathBuf,
    },
    Lex {
        file: PathBuf,
    },
    Debug {
        file: PathBuf,
    },
    Serve {
        file: PathBuf,
        #[arg(short, long, default_value = "127.0.0.1:8000")]
        addr: String,
        #[arg(short, long)]
        static_dir: Option<PathBuf>,
        #[arg(long)]
        tls_cert: Option<PathBuf>,
        #[arg(long)]
        tls_key: Option<PathBuf>,
    },
    New {
        name: String,
    },
    Install,
    Lsp,
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
        Commands::Debug { file } => {
            if debug_script(&file).is_err() {
                process::exit(1);
            }
        }
        Commands::Serve {
            file,
            addr,
            static_dir,
            tls_cert,
            tls_key,
        } => {
            if serve_script(
                &file,
                &addr,
                static_dir.as_ref(),
                tls_cert.as_ref(),
                tls_key.as_ref(),
            )
            .is_err()
            {
                process::exit(1);
            }
        }
        Commands::New { name } => {
            if new_project(&name).is_err() {
                process::exit(1);
            }
        }
        Commands::Install => {
            if install_project().is_err() {
                process::exit(1);
            }
        }
        Commands::Lsp => {
            if sfex_lang::lsp::run().is_err() {
                process::exit(1);
            }
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
        eprintln!("Lexer error: {}", e);
    })?;

    // for token in &tokens {
    //     println!("{:?}", token.token_type);
    // }

    let mut parser = SFXParser::new(tokens);
    let program = parser.parse().map_err(|e| {
        eprintln!("Parser error: {}", e);
    })?;

    let mut interpreter = Interpreter::new();
    interpreter.run(program).map_err(|e| {
        eprintln!("Runtime error: {}", e);
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
        eprintln!("Lexer error: {}", e);
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

fn debug_script(path: &PathBuf) -> Result<(), ()> {
    println!("Debugging SFX script: {}", path.display());
    println!();

    let source = fs::read_to_string(path).map_err(|e| {
        eprintln!("Error reading file: {}", e);
    })?;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| {
        eprintln!("Lexer error: {}", e);
    })?;

    let mut parser = SFXParser::new(tokens);
    let program = parser.parse().map_err(|e| {
        eprintln!("Parser error: {}", e);
    })?;

    let mut interpreter = Interpreter::new();
    interpreter.enable_trace();
    interpreter.run(program).map_err(|e| {
        eprintln!("Runtime error: {}", e);
    })?;

    Ok(())
}

fn serve_script(
    path: &PathBuf,
    addr: &str,
    static_dir: Option<&PathBuf>,
    tls_cert: Option<&PathBuf>,
    tls_key: Option<&PathBuf>,
) -> Result<(), ()> {
    let handler_path = path
        .to_str()
        .ok_or_else(|| {
            eprintln!("Invalid handler path");
        })
        .map(|s| s.to_string())?;

    let static_str = static_dir.and_then(|p| p.to_str()).map(|s| s.to_string());
    let tls_cert_str = tls_cert.and_then(|p| p.to_str()).map(|s| s.to_string());
    let tls_key_str = tls_key.and_then(|p| p.to_str()).map(|s| s.to_string());

    match (tls_cert_str.as_deref(), tls_key_str.as_deref()) {
        (Some(cert), Some(key)) => {
            web::serve_tls(addr, &handler_path, cert, key, static_str.as_deref()).map_err(|e| {
                eprintln!("Serve error: {}", e);
            })?;
        }
        (None, None) => {
            web::serve(addr, &handler_path, static_str.as_deref()).map_err(|e| {
                eprintln!("Serve error: {}", e);
            })?;
        }
        _ => {
            eprintln!("Serve error: --tls-cert and --tls-key must be provided together");
            return Err(());
        }
    }

    Ok(())
}

fn new_project(name: &str) -> Result<(), ()> {
    let project_dir = Path::new(name);
    if project_dir.exists() {
        eprintln!("Directory '{}' already exists", name);
        return Err(());
    }

    fs::create_dir_all(project_dir).map_err(|e| {
        eprintln!("Failed to create project directory: {}", e);
    })?;

    let packages_dir = project_dir.join("packages");
    fs::create_dir_all(&packages_dir).map_err(|e| {
        eprintln!("Failed to create packages directory: {}", e);
    })?;

    let manifest = format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\n\n[dependencies]\n",
        name
    );
    fs::write(project_dir.join("sfex.toml"), manifest).map_err(|e| {
        eprintln!("Failed to write sfex.toml: {}", e);
    })?;

    let main_sfex = "Story:\n    Print \"Hello, SFX!\"\n";
    fs::write(project_dir.join("main.sfex"), main_sfex).map_err(|e| {
        eprintln!("Failed to write main.sfex: {}", e);
    })?;

    let readme = format!("# {}\n\nRun:\n\n```\nsfex run main.sfex\n```\n", name);
    fs::write(project_dir.join("README.md"), readme).map_err(|e| {
        eprintln!("Failed to write README.md: {}", e);
    })?;

    println!("Created new SFX project at {}", project_dir.display());
    Ok(())
}

fn install_project() -> Result<(), ()> {
    let cwd = std::env::current_dir().map_err(|e| {
        eprintln!("Failed to resolve current directory: {}", e);
    })?;

    let root = project::find_project_root(&cwd).ok_or_else(|| {
        eprintln!("No sfex.toml found (run from a project directory).");
    })?;

    let installed = project::install_dependencies(&root).map_err(|e| {
        eprintln!("Install failed: {}", e);
    })?;

    if installed.is_empty() {
        println!("No dependencies to install.");
    } else {
        println!("Installed dependencies:");
        for name in installed {
            println!("  - {}", name);
        }
    }

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
