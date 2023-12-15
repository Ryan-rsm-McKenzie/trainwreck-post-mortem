use winres::WindowsResource;

fn main() {
    if cfg!(target_os = "windows") {
        let res = WindowsResource::new();
        res.compile().unwrap();
    }
}
