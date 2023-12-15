use post_mortem::*;
use std::io;
use windows::{
    core::HSTRING,
    Win32::UI::WindowsAndMessaging::{self, MB_ICONWARNING, MB_OK},
};

fn main() {
    let mut stream = io::stdin().lock();
    if let Err(err) = real_main(&mut stream) {
        let message = HSTRING::from(&format!("{err:?}"));
        _ = unsafe {
            WindowsAndMessaging::MessageBoxW(None, &message, None, MB_OK | MB_ICONWARNING)
        };
    }
}
