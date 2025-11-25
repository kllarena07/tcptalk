use clap::Parser;

#[derive(Parser)]
#[command(name = "tailtalk")]
#[command(about = "A TUI chat client")]
pub struct Args {
    #[arg(short, long)]
    pub username: String,

    #[arg(short, long, default_value = "127.0.0.1")]
    pub ip: String,
}
