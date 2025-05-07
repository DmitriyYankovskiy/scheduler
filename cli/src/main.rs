use {
    clap::{Parser, arg},
    indicatif::ProgressBar,
    scheduler::{
        Schedule,
        models::{Result, ScheduleModel, csv},
    },
    std::path::PathBuf,
};

fn validate_input_path(s: &str) -> std::result::Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if path.exists() {
        Ok(path)
    } else {
        Err("Path does not exist".to_string())
    }
}

fn validate_lambda_opt(s: &str) -> std::result::Result<f64, String> {
    let val = s.parse::<f64>().map_err(|e| format!("{e}"))?;
    if 0.0 < val && val < 1.0 {
        Ok(val)
    } else {
        Err("Value is not between 0..1".to_string())
    }
}

#[derive(Parser)]
struct Args {
    #[arg(
        value_parser = validate_input_path,
        help = "Input file (must exist)"
    )]
    input_path: PathBuf,

    #[arg(
        short,
        value_parser = clap::value_parser!(PathBuf),
        help = "Output file (can be non-existent)"
    )]
    output_path: PathBuf,

    #[arg(short, long, value_parser = validate_lambda_opt)]
    lamda_opt: Option<f64>,
    #[arg(short, long)]
    aging_opt: Option<usize>,
    #[arg(short, long)]
    shuffling: bool,
    #[arg(short, long)]
    greedily: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(&args.input_path)
        .unwrap();
    // let mut schedule: Schedule = serde_json::from_str::<ScheduleModel>(&file).unwrap().into();
    let mut schedule = Schedule::new(ScheduleModel::deserialize_csv(&mut reader)?.into());

    let aging = args.aging_opt.unwrap_or(scheduler::AGING_OPT_DEFAULT);

    let pb = ProgressBar::new(aging as u64);

    let time = std::time::Instant::now();

    schedule.optimize(
        args.lamda_opt.unwrap_or(scheduler::LAMBDA_OPT_DEFAULT),
        aging,
        args.shuffling,
        args.greedily,
        || pb.inc(1),
    );
    pb.finish();
    let dur = time.elapsed();
    println!("results cost: {}", schedule.cost);
    println!("calculation time: {}", dur.as_secs_f32());

    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_path(&args.output_path)
        .unwrap();
    ScheduleModel::from(schedule)
        .serialize_csv(&mut writer)
        .unwrap();
    Ok(())
}
