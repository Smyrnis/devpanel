mod state;

pub use state::{FirstRunState, mark_done};

#[allow(dead_code)]
pub fn is_first_run() -> bool {
    state::is_first_run()
}
