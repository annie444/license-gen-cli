use clap::{Args, CommandFactory, Parser, Subcommand};
use color_print::cprintln;
use devx_cmd::{cmd, run};
use devx_pre_commit::{PreCommitContext, locate_project_root};
use license_gen_bin::cli::Cli;
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::env;
use std::ffi::OsStr;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::sync::Mutex;
use tar::{Builder, EntryType, Header, HeaderMode};
use tracing::{error, info, instrument, warn};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use zstd::bulk::Compressor;

#[derive(Parser, Debug, Clone)]
struct XtaskCli {
    #[command(subcommand)]
    pub cmd: Option<XtaskCmds>,
}

#[derive(Subcommand, Debug, Clone)]
enum XtaskCmds {
    Dist(OptOutDir),
    Man(OptOutDir),
    Completion(OptOutDir),
    Clean(OptOutDirWithAll),
    InstallPreCommitHook,
}

static PROJECT_ROOT: LazyLock<PathBuf> = LazyLock::new(|| {
    locate_project_root().unwrap_or_else(|e| {
        error!("Failed to locate project root: {}", e);
        std::process::exit(1);
    })
});
static OUT_ENV: LazyLock<String> = LazyLock::new(|| {
    env::var("OUT_DIR").unwrap_or_else(|e| {
        error!(
            "OUT_DIR environment variable is not set. Received error: {}",
            e
        );
        PROJECT_ROOT.join("target").to_string_lossy().to_string()
    })
});

#[derive(Args, Debug, Clone)]
struct OptOutDir {
    /// The output directory for generated files.
    #[arg(long, default_value = OUT_ENV.clone())]
    out_dir: PathBuf,
}

#[derive(Args, Debug, Clone)]
struct OptOutDirWithAll {
    /// The output directory for generated files.
    #[arg(long, default_value = OUT_ENV.clone())]
    out_dir: PathBuf,

    /// Clean all generated files, including archives.
    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    all: bool,
}

#[derive(Debug)]
struct Status<'a> {
    succeeded: RefCell<Vec<&'a str>>,
    failed: RefCell<Vec<&'a str>>,
}

impl<'a> Status<'a> {
    pub fn new() -> Self {
        Self {
            succeeded: RefCell::new(Vec::new()),
            failed: RefCell::new(Vec::new()),
        }
    }

    pub fn success(&self, target: &'a str) {
        self.succeeded.borrow_mut().push(target);
    }

    pub fn fail(&self, target: &'a str) {
        self.failed.borrow_mut().push(target);
    }

    pub fn ok(&self) -> bool {
        self.failed.borrow().is_empty()
    }

    pub fn report(&self) {
        println!();
        cprintln!("<bold>Build succeeded for targets:</>");
        for target in self.succeeded.borrow().iter() {
            cprintln!("  <green>{}</>", target);
        }
        cprintln!("<bold>Build failed for targets:</>");
        for target in self.failed.borrow().iter() {
            cprintln!("  <red>{}</red>", target);
        }
    }
}

static STATUS: LazyLock<Mutex<Status<'static>>> = LazyLock::new(|| Mutex::new(Status::new()));

type XtaskError = Box<dyn std::error::Error + Send + Sync + 'static>;
type XtaskResult<T> = Result<T, XtaskError>;

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    ctrlc::set_handler(move || {
        STATUS.lock().unwrap().report();
        std::process::exit(2);
    })
    .expect("Failed to set Ctrl-C handler");

    let cli = XtaskCli::parse();
    match cli.cmd {
        Some(cmd) => match cmd {
            XtaskCmds::Dist(out) => {
                info!("Building distribution...");
                dist(out.out_dir);
            }
            XtaskCmds::Man(out) => {
                info!("Building manpages...");
                if let Err(e) = build_manpages(out.out_dir) {
                    warn!("Failed to build manpages: {}", e);
                    STATUS.lock().unwrap().fail("manpages");
                } else {
                    STATUS.lock().unwrap().success("manpages");
                }
            }
            XtaskCmds::Completion(out) => {
                info!("Building completions...");
                if let Err(e) = build_completion(out.out_dir) {
                    warn!("Failed to build completions: {}", e);
                    STATUS.lock().unwrap().fail("completion");
                } else {
                    STATUS.lock().unwrap().success("completion");
                }
            }
            XtaskCmds::Clean(out) => {
                info!("Cleaning generated files...");
                if let Err(e) = clean(out.out_dir, out.all) {
                    warn!("Failed to clean generated files: {}", e);
                    STATUS.lock().unwrap().fail("clean");
                } else {
                    STATUS.lock().unwrap().success("clean");
                }
            }
            XtaskCmds::InstallPreCommitHook => {
                info!("Installing pre-commit hook...");
                if let Err(e) = install_pre_commit_hook() {
                    error!("Failed to install pre-commit hook: {}", e);
                    STATUS.lock().unwrap().fail("install-pre-commit-hook");
                } else {
                    STATUS.lock().unwrap().success("install-pre-commit-hook");
                }
            }
        },
        None => {
            info!("Running pre-commit hook...");
            if let Err(e) = run_pre_commit_hook() {
                error!("Pre-commit hook failed: {}", e);
                STATUS.lock().unwrap().fail("pre-commit");
            } else {
                STATUS.lock().unwrap().success("pre-commit");
            }
        }
    }
    if !STATUS.lock().unwrap().ok() {
        std::process::exit(1);
    }
}

#[instrument]
fn install_pre_commit_hook() -> XtaskResult<()> {
    devx_pre_commit::install_self_as_hook(&*PROJECT_ROOT)?;
    Ok(())
}

#[instrument]
fn run_pre_commit_hook() -> XtaskResult<()> {
    let mut ctx = PreCommitContext::from_git_diff(&*PROJECT_ROOT)?;

    ctx.retain_staged_files(|path| {
        path.components()
            .all(|it| it.as_os_str() != OsStr::new("generated"))
    });

    ctx.rustfmt()?;

    ctx.stage_new_changes()?;
    Ok(())
}

const TARGETS: [&str; 14] = [
    "aarch64-unknown-linux-gnu",
    "aarch64-unknown-linux-musl",
    "armv7-unknown-linux-gnueabi",
    "armv7-unknown-linux-gnueabihf",
    "armv7-unknown-linux-musleabi",
    "armv7-unknown-linux-musleabihf",
    "i686-unknown-linux-gnu",
    "i686-unknown-linux-musl",
    "riscv64gc-unknown-linux-gnu",
    "riscv64gc-unknown-linux-musl",
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
];

#[instrument]
fn mkdir<P: AsRef<Path> + std::fmt::Debug>(dir: P, clean: bool) -> XtaskResult<()> {
    if !dir.as_ref().exists() {
        std::fs::create_dir_all(dir)?;
    } else if clean {
        std::fs::remove_dir_all(&dir)?;
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}

#[instrument]
fn rmdir<P: AsRef<Path> + std::fmt::Debug>(dir: P) -> XtaskResult<()> {
    if dir.as_ref().exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

#[instrument]
fn clean<P: AsRef<Path> + std::fmt::Debug>(dir: P, all: bool) -> XtaskResult<()> {
    for out_path in TARGETS
        .iter()
        .chain(
            [
                "manpages",
                "bundle",
                "bash_completions",
                "zsh_completions",
                "fish_completions",
                "powershell_completions",
                "elvish_completions",
            ]
            .iter(),
        )
        .map(|t| dir.as_ref().join(t))
    {
        rmdir(&out_path).unwrap_or_else(|e| {
            warn!("Failed to remove directory {}: {}", out_path.display(), e);
        });
    }

    if all {
        let name = Cli::command().get_name().to_string();
        for file in TARGETS
            .iter()
            .map(|t| dir.as_ref().join(format!("{name}-{t}.tar.zst")))
        {
            if file.exists() {
                std::fs::remove_file(&file).unwrap_or_else(|e| {
                    warn!("Failed to remove file {}: {}", file.display(), e);
                });
            }
        }
    }
    Ok(())
}

#[instrument]
fn dist<P: AsRef<Path> + std::fmt::Debug>(out: P) {
    info!("Output directory: {:?}", out.as_ref());
    info!("Generating manpages...");
    build_manpages(&out).unwrap_or_else(|e| {
        warn!("Failed to build manpages: {}", e);
        STATUS.lock().unwrap().fail("manpages");
    });
    info!("Generating completions...");
    build_completion(&out).unwrap_or_else(|e| {
        warn!("Failed to build completions: {}", e);
        STATUS.lock().unwrap().fail("completion");
    });
    info!("Building targets...");
    for target in TARGETS {
        info!("Building target: {}", target);
        if let Err(e) = build(target) {
            warn!("Failed to build target {}: {}", target, e);
            STATUS.lock().unwrap().fail(target);
        } else {
            info!("Successfully built target: {}", target);
            STATUS.lock().unwrap().success(target);
        }
    }
    info!("Bundling distribution...");
    if let Err(e) = bundle(&out) {
        warn!("Failed to bundle distribution: {}", e);
        STATUS.lock().unwrap().fail("bundle");
    } else {
        info!("Successfully bundled distribution");
        STATUS.lock().unwrap().success("bundle");
    }
}

#[instrument]
fn build_manpages<P: AsRef<Path> + std::fmt::Debug>(out: P) -> XtaskResult<()> {
    let cmd = Cli::command();
    let name = cmd.get_name().to_string();
    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;
    let out = out.as_ref().join("manpages");
    info!("Generating manpages for {} in {}", name, out.display());
    mkdir(&out, false)?;
    std::fs::write(out.join(format!("{name}.1")), buffer)?;
    Ok(())
}

#[instrument]
fn build_completion<P: AsRef<Path> + std::fmt::Debug>(out: P) -> XtaskResult<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    info!(
        "Generating completions for {} in {}",
        name,
        &out.as_ref().display()
    );
    clap_complete::aot::generate(
        clap_complete::aot::Bash,
        &mut cmd,
        &name,
        &mut std::fs::File::create(
            completion_dir(&out, clap_complete::Shell::Bash)?.join(format!("{name}.sh")),
        )?,
    );
    clap_complete::generate(
        clap_complete::aot::Zsh,
        &mut cmd,
        &name,
        &mut std::fs::File::create(
            completion_dir(&out, clap_complete::Shell::Zsh)?.join(format!("{name}.zsh")),
        )?,
    );
    clap_complete::generate(
        clap_complete::aot::Fish,
        &mut cmd,
        &name,
        &mut std::fs::File::create(
            completion_dir(&out, clap_complete::Shell::Fish)?.join(format!("{name}.fish")),
        )?,
    );
    clap_complete::generate(
        clap_complete::aot::PowerShell,
        &mut cmd,
        &name,
        &mut std::fs::File::create(
            completion_dir(&out, clap_complete::Shell::PowerShell)?.join(format!("{name}.ps1")),
        )?,
    );
    clap_complete::generate(
        clap_complete::aot::Elvish,
        &mut cmd,
        &name,
        &mut std::fs::File::create(
            completion_dir(&out, clap_complete::Shell::Elvish)?.join(format!("{name}.elv")),
        )?,
    );
    Ok(())
}

#[instrument]
fn completion_dir<P: AsRef<Path> + std::fmt::Debug>(
    out: P,
    shell: clap_complete::Shell,
) -> XtaskResult<PathBuf> {
    info!(
        "Creating completion directory for {} in {:?}",
        shell.to_string(),
        out
    );
    let shell = match shell {
        clap_complete::Shell::Bash => "bash_completions",
        clap_complete::Shell::Zsh => "zsh_completions",
        clap_complete::Shell::Fish => "fish_completions",
        clap_complete::Shell::PowerShell => "powershell_completions",
        clap_complete::Shell::Elvish => "elvish_completions",
        _ => {
            return Err(format!("Unsupported shell: {shell:?}").into());
        }
    };
    let out = out.as_ref().join(shell);
    mkdir(&out, false)?;
    Ok(out)
}

#[instrument]
fn bundle<P: AsRef<Path> + std::fmt::Debug>(out: P) -> XtaskResult<()> {
    info!("Bundling distribution in {}", out.as_ref().display());
    let cur_dir = env::current_dir()?;
    let bundle_dir = out.as_ref().join("bundle");
    mkdir(&bundle_dir, true)?;
    let cmd = Cli::command();
    let name = cmd.get_name().to_string();
    let mut files: Vec<(PathBuf, Option<String>, String)> = vec![
        (
            PROJECT_ROOT.join("README.md"),
            None,
            "README.md".to_string(),
        ),
        (
            PROJECT_ROOT.join("LICENSE.MIT"),
            None,
            "LICENSE.MIT".to_string(),
        ),
        (
            PROJECT_ROOT.join("LICENSE.Apache-2.0"),
            None,
            "LICENSE.Apache-2.0".to_string(),
        ),
    ];
    for file in out.as_ref().join("manpages").read_dir()? {
        let file = file?;
        if file.file_type()?.is_file() {
            files.push((
                file.path(),
                Some("manpages".to_string()),
                file.file_name().to_string_lossy().to_string(),
            ));
        }
    }
    for complete in [
        "bash_completions",
        "zsh_completions",
        "fish_completions",
        "powershell_completions",
        "elvish_completions",
    ] {
        let shell = complete.trim_end_matches("_completions");
        for file in out.as_ref().join(complete).read_dir()? {
            let file = file?;
            if file.file_type()?.is_file() {
                files.push((
                    file.path(),
                    Some(shell.to_string()),
                    file.file_name().to_string_lossy().to_string(),
                ));
            }
        }
    }
    info!("Files to include in bundle: {:?}", files);
    let mut dirs: Vec<PathBuf> = Vec::new();
    for tgt in TARGETS {
        if !out.as_ref().join(tgt).join("release").join(&name).exists() {
            warn!("Skipping target {}: binary not found", tgt);
            continue;
        }
        info!("Bundling target: {}", tgt);
        let exe = if tgt.contains("windows") {
            format!("{name}.exe")
        } else {
            name.clone()
        };
        let tgt_dir = bundle_dir.join(tgt);
        mkdir(&tgt_dir, true)?;
        for (src, subdir, fname) in &files {
            let dir = match subdir {
                Some(subdir) => tgt_dir.join(subdir),
                None => tgt_dir.clone(),
            };
            mkdir(&dir, false)?;
            std::fs::copy(src, dir.join(fname))?;
        }
        std::fs::copy(
            out.as_ref().join(tgt).join("release").join(&exe),
            tgt_dir.join(&exe),
        )?;
        dirs.push(tgt_dir);
    }
    info!("Directories to compress: {:?}", dirs);
    let mut compressor = Compressor::new(3)?;
    compressor.include_checksum(true)?;
    compressor.multithread(num_cpus::get() as u32)?;
    compressor.include_contentsize(true)?;
    let mut all_hashes: String = String::new();
    for dir in &dirs {
        info!("Compressing directory: {}", dir.display());
        env::set_current_dir(dir)?;
        let mut tar_builder = Builder::new(Vec::new());
        tar_builder.mode(HeaderMode::Deterministic);
        tar_builder.follow_symlinks(false);
        let mut hashes: String = String::new();
        for file in dir.read_dir()? {
            let file = file?;
            if file.file_type()?.is_file() {
                let rel_path = file.path().strip_prefix(dir)?.to_owned();
                let mut f = std::fs::File::open(file.path())?;
                let mut hasher = Sha256::new();
                std::io::copy(&mut f, &mut hasher)?;
                let hash = hasher.finalize();
                drop(f);
                hashes.push_str(&format!("{hash:x}  {}\n", rel_path.display()));
                tar_builder.append_path_with_name(
                    file.path(),
                    PathBuf::from(format!("license/{}", rel_path.display())),
                )?;
            }
            let mut header = Header::new_gnu();
            header.set_entry_type(EntryType::Regular);
            header.set_mode(0o644);
            header.set_size(std::mem::size_of_val(&hashes) as u64);
            tar_builder.append_data(&mut header, "./sha256sums.txt", hashes.as_bytes())?;
        }
        let tarball = format!(
            "{name}-{}.tar.zst",
            dir.file_name().unwrap().to_string_lossy()
        );
        info!(
            "Finalizing archive {tarball} for directory {}",
            dir.display()
        );
        std::fs::File::create(out.as_ref().join(&tarball))?
            .write_all(&compressor.compress(&tar_builder.into_inner()?)?)?;
        let mut f = std::fs::File::open(out.as_ref().join(&tarball))?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut f, &mut hasher)?;
        let hash = hasher.finalize();
        let tar_hash = format!("{hash:x}  {tarball}\n");
        all_hashes.push_str(&tar_hash);
        std::fs::File::create(out.as_ref().join(format!("{tarball}.sha256")))?
            .write_all(tar_hash.as_bytes())?;
        std::fs::set_permissions(
            out.as_ref().join(format!("{tarball}.sha256")),
            std::fs::Permissions::from_mode(0o644),
        )?;
    }
    std::fs::File::create(out.as_ref().join("sha256sums.txt"))?.write_all(all_hashes.as_bytes())?;
    env::set_current_dir(cur_dir)?;
    Ok(())
}

#[instrument]
fn build(target: &str) -> XtaskResult<()> {
    info!("Building for target: {}", target);
    let mut cmd = if target == current_platform::CURRENT_PLATFORM {
        cmd!(
            "cargo",
            "build",
            "--package=license-gen-bin",
            "--release",
            format!("--target={target}")
        )
    } else {
        cmd!(
            "cross",
            "build",
            "--package=license-gen-bin",
            "--release",
            format!("--target={target}")
        )
    };
    cmd.env("CROSS_CONTAINER_ENGINE", "podman");
    cmd.current_dir(&*PROJECT_ROOT);
    if let Err(e) = cmd.run() {
        error!("Build failed for target {}: {}", target, e);
        run!("rustup", "target", "add", target)?;
        let mut cmd = cmd!(
            "cargo",
            "build",
            "--package=license-gen-bin",
            "--release",
            format!("--target={target}")
        );
        cmd.current_dir(&*PROJECT_ROOT);
        cmd.run()?;
    };
    Ok(())
}
