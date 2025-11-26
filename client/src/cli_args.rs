use clap::Parser;

#[derive(Parser)]
#[command(name = "tailtalk")]
#[command(about = "A TUI chat client")]
pub struct Args {
    #[arg(index = 1)]
    pub username: String,

    #[arg(index = 2, default_value = "0.0.0.0")]
    pub ip: String,

    #[arg(short = 'p', long, default_value = "2133")]
    pub port: u16,
}
