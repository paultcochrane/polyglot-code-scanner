#![warn(clippy::all)]
use failure::Error;

use pretty_assertions::assert_eq;
use serde::Serialize;
use serde_json::Value;
use std::fs::File;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// adapted from https://github.com/mvdnes/zip-rs/blob/master/examples/extract.rs
pub fn unzip_to_dir(dest: &Path, zipname: &str) -> Result<(), Error> {
    let fname = std::path::Path::new(zipname);
    let file = File::open(&fname)?;

    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = PathBuf::from(dest).join(file.sanitized_name());

        if (&*file.name()).ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

pub fn unzip_git_sample(workdir: &Path) -> Result<PathBuf, Error> {
    unzip_to_dir(workdir, "tests/data/git/git_sample.zip")?;
    Ok(PathBuf::from(workdir).join("git_sample"))
}

pub fn assert_eq_json_file<T: ?Sized>(actual: &T, expected_file: &str)
where
    T: Serialize,
{
    let expected = std::fs::read_to_string(Path::new(expected_file)).unwrap();

    assert_eq_json_str(&actual, &expected)
}

pub fn assert_eq_json_str<T: ?Sized>(actual_serializable: &T, expected_json: &str)
where
    T: Serialize,
{
    let actual = serde_json::to_value(&actual_serializable).unwrap();

    let expected: Value = serde_json::from_str(expected_json).unwrap();
    assert_eq!(&actual, &expected)
}

pub fn assert_eq_json_value<T: ?Sized>(actual_serializable: &T, expected_json: &Value)
where
    T: Serialize,
{
    let actual = serde_json::to_value(&actual_serializable).unwrap();

    assert_eq!(&actual, expected_json)
}

pub fn assert_eq_json(left: &str, right: &str) {
    let left: Value = serde_json::from_str(left).unwrap();
    let right: Value = serde_json::from_str(right).unwrap();
    assert_eq!(left, right);
}
