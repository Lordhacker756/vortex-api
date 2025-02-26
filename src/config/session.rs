use tower_sessions::{
    cookie::{time::Duration, SameSite},
    Expiry, MemoryStore, SessionManagerLayer,
};

pub fn init_session() -> SessionManagerLayer<MemoryStore> {
    let session_store = MemoryStore::default();
    SessionManagerLayer::new(session_store)
        .with_name("webauthnrs")
        .with_same_site(SameSite::None) // Required for cross-origin
        .with_secure(true) // Required for cross-origin
        .with_domain("votx.vercel.app") // Set to frontend domain
        .with_path("/")
        .with_expiry(Expiry::OnInactivity(Duration::seconds(3600)))
}
