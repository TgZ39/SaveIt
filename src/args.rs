use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Resets the config
    #[clap(long, action)]
    pub reset_config: bool,

    /// Reset the source database
    #[clap(long, action)]
    pub reset_database: bool,
}
