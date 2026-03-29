use clap::Parser;

/// Kiki's Courier (Delivery sounds better) Service
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    // Just a placeholder for now.
}

fn main() {
    let _cli = Cli::parse();
    todo!("CLI to be implemented")
}
