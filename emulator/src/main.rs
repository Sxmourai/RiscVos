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


fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    let args = Cli::parse();
    let program = std::fs::read(args.kernel_file)?;
    assert!(program.len()%2==0);
    emulator::vm::run(program)?;
    Ok(())
}
