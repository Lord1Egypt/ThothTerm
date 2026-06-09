#[no_mangle]
pub extern "C" fn on_command_complete() {
    // In a real plugin, this would call thoth_status_bar_set() via the SDK.
    // For now it just prints the git branch to stdout (captured by the host).
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let branch = String::from_utf8_lossy(&out.stdout);
            println!("git:{}", branch.trim());
        }
    }
}
