use clap::Parser;

#[derive(Parser, Debug)]
pub(crate) struct Args {
    #[clap(short, long)]
    pub config: String,

    #[clap(short, long, default_value = "false")]
    pub stdout: bool,
}
