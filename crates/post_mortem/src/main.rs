use post_mortem::*;
use std::env;

fn main() {
    if let Err(err) = real_main(env::args_os()) {
        error::report(&err);
    }
}
