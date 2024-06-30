#![allow(unused, dead_code, non_snake_case)]
mod args;
mod vm;
mod cpu;
use clap::Parser;
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    kernel_file: String,

    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    // #[command(subcommand)]
    // command: Option<Commands>,
}
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Cli::parse();
    let program = std::fs::read(args.kernel_file).unwrap();
    assert!(program.len()%2==0);
    vm::VM::new(program).run();
    Ok(())
}
