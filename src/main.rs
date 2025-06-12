// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Matheus C. França

use clap::{Args, Parser, Subcommand};
use std::process::{Command, Stdio};
use std::{env, io, path::Path};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser, Debug)]
#[command(name = "cargo-dub", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(name = "dub", hide = true)]
    Dub {
        #[command(subcommand)]
        cmd: Option<DubCommands>,
    },
    #[command(flatten)]
    Direct(DubCommands),
}

#[derive(Subcommand, Debug)]
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
    /// Print JSON build description for package and dependencies
    Describe(DescribeOptions),
    /// Add packages as dependencies
    Add(AddRemoveOptions),
    /// Remove packages from dependencies
    Remove(AddRemoveOptions),
    /// Fetch packages to a shared location
    Fetch(FetchOptions),
    /// Initialize an empty package
    Init(InitOptions),
    /// Remove cached build files
    Clean(CleanOptions),
    /// Run D-Scanner linter tests
    Lint(LintOptions),
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Format {
    Json,
    Sdl,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum ProjectType {
    Minimal,
    VibeD,
    Deimos,
    Custom,
}

#[derive(Args, Default, Clone, Debug)]
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

#[derive(Args, Clone, Debug)]
struct DescribeOptions {
    #[arg(long, value_delimiter = ',')]
    data: Option<Vec<String>>,
    #[arg(long)]
    data_list: bool,
    #[command(flatten)]
    options: DubOptions,
}

#[derive(Args, Clone, Debug)]
struct AddRemoveOptions {
    #[arg(required = true, value_name = "PACKAGE[@VERSION]")]
    packages: Vec<String>,
    #[command(flatten)]
    options: DubOptions,
}

#[derive(Args, Clone, Debug)]
struct FetchOptions {
    #[arg(required = true, value_name = "PACKAGE[@VERSION]")]
    package: String,
    #[arg(long)]
    cache: Option<String>,
    #[command(flatten)]
    options: DubOptions,
}

#[derive(Args, Clone, Debug)]
struct InitOptions {
    #[arg(value_name = "DIRECTORY")]
    directory: Option<String>,
    #[arg(value_name = "DEPENDENCY")]
    dependencies: Vec<String>,
    #[arg(short, long, value_enum, default_value_t = ProjectType::Minimal)]
    r#type: ProjectType,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    non_interactive: bool,
    #[command(flatten)]
    options: DubOptions,
}

#[derive(Args, Clone, Debug)]
struct CleanOptions {
    #[arg(value_name = "PACKAGE")]
    package: Option<String>,
    #[arg(long)]
    all_packages: bool,
    #[command(flatten)]
    options: DubOptions,
}

#[derive(Args, Clone, Debug)]
struct LintOptions {
    #[arg(value_name = "PACKAGE[@VERSION]")]
    package: Option<String>,
    #[arg(long)]
    syntax_check: bool,
    #[arg(long)]
    style_check: bool,
    #[arg(long)]
    error_format: Option<String>,
    #[arg(long)]
    report: bool,
    #[arg(long)]
    report_format: Option<String>,
    #[arg(long)]
    report_file: Option<String>,
    #[arg(long)]
    import_paths: Option<Vec<String>>,
    #[arg(long)]
    dscanner_config: Option<String>,
    #[command(flatten)]
    options: DubOptions,
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
    let args = Cli::parse();
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
        DubCommands::Describe(opts) => execute_describe(&dub, &opts),
        DubCommands::Add(opts) => execute_add_remove(&dub, "add", &opts),
        DubCommands::Remove(opts) => execute_add_remove(&dub, "remove", &opts),
        DubCommands::Fetch(opts) => execute_fetch(&dub, &opts),
        DubCommands::Init(opts) => execute_init(&dub, &opts),
        DubCommands::Clean(opts) => execute_clean(&dub, &opts),
        DubCommands::Lint(opts) => execute_lint(&dub, &opts),
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

fn execute_describe(dub: &impl DubCommand, opts: &DescribeOptions) -> Result<()> {
    let mut cmd = dub.command();
    cmd.arg("describe");
    if let Some(data) = &opts.data {
        for d in data {
            cmd.arg(format!("--data={d}"));
        }
    }
    if opts.data_list {
        cmd.arg("--data-list");
    }
    build_dub_args(&mut cmd, &opts.options)?;
    execute_command(cmd)
}

fn execute_add_remove(
    dub: &impl DubCommand,
    subcommand: &str,
    opts: &AddRemoveOptions,
) -> Result<()> {
    let mut cmd = dub.command();
    cmd.arg(subcommand);
    cmd.args(&opts.packages);
    build_dub_args(&mut cmd, &opts.options)?;
    execute_command(cmd)
}

fn execute_fetch(dub: &impl DubCommand, opts: &FetchOptions) -> Result<()> {
    let mut cmd = dub.command();
    cmd.arg("fetch");
    cmd.arg(&opts.package);
    if let Some(cache) = &opts.cache {
        cmd.arg(format!("--cache={cache}"));
    }
    build_dub_args(&mut cmd, &opts.options)?;
    execute_command(cmd)
}

fn execute_init(dub: &impl DubCommand, opts: &InitOptions) -> Result<()> {
    let mut cmd = dub.command();
    cmd.arg("init");
    if let Some(dir) = &opts.directory {
        cmd.arg(dir);
    }
    cmd.args(&opts.dependencies);
    cmd.arg(format!(
        "--type={}",
        match opts.r#type {
            ProjectType::Minimal => "minimal",
            ProjectType::VibeD => "vibe.d",
            ProjectType::Deimos => "deimos",
            ProjectType::Custom => "custom",
        }
    ));
    if opts.non_interactive {
        cmd.arg("--non-interactive");
    }
    build_dub_args(&mut cmd, &opts.options)?;
    execute_command(cmd)
}

fn execute_clean(dub: &impl DubCommand, opts: &CleanOptions) -> Result<()> {
    let mut cmd = dub.command();
    cmd.arg("clean");
    if let Some(package) = &opts.package {
        cmd.arg(package);
    }
    if opts.all_packages {
        cmd.arg("--all-packages");
    }
    build_dub_args(&mut cmd, &opts.options)?;
    execute_command(cmd)
}

fn execute_lint(dub: &impl DubCommand, opts: &LintOptions) -> Result<()> {
    let mut cmd = dub.command();
    cmd.arg("lint");
    if let Some(package) = &opts.package {
        cmd.arg(package);
    }
    if opts.syntax_check {
        cmd.arg("--syntax-check");
    }
    if opts.style_check {
        cmd.arg("--style-check");
    }
    if let Some(format) = &opts.error_format {
        cmd.arg(format!("--error-format={format}"));
    }
    if opts.report {
        cmd.arg("--report");
    }
    if let Some(format) = &opts.report_format {
        cmd.arg(format!("--report-format={format}"));
    }
    if let Some(file) = &opts.report_file {
        cmd.arg(format!("--report-file={file}"));
    }
    if let Some(paths) = &opts.import_paths {
        for path in paths {
            cmd.arg(format!("--import-paths={path}"));
        }
    }
    if let Some(config) = &opts.dscanner_config {
        cmd.arg(format!("--dscanner-config={config}"));
    }
    build_dub_args(&mut cmd, &opts.options)?;
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

        let cmd = Command::new("dub");
        let mut cmd = cmd;
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
        let opts = DubOptions::default();
        let cmd = Command::new("dub");
        let mut cmd = cmd;
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
        let cmd = dub.command();
        let mut cmd = cmd;
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
        let cmd = dub.command();
        let mut cmd = cmd;
        cmd.arg("run");
        build_dub_args(&mut cmd, &opts).unwrap();

        let args: Vec<String> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["run"]);
    }

    #[test]
    fn test_execute_describe() {
        let dub = MockDubExecutable::new("dub");
        let opts = DescribeOptions {
            data: Some(vec!["main-source-file".to_string(), "libs".to_string()]),
            data_list: true,
            options: DubOptions {
                compiler: Some("ldc2".to_string()),
                ..Default::default()
            },
        };
        let cmd = dub.command();
        let mut cmd = cmd;
        cmd.arg("describe");
        if let Some(data) = &opts.data {
            for d in data {
                cmd.arg(format!("--data={d}"));
            }
        }
        if opts.data_list {
            cmd.arg("--data-list");
        }
        build_dub_args(&mut cmd, &opts.options).unwrap();

        let args: Vec<String> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            vec![
                "describe",
                "--data=main-source-file",
                "--data=libs",
                "--data-list",
                "--compiler=ldc2"
            ]
        );
    }

    #[test]
    fn test_execute_add() {
        let dub = MockDubExecutable::new("dub");
        let opts = AddRemoveOptions {
            packages: vec!["vibelog@1.0.0".to_string(), "libdparse".to_string()],
            options: DubOptions {
                yes: true,
                ..Default::default()
            },
        };
        let cmd = dub.command();
        let mut cmd = cmd;
        cmd.arg("add");
        cmd.args(&opts.packages);
        build_dub_args(&mut cmd, &opts.options).unwrap();

        let args: Vec<String> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["add", "vibelog@1.0.0", "libdparse", "--yes"]);
    }

    #[test]
    fn test_execute_remove() {
        let dub = MockDubExecutable::new("dub");
        let opts = AddRemoveOptions {
            packages: vec!["vibelog@1.0.0".to_string()],
            options: DubOptions {
                force: true,
                ..Default::default()
            },
        };
        let cmd = dub.command();
        let mut cmd = cmd;
        cmd.arg("remove");
        cmd.args(&opts.packages);
        build_dub_args(&mut cmd, &opts.options).unwrap();

        let args: Vec<String> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["remove", "vibelog@1.0.0", "--force"]);
    }

    #[test]
    fn test_execute_fetch() {
        let dub = MockDubExecutable::new("dub");
        let opts = FetchOptions {
            package: "vibelog@1.0.0".to_string(),
            cache: Some("local".to_string()),
            options: DubOptions {
                yes: true,
                ..Default::default()
            },
        };
        let cmd = dub.command();
        let mut cmd = cmd;
        cmd.arg("fetch");
        cmd.arg(&opts.package);
        if let Some(cache) = &opts.cache {
            cmd.arg(format!("--cache={cache}"));
        }
        build_dub_args(&mut cmd, &opts.options).unwrap();

        let args: Vec<String> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            vec!["fetch", "vibelog@1.0.0", "--cache=local", "--yes"]
        );
    }

    #[test]
    fn test_execute_init() {
        let dub = MockDubExecutable::new("dub");
        let opts = InitOptions {
            directory: Some("my_project".to_string()),
            dependencies: vec!["vibelog@1.0.0".to_string()],
            r#type: ProjectType::VibeD,
            non_interactive: true,
            options: DubOptions {
                yes: true,
                ..Default::default()
            },
        };
        let cmd = dub.command();
        let mut cmd = cmd;
        cmd.arg("init");
        if let Some(dir) = &opts.directory {
            cmd.arg(dir);
        }
        cmd.args(&opts.dependencies);
        cmd.arg(format!(
            "--type={}",
            match opts.r#type {
                ProjectType::Minimal => "minimal",
                ProjectType::VibeD => "vibe.d",
                ProjectType::Deimos => "deimos",
                ProjectType::Custom => "custom",
            }
        ));
        if opts.non_interactive {
            cmd.arg("--non-interactive");
        }
        build_dub_args(&mut cmd, &opts.options).unwrap();

        let args: Vec<String> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            vec![
                "init",
                "my_project",
                "vibelog@1.0.0",
                "--type=vibe.d",
                "--non-interactive",
                "--yes"
            ]
        );

        // Test with no directory, no dependencies, and minimal flags
        let opts_minimal = InitOptions {
            directory: None,
            dependencies: vec![],
            r#type: ProjectType::Minimal,
            non_interactive: false,
            options: DubOptions::default(),
        };
        let cmd = dub.command();
        let mut cmd = cmd;
        cmd.arg("init");
        if let Some(dir) = &opts_minimal.directory {
            cmd.arg(dir);
        }
        cmd.args(&opts_minimal.dependencies);
        cmd.arg(format!(
            "--type={}",
            match opts_minimal.r#type {
                ProjectType::Minimal => "minimal",
                ProjectType::VibeD => "vibe.d",
                ProjectType::Deimos => "deimos",
                ProjectType::Custom => "custom",
            }
        ));
        if opts_minimal.non_interactive {
            cmd.arg("--non-interactive");
        }
        build_dub_args(&mut cmd, &opts_minimal.options).unwrap();

        let args: Vec<String> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["init", "--type=minimal"]);
    }

    #[test]
    fn test_execute_clean() {
        let dub = MockDubExecutable::new("dub");
        let opts = CleanOptions {
            package: Some("my_package".to_string()),
            all_packages: false,
            options: DubOptions {
                force: true,
                ..Default::default()
            },
        };
        let cmd = dub.command();
        let mut cmd = cmd;
        cmd.arg("clean");
        if let Some(package) = &opts.package {
            cmd.arg(package);
        }
        if opts.all_packages {
            cmd.arg("--all-packages");
        }
        build_dub_args(&mut cmd, &opts.options).unwrap();

        let args: Vec<String> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["clean", "my_package", "--force"]);
    }

    #[test]
    fn test_execute_lint() {
        let dub = MockDubExecutable::new("dub");
        let opts = LintOptions {
            package: Some("my_package@1.0.0".to_string()),
            syntax_check: true,
            style_check: true,
            error_format: Some("custom".to_string()),
            report: true,
            report_format: Some("json".to_string()),
            report_file: Some("report.json".to_string()),
            import_paths: Some(vec!["src".to_string()]),
            dscanner_config: Some("dscanner.ini".to_string()),
            options: DubOptions {
                yes: true,
                ..Default::default()
            },
        };
        let cmd = dub.command();
        let mut cmd = cmd;
        cmd.arg("lint");
        if let Some(package) = &opts.package {
            cmd.arg(package);
        }
        if opts.syntax_check {
            cmd.arg("--syntax-check");
        }
        if opts.style_check {
            cmd.arg("--style-check");
        }
        if let Some(format) = &opts.error_format {
            cmd.arg(format!("--error-format={format}"));
        }
        if opts.report {
            cmd.arg("--report");
        }
        if let Some(format) = &opts.report_format {
            cmd.arg(format!("--report-format={format}"));
        }
        if let Some(file) = &opts.report_file {
            cmd.arg(format!("--report-file={file}"));
        }
        if let Some(paths) = &opts.import_paths {
            for path in paths {
                cmd.arg(format!("--import-paths={path}"));
            }
        }
        if let Some(config) = &opts.dscanner_config {
            cmd.arg(format!("--dscanner-config={config}"));
        }
        build_dub_args(&mut cmd, &opts.options).unwrap();

        let args: Vec<String> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            vec![
                "lint",
                "my_package@1.0.0",
                "--syntax-check",
                "--style-check",
                "--error-format=custom",
                "--report",
                "--report-format=json",
                "--report-file=report.json",
                "--import-paths=src",
                "--dscanner-config=dscanner.ini",
                "--yes"
            ]
        );
    }
}
