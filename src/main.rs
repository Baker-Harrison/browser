mod error;

use clap::Parser;
use error::{BrowserError, Result};

/// A modern web browser built from scratch in Rust
#[derive(Parser)]
#[command(name = "browser")]
#[command(about = "A modern, cross-platform web browser", long_about = None)]
struct Cli {
    /// URL to navigate to
    #[arg(value_name = "URL")]
    url: Option<String>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    if let Some(url) = cli.url {
        if cli.verbose {
            println!("Navigating to: {}", url);
        }
        match navigate_to_url(&url) {
            Ok(_) => println!("Loading: {}", url),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("Browser - A modern web browser");
        println!("\nUsage: browser [OPTIONS] [URL]");
        println!("\nOptions:");
        println!("  -h, --help     Print help information");
        println!("  -v, --verbose  Enable verbose output");
        println!("\nArguments:");
        println!("  [URL]          URL to navigate to");
        println!("\nExamples:");
        println!("  browser https://example.com");
        println!("  browser --verbose https://example.com");
    }
}

fn navigate_to_url(_url: &str) -> Result<()> {
    // TODO: Implement actual URL loading and rendering
    // For now, return a not implemented error to demonstrate error handling
    Err(BrowserError::NotImplemented(
        "URL loading will be implemented soon".to_string(),
    ))
}
