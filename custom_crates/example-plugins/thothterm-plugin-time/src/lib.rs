use std::time::{SystemTime, UNIX_EPOCH};

#[no_mangle]
pub extern "C" fn on_startup() {
    println!("⏱ time plugin ready");
}

#[no_mangle]
pub extern "C" fn on_command_complete() {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    println!("{:02}:{:02}:{:02}", h, m, s);
}
