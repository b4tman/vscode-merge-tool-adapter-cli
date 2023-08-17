use anyhow::Result;
use clap::{Parser, Subcommand};
use utils::{remove_all_files, set_ext_to_all, VSCodeComand, WrappedCommand};

use std::vec;
use std::{fs, path::PathBuf, process};

mod utils;

/// commond args for VSCode
pub const COMMON_CODE_ARGS: [&str; 4] = ["--new-window", "--sync", "off", "--wait"];
pub const CODE_CMD_DIFF: &str = "--diff";
pub const CODE_CMD_MERGE: &str = "--merge";
/// filename extension for syntax highlights
pub const EXTENSION_BSL: &str = "bsl";

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

struct Program<C: WrappedCommand> {
    remove_files: bool,
    rename_files: bool,
    vscmd: C,
    action: Option<Action>,
}

impl<C: WrappedCommand> Program<C> {
    fn new(cli: Cli, vscmd: C) -> Self {
        Self {
            remove_files: cli.remove_files,
            rename_files: cli.rename_files,
            vscmd,
            action: Some(cli.command),
        }
    }

    /// diff 2 files
    fn command_diff(&mut self, mut base_cfg: PathBuf, mut second_cfg: PathBuf) -> Result<i32> {
        let mut files = [&mut base_cfg, &mut second_cfg];
        set_ext_to_all(&mut files, EXTENSION_BSL, self.rename_files)?;

        let mut args: Vec<&str> = vec![];
        args.extend_from_slice(&COMMON_CODE_ARGS);
        args.push(CODE_CMD_DIFF);
        args.extend(files.iter().map(|p| p.to_str().unwrap()));

        let status = self.vscmd.args(args).status()?;

        if self.remove_files {
            remove_all_files(files.into_iter())?;
        }

        Ok(status.code().unwrap_or_default())
    }

    /// merge 3 files into 1
    fn command_merge(
        &mut self,
        mut base_cfg: PathBuf,
        mut second_cfg: PathBuf,
        mut old_vendor_cfg: PathBuf,
        mut merged: PathBuf,
        from_second: bool,
    ) -> Result<i32> {
        let merged_orig = merged.clone();
        let default_src = if from_second { &second_cfg } else { &base_cfg };
        fs::copy(default_src, &merged)?;

        let mut files = [
            &mut base_cfg,
            &mut second_cfg,
            &mut old_vendor_cfg,
            &mut merged,
        ];
        set_ext_to_all(&mut files, EXTENSION_BSL, self.rename_files)?;

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

        let status = self.vscmd.args(args).status()?;

        if merged_orig != merged_new {
            fs::rename(merged_new, merged_orig)?;
        }

        if self.remove_files {
            remove_all_files(files.into_iter().take(3))?;
        }

        Ok(status.code().unwrap_or_default())
    }

    fn run(&mut self) -> Result<i32> {
        match self.action.take().unwrap() {
            Action::Diff {
                base_cfg,
                second_cfg,
            } => self.command_diff(base_cfg, second_cfg),
            Action::Merge {
                base_cfg,
                second_cfg,
                old_vendor_cfg,
                merged,
                from_second,
            } => self.command_merge(base_cfg, second_cfg, old_vendor_cfg, merged, from_second),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let vscmd = VSCodeComand::new();

    let mut program = Program::new(cli, vscmd);
    let status_code = program.run()?;

    process::exit(status_code);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
