use anyhow::Context as _;
use post_mortem as pm;
use std::{ffi::OsStr, fs::File, io::Write, path::Path};

#[test]
fn test_main() -> anyhow::Result<()> {
    let crash_log_path = {
        let crash_log_path = Path::new(env!("CARGO_TARGET_TMPDIR")).with_file_name("crash.log");
        let mut file = File::create(&crash_log_path).context("Failed to create crash log")?;
        file.write_all(b"Hello world!")
            .context("Failed to write to crash log")?;
        crash_log_path
    };
    pm::real_main([&OsStr::new("--crash-log-path"), crash_log_path.as_os_str()])
        .context("Failure in real main")
}
