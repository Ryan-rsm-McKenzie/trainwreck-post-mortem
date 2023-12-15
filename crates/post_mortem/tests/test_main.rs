use anyhow::Context as _;
use eulogy::protocol::Eulogy;
use post_mortem as pm;
use std::{
    fs::File,
    io::{Cursor, Write},
    path::Path,
};

#[test]
fn test_main() -> anyhow::Result<()> {
    let eulogy = {
        let crash_log_path = Path::new(env!("CARGO_TARGET_TMPDIR")).with_file_name("crash.log");
        let mut file = File::create(&crash_log_path).context("Failed to create crash log")?;
        file.write_all(b"Hello world!")
            .context("Failed to write to crash log")?;
        Eulogy { crash_log_path }
    };
    let mut stream = Cursor::new(Vec::<u8>::new());
    eulogy
        .write_to(&mut stream)
        .context("Failed to write eulogy to stream")?;
    stream.set_position(0);

    pm::real_main(&mut stream).context("Failure in real main")
}
