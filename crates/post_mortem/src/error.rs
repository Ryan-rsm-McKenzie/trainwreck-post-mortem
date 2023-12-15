use windows::{
    core::HSTRING,
    Win32::UI::WindowsAndMessaging::{self, MB_ICONWARNING, MB_OK},
};

pub fn report_error(err: &anyhow::Error) {
    let message = HSTRING::from(&format!("{err:?}"));
    _ = unsafe { WindowsAndMessaging::MessageBoxW(None, &message, None, MB_OK | MB_ICONWARNING) };
}
