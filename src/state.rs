use std::sync::Arc;
use takumi::GlobalContext;
use tokio::sync::RwLock;

use crate::config::Config;

pub struct AppState {
    pub context: Arc<RwLock<GlobalContext>>,
    pub config: Config,
    pub fonts_loaded: usize,
}

impl AppState {
    pub fn new(config: Config, context: GlobalContext, fonts_loaded: usize) -> Self {
        Self {
            context: Arc::new(RwLock::new(context)),
            config,
            fonts_loaded,
        }
    }
}

pub type SharedState = Arc<AppState>;
