//#![windows_subsystem = "windows"]

use anyhow::Result;
use clap::{Parser, Subcommand};

use std::vec;
use std::{fs, path::PathBuf, process};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    /// diff or merge
    #[clap(subcommand)]
    command: Action,
    /// remove source files
    #[clap(short, long, action)]
    remove_files: bool,
    /// rename source files with bsl extension instead of copy
    #[clap(short = 'n', long, action)]
    rename_files: bool,
}

/// diff or merge
#[derive(Subcommand, Debug)]
enum Action {
    /// diff 2 files
    Diff {
        /// first file / %baseCfg
        #[clap(value_parser)]
        base_cfg: PathBuf,
        /// second file / %secondCfg
        #[clap(value_parser)]
        second_cfg: PathBuf,
    },
    /// merge 3 files into 1
    Merge {
        /// first file (from base config) / %baseCfg
        #[clap(value_parser)]
        base_cfg: PathBuf,
        /// second file (from new config) / %secondCfg
        #[clap(value_parser)]
        second_cfg: PathBuf,
        /// third file (from old vendor config) / %oldVendorCfg
        #[clap(value_parser)]
        old_vendor_cfg: PathBuf,
        /// merge result file / %merged
        #[clap(value_parser)]
        merged: PathBuf,
        /// use(copy) `second_cfg` as `merged`(result) (default is `base_cfg`)
        #[clap(short = 's', long, action)]
        from_second: bool,
    },
}

#[cfg(windows)]
const CODE_CMD: &str = "code.cmd";
#[cfg(not(windows))]
const CODE_CMD: &str = "code";
const COMMON_CODE_ARGS: [&str; 4] = ["--new-window", "--sync", "off", "--wait"];
const CODE_CMD_DIFF: &str = "--diff";
const CODE_CMD_MERGE: &str = "--merge";
const EXTENSION_BSL: &str = "bsl";

/// copy/rename mutable slice of files (set extension)
fn set_ext_to_all(files: &mut [&mut PathBuf], extension: &str, rename_files: bool) -> Result<()> {
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
fn remove_all_files<'a, I: Iterator<Item = &'a mut PathBuf>>(files: I) -> Result<()> {
    for file in files {
        fs::remove_file(file)?
    }
    Ok(())
}

/// diff 2 files
fn command_diff(
    mut base_cfg: PathBuf,
    mut second_cfg: PathBuf,
    remove_files: bool,
    rename_files: bool,
) -> Result<i32> {
    let mut files = [&mut base_cfg, &mut second_cfg];
    set_ext_to_all(&mut files, EXTENSION_BSL, rename_files)?;

    let mut args: Vec<&str> = vec![];
    args.extend_from_slice(&COMMON_CODE_ARGS);
    args.push(CODE_CMD_DIFF);
    args.extend(files.iter().map(|p| p.to_str().unwrap()));

    let status = process::Command::new(CODE_CMD).args(args).status()?;

    if remove_files {
        remove_all_files(files.into_iter())?;
    }

    Ok(status.code().unwrap_or_default())
}

/// merge 3 files into 1
fn command_merge(
    mut base_cfg: PathBuf,
    mut second_cfg: PathBuf,
    mut old_vendor_cfg: PathBuf,
    mut merged: PathBuf,
    remove_files: bool,
    rename_files: bool,
    from_second: bool,
) -> Result<i32> {
    let merged_orig = merged.clone();
    let default_src = if from_second { &base_cfg } else { &second_cfg };
    fs::copy(default_src, &merged)?;

    let mut files = [
        &mut base_cfg,
        &mut second_cfg,
        &mut old_vendor_cfg,
        &mut merged,
    ];
    set_ext_to_all(&mut files, EXTENSION_BSL, rename_files)?;

    let merged_new = merged.clone();

    let files = [
        &mut base_cfg,
        &mut second_cfg,
        &mut old_vendor_cfg,
        &mut merged,
    ];

    let mut args: Vec<&str> = vec![];
    args.extend_from_slice(&COMMON_CODE_ARGS);
    args.push(CODE_CMD_MERGE);
    args.extend(files.iter().map(|p| p.to_str().unwrap()));

    let status = process::Command::new(CODE_CMD).args(args).status()?;

    if merged_orig != merged_new {
        fs::rename(merged_new, merged_orig)?;
    }

    if remove_files {
        remove_all_files(files.into_iter().take(3))?;
    }

    Ok(status.code().unwrap_or_default())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let status_code = match cli.command {
        Action::Diff {
            base_cfg,
            second_cfg,
        } => command_diff(base_cfg, second_cfg, cli.remove_files, cli.rename_files)?,
        Action::Merge {
            base_cfg,
            second_cfg,
            old_vendor_cfg,
            merged,
            from_second,
        } => command_merge(
            base_cfg,
            second_cfg,
            old_vendor_cfg,
            merged,
            cli.remove_files,
            cli.rename_files,
            from_second,
        )?,
    };

    process::exit(status_code);
}
