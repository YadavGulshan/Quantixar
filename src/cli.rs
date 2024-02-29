use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[arg(long, value_name = "PATH")]
    pub config_path: Option<String>,
}
