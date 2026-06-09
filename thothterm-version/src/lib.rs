pub fn thothterm_version() -> &'static str {
    // See build.rs
    env!("THOTHTERM_CI_TAG")
}

pub fn thothterm_target_triple() -> &'static str {
    // See build.rs
    env!("THOTHTERM_TARGET_TRIPLE")
}
