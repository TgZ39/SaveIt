use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Resets the config
    #[clap(long, action)]
    pub reset_config: bool,

    /// Resets the source database
    #[clap(long, action)]
    pub reset_database: bool,

    /// Set logging verbosity level
    #[clap(value_enum, long, default_value_t = VerbosityLevel::INFO)]
    pub verbosity: VerbosityLevel,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, clap::ValueEnum)]
pub enum VerbosityLevel {
    TRACE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
}
