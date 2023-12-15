use crate::error;
use anyhow::Context;
use serde::Deserialize;
use std::{
    ffi::OsString,
    fs::{self},
    io,
    os::windows::prelude::*,
    path::PathBuf,
    ptr,
};
use windows::{
    core as wincore,
    Win32::{
        Foundation::{ERROR_INSUFFICIENT_BUFFER, HMODULE, MAX_PATH},
        System::{LibraryLoader, SystemServices::IMAGE_DOS_HEADER},
    },
};

extern "C" {
    static __ImageBase: IMAGE_DOS_HEADER;
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase", default)]
struct General {
    auto_open_log: bool,
    prompt_upload: bool,
}

impl Default for General {
    fn default() -> Self {
        Self {
            auto_open_log: true,
            prompt_upload: true,
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase", default)]
struct Config {
    general: General,
}

struct CachedConfig {
    config: Config,
}

impl CachedConfig {
    fn get_module_file_path(handle: HMODULE) -> anyhow::Result<PathBuf> {
        let mut filename = Vec::<u16>::new();
        filename.resize_with(MAX_PATH as usize, Default::default);
        loop {
            match unsafe { LibraryLoader::GetModuleFileNameW(handle, &mut filename[..]) } {
                0 => {
                    return Err(wincore::Error::from_win32())
                        .context("Failed to get module file name")
                }
                len => {
                    let last_error = wincore::Error::from_win32().code();
                    if last_error == ERROR_INSUFFICIENT_BUFFER.to_hresult() {
                        filename.resize_with(filename.len() * 2, Default::default);
                    } else {
                        let str = OsString::from_wide(&filename[0..len as usize]);
                        let path = PathBuf::from(str);
                        return Ok(path);
                    }
                }
            }
        }
    }

    pub fn new() -> Self {
        let image_base = unsafe { ptr::addr_of!(__ImageBase) };
        let self_handle = HMODULE(image_base as isize);
        let config = Self::get_module_file_path(self_handle)
            .and_then(|mut file_path| {
                file_path.set_file_name("post_mortem.toml");
                match fs::read_to_string(&file_path) {
                    Ok(file_contents) => toml::from_str::<Config>(&file_contents)
                        .context("Failed to parse config from file"),
                    Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Config::default()),
                    Err(err) => anyhow::bail!("Failed to open config file: {err:?}"),
                }
            })
            .unwrap_or_else(|err| {
                error::report(&err);
                Config::default()
            });
        Self { config }
    }
}

lazy_static::lazy_static! {
    static ref CONFIG: CachedConfig = CachedConfig::new();
}

pub fn auto_open_log() -> bool {
    CONFIG.config.general.auto_open_log
}

pub fn prompt_upload() -> bool {
    CONFIG.config.general.prompt_upload
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn test_config_loading() -> anyhow::Result<()> {
        let config_string = r#"
			[General]
			AutoOpenLog = true
			PromptUpload = false
		"#;
        let Config { general } = toml::from_str(config_string).context("Failed to parse config")?;
        assert_eq!(general.auto_open_log, true);
        assert_eq!(general.prompt_upload, false);
        Ok(())
    }
}
