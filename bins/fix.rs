use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let path = env::args().nth(1).unwrap_or(".".to_owned());
    fix(&path);
}

fn fix(path: impl AsRef<Path>) {
    for dir in fs::read_dir(path).unwrap() {
        if let Ok(f) = dir {
            let path = f.path();

            if path.is_dir() {
                fix(path);
                continue;
            }

            if path.extension().unwrap_or_default() != "rs" {
                continue;
            }

            let mut text = fs::read_to_string(&path).unwrap_or_default();

            if text.contains("&mut Context") {
                text = text.replace("&mut Context", "&Context");
                fs::write(path, text).unwrap();
            }
        }
    }
}
