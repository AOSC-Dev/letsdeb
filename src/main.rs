use clap::Parser;
use eyre::{bail, Result};
use letsdeb_core::{do_build_deb, CompressType};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    root_path: String,
    control_path: String,
    output_dir: String,
    compress_type: String,
    #[clap(default_value_t = 6)]
    level: u32,
    pkg_name: String,
}

fn main() -> Result<()> {
    let Args {
        root_path,
        control_path,
        output_dir,
        compress_type,
        level,
        pkg_name,
    } = Args::parse();

    do_build_deb(
        root_path,
        control_path,
        match compress_type.as_str() {
            "xz" => CompressType::Xz { level },
            "gz" => CompressType::Gz { level },
            "zstd" => CompressType::Zstd {
                level: level as i32,
            },
            _ => bail!("Unsupport compression type"),
        },
        output_dir,
        &pkg_name,
    )?;

    Ok(())
}
