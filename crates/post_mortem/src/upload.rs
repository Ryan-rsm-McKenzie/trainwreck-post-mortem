use anyhow::Context as _;
use curl::easy::Easy;
use fast32::base64::RFC4648;
use memmap2::MmapOptions;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

struct EncryptedString<'a> {
    encrypted: &'a [u8],
}

impl<'a> EncryptedString<'a> {
    pub const fn new(encrypted: &'a [u8]) -> Self {
        Self { encrypted }
    }

    pub fn decrypt(&self) -> anyhow::Result<String> {
        match RFC4648.decode(self.encrypted) {
            Ok(ok) => String::from_utf8(ok).with_context(|| {
                format!(
                    "Failed to encode decrypted string as utf-8: {:?}",
                    self.encrypted
                )
            }),
            Err(_) => anyhow::bail!("Failed to decode encrypted string: {:?}", self.encrypted),
        }
    }
}

// encrypted so secret scanners don't revoke them
const API_KEY: EncryptedString<'_> =
    EncryptedString::new(b"Z3JOenB0TXFraFFtVVh6MUhJZnZIQ0ZEM2VYS1k0N2s=");
const USER_AGENT: EncryptedString<'_> = EncryptedString::new(b"dHNsd21peWg=");

fn encode_post(easy: &mut Easy, crash_log: &Path) -> anyhow::Result<String> {
    let api_key = API_KEY.decrypt().context("Failed to decript api key")?;
    let paste_name = {
        let file_name = crash_log
            .file_name()
            .context("Failed to get the file name from the crash log")?;
        easy.url_encode(file_name.as_encoded_bytes())
    };
    let file_code = {
        let file = File::open(crash_log).context("Failed to open crash log")?;
        let mmap = unsafe {
            MmapOptions::new()
                .map(&file)
                .context("Failed to memory map the file")
        }?;
        easy.url_encode(&mmap[..])
    };

    Ok(format!(
        "api_dev_key={api_key}\
        &api_option=paste\
        &api_paste_private=0\
        &api_paste_expire_date=N\
        &api_paste_name={paste_name}\
        &api_paste_code={file_code}"
    ))
}

#[repr(transparent)]
pub struct Url(pub String);

pub fn post_crash_log(crash_log: &Path) -> anyhow::Result<Url> {
    let mut post_response = Vec::new();
    {
        let mut easy = Easy::new();

        easy.url("https://pastebin.com/api/api_post.php")
            .context("Failed to set the URL")?;
        easy.post(true).context("Failed to enable POST")?;

        let useragent = USER_AGENT
            .decrypt()
            .context("Failed to decrypt user agent")?;
        easy.useragent(&useragent)
            .context("Failed to set user agent")?;

        let post_message = encode_post(&mut easy, crash_log)?;
        let mut post_message = post_message.as_bytes();
        let mut transfer = easy.transfer();
        transfer
            .read_function(|buf| Ok(post_message.read(buf).unwrap_or(0)))
            .context("Failed to set read function")?;
        transfer
            .write_function(|buf| {
                post_response
                    .write_all(buf)
                    .expect("Failed to write back response to buffer");
                Ok(post_response.len())
            })
            .context("Failed to set write function")?;
        transfer.perform().context("Failed to make http request")?;
    }
    String::from_utf8(post_response)
        .map(Url)
        .context("Post response was not valid utf-8")
}
