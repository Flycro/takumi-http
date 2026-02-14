use std::{borrow::Cow, fs::read, net::SocketAddr, sync::Arc};

use mimalloc::MiMalloc;
use takumi::GlobalContext;
use tokio::net::TcpListener;
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use walkdir::WalkDir;

use takumi_http::{AppState, Config, create_router};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const GEIST_FONT: &[u8] = include_bytes!("../assets/fonts/Geist[wght].woff2");
const GEIST_MONO_FONT: &[u8] = include_bytes!("../assets/fonts/GeistMono[wght].woff2");
const TWEMOJI_FONT: &[u8] = include_bytes!("../assets/fonts/TwemojiMozilla-colr.woff2");

fn load_default_fonts(context: &mut GlobalContext) -> usize {
    let mut count = 0;

    if context
        .font_context
        .load_and_store(Cow::Borrowed(GEIST_FONT), None, None)
        .is_ok()
    {
        info!("Loaded embedded font: Geist");
        count += 1;
    }

    if context
        .font_context
        .load_and_store(Cow::Borrowed(GEIST_MONO_FONT), None, None)
        .is_ok()
    {
        info!("Loaded embedded font: Geist Mono");
        count += 1;
    }

    if context
        .font_context
        .load_and_store(Cow::Borrowed(TWEMOJI_FONT), None, None)
        .is_ok()
    {
        info!("Loaded embedded font: Twemoji");
        count += 1;
    }

    count
}

fn load_fonts_from_dir(config: &Config, context: &mut GlobalContext) -> usize {
    let mut count = 0;

    if let Some(font_dir) = &config.font_dir {
        for entry in WalkDir::new(font_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "ttf" | "otf" | "woff" | "woff2") {
                continue;
            }

            match read(path) {
                Ok(data) => {
                    if let Err(e) = context.font_context.load_and_store(Cow::Owned(data), None, None) {
                        error!("Failed to load font {}: {e:?}", path.display());
                    } else {
                        info!("Loaded font: {}", path.display());
                        count += 1;
                    }
                }
                Err(e) => {
                    error!("Failed to read font file {}: {e}", path.display());
                }
            }
        }
    }

    count
}

#[tokio::main]
async fn main() {
    let config = Config::parse_args();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.log_level.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut context = GlobalContext::default();
    let mut fonts_loaded = 0;

    if config.load_default_fonts {
        fonts_loaded += load_default_fonts(&mut context);
    }

    fonts_loaded += load_fonts_from_dir(&config, &mut context);

    info!("Loaded {fonts_loaded} fonts");

    let state = Arc::new(AppState::new(config.clone(), context, fonts_loaded));

    let app = create_router(state)
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(config.body_limit));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await.unwrap();

    info!("Server listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
