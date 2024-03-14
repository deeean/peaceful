use clap::{Args, Parser, Subcommand};
use peaceful::{convert, resize};

#[derive(Debug, Parser)]
#[command(name = "peaceful", about = "A simple image processing tool")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Convert(ConvertArgs),
    Resize(ResizeArgs),
}

#[derive(Debug, Args)]
pub struct ConvertArgs {
    #[arg(short, long)]
    input: String,
    #[arg(short, long)]
    output: String,
    #[arg(short, long)]
    format: String,
}

#[derive(Debug, Args)]
pub struct ResizeArgs {
    #[arg(short, long)]
    input: String,
    #[arg(short, long)]
    output: String,
    #[arg(short, long)]
    size: String,
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Command::Convert(args) => convert(
            args.input.as_str(),
            args.output.as_str(),
            args.format.as_str()
        ),
        Command::Resize(args) => resize(
            args.input.as_str(),
            args.output.as_str(),
            args.size.as_str()
        )
    }
}
