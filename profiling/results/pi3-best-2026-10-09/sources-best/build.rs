fn main() {
    println!("cargo:rerun-if-changed=../../profiling/signposts.c");
    #[cfg(target_os = "macos")]
    if std::env::var_os("CARGO_FEATURE_SIGNPOSTS").is_some()
        && std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos")
    {
        cc::Build::new()
            .file("../../profiling/signposts.c")
            .compile("repro_signposts");
    }
}
