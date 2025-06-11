// main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cargo-dub", version = "0.1.0", author = "Matheus C. França", about = "A cargo subcommand for dub")]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Build and run the package (default)
    Run(DubOptions),

    /// Only build the package
    Build(DubOptions),

    /// Convert between dub.json and dub.sdl
    Convert {
        #[arg(short = 'f', long)]
        format: String,
    },

    /// Pass-through to dub with raw arguments
    Raw {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

#[derive(clap::Args, Default)]
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

fn main() {
    let args = Args::parse();

    match args.command.unwrap_or(Command::Run(DubOptions::default())) {
        Command::Run(opts) => run_dub("run", opts),
        Command::Build(opts) => run_dub("build", opts),
        Command::Convert { format } => convert_format(&format),
        Command::Raw { args } => {
            std::process::Command::new("dub")
                .args(args)
                .status()
                .expect("failed to run dub");
        }
    }
}

fn run_dub(sub: &str, opts: DubOptions) {
    let mut cmd = std::process::Command::new("dub");
    cmd.arg(sub);

    if let Some(c) = opts.compiler.or_else(|| std::env::var("DC").ok()) {
        cmd.arg(format!("--compiler={}", c));
    }

    if let Some(b) = opts.build {
        cmd.arg(format!("--build={}", b));
    }

    if let Some(cfg) = opts.config {
        cmd.arg(format!("--config={}", cfg));
    }

    if let Some(arch) = opts.arch {
        cmd.arg(format!("--arch={}", arch));
    }

    if opts.rdmd { cmd.arg("--rdmd"); }
    if opts.temp_build { cmd.arg("--temp-build"); }
    if opts.force { cmd.arg("--force"); }
    if opts.deep { cmd.arg("--deep"); }
    if opts.nodeps { cmd.arg("--nodeps"); }
    if opts.yes { cmd.arg("--yes"); }
    if opts.non_interactive { cmd.arg("--non-interactive"); }

    for v in opts.d_versions {
        cmd.arg(format!("--d-version={}", v));
    }

    for d in opts.debug {
        cmd.arg(format!("--debug={}", d));
    }

    for oc in opts.override_config {
        cmd.arg(format!("--override-config={}", oc));
    }

    let status = cmd.status().expect("failed to run dub");
    std::process::exit(status.code().unwrap_or(1));
}

fn convert_format(fmt: &str) {
    match fmt {
        "json" => {
            if std::path::Path::new("dub.sdl").exists() {
                std::process::Command::new("dub")
                    .args(["convert", "--format=json"])
                    .status()
                    .expect("failed to convert to json");
            } else {
                eprintln!("dub.sdl not found");
            }
        }
        "sdl" => {
            if std::path::Path::new("dub.json").exists() {
                std::process::Command::new("dub")
                    .args(["convert", "--format=sdl"])
                    .status()
                    .expect("failed to convert to sdl");
            } else {
                eprintln!("dub.json not found");
            }
        }
        _ => {
            eprintln!("invalid format: '{}'", fmt);
        }
    }
}
