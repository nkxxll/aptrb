use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cmd {
    #[arg(help = "The packages that should be installed and rolled back later")]
    command: Vec<String>,
    #[arg(
        default_value = "$HOME/.local/share/aptrb/rb.toml",
        help = "Optional flag that indicates a specific file to rollback from or to store rollback data",
        short,
        long
    )]
    file: Option<String>,
    #[arg(short, long, help = "If set the rollback command is executed")]
    rollback: bool,
}

fn main() {
    let args = Cmd::parse();
    dbg!(args);
    println!("Hello, world!");
}
