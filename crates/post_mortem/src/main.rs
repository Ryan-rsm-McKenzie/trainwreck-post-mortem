use post_mortem::*;
use std::io;

fn main() {
    let mut stream = io::stdin().lock();
    if let Err(err) = real_main(&mut stream) {
        error::report(&err);
    }
}
