use std::{collections::HashMap, sync::Arc};
use takumi::prelude::{Fonts, ImageSource};
use tokio::sync::RwLock;

use crate::config::Config;

pub struct AppState {
    pub fonts: Arc<RwLock<Fonts>>,
    pub images: Arc<RwLock<HashMap<Arc<str>, ImageSource>>>,
    pub fetched_bytes: Arc<RwLock<HashMap<String, Arc<[u8]>>>>,
    pub config: Config,
    pub fonts_loaded: usize,
}

impl AppState {
    pub fn new(config: Config, fonts: Fonts, fonts_loaded: usize) -> Self {
        Self {
            fonts: Arc::new(RwLock::new(fonts)),
            images: Arc::new(RwLock::new(HashMap::new())),
            fetched_bytes: Arc::new(RwLock::new(HashMap::new())),
            config,
            fonts_loaded,
        }
    }
}

pub type SharedState = Arc<AppState>;
