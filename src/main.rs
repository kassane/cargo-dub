// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Matheus C. França

use clap::{Parser, Subcommand};
use std::process::{Command, Stdio};
use std::{env, io, path::Path};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(
    name = "cargo-dub",
    bin_name = "cargo dub",
    version,
    about = "Cargo subcommand for dub"
)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Internal: Skip cargo detection
    #[command(name = "dub", hide = true)]
    Dub {
        #[command(subcommand)]
        cmd: Option<DubCommands>,
    },
    #[command(flatten)]
    Direct(DubCommands),
}

#[derive(Subcommand)]
enum DubCommands {
    /// Build and run package
    #[command(alias = "r")]
    Run(DubOptions),
    /// Build package
    #[command(alias = "b")]
    Build(DubOptions),
    /// Convert dub.json/dub.sdl
    Convert {
        #[arg(short, long, value_enum)]
        format: Format,
    },
    /// Pass raw arguments to dub
    Raw {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum Format {
    Json,
    Sdl,
}

#[derive(clap::Args, Default, Clone)]
struct DubOptions {
    #[arg(long)]
    compiler: Option<String>,
    #[arg(short = 'b', long)]
    build: Option<String>,
    #[arg(short = 'c', long)]
    config: Option<String>,
    #[arg(short = 'a', long)]
    arch: Option<String>,
    #[arg(long)]
    rdmd: bool,
    #[arg(long)]
    temp_build: bool,
    #[arg(short = 'f', long)]
    force: bool,
    #[arg(long)]
    nodeps: bool,
    #[arg(long)]
    deep: bool,
    #[arg(long = "d-version")]
    d_versions: Vec<String>,
    #[arg(short = 'd', long)]
    debug: Vec<String>,
    #[arg(long = "override-config")]
    override_config: Vec<String>,
    #[arg(long)]
    yes: bool,
    #[arg(long)]
    non_interactive: bool,
}

/// Trait for DUB executable command creation
trait DubCommand {
    fn command(&self) -> Command;
}

/// Cached DUB executable path
struct DubExecutable {
    path: String,
}

impl DubExecutable {
    fn new() -> Result<Self> {
        let candidates = if cfg!(windows) {
            vec!["dub.exe", "dub"]
        } else {
            vec!["dub"]
        };
        for candidate in candidates {
            if Command::new(candidate)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            {
                return Ok(Self {
                    path: candidate.to_string(),
                });
            }
        }
        Err("dub executable not found. Install DUB from https://dub.pm".into())
    }
}

impl DubCommand for DubExecutable {
    fn command(&self) -> Command {
        let mut cmd = Command::new(&self.path);
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        cmd
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    let dub = DubExecutable::new()?;

    let cmd = match args.command {
        Some(Commands::Dub { cmd }) => cmd.unwrap_or(DubCommands::Run(DubOptions::default())),
        Some(Commands::Direct(cmd)) => cmd,
        None => DubCommands::Run(DubOptions::default()),
    };

    match cmd {
        DubCommands::Run(opts) => execute_dub(&dub, "run", &opts),
        DubCommands::Build(opts) => execute_dub(&dub, "build", &opts),
        DubCommands::Convert { format } => convert_format(&dub, format),
        DubCommands::Raw { args } => execute_raw(&dub, &args),
    }
}

fn execute_dub(dub: &impl DubCommand, subcommand: &str, opts: &DubOptions) -> Result<()> {
    let mut cmd = dub.command();
    cmd.arg(subcommand);
    build_dub_args(&mut cmd, opts)?;
    execute_command(cmd)
}

fn execute_raw(dub: &impl DubCommand, args: &[String]) -> Result<()> {
    let mut cmd = dub.command();
    cmd.args(args);
    execute_command(cmd)
}

fn convert_format(dub: &impl DubCommand, format: Format) -> Result<()> {
    let (source, target) = match format {
        Format::Json => ("dub.sdl", "json"),
        Format::Sdl => ("dub.json", "sdl"),
    };

    if !Path::new(source).exists() {
        return Err(format!("Source file '{source}' not found").into());
    }

    let mut cmd = dub.command();
    cmd.args(["convert", &format!("--format={target}")]);
    execute_command(cmd)
}

fn build_dub_args(cmd: &mut Command, opts: &DubOptions) -> Result<()> {
    if let Some(compiler) = opts.compiler.clone().or_else(|| env::var("DC").ok()) {
        cmd.arg(format!("--compiler={compiler}"));
    }
    if let Some(build) = &opts.build {
        cmd.arg(format!("--build={build}"));
    }
    if let Some(config) = &opts.config {
        cmd.arg(format!("--config={config}"));
    }
    if let Some(arch) = &opts.arch {
        cmd.arg(format!("--arch={arch}"));
    }
    for (flag, enabled) in [
        ("--rdmd", opts.rdmd),
        ("--temp-build", opts.temp_build),
        ("--force", opts.force),
        ("--deep", opts.deep),
        ("--nodeps", opts.nodeps),
        ("--yes", opts.yes),
        ("--non-interactive", opts.non_interactive),
    ] {
        if enabled {
            cmd.arg(flag);
        }
    }
    for version in &opts.d_versions {
        cmd.arg(format!("--d-version={version}"));
    }
    for debug in &opts.debug {
        cmd.arg(format!("--debug={debug}"));
    }
    for config in &opts.override_config {
        cmd.arg(format!("--override-config={config}"));
    }
    Ok(())
}

fn execute_command(mut cmd: Command) -> Result<()> {
    match cmd.status() {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(e) => Err(match e.kind() {
            io::ErrorKind::NotFound => "dub executable not found or not accessible",
            io::ErrorKind::PermissionDenied => "Permission denied when executing dub",
            io::ErrorKind::WouldBlock => "System resources temporarily unavailable",
            _ => "Failed to execute dub",
        }
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    // Mock DubExecutable for tests
    struct MockDubExecutable {
        path: String,
    }

    impl MockDubExecutable {
        fn new(path: &str) -> Self {
            MockDubExecutable {
                path: path.to_string(),
            }
        }
    }

    impl DubCommand for MockDubExecutable {
        fn command(&self) -> Command {
            let mut cmd = Command::new(&self.path);
            cmd.stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            cmd
        }
    }

    #[test]
    fn test_build_dub_args() {
        let opts = DubOptions {
            compiler: Some("ldc2".to_string()),
            build: Some("release".to_string()),
            config: Some("test-config".to_string()),
            arch: Some("x86_64".to_string()),
            rdmd: true,
            temp_build: true,
            force: true,
            nodeps: false,
            deep: true,
            d_versions: vec!["ver1".to_string(), "ver2".to_string()],
            debug: vec!["debug1".to_string()],
            override_config: vec!["conf1".to_string()],
            yes: true,
            non_interactive: false,
        };

        let mut cmd = Command::new("dub");
        build_dub_args(&mut cmd, &opts).unwrap();

        let args: Vec<String> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            vec![
                "--compiler=ldc2",
                "--build=release",
                "--config=test-config",
                "--arch=x86_64",
                "--rdmd",
                "--temp-build",
                "--force",
                "--deep",
                "--yes",
                "--d-version=ver1",
                "--d-version=ver2",
                "--debug=debug1",
                "--override-config=conf1",
            ]
        );
    }

    #[test]
    fn test_build_dub_args_with_env_dc() {
        env::set_var("DC", "dmd");
        let opts = DubOptions {
            compiler: None,
            build: None,
            config: None,
            arch: None,
            rdmd: false,
            temp_build: false,
            force: false,
            nodeps: false,
            deep: false,
            d_versions: vec![],
            debug: vec![],
            override_config: vec![],
            yes: false,
            non_interactive: false,
        };

        let mut cmd = Command::new("dub");
        build_dub_args(&mut cmd, &opts).unwrap();

        let args: Vec<String> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["--compiler=dmd"]);
        env::remove_var("DC");
    }

    #[test]
    fn test_convert_format_file_missing() {
        let dub = MockDubExecutable::new("dub");
        let result = convert_format(&dub, Format::Json);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Source file 'dub.sdl' not found"
        );
    }

    #[test]
    fn test_convert_format_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("dub.sdl");
        File::create(&source_path).unwrap().write_all(b"").unwrap();

        let dub = MockDubExecutable::new("dub");
        let mut cmd = dub.command();
        cmd.current_dir(temp_dir.path());
        cmd.args(["convert", "--format=json"]);

        let args: Vec<String> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["convert", "--format=json"]);
    }

    #[test]
    fn test_execute_dub_command() {
        let dub = MockDubExecutable::new("dub");
        let opts = DubOptions::default();
        let mut cmd = dub.command();
        cmd.arg("run");
        build_dub_args(&mut cmd, &opts).unwrap();

        let args: Vec<String> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["run"]);
    }
}
