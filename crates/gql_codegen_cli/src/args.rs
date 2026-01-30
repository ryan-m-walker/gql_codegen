use clap::Parser;
use gql_codegen_logger::LogLevel;

#[derive(Parser, Debug)]
pub(crate) struct Args {
    #[clap(short, long)]
    pub config: String,

    #[clap(short, long, default_value = "false")]
    pub stdout: bool,
}
