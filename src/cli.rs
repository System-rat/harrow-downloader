use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct CliArgs {
    #[arg(long)]
    pub api_delay: Option<u32>,
    #[arg(long)]
    pub data_directory: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub skip_db_regen: bool,
    #[arg(long, default_value_t = false)]
    pub skip_metadata_files: bool,
    #[arg(long, default_value_t = false)]
    pub skip_generators: bool,
    #[arg(long, default_value_t = false)]
    pub clean_data_directory: bool,
}
