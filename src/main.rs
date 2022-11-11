use common_failures::display::DisplayCausesAndBacktraceExt;
use common_failures::Result;
use env_logger::Env;
use structopt::StructOpt;

fn run() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let opts = docker_source_checksum::opts::Opts::from_args();

    let dockerfile_path = opts
        .dockerfile_path
        .clone()
        .unwrap_or_else(|| opts.context_path.join("Dockerfile"));

    let checksum = docker_source_checksum::hash(&dockerfile_path, opts)?;
    println!("{}", checksum);
    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e.display_causes_and_backtrace());
            std::process::exit(-2)
        }
    }
}
