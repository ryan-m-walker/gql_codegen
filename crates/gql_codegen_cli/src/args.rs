use clap::Parser;

#[derive(Parser, Debug)]
pub(crate) struct Args {
    #[clap(short, long)]
    pub config: String,
}
