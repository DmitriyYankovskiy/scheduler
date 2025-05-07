use {
    clap::{Parser, arg},
    std::{
        fs::File,
        io::{Result, Write},
        path::PathBuf,
    },
};

#[derive(Parser)]
struct Args {
    n: usize,

    #[arg(
        short,
        value_parser = clap::value_parser!(PathBuf),
        help = "Output file (can be non-existent)"
    )]
    output_path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut file = File::create(args.output_path).unwrap();

    let mut s = String::new();
    for l in 0..args.n {
        for i in 1..=args.n {
            s += &format!("{i}:{i},");
        }
        s.pop();
        s.push('\n');
    }

    file.write_all(s.as_bytes()).unwrap();
    Ok(())
}
