use post_mortem::*;
use std::io;

fn main() -> anyhow::Result<()> {
    let mut stream = io::stdin().lock();
    real_main(&mut stream)
}
