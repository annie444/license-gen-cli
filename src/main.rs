use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Parser, value_parser};
use clap_verbosity_flag::Verbosity;
use std::path::PathBuf;

pub mod io;
pub mod license;
pub mod texts;

pub const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default())
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD));

/// Command line interface for generating license texts.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, styles = STYLES)]
pub struct Cli {
    /// Whether to add the license comment headers to the
    /// source files. If this is not set, the program will
    /// only print the license comment header to the console.
    #[arg(short = 'c', long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub add_comment: bool,

    /// How to denote comments in the license header comment.
    /// (e.g., `//` in rust vs. `#` in python).
    #[arg(long, default_value = "//")]
    pub comment: String,

    /// The path to the source files to add the license headers to.
    /// If '--add-comment' is set:
    /// (A) and this points to a file, the license header will be added
    /// to that file.
    /// (B) and this points to a directory, the license header will be added
    /// to all files in that directory recursively.
    #[arg(short, long, default_value = "src")]
    pub source_path: PathBuf,

    /// The output file to write the license text to.
    #[arg(short, long, default_value = "LICENSE.txt")]
    pub output: PathBuf,

    #[command(flatten)]
    pub verbosity: Verbosity,

    /// The license to generate text for.
    #[arg(value_parser = value_parser!(license::Licenses))]
    pub license: license::Licenses,
}

fn main() {
    let cli = Cli::parse();
    let Cli {
        add_comment,
        comment,
        source_path,
        output,
        license,
        verbosity,
    } = cli;

    tracing_subscriber::fmt().with_max_level(verbosity).init();

    // Here you would typically call a function to handle the CLI arguments,
    // such as generating the license text based on the provided options.
    // For example:
    let text = license::generate_license_text(&license);
    io::output(&text, add_comment, &comment, source_path, output);
}
