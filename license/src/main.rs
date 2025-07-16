use clap::Parser;
use license_gen_bin::{cli::Cli, io, license};

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
