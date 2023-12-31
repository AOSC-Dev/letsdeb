use std::{fs, path::Path};

use clap::{Parser, ValueEnum};
use eyre::{Result, bail};
use letsdeb_core::build::{do_build_deb, CompressType};
use log::info;
use rustix::process;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    root_path: String,
    #[clap(long)]
    control_path: Option<String>,
    #[clap(long)]
    #[arg(default_value_t = Self::default_output_dir())]
    output_dir: String,
    #[clap(long)]
    #[arg(value_enum)]
    compress_type: CompressTypeArg,
    #[clap(long)]
    #[arg(default_value_t = 6)]
    level: u32,
    #[clap(long)]
    pkg_name: String,
}

#[derive(Debug, Clone, ValueEnum)]
enum CompressTypeArg {
    Xz,
    Gz,
    Zstd,
}

impl Args {
    fn default_output_dir() -> String {
        ".".to_string()
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let Args {
        root_path,
        control_path,
        output_dir,
        compress_type,
        level,
        pkg_name,
    } = Args::parse();

    if !process::geteuid().is_root() {
        bail!("Please run me as root!")
    }

    info!("Building deb ...");

    let control_path = if let Some(control_path) = control_path {
        control_path
    } else {
        let p = Path::new(&root_path).canonicalize()?.join("letsdeb-control");
        fs::create_dir_all(&p)?;

        p.display().to_string()
    };

    do_build_deb(
        root_path,
        control_path,
        match compress_type {
            CompressTypeArg::Xz => CompressType::Xz { level },
            CompressTypeArg::Gz => CompressType::Gz { level },
            CompressTypeArg::Zstd => CompressType::Zstd {
                level: level as i32,
            },
        },
        output_dir,
        &pkg_name,
    )?;

    Ok(())
}
