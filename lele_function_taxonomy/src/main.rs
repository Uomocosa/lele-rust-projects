use std::path::PathBuf;

use clap::Parser;

use lele_function_taxonomy::driver;

#[derive(Parser, Debug)]
#[command(
    name = "lele_function_taxonomy",
    about = "Precise function honesty taxonomy via rustc TyCtxt"
)]
struct Args {
    #[arg(long)]
    manifest_path: Option<PathBuf>,
    #[arg(long, default_value_t = 1)]
    honesty_depth: usize,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let code = driver::run(args.manifest_path, Some(args.honesty_depth));
    std::process::exit(code);
}
