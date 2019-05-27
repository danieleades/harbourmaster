use std::io::Write;
use std::{fs, path};

fn main() {
    // Generate README.md with cargo_readme.

    let mut f = fs::File::open("src/lib.rs").unwrap();
    let mut template = fs::File::open("./README.tpl").unwrap();
    let content = cargo_readme::generate_readme(
        &path::PathBuf::from("./"),
        &mut f,
        Some(&mut template),
        false,
        true,
        true,
        true,
    )
    .unwrap();

    let mut f = fs::File::create("README.md").unwrap();
    f.write_all(content.as_bytes()).unwrap();
}
