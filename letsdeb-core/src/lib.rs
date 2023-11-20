use std::{
    fmt::Display,
    fs,
    io::{self, Write},
    path::Path,
};

use flate2::{write::GzEncoder, Compression};
use xz::write::XzEncoder;

pub enum CompressType {
    Xz { level: u32 },
    Gz { level: u32 },
    Zstd { level: i32 },
}

impl Display for CompressType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressType::Xz { .. } => f.write_str("xz"),
            CompressType::Gz { .. } => f.write_str("gz"),
            CompressType::Zstd { .. } => f.write_str("zstd"),
        }
    }
}

pub fn do_build_deb<P: AsRef<Path>>(
    root_path: P,
    control_dir_path: P,
    compress_type: CompressType,
    output_dir: P,
    pkg_name: &str,
) -> io::Result<()> {
    let output_dir = output_dir.as_ref();
    let control_dir_path = control_dir_path.as_ref();
    let root_path = root_path.as_ref();

    let root_blocklist = if control_dir_path.starts_with(root_path) {
        let mut v = vec![control_dir_path.to_path_buf()];
        for i in fs::read_dir(control_dir_path)? {
            v.push(i?.path().to_path_buf())
        }
        v
    } else {
        vec![]
    };

    compress_files(
        root_path,
        &compress_type,
        output_dir,
        &root_blocklist
            .iter()
            .map(|x| x.as_ref())
            .collect::<Vec<&Path>>(),
        "data",
    )?;

    compress_files(control_dir_path, &compress_type, output_dir, &[], "control")?;

    let mut debian_binary = fs::File::create(output_dir.join("debian-binary"))?;
    debian_binary.write_all(b"2.0")?;

    let mut builder = ar::Builder::new(fs::File::create(
        output_dir.join(format!("{pkg_name}.deb")),
    )?);

    let control_path = output_dir.join(format!("control.tar.{compress_type}"));
    let data_path = output_dir.join(format!("data.tar.{compress_type}"));
    let debian_binary_path = output_dir.join("debian-binary");

    builder.append_path(&control_path)?;
    builder.append_path(&data_path)?;
    builder.append_path(&debian_binary_path)?;

    fs::remove_file(control_path)?;
    fs::remove_file(&data_path)?;
    fs::remove_file(&debian_binary_path)?;

    Ok(())
}

fn compress_files<P: AsRef<Path>>(
    compress_path: P,
    compress_type: &CompressType,
    output_dir: P,
    blocklist: &[P],
    name: &str,
) -> io::Result<()> {
    let output_dir = output_dir.as_ref();
    let mut tar = tar::Builder::new(vec![]);

    for i in fs::read_dir(compress_path)? {
        let p = i?.path();
        if blocklist
            .iter()
            .map(|x| x.as_ref())
            .position(|x| x == p)
            .is_none()
        {
            tar.append_path(p)?;
        }
    }

    tar.finish()?;

    let mut compresser: Box<dyn Write> = match compress_type {
        CompressType::Xz { level } => Box::new(XzEncoder::new(
            fs::File::open(output_dir.join(format!("{name}.tar.xz")))?,
            *level,
        )),
        CompressType::Gz { level } => Box::new(GzEncoder::new(
            fs::File::open(output_dir.join(format!("{name}.tar.gz")))?,
            Compression::new(*level),
        )),
        CompressType::Zstd { level } => Box::new(zstd::Encoder::new(
            fs::File::open(output_dir.join(format!("{name}.tar.zstd")))?,
            *level,
        )?),
    };

    compresser.write_all(&tar.into_inner()?)?;

    Ok(())
}
