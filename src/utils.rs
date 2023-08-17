use anyhow::Result;
use std::ffi::OsStr;
use std::io;
use std::process::ExitStatus;
use std::{fs, path::PathBuf, process};
use which::which;

use lazy_static::lazy_static;

lazy_static! {
    /// command to launch VSCode
    pub static ref CODE_CMD: PathBuf = which("code").expect(r#"no "code" found in path"#);
}

pub trait WrappedCommand {
    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;
    fn status(&mut self) -> io::Result<ExitStatus>;
}

pub struct VSCodeComand {
    inner: process::Command,
}

impl VSCodeComand {
    pub fn new() -> Self {
        Self {
            inner: process::Command::new(CODE_CMD.as_os_str()),
        }
    }
}

impl WrappedCommand for VSCodeComand {
    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.inner.args(args);
        self
    }

    fn status(&mut self) -> io::Result<ExitStatus> {
        self.inner.status()
    }
}

/// copy/rename mutable slice of files (set extension)
pub fn set_ext_to_all(
    files: &mut [&mut PathBuf],
    extension: &str,
    rename_files: bool,
) -> Result<()> {
    for file in files.iter_mut() {
        let src = file.clone();
        file.set_extension(extension);
        if src.extension() == file.extension() {
            continue;
        }
        if rename_files {
            fs::rename(src, &file)?;
        } else {
            fs::copy(src, &file)?;
        }
    }
    Ok(())
}

/// remove files
pub fn remove_all_files<'a, I: Iterator<Item = &'a mut PathBuf>>(files: I) -> Result<()> {
    for file in files {
        fs::remove_file(file)?
    }
    Ok(())
}
