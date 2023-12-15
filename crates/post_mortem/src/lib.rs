#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

use anyhow::Context as _;
use eulogy::protocol::Eulogy;
use std::{io::Read, path::Path};
use windows::{
    core::HSTRING,
    Win32::UI::{
        Shell,
        WindowsAndMessaging::{
            self, IDYES, MB_DEFBUTTON2, MB_ICONWARNING, MB_OK, MB_YESNO, SW_SHOW,
        },
    },
};

pub mod error;
mod settings;
mod upload;

enum Action {
    OpenCrashLog,
    UploadCrashLog,
}

fn report_crash(crash_log: &Path) -> anyhow::Result<Action> {
    let prompt_upload = settings::prompt_upload();
    let (message, buttons) = {
        let mut message = format!(
            "The game has crashed.\n\nA crash log has been written to: {}",
            crash_log.to_string_lossy()
        );
        if prompt_upload {
            message += "\n\nWould you like to upload your crash log to pastebin for easy sharing?";
            (message, MB_YESNO | MB_DEFBUTTON2)
        } else {
            (message, MB_OK)
        }
    };
    let message = HSTRING::from(message);
    let result =
        unsafe { WindowsAndMessaging::MessageBoxW(None, &message, None, buttons | MB_ICONWARNING) };
    if prompt_upload {
        match result {
            IDYES => {
                let url =
                    upload::post_crash_log(crash_log).context("Failed to upload crash log")?;
                let url = HSTRING::from(url.0);
                unsafe { Shell::ShellExecuteW(None, None, &url, None, None, SW_SHOW) };
                Ok(Action::UploadCrashLog)
            }
            _ => Ok(Action::OpenCrashLog),
        }
    } else {
        Ok(Action::OpenCrashLog)
    }
}

pub fn real_main<R: Read>(stream: &mut R) -> anyhow::Result<()> {
    let eulogy = Eulogy::read_from(stream).context("Failed to read eulogy from stdin")?;
    let result = report_crash(&eulogy.crash_log_path);
    match result {
        Ok(Action::OpenCrashLog) | Err(_) => {
            if settings::auto_open_log() {
                let parameters = HSTRING::from(eulogy.crash_log_path.as_os_str());
                unsafe { Shell::ShellExecuteW(None, None, &parameters, None, None, SW_SHOW) };
            }
        }
        Ok(Action::UploadCrashLog) => (),
    };
    result.map(|_| ())
}
