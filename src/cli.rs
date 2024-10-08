use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    /// Log level:
    /// 0 quiet,
    /// 1 errors,
    /// 2 warnings,
    /// 3 info,
    #[clap(short, long)]
    #[clap(default_value_t = 2)]
    pub loglevel: u8,
    // /// Valume level in percetage
    // #[clap(short, long)]
    // #[clap(default_value_t = 100)]
    // pub volume: u8,
    //
    // #[clap(short, long)]
    // pub add_path: bool,
    //
    // pub opus_file: String,
}
