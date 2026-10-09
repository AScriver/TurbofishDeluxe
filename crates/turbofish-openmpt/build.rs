fn main() {
    let directory = std::env::var("OPENMPT_LIB_DIR")
        .expect("OPENMPT_LIB_DIR must point to the official libopenmpt 0.8.9 amd64 import library");
    println!("cargo:rustc-link-search=native={directory}");
    println!("cargo:rerun-if-env-changed=OPENMPT_LIB_DIR");
}
