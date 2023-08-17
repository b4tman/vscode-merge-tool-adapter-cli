use std::{ffi::OsStr, io, path::Path, process::ExitStatus};

use crate::*;
use tempfile::tempdir;

struct TestComand {
    inner: process::Command,
}

impl TestComand {
    fn new() -> Self {
        Self {
            inner: process::Command::new("echo"),
        }
    }
    fn into_iner(self) -> process::Command {
        self.inner
    }
}

impl WrappedCommand for TestComand {
    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.inner.args(args);
        self
    }

    fn status(&mut self) -> io::Result<ExitStatus> {
        #[cfg(unix)]
        use std::os::unix::process::ExitStatusExt;
        #[cfg(windows)]
        use std::os::windows::process::ExitStatusExt;
        Ok(ExitStatus::from_raw(0))
    }
}

impl<C: WrappedCommand> Program<C> {
    fn new_test(action: Action, vscmd: C, remove_files: bool, rename_files: bool) -> Self {
        Self {
            remove_files,
            rename_files,
            vscmd,
            action: Some(action),
        }
    }
    fn into_vscmd(self) -> C {
        self.vscmd
    }
}

fn prepare_merge(
    base_cfg: &PathBuf,
    second_cfg: &PathBuf,
    old_vendor_cfg: &PathBuf,
    merged: &Path,
    from_second: bool,
    remove_files: bool,
    rename_files: bool,
) -> Program<TestComand> {
    // write src files
    fs::write(base_cfg, "base_cfg").unwrap();
    fs::write(second_cfg, "second_cfg").unwrap();
    fs::write(old_vendor_cfg, "old_vendor_cfg").unwrap();

    let vscmd: TestComand = TestComand::new();
    Program::new_test(
        Action::Merge {
            base_cfg: base_cfg.clone(),
            second_cfg: second_cfg.clone(),
            old_vendor_cfg: old_vendor_cfg.clone(),
            merged: merged.to_path_buf(),
            from_second,
        },
        vscmd,
        remove_files,
        rename_files,
    )
}

fn prepare_diff(
    base_cfg: &PathBuf,
    second_cfg: &PathBuf,
    remove_files: bool,
    rename_files: bool,
) -> Program<TestComand> {
    // write src files
    fs::write(base_cfg, "").unwrap();
    fs::write(second_cfg, "").unwrap();

    let vscmd: TestComand = TestComand::new();
    Program::new_test(
        Action::Diff {
            base_cfg: base_cfg.clone(),
            second_cfg: second_cfg.clone(),
        },
        vscmd,
        remove_files,
        rename_files,
    )
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}

/// test diff (no remove, no rename)
#[test]
fn test_cmd_diff() {
    let dir = tempdir().expect("tempdir");
    let base_cfg = dir.path().join("base_cfg.txt");
    let second_cfg = dir.path().join("second_cfg.txt");
    let base_cfg_bsl = base_cfg.with_extension(EXTENSION_BSL);
    let second_cfg_bsl = second_cfg.with_extension(EXTENSION_BSL);

    // create prog instance
    let mut prog = prepare_diff(&base_cfg, &second_cfg, false, false);

    // check status code
    assert_eq!(prog.run().unwrap(), 0);

    // check original files exists
    assert!(fs::metadata(&base_cfg).is_ok());
    assert!(fs::metadata(&second_cfg).is_ok());

    // check bsl files exists
    assert!(fs::metadata(&base_cfg_bsl).is_ok());
    assert!(fs::metadata(&second_cfg_bsl).is_ok());

    // check args
    let vscmd = prog.into_vscmd().into_iner();
    let mut expected: Vec<String> = vec![];
    expected.extend(COMMON_CODE_ARGS.into_iter().map(|x| x.to_string()));
    expected.extend_from_slice(&[
        CODE_CMD_DIFF.to_string(),
        base_cfg_bsl.into_os_string().into_string().unwrap(),
        second_cfg_bsl.into_os_string().into_string().unwrap(),
    ]);

    let actual: Vec<String> = vscmd
        .get_args()
        .map(|x| x.to_str().unwrap().to_string())
        .collect();
    assert_eq!(expected, actual);
}

/// test diff (with remove, no rename)
#[test]
fn test_cmd_diff_r() {
    let dir = tempdir().expect("tempdir");
    let base_cfg = dir.path().join("base_cfg.txt");
    let second_cfg = dir.path().join("second_cfg.txt");
    let base_cfg_bsl = base_cfg.with_extension(EXTENSION_BSL);
    let second_cfg_bsl = second_cfg.with_extension(EXTENSION_BSL);

    // create prog instance
    let mut prog = prepare_diff(&base_cfg, &second_cfg, true, false);

    // check status code
    assert_eq!(prog.run().unwrap(), 0);

    // check original files exists
    assert!(fs::metadata(&base_cfg).is_ok());
    assert!(fs::metadata(&second_cfg).is_ok());

    // check bsl files not exists
    assert!(fs::metadata(&base_cfg_bsl).is_err());
    assert!(fs::metadata(&second_cfg_bsl).is_err());

    // check args
    let vscmd = prog.into_vscmd().into_iner();
    let mut expected: Vec<String> = vec![];
    expected.extend(COMMON_CODE_ARGS.into_iter().map(|x| x.to_string()));
    expected.extend_from_slice(&[
        CODE_CMD_DIFF.to_string(),
        base_cfg_bsl.into_os_string().into_string().unwrap(),
        second_cfg_bsl.into_os_string().into_string().unwrap(),
    ]);

    let actual: Vec<String> = vscmd
        .get_args()
        .map(|x| x.to_str().unwrap().to_string())
        .collect();
    assert_eq!(expected, actual);
}

/// test diff (no remove, with rename)
#[test]
fn test_cmd_diff_n() {
    let dir = tempdir().expect("tempdir");
    let base_cfg = dir.path().join("base_cfg.txt");
    let second_cfg = dir.path().join("second_cfg.txt");
    let base_cfg_bsl = base_cfg.with_extension(EXTENSION_BSL);
    let second_cfg_bsl = second_cfg.with_extension(EXTENSION_BSL);

    // create prog instance
    let mut prog = prepare_diff(&base_cfg, &second_cfg, false, true);

    // check status code
    assert_eq!(prog.run().unwrap(), 0);

    // check original files not exists
    assert!(fs::metadata(&base_cfg).is_err());
    assert!(fs::metadata(&second_cfg).is_err());

    // check bsl files exists
    assert!(fs::metadata(&base_cfg_bsl).is_ok());
    assert!(fs::metadata(&second_cfg_bsl).is_ok());

    // check args
    let vscmd = prog.into_vscmd().into_iner();
    let mut expected: Vec<String> = vec![];
    expected.extend(COMMON_CODE_ARGS.into_iter().map(|x| x.to_string()));
    expected.extend_from_slice(&[
        CODE_CMD_DIFF.to_string(),
        base_cfg_bsl.into_os_string().into_string().unwrap(),
        second_cfg_bsl.into_os_string().into_string().unwrap(),
    ]);

    let actual: Vec<String> = vscmd
        .get_args()
        .map(|x| x.to_str().unwrap().to_string())
        .collect();
    assert_eq!(expected, actual);
}

/// test diff (with remove, with rename)
#[test]
fn test_cmd_diff_rn() {
    let dir = tempdir().expect("tempdir");
    let base_cfg = dir.path().join("base_cfg.txt");
    let second_cfg = dir.path().join("second_cfg.txt");
    let base_cfg_bsl = base_cfg.with_extension(EXTENSION_BSL);
    let second_cfg_bsl = second_cfg.with_extension(EXTENSION_BSL);

    // create prog instance
    let mut prog = prepare_diff(&base_cfg, &second_cfg, true, true);

    // check status code
    assert_eq!(prog.run().unwrap(), 0);

    // check original not exists
    assert!(fs::metadata(&base_cfg).is_err());
    assert!(fs::metadata(&second_cfg).is_err());

    // check bsl files not exists
    assert!(fs::metadata(&base_cfg_bsl).is_err());
    assert!(fs::metadata(&second_cfg_bsl).is_err());

    // check args
    let vscmd = prog.into_vscmd().into_iner();
    let mut expected: Vec<String> = vec![];
    expected.extend(COMMON_CODE_ARGS.into_iter().map(|x| x.to_string()));
    expected.extend_from_slice(&[
        CODE_CMD_DIFF.to_string(),
        base_cfg_bsl.into_os_string().into_string().unwrap(),
        second_cfg_bsl.into_os_string().into_string().unwrap(),
    ]);

    let actual: Vec<String> = vscmd
        .get_args()
        .map(|x| x.to_str().unwrap().to_string())
        .collect();
    assert_eq!(expected, actual);
}

/// test merge (no remove, no rename, no from_second)
#[test]
fn test_cmd_merge() {
    let dir = tempdir().expect("tempdir");
    let base_cfg = dir.path().join("base_cfg.txt");
    let second_cfg = dir.path().join("second_cfg.txt");
    let old_vendor_cfg = dir.path().join("old_vendor_cfg.txt");
    let merged = dir.path().join("merged.txt");
    let base_cfg_bsl = base_cfg.with_extension(EXTENSION_BSL);
    let second_cfg_bsl = second_cfg.with_extension(EXTENSION_BSL);
    let old_vendor_bsl = old_vendor_cfg.with_extension(EXTENSION_BSL);
    let merged_bsl = merged.with_extension(EXTENSION_BSL);

    // create prog instance
    let mut prog = prepare_merge(
        &base_cfg,
        &second_cfg,
        &old_vendor_cfg,
        &merged,
        false,
        false,
        false,
    );

    // check status code
    assert_eq!(prog.run().unwrap(), 0);

    // check original + merged files exists
    assert!(fs::metadata(&base_cfg).is_ok());
    assert!(fs::metadata(&second_cfg).is_ok());
    assert!(fs::metadata(&old_vendor_cfg).is_ok());
    assert!(fs::metadata(&merged).is_ok());

    // check bsl files exists
    assert!(fs::metadata(&base_cfg_bsl).is_ok());
    assert!(fs::metadata(&second_cfg_bsl).is_ok());
    assert!(fs::metadata(&old_vendor_bsl).is_ok());

    // check merged_bsl not exist
    assert!(fs::metadata(&merged_bsl).is_err());

    // check args
    let vscmd = prog.into_vscmd().into_iner();
    let mut expected: Vec<String> = vec![];
    expected.extend(COMMON_CODE_ARGS.into_iter().map(|x| x.to_string()));
    expected.extend_from_slice(&[
        CODE_CMD_MERGE.to_string(),
        base_cfg_bsl.into_os_string().into_string().unwrap(),
        second_cfg_bsl.into_os_string().into_string().unwrap(),
        old_vendor_bsl.into_os_string().into_string().unwrap(),
        merged_bsl.into_os_string().into_string().unwrap(),
    ]);

    let actual: Vec<String> = vscmd
        .get_args()
        .map(|x| x.to_str().unwrap().to_string())
        .collect();
    assert_eq!(expected, actual);

    // check merged (default) content
    assert_eq!(
        fs::read_to_string(&merged).unwrap(),
        fs::read_to_string(&base_cfg).unwrap()
    )
}

/// test merge (no remove, no rename, with from_second)
#[test]
fn test_cmd_merge_s() {
    let dir = tempdir().expect("tempdir");
    let base_cfg = dir.path().join("base_cfg.txt");
    let second_cfg = dir.path().join("second_cfg.txt");
    let old_vendor_cfg = dir.path().join("old_vendor_cfg.txt");
    let merged = dir.path().join("merged.txt");
    let base_cfg_bsl = base_cfg.with_extension(EXTENSION_BSL);
    let second_cfg_bsl = second_cfg.with_extension(EXTENSION_BSL);
    let old_vendor_bsl = old_vendor_cfg.with_extension(EXTENSION_BSL);
    let merged_bsl = merged.with_extension(EXTENSION_BSL);

    // create prog instance
    let mut prog = prepare_merge(
        &base_cfg,
        &second_cfg,
        &old_vendor_cfg,
        &merged,
        true,
        false,
        false,
    );

    // check status code
    assert_eq!(prog.run().unwrap(), 0);

    // check original + merged files exists
    assert!(fs::metadata(&base_cfg).is_ok());
    assert!(fs::metadata(&second_cfg).is_ok());
    assert!(fs::metadata(&old_vendor_cfg).is_ok());
    assert!(fs::metadata(&merged).is_ok());

    // check bsl files exists
    assert!(fs::metadata(&base_cfg_bsl).is_ok());
    assert!(fs::metadata(&second_cfg_bsl).is_ok());
    assert!(fs::metadata(&old_vendor_bsl).is_ok());

    // check merged_bsl not exist
    assert!(fs::metadata(&merged_bsl).is_err());

    // check args
    let vscmd = prog.into_vscmd().into_iner();
    let mut expected: Vec<String> = vec![];
    expected.extend(COMMON_CODE_ARGS.into_iter().map(|x| x.to_string()));
    expected.extend_from_slice(&[
        CODE_CMD_MERGE.to_string(),
        base_cfg_bsl.into_os_string().into_string().unwrap(),
        second_cfg_bsl.into_os_string().into_string().unwrap(),
        old_vendor_bsl.into_os_string().into_string().unwrap(),
        merged_bsl.into_os_string().into_string().unwrap(),
    ]);

    let actual: Vec<String> = vscmd
        .get_args()
        .map(|x| x.to_str().unwrap().to_string())
        .collect();
    assert_eq!(expected, actual);

    // check merged (default) content
    assert_eq!(
        fs::read_to_string(&merged).unwrap(),
        fs::read_to_string(&second_cfg).unwrap()
    )
}

/// test merge (with remove, no rename, no from_second)
#[test]
fn test_cmd_merge_r() {
    let dir = tempdir().expect("tempdir");
    let base_cfg = dir.path().join("base_cfg.txt");
    let second_cfg = dir.path().join("second_cfg.txt");
    let old_vendor_cfg = dir.path().join("old_vendor_cfg.txt");
    let merged = dir.path().join("merged.txt");
    let base_cfg_bsl = base_cfg.with_extension(EXTENSION_BSL);
    let second_cfg_bsl = second_cfg.with_extension(EXTENSION_BSL);
    let old_vendor_bsl = old_vendor_cfg.with_extension(EXTENSION_BSL);
    let merged_bsl = merged.with_extension(EXTENSION_BSL);

    // create prog instance
    let mut prog = prepare_merge(
        &base_cfg,
        &second_cfg,
        &old_vendor_cfg,
        &merged,
        false,
        true,
        false,
    );

    // check status code
    assert_eq!(prog.run().unwrap(), 0);

    // check original files +merged exists
    assert!(fs::metadata(&base_cfg).is_ok());
    assert!(fs::metadata(&second_cfg).is_ok());
    assert!(fs::metadata(&old_vendor_cfg).is_ok());
    assert!(fs::metadata(&merged).is_ok());

    // check bsl files + merged_bsl not exists
    assert!(fs::metadata(&base_cfg_bsl).is_err());
    assert!(fs::metadata(&second_cfg_bsl).is_err());
    assert!(fs::metadata(&old_vendor_bsl).is_err());
    assert!(fs::metadata(&merged_bsl).is_err());

    // check args
    let vscmd = prog.into_vscmd().into_iner();
    let mut expected: Vec<String> = vec![];
    expected.extend(COMMON_CODE_ARGS.into_iter().map(|x| x.to_string()));
    expected.extend_from_slice(&[
        CODE_CMD_MERGE.to_string(),
        base_cfg_bsl.into_os_string().into_string().unwrap(),
        second_cfg_bsl.into_os_string().into_string().unwrap(),
        old_vendor_bsl.into_os_string().into_string().unwrap(),
        merged_bsl.into_os_string().into_string().unwrap(),
    ]);

    let actual: Vec<String> = vscmd
        .get_args()
        .map(|x| x.to_str().unwrap().to_string())
        .collect();
    assert_eq!(expected, actual);

    // check merged (default) content
    assert_eq!(
        fs::read_to_string(&merged).unwrap(),
        fs::read_to_string(&base_cfg).unwrap()
    )
}

/// test merge (no remove, with rename, no from_second)
#[test]
fn test_cmd_merge_n() {
    let dir = tempdir().expect("tempdir");
    let base_cfg = dir.path().join("base_cfg.txt");
    let second_cfg = dir.path().join("second_cfg.txt");
    let old_vendor_cfg = dir.path().join("old_vendor_cfg.txt");
    let merged = dir.path().join("merged.txt");
    let base_cfg_bsl = base_cfg.with_extension(EXTENSION_BSL);
    let second_cfg_bsl = second_cfg.with_extension(EXTENSION_BSL);
    let old_vendor_bsl = old_vendor_cfg.with_extension(EXTENSION_BSL);
    let merged_bsl = merged.with_extension(EXTENSION_BSL);

    // create prog instance
    let mut prog = prepare_merge(
        &base_cfg,
        &second_cfg,
        &old_vendor_cfg,
        &merged,
        false,
        false,
        true,
    );

    // check status code
    assert_eq!(prog.run().unwrap(), 0);

    // check original files bot exists
    assert!(fs::metadata(&base_cfg).is_err());
    assert!(fs::metadata(&second_cfg).is_err());
    assert!(fs::metadata(&old_vendor_cfg).is_err());

    // check merged exist
    assert!(fs::metadata(&merged).is_ok());

    // check bsl files exists
    assert!(fs::metadata(&base_cfg_bsl).is_ok());
    assert!(fs::metadata(&second_cfg_bsl).is_ok());
    assert!(fs::metadata(&old_vendor_bsl).is_ok());

    // check merged_bsl not exist
    assert!(fs::metadata(&merged_bsl).is_err());

    let base_content = fs::read_to_string(&base_cfg_bsl).unwrap();

    // check args
    let vscmd = prog.into_vscmd().into_iner();
    let mut expected: Vec<String> = vec![];
    expected.extend(COMMON_CODE_ARGS.into_iter().map(|x| x.to_string()));
    expected.extend_from_slice(&[
        CODE_CMD_MERGE.to_string(),
        base_cfg_bsl.into_os_string().into_string().unwrap(),
        second_cfg_bsl.into_os_string().into_string().unwrap(),
        old_vendor_bsl.into_os_string().into_string().unwrap(),
        merged_bsl.into_os_string().into_string().unwrap(),
    ]);

    let actual: Vec<String> = vscmd
        .get_args()
        .map(|x| x.to_str().unwrap().to_string())
        .collect();
    assert_eq!(expected, actual);

    // check merged (default) content
    assert_eq!(fs::read_to_string(&merged).unwrap(), base_content)
}

/// test merge (with remove, with rename, no from_second)
#[test]
fn test_cmd_merge_rn() {
    let dir = tempdir().expect("tempdir");
    let base_cfg = dir.path().join("base_cfg.txt");
    let second_cfg = dir.path().join("second_cfg.txt");
    let old_vendor_cfg = dir.path().join("old_vendor_cfg.txt");
    let merged = dir.path().join("merged.txt");
    let base_cfg_bsl = base_cfg.with_extension(EXTENSION_BSL);
    let second_cfg_bsl = second_cfg.with_extension(EXTENSION_BSL);
    let old_vendor_bsl = old_vendor_cfg.with_extension(EXTENSION_BSL);
    let merged_bsl = merged.with_extension(EXTENSION_BSL);

    // create prog instance
    let mut prog = prepare_merge(
        &base_cfg,
        &second_cfg,
        &old_vendor_cfg,
        &merged,
        false,
        true,
        true,
    );

    let base_content = fs::read_to_string(&base_cfg).unwrap();

    // check status code
    assert_eq!(prog.run().unwrap(), 0);

    // check original files not exists
    assert!(fs::metadata(&base_cfg).is_err());
    assert!(fs::metadata(&second_cfg).is_err());
    assert!(fs::metadata(&old_vendor_cfg).is_err());

    // check merged exist
    assert!(fs::metadata(&merged).is_ok());

    // check bsl files not exists
    assert!(fs::metadata(&base_cfg_bsl).is_err());
    assert!(fs::metadata(&second_cfg_bsl).is_err());
    assert!(fs::metadata(&old_vendor_bsl).is_err());
    assert!(fs::metadata(&merged_bsl).is_err());

    // check args
    let vscmd = prog.into_vscmd().into_iner();
    let mut expected: Vec<String> = vec![];
    expected.extend(COMMON_CODE_ARGS.into_iter().map(|x| x.to_string()));
    expected.extend_from_slice(&[
        CODE_CMD_MERGE.to_string(),
        base_cfg_bsl.into_os_string().into_string().unwrap(),
        second_cfg_bsl.into_os_string().into_string().unwrap(),
        old_vendor_bsl.into_os_string().into_string().unwrap(),
        merged_bsl.into_os_string().into_string().unwrap(),
    ]);

    let actual: Vec<String> = vscmd
        .get_args()
        .map(|x| x.to_str().unwrap().to_string())
        .collect();
    assert_eq!(expected, actual);

    // check merged (default) content
    assert_eq!(fs::read_to_string(&merged).unwrap(), base_content)
}
