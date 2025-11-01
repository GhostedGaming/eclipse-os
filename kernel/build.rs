use std::fs;

fn main() {
    let entries = match fs::read_dir("src") {
        Ok(e) => e,
        Err(e) => {
            println!("cargo:error=Error reading src/: {}", e);
            return;
        }
    };

    let mut found_c = false;
    let mut build = cc::Build::new();

    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    
    // Tell cargo to pass the linker script to the linker..
    println!("cargo:rustc-link-arg=-Tlinker-{arch}.ld");
    // ..and to re-run if it changes.
    println!("cargo:rerun-if-changed=linker-{arch}.ld");

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "c" || ext == "C" {
                println!("cargo:warning=Found: C file: {}", path.display());
                build.file(&path);
                found_c = true;
            }
        }
    }

    if found_c {
        println!("cargo:warning=Compiling C sources...");
        build.compile("clib");
        println!("cargo:rustc-link-lib=static=clib");
    } else {
        println!("cargo:warning=No C files found in src/");
    }

    println!("cargo:warning=Finished Compiling!");
}