use std::sync::Once;

static INIT: Once = Once::new();

pub fn init_test_env() {
    INIT.call_once(|| {
        let _ = std::fs::create_dir_all("./test_logs");
        let _ = std::fs::create_dir_all("./test_snapshots");
    });
}
