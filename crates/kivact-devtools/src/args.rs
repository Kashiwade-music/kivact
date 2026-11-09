//! Role: The command line arguments of the DevTools app.

use std::path::PathBuf;

/// What the DevTools app was asked to do.
pub struct Args {
    /// The discovery file to connect to; `None` lists the running targets.
    pub target: Option<PathBuf>,
}

impl Args {
    /// Read `--target <path>`; every other argument is an error.
    pub fn parse(mut args: impl Iterator<Item = String>) -> Result<Args, String> {
        let mut target = None;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--target" => {
                    let Some(value) = args.next() else {
                        return Err("--target needs a path".to_owned());
                    };
                    target = Some(PathBuf::from(value));
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }
        Ok(Args { target })
    }
}
