use std::fs;

use cc::Build;

fn main() {
  for entry in fs::read_dir("languages").expect("read_dir") {
    let entry = entry.expect("entry");
    let dir = entry.path().join("src");

    Build::new()
      .include(&dir)
      .file(dir.join("parser.c"))
      .file(dir.join("scanner.c"))
      .compile(entry.file_name().to_str().expect("to_str"));
  }
}
