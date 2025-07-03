use crate::texts::LicenseTexts;
use color_print::{ceprintln, cformat, cprint, cprintln};
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::str::FromStr;
use tempfile::NamedTempFile;

#[tracing::instrument]
pub fn output(
    license: &LicenseTexts,
    add_comment: bool,
    comment: &str,
    source_path: PathBuf,
    output: PathBuf,
) {
    match (
        add_comment,
        source_path.exists(),
        source_path.is_dir(),
        source_path.is_file(),
    ) {
        (true, true, true, _) => iterate_dir(&source_path, comment, &license.comment),
        (true, true, _, true) => {
            if let Err(e) = write_comment(comment, &license.comment, &source_path) {
                ceprintln!(
                    "<bold><red>Failed to write comment for file {}</></>: {}",
                    source_path.display(),
                    e
                );
                return;
            }
        }
        (true, true, false, false) => {
            ceprintln!(
                "<bold><red>Source path is neither a file nor a directory</></>: {}",
                source_path.display()
            );
            return;
        }
        (true, false, _, _) => {
            ceprintln!(
                "<bold><red>Source path does not exist</></>: {}",
                source_path.display()
            );
            return;
        }
        (false, _, _, _) => {
            cprintln!(
                "<bold><magenta>Add this as a comment to the top of your source file(s):</></>\n"
            );
            println!("{}", license.comment);
        }
    };

    let mut license_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&output)
    {
        Ok(file) => file,
        Err(e) => {
            ceprintln!(
                "<bold><red>Failed to open license file {}</></>: {}",
                output.display(),
                e
            );
            return;
        }
    };

    match license_file.write_all(license.text.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            ceprintln!(
                "<bold><red>Failed to write license text to {}</></>: {}",
                output.display(),
                e
            );
            return;
        }
    };

    if let Err(e) = license_file.flush() {
        ceprintln!(
            "<bold><red>Failed to flush license file {}</></>: {}",
            output.display(),
            e
        );
        return;
    };

    if let Some(alt) = &license.alt {
        cprintln!(
            r#"<magenta><bold>
You'll need to include the following amendment to your license.</> 
This is usually added to the end of the license file, but there is no strict requirement 
for where it goes. Another common place is to add it as a comment at the top of your source 
files or to the readme.</>
"#,
        );
        println!("{}", alt);
    };

    if let Some(interactive) = &license.interactive {
        cprintln!(
            r#"<magenta><bold>
Since your program is interactive, you should also include the following notice in your program's output.</>
This needs to be easily accessible to users, such as in a help command, at the start of the program, in 
a footer section, or in an about section.</>
"#,
        );
        println!("{}", interactive);
    };
}

#[tracing::instrument]
fn write_comment<P: AsRef<Path> + std::fmt::Debug>(
    comment: &str,
    comment_block: &str,
    output_file: P,
) -> io::Result<()> {
    let tmp_path = NamedTempFile::new()?.into_temp_path();
    let mut tmp_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&tmp_path)?;
    for line in comment_block.lines() {
        writeln!(tmp_file, "{} {}", comment, line)?;
    }
    let src = OpenOptions::new().read(true).open(&output_file)?;
    for line in io::BufReader::new(src).lines() {
        writeln!(tmp_file, "{}", line?)?;
    }
    tmp_file.flush()?;
    fs::remove_file(&output_file)?;
    fs::rename(tmp_path, output_file)?;
    Ok(())
}

#[tracing::instrument]
fn iterate_dir<P: AsRef<Path> + std::fmt::Debug>(
    path: P,
    comment: &str,
    comment_block: &str,
) -> () {
    let files = match path.as_ref().read_dir() {
        Ok(files) => files,
        Err(e) => {
            ceprintln!(
                "<bold><red>Failed to read directory {}</></>: {}",
                path.as_ref().display(),
                e
            );
            return;
        }
    };
    for file in files {
        match file {
            Ok(entry) => {
                if let Err(e) = write_comment(comment, comment_block, entry.path()) {
                    ceprintln!(
                        "<bold><red>Failed to write comment for file {}</></>: {}",
                        entry.path().display(),
                        e
                    );
                    return;
                }
            }
            Err(e) => {
                ceprintln!(
                    "<bold><red>Failed to read entry in directory {}</></>: {}",
                    path.as_ref().display(),
                    e
                );
                return;
            }
        }
    }
}

#[tracing::instrument]
pub fn prompt<T>(q: &str) -> T
where
    T: FromStr,
{
    loop {
        cprint!("<bold><cyan>{}</></>: ", q);
        match io::stdout().flush() {
            Ok(_) => (),
            Err(e) => {
                ceprintln!("<bold><red>Failed to flush stdout</></>: {}", e);
                process::exit(1);
            }
        };

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(e) => {
                ceprintln!("<bold><red>Failed to read line</></>: {}", e);
                process::exit(1);
            }
        }

        let trimmed_input = input.trim();

        match trimmed_input.parse::<T>() {
            Ok(value) => return value,
            Err(_) => {
                ceprintln!("<bold><yellow>Invalid input</></>: {}.", trimmed_input);
                ceprintln!("<bold><yellow>Please try again.</></>");
            }
        }
    }
}

#[tracing::instrument]
pub fn prompt_optional<T>(q: &str) -> Option<T>
where
    T: FromStr,
{
    loop {
        cprint!("<bold><cyan>{}</></> <dim>(<italics>optional</>)</>: ", q);
        match io::stdout().flush() {
            Ok(_) => (),
            Err(e) => {
                ceprintln!("<bold><red>Failed to flush stdout</></>: {}", e);
                process::exit(1);
            }
        };

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(e) => {
                ceprintln!("<bold><red>Failed to read line</></>: {}", e);
                process::exit(1);
            }
        }

        let trimmed_input = input.trim();

        if trimmed_input.is_empty() {
            return None;
        } else {
            match trimmed_input.parse::<T>() {
                Ok(value) => return Some(value),
                Err(_) => {
                    ceprintln!("<bold><yellow>Invalid input</></>: {}.", trimmed_input);
                    ceprintln!("<bold><yellow>Please try again or leave blank for none.</></>");
                }
            }
        }
    }
}

#[tracing::instrument]
pub fn prompt_bool(q: &str) -> bool {
    loop {
        let response = prompt::<String>(&cformat!(
            "{} <dim>(<italics>[<bold>y</bold>]es</italics>/<italics>[<bold>n</bold>]o</italics>)</dim>",
            q
        ));
        match response.to_lowercase().as_str() {
            "yes" | "y" | "true" | "t" => return true,
            "no" | "n" | "false" | "f" => return false,
            _ => ceprintln!(
                "<bold><yellow>Please answer '<italics>yes</>' or '<italics>no</>'.</></>"
            ),
        }
    }
}

#[tracing::instrument]
pub fn prompt_optional_bool(q: &str) -> Option<bool> {
    loop {
        let response = prompt_optional::<String>(&cformat!(
            "{} <dim>(<italics>[<bold>y</bold>]es</italics>/<italics>[<bold>n</bold>]o</italics>)</dim>",
            q
        ));
        match response {
            Some(r) => match r.to_lowercase().as_str() {
                "yes" | "y" | "true" | "t" => return Some(true),
                "no" | "n" | "false" | "f" => return Some(false),
                "" => return None,
                _ => ceprintln!(
                    "<bold><yellow>Please answer '<italics>yes</>', '<italics>no</>', or <italics>leave blank</> for none.</></>"
                ),
            },
            None => return None,
        }
    }
}
