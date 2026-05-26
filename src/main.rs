mod error;
mod html;
mod network;
mod renderer;
mod url;
mod window;

use clap::Parser;
use error::Result;
use html::HtmlDocument;
use network::HttpClient;
use url::BrowserUrl;
use window::run_browser_window;

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

    /// Launch GUI version of the browser
    #[arg(long)]
    gui: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.gui {
        // Launch GUI version
        if let Err(e) = run_browser_window() {
            eprintln!("Error launching browser window: {}", e);
            std::process::exit(1);
        }
    } else if let Some(url) = cli.url {
        // CLI version
        if cli.verbose {
            println!("Navigating to: {}", url);
        }
        match navigate_to_url(&url, cli.verbose) {
            Ok(_) => println!("Successfully loaded: {}", url),
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
        println!("  --gui          Launch GUI version");
        println!("\nArguments:");
        println!("  [URL]          URL to navigate to");
        println!("\nExamples:");
        println!("  browser https://example.com");
        println!("  browser --verbose https://example.com");
        println!("  browser --gui");
    }
}

/// Navigate to a URL: validate URL, fetch content, and parse HTML
///
/// This function demonstrates the full flow:
/// 1. URL validation and parsing
/// 2. Network request to fetch content
/// 3. HTML parsing to extract structure
fn navigate_to_url(url_str: &str, verbose: bool) -> Result<()> {
    // Step 1: Validate and parse the URL
    if verbose {
        println!("Step 1: Validating URL...");
    }
    let url = BrowserUrl::parse(url_str)?;
    if verbose {
        println!("  ✓ URL is valid");
        println!("  Scheme: {}", url.scheme());
        println!("  Host: {}", url.host());
        println!("  Secure: {}", url.is_secure());
        println!("  Path: {}", url.path());
        println!("  Original: {}", url.as_str());
        if let Some(port) = url.port() {
            println!("  Port: {}", port);
        }
        if let Some(query) = url.query() {
            println!("  Query: {}", query);
        }
        if let Some(fragment) = url.fragment() {
            println!("  Fragment: {}", fragment);
        }
        let _ = url.as_url();
    }

    // Step 2: Fetch the content using the network stack
    if verbose {
        println!("\nStep 2: Fetching content...");
    }
    let client = HttpClient::new()?;
    if verbose {
        println!("  Timeout: {:?}", client.timeout());
    }
    let html_content = client.get(url_str)?;
    if verbose {
        println!("  ✓ Fetched {} bytes", html_content.len());
    }

    // Step 3: Parse the HTML
    if verbose {
        println!("\nStep 3: Parsing HTML...");
    }
    let document = HtmlDocument::parse(&html_content)?;
    if verbose {
        println!("  ✓ HTML parsed successfully");
        println!("  Document size: {} bytes", document.len());
        println!("  Is empty: {}", document.is_empty());
    }

    // Display extracted information
    println!("\n=== Page Information ===");

    if let Some(title) = document.title() {
        println!("Title: {}", title);
    } else {
        println!("Title: (none)");
    }

    let links = document.links();
    println!("Links found: {}", links.len());
    if verbose && !links.is_empty() {
        println!("  First few links:");
        for link in links.iter().take(5) {
            println!("    - {}", link);
        }
    }

    let images = document.images();
    println!("Images found: {}", images.len());
    if verbose && !images.is_empty() {
        println!("  First few images:");
        for img in images.iter().take(5) {
            println!("    - {}", img);
        }
    }

    let headings = document.headings();
    println!("Headings found: {}", headings.len());
    if verbose && !headings.is_empty() {
        println!("  Headings:");
        for (level, text) in headings.iter().take(5) {
            println!("    H{}: {}", level, text);
        }
    }

    let text = document.text_content();
    let preview: String = text.chars().take(200).collect();
    println!(
        "\nText preview: {}{}",
        preview,
        if text.len() > 200 { "..." } else { "" }
    );

    if verbose {
        let html_preview: String = document.as_html().chars().take(300).collect();
        println!(
            "\nHTML preview: {}{}",
            html_preview,
            if document.as_html().len() > 300 {
                "..."
            } else {
                ""
            }
        );
    }

    Ok(())
}
