use assert_cmd::Command as AssertCommand;
use assert_cmd::cargo::cargo_bin;
use assert_fs::prelude::*;
use predicates::prelude::*;
use rexpect::reader::Options;
use rexpect::session::spawn_with_options;
use std::env;
use std::path::Path;
use std::process::Command as StdCommand;

// TODO: Write tests for all license types:
// - [x] Apache-2.0
// - [x] MIT
// - [x] BSD-3-Clause
// - [x] BSD-3-Clause-Modification
// - [x] BSD-3-Clause-No-Military-License
// - [x] BSD-3-Clause-Attribution
// - [x] AGPL-3.0
// - [x] GPL-3.0
// - [x] LGPL-3.0
// - [x] MPL-2.0
// - [x] Unlicense
// - [x] CDDL-1.0
// - [ ] EPL-2.0
// - [x] BSL-1.0

#[test]
fn test_cli_help() {
    AssertCommand::new(cargo_bin!("license"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_cli_version() {
    AssertCommand::new(cargo_bin!("license"))
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("license"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

fn setup_test_env() -> assert_fs::TempDir {
    let temp = assert_fs::TempDir::new().unwrap();
    let py_file = temp.child("python/__init__.py");
    py_file
        .write_file(Path::new("tests/data/__init__.py"))
        .unwrap();
    let rs_file = temp.child("rust/main.rs");
    rs_file.write_file(Path::new("tests/data/main.rs")).unwrap();
    temp
}

const BASIC_LICENSES: [&str; 5] = [
    "BSD-3-Clause",
    "BSD-3-Clause-Modification",
    "BSD-3-Clause-No-Military-License",
    "Apache-2.0",
    "MIT",
];

#[derive(Debug, Clone, Copy)]
enum Lang {
    Python,
    Rust,
}

fn gen_cmd<P: AsRef<std::path::Path>>(lang: Lang, path: P, license: &str) -> StdCommand {
    let mut cmd = StdCommand::new(cargo_bin!("license"));
    cmd.current_dir(path)
        .arg("--source-path")
        .arg(match lang {
            Lang::Python => "python/",
            Lang::Rust => "rust/",
        })
        .arg("--add-comment")
        .arg("--comment")
        .arg(match lang {
            Lang::Python => "#",
            Lang::Rust => "//",
        })
        .arg("--output")
        .arg(match lang {
            Lang::Python => "LICENSE.python.txt",
            Lang::Rust => "LICENSE.rust.txt",
        })
        .arg(license);
    cmd
}

fn gen_assert_cmd<P: AsRef<std::path::Path>>(lang: Lang, path: P, license: &str) -> AssertCommand {
    let mut cmd = AssertCommand::new(cargo_bin!("license"));
    cmd.current_dir(path)
        .arg("--source-path")
        .arg(match lang {
            Lang::Python => "python/",
            Lang::Rust => "rust/",
        })
        .arg("--add-comment")
        .arg("--comment")
        .arg(match lang {
            Lang::Python => "#",
            Lang::Rust => "//",
        })
        .arg("--output")
        .arg(match lang {
            Lang::Python => "LICENSE.python.txt",
            Lang::Rust => "LICENSE.rust.txt",
        })
        .arg(license);
    cmd
}

fn spawn_session(cmd: StdCommand) -> rexpect::session::PtySession {
    let session = spawn_with_options(
        cmd,
        Options {
            timeout_ms: Some(115),
            strip_ansi_escape_codes: true,
        },
    )
    .unwrap();
    session
}

fn basic_interact(session: &mut rexpect::session::PtySession) {
    session.exp_string("Enter the copyright year:").unwrap();
    session.send_line("2025").unwrap();
    session
        .exp_string("Enter the full name of the copyright holder:")
        .unwrap();
    session.send_line("Your Name").unwrap();
    session.exp_eof().unwrap();
}

fn run_basic_gen(lang: Lang, license: &str) -> assert_fs::TempDir {
    let temp = setup_test_env();
    let cmd = gen_cmd(lang, temp.path(), license);
    let mut session = spawn_session(cmd);
    basic_interact(&mut session);
    temp
}

#[derive(Debug, Clone, Copy)]
enum Location<'a> {
    License(&'a str),
    Source(&'a str),
}

fn assert_files(
    temp: &assert_fs::TempDir,
    lang: Lang,
    license: &str,
    copyright: Option<Location<'_>>,
) -> (assert_fs::fixture::ChildPath, assert_fs::fixture::ChildPath) {
    let license_file = match lang {
        Lang::Python => temp.child("LICENSE.python.txt"),
        Lang::Rust => temp.child("LICENSE.rust.txt"),
    };
    license_file
        .assert(predicate::path::exists())
        .assert(predicate::path::is_file());
    let source_file = match lang {
        Lang::Python => temp.child("python/__init__.py"),
        Lang::Rust => temp.child("rust/main.rs"),
    };
    let comment = match lang {
        Lang::Python => "#",
        Lang::Rust => "//",
    };
    source_file
        .assert(predicate::path::exists())
        .assert(predicate::path::is_file())
        .assert(predicate::str::contains(format!(
            "{comment} SPDX-License-Identifier: {license}"
        )));
    if let Some(copyright) = copyright {
        match copyright {
            Location::License(copyright) => {
                license_file.assert(predicate::str::contains(copyright));
            }
            Location::Source(copyright) => {
                source_file.assert(predicate::str::contains(format!("{comment} {copyright}")));
            }
        }
    }
    (license_file, source_file)
}

fn test_basic_licenses(lang: Lang) {
    for license in BASIC_LICENSES.iter() {
        let temp = run_basic_gen(lang, license);
        let copyright = if license == &"Apache-2.0" {
            Location::Source("Copyright 2025 Your Name")
        } else {
            Location::License("Copyright (c) 2025 Your Name")
        };
        let (license_file, _) = assert_files(&temp, lang, license, Some(copyright));
        if license == &"Apache-2.0" {
            license_file.assert(predicate::str::contains("Apache License"));
        }
        temp.close().unwrap();
    }
}

#[test]
fn test_basic_licenses_python() {
    test_basic_licenses(Lang::Python);
}

#[test]
fn test_basic_licenses_rust() {
    test_basic_licenses(Lang::Rust);
}

const GNU_LICENSES: [&str; 9] = [
    "AGPL-3.0",
    "AGPL-3.0-only",
    "AGPL-3.0-or-later",
    "GPL-3.0",
    "GPL-3.0-only",
    "GPL-3.0-or-later",
    "LGPL-3.0",
    "LGPL-3.0-only",
    "LGPL-3.0-or-later",
];

fn run_gnu_gen(lang: Lang, license: &str) -> assert_fs::TempDir {
    let temp = setup_test_env();
    let cmd = gen_cmd(lang, temp.path(), license);
    let mut session = spawn_session(cmd);
    session.exp_string("Enter the copyright year:").unwrap();
    session.send_line("2025").unwrap();
    session
        .exp_string("Enter the full name of the copyright holder:")
        .unwrap();
    session.send_line("Your Name").unwrap();
    session
        .exp_string("Enter the name of the program:")
        .unwrap();
    session.send_line("license").unwrap();
    if !license.contains("LGPL") {
        session
            .exp_string("Enter the version of the program (optional):")
            .unwrap();
        session.send_line("1.0.0").unwrap();
    }
    session
        .exp_string("Enter a short description of the program (5-10 words):")
        .unwrap();
    session.send_line("A tool for managing licenses").unwrap();
    if !license.contains("LGPL") {
        session
            .exp_string(
                "Is this program interactive? (e.g., a website, CLI tool, etc.) ([y]es/[n]o):",
            )
            .unwrap();
        session.send_line("y").unwrap();
    }
    session.exp_string("Do you need a signed release for this software? (e.g., for an organization) ([y]es/[n]o):").unwrap();
    session.send_line("yes").unwrap();
    session
        .exp_string("Enter the name of the organization:")
        .unwrap();
    session.send_line("ACME, Inc.").unwrap();
    session
        .exp_string("Enter the name of the signer from the organization:")
        .unwrap();
    session.send_line("Road Runner").unwrap();
    session
        .exp_string("Enter the position within the organization of the signer:")
        .unwrap();
    session.send_line("The Boss").unwrap();
    session.exp_string("Enter the day of the signing:").unwrap();
    session.send_line("7").unwrap();
    session
        .exp_string("Enter the month of the signing:")
        .unwrap();
    session.send_line("April").unwrap();
    session
        .exp_string("Enter the year of the signing:")
        .unwrap();
    session.send_line("2025").unwrap();

    // Expected stdout output
    session
        .exp_string("You'll need to include the following amendment to your license.")
        .unwrap();
    session
        .exp_string("ACME, Inc., hereby disclaims all copyright interest")
        .unwrap();
    session.exp_string("Road Runner, The Boss").unwrap();
    if !license.contains("LGPL") {
        session
            .exp_string("Since your program is interactive, you should")
            .unwrap();
        session
            .exp_string("license version 1.0.0, Copyright (C) 2025 Your Name")
            .unwrap();
        session
            .exp_string("license comes with ABSOLUTELY NO WARRANTY.")
            .unwrap();
    }
    session.exp_eof().unwrap();
    temp
}

fn test_gnu(lang: Lang, license: &str) {
    let temp = run_gnu_gen(lang, license);
    let (_, _) = assert_files(
        &temp,
        lang,
        license,
        Some(Location::Source("Copyright (C) 2025 Your Name")),
    );
    temp.close().unwrap();
}

#[test]
fn test_gnu_licenses_python() {
    for license in GNU_LICENSES.iter() {
        test_gnu(Lang::Python, license);
    }
}

#[test]
fn test_gnu_licenses_rust() {
    for license in GNU_LICENSES.iter() {
        test_gnu(Lang::Rust, license);
    }
}

const PLAIN_LICENSES: [&str; 4] = ["MPL-2.0", "CDDL-1.0", "Unlicense", "BSL-1.0"];

fn test_plain(lang: Lang, license: &str) {
    let temp = setup_test_env();
    gen_assert_cmd(lang, temp.path(), license)
        .assert()
        .success();

    assert_files(&temp, lang, license, None);
    temp.close().unwrap();
}

#[test]
fn test_plain_licenses_python() {
    for license in PLAIN_LICENSES.iter() {
        test_plain(Lang::Python, license);
    }
}

#[test]
fn test_plain_licenses_rust() {
    for license in PLAIN_LICENSES.iter() {
        test_plain(Lang::Rust, license);
    }
}

fn run_attribution_gen(lang: Lang) -> assert_fs::TempDir {
    let temp = setup_test_env();
    let cmd = gen_cmd(lang, temp.path(), "BSD-3-Clause-Attribution");
    let mut session = spawn_session(cmd);
    session.exp_string("Enter the copyright year:").unwrap();
    session.send_line("2025").unwrap();
    session
        .exp_string("Enter the full name of the copyright holder:")
        .unwrap();
    session.send_line("Your Name").unwrap();
    session
        .exp_string("Enter the name of the organization (optional):")
        .unwrap();
    session.send_line("ACME, Inc.").unwrap();
    session
        .exp_string("Enter the website of the organization (optional):")
        .unwrap();
    session.send_line("https://acme.example.com").unwrap();
    session.exp_eof().unwrap();

    temp
}

fn test_attribution(lang: Lang) {
    let temp = run_attribution_gen(lang);
    assert_files(&temp, lang, "BSD-3-Clause-Attribution", None);
    temp.close().unwrap();
}

#[test]
fn test_bsd_attribution_license_python() {
    test_attribution(Lang::Python);
}

#[test]
fn test_bsd_attribution_license_rust() {
    test_attribution(Lang::Rust);
}

const EPL_LICENSES: [&str; 13] = [
    "Apache-2.0",
    "MIT",
    "BSD-3-Clause",
    "BSD-3-Clause-Modification",
    "BSD-3-Clause-No-Military-License",
    "BSD-3-Clause-Attribution",
    "AGPL-3.0-or-later",
    "GPL-3.0-or-later",
    "LGPL-3.0-or-later",
    "MPL-2.0",
    "Unlicense",
    "CDDL-1.0",
    "BSL-1.0",
];

fn run_epl_gen(lang: Lang) -> assert_fs::TempDir {
    let temp = setup_test_env();
    let cmd = gen_cmd(lang, temp.path(), "EPL-2.0");
    let mut session = spawn_session(cmd);
    session
        .exp_string("Enter the secondary licenses that are permitted (comma separated) (optional):")
        .unwrap();
    session.send_line(&EPL_LICENSES.join(", ")).unwrap();
    session
        .exp_string("You'll need to include the following amendment to your license.")
        .unwrap();
    session
        .exp_string("This Source Code may also be made available")
        .unwrap();
    for license in EPL_LICENSES.iter() {
        session.exp_string(&format!("- {license}")).unwrap();
    }
    session.exp_eof().unwrap();

    temp
}

fn test_epl(lang: Lang) {
    let temp = run_epl_gen(lang);
    assert_files(&temp, lang, "EPL-2.0", None);
    temp.close().unwrap();
}

#[test]
fn test_epl_license_python() {
    test_epl(Lang::Python);
}

#[test]
fn test_epl_license_rust() {
    test_epl(Lang::Rust);
}
