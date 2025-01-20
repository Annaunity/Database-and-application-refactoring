use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct Globals {
    pub data_path: Arc<Path>,
}
