use std::{
    ffi::OsStr,
    fmt::Display,
    fs,
    io::{self, ErrorKind, Write},
    os::unix::{self, fs::PermissionsExt},
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
    let output_dir = output_dir.as_ref().canonicalize()?;
    let control_dir_path = control_dir_path.as_ref().canonicalize()?;
    let root_path = root_path.as_ref().canonicalize()?;

    let root_blocklist = if control_dir_path.starts_with(&root_path) {
        let mut v = vec![control_dir_path.to_path_buf()];
        for i in fs::read_dir(&control_dir_path)? {
            let p = i?.path();
            v.push(p)
        }
        v
    } else {
        vec![]
    };

    let blocklist = root_blocklist.iter().collect::<Vec<_>>();

    for i in fs::read_dir(&root_path)? {
        let p = i?.path();
        unix::fs::chown(&p, Some(0), Some(0))?;
    }

    compress_files(&root_path, &compress_type, &output_dir, &blocklist, "data")?;

    for i in fs::read_dir(&control_dir_path)? {
        let p = i?.path();
        let file_name = get_file_name(p.file_name())
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "Can not parse file name"))?;

        if file_name == "control" {
            let mut perms = fs::metadata(&p)?.permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&p, perms)?;
        } else {
            let mut perms = fs::metadata(&p)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&p, perms)?;
        }

        unix::fs::chown(&p, Some(0), Some(0))?;
    }

    compress_files(
        &control_dir_path,
        &compress_type,
        &output_dir,
        &[],
        "control",
    )?;

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
            let file_name = get_file_name(p.file_name()).ok_or_else(|| {
                io::Error::new(ErrorKind::InvalidInput, "Can not parse file name")
            })?;

            if p.is_dir() {
                tar.append_dir_all(file_name, p)?;
            } else {
                tar.append_file(file_name, &mut fs::File::open(p)?)?;
            }
        }
    }

    tar.finish()?;

    let mut compresser: Box<dyn Write> = match compress_type {
        CompressType::Xz { level } => Box::new(XzEncoder::new(
            fs::File::create(output_dir.join(format!("{name}.tar.xz")))?,
            *level,
        )),
        CompressType::Gz { level } => Box::new(GzEncoder::new(
            fs::File::create(output_dir.join(format!("{name}.tar.gz")))?,
            Compression::new(*level),
        )),
        CompressType::Zstd { level } => Box::new(zstd::Encoder::new(
            fs::File::create(output_dir.join(format!("{name}.tar.zstd")))?,
            *level,
        )?),
    };

    compresser.write_all(&tar.into_inner()?)?;

    Ok(())
}

fn get_file_name(file_name: Option<&OsStr>) -> Option<String> {
    Some(file_name?.to_str()?.to_string())
}
