use std::io;
use trainwreck_post_mortem::*;

fn main() -> anyhow::Result<()> {
    let mut stream = io::stdin().lock();
    real_main(&mut stream)
}
