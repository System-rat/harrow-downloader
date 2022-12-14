use std::path::PathBuf;

use clap::Parser;

/// A twitter utility for downloading/archiving personal likes/bookrmarks
///
/// During the first run you will be asked to login with your credentials
/// after which the credentials will be stored for re-use.
///
/// The downloader will go through each liked/bookmarked tweet and will
/// download their respective media content (video/image) with the highest
/// possible resolution available. Optionally (enabled by default) a `.txt`
/// metadata file will be generated for every post specifying the post's text,
/// author, and filenames for all the related media.
///
/// After all the data, various other organizational folders will be generated
/// with symlinks for ease-of-browsing. (optional, enabled by default)
///
/// WARNING: the access token is stored in a plain-text JSON file inside
/// the data directory, make sure you delete it if you're feeling security
/// conscious.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
pub struct CliArgs {
    /// Adds a delay to each tweet listing request
    #[arg(long, short = 't')]
    pub api_delay: Option<u32>,
    /// Change the default data directory where harrow-downloader
    /// stores it's data and configuration. By defaults it stores data
    /// within the `.local/share/harrow-downloader/' folder on Linux,
    /// `AppData/Roaming/harrow-downloader/` on Windows, and `Library/Application
    /// Support/harrow-downloader/`
    /// on macOS. All of these are within the user's system-specified HOME
    /// directory
    #[arg(long, short = 'd')]
    pub data_directory: Option<PathBuf>,
    /// Skip the regeneration of the internal tweet database, this will
    /// prevent the fetching of new tweets
    #[arg(long, short = 'r')]
    pub skip_db_regen: bool,
    /// Do not generate the additional .txt metadata files for each post
    #[arg(long, short = 'm')]
    pub skip_metadata_files: bool,
    /// Skip the generation of symlink organizational folders
    #[arg(long, short = 'g')]
    pub skip_generators: bool,
    /// Clean the entire data directory before downloading files
    #[arg(long, short = 'c')]
    pub clean_data_directory: bool,
}
