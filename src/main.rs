use clap::{Parser, Subcommand};
use sage::lexer::Lexer;

#[derive(Parser)]
#[command(
    name = "sage",
    version,
    about = "Fast like C. Productive like Python. AI-native."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a .sg file
    Build {
        /// Path to the source file
        file: String,
    },
    /// Compile and run a .sg file
    Run {
        /// Path to the source file
        file: String,
    },
    /// Start the interactive REPL
    Repl,
    /// Check types without compiling
    Check {
        /// Path to the source file
        file: String,
    },
    /// Format source files
    Fmt {
        /// Path to the source file
        file: String,
    },
    /// Run tests
    Test,
    /// Initialize a new project
    Init {
        /// Project name
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { file } => {
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading '{}': {}", file, e);
                    std::process::exit(1);
                }
            };

            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize();

            for error in lexer.errors() {
                eprintln!("{}", error);
            }

            if !lexer.errors().is_empty() {
                std::process::exit(1);
            }

            println!("Lexed {} tokens from '{}'", tokens.len(), file);
            for token in &tokens {
                println!(
                    "  {:>4}:{:<3} {:?}",
                    token.span.line, token.span.column, token.kind
                );
            }

            // TODO: parser, type checker, codegen
            println!("\nParser not implemented yet.");
        }
        Commands::Run { file } => {
            println!("sage run: not implemented yet (file: {})", file);
        }
        Commands::Repl => {
            println!("sage repl: not implemented yet");
        }
        Commands::Check { file } => {
            println!("sage check: not implemented yet (file: {})", file);
        }
        Commands::Fmt { file } => {
            println!("sage fmt: not implemented yet (file: {})", file);
        }
        Commands::Test => {
            println!("sage test: not implemented yet");
        }
        Commands::Init { name } => {
            println!("sage init: not implemented yet (project: {})", name);
        }
    }
}
