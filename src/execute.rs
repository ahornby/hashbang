use std::{
    ffi::OsString,
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
};

use crate::config::{HASHBANG_BINARY, HASHBANG_CONFIG_URL};

pub(crate) fn execute_command<I, A>(
    binary_path: &PathBuf,
    envs: I,
    args: A,
    script_mode: bool,
) -> ExitStatus
where
    A: IntoIterator<Item = OsString>,
    I: IntoIterator<Item = (OsString, OsString)>,
{
    // Remove so any recursive calls need their own #! to be in script mode
    let envs: Vec<(OsString, OsString)> = envs
        .into_iter()
        .filter(|(k, _)| k != HASHBANG_CONFIG_URL && k != HASHBANG_BINARY)
        .collect();

    eprintln!("Running {:?}", &binary_path);

    let mut args = args.into_iter();
    if script_mode {
        args.next(); // Remove the script name
    }

    // Pass all file descriptors through as well.
    Command::new(binary_path)
        .args(args)
        .envs(envs.into_iter())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute {:?}", &binary_path))
        .status
}
