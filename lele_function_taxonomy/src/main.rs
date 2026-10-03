use std::path::PathBuf;

use clap::Parser;

use lele_function_taxonomy::run;

#[derive(Parser, Debug)]
#[command(
    name = "lele-function-taxonomy",
    about = "Checks that functions in require=honest boundary folders are honest (rustc MIR)"
)]
struct Args {
    #[arg(long)]
    manifest_path: Option<PathBuf>,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    std::process::exit(run(args.manifest_path));
}
