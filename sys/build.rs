use std::env;
use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let whisper_root = out.join("april-asr/");

    if !whisper_root.exists() {
        std::fs::create_dir_all(&whisper_root).unwrap();
        fs_extra::dir::copy("./april-asr", &out, &Default::default()).unwrap_or_else(|e| {
            panic!(
                "Failed to copy whisper sources into {}: {}",
                whisper_root.display(),
                e
            )
        });
    }

    if env::var("APRIL_DONT_GENERATE_BINDINGS").is_ok() {
        let _: u64 = std::fs::copy("src/bindings.rs", out.join("bindings.rs"))
            .expect("Failed to copy bindings.rs");
    } else {
        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            .clang_arg("-I./april-asr")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate();

        match bindings {
            Ok(b) => {
                let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
                b.write_to_file(out_path.join("bindings.rs"))
                    .expect("Couldn't write bindings!");
            }
            Err(e) => {
                println!("cargo:warning=Unable to generate bindings: {}", e);
                println!("cargo:warning=Using bundled bindings.rs, which may be out of date");
                // copy src/bindings.rs to OUT_DIR
                std::fs::copy("src/bindings.rs", out.join("bindings.rs"))
                    .expect("Unable to copy bindings.rs");
            }
        }
    };

    cmake::Config::new(&whisper_root).build();

    let out_dir = env::var("OUT_DIR").expect("Expecting output directory");
    let out_dir = PathBuf::from(out_dir);
    let input_dir = out_dir.join("build/libaprilasr_static.a");
    let output_dir = out_dir.join("libaprilasr.a");
    std::fs::copy(input_dir, output_dir).expect("failed to copy to OUT_DIR");

    println!("cargo:rustc-link-lib=static=aprilasr");
    println!("cargo:rustc-link-lib=dylib=onnxruntime");
    println!("cargo:rustc-link-search={}", out_dir.display());
}
