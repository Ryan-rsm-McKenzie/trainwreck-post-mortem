#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

use anyhow::Context as _;
use eulogy::protocol::Eulogy;
use std::{io::Read, path::Path};
use windows::{
    core::HSTRING,
    Win32::UI::{
        Shell,
        WindowsAndMessaging::{self, IDYES, MB_DEFBUTTON2, MB_ICONWARNING, MB_YESNO, SW_SHOW},
    },
};

mod upload;

enum Action {
    OpenCrashLog,
    UploadCrashLog,
}

fn report_crash(crash_log: &Path) -> anyhow::Result<Action> {
    let message = format!(
        "The game has crashed.\n\n\
		A crash log has been written to: {}\n\n\
		Would you like to upload your crash log to pastebin for easy sharing?",
        crash_log.to_string_lossy()
    );
    let message = HSTRING::from(message);
    let result = unsafe {
        WindowsAndMessaging::MessageBoxW(
            None,
            &message,
            None,
            MB_YESNO | MB_DEFBUTTON2 | MB_ICONWARNING,
        )
    };

    match result {
        IDYES => {
            let url = upload::upload_crash_log(crash_log).context("Failed to upload crash log")?;
            let url = HSTRING::from(url.0);
            unsafe { Shell::ShellExecuteW(None, None, &url, None, None, SW_SHOW) };
            Ok(Action::UploadCrashLog)
        }
        _ => Ok(Action::OpenCrashLog),
    }
}

pub fn real_main<R: Read>(stream: &mut R) -> anyhow::Result<()> {
    let eulogy = Eulogy::read_from(stream).context("Failed to read eulogy from stdin")?;
    let result = report_crash(&eulogy.crash_log_path);
    match result {
        Ok(Action::OpenCrashLog) | Err(_) => {
            let parameters = HSTRING::from(eulogy.crash_log_path.as_os_str());
            unsafe { Shell::ShellExecuteW(None, None, &parameters, None, None, SW_SHOW) };
        }
        Ok(Action::UploadCrashLog) => (),
    };
    result.map(|_| ())
}
