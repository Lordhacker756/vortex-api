use tower_sessions::{
    cookie::{time::Duration, SameSite},
    Expiry, MemoryStore, SessionManagerLayer,
};

pub fn init_session() -> SessionManagerLayer<MemoryStore> {
    let session_store = MemoryStore::default();
    SessionManagerLayer::new(session_store)
        .with_name("webauthnrs")
        .with_same_site(SameSite::None)
        .with_secure(true)
        .with_path("/")
        .with_domain("vortex-api-koba.onrender.com")
        .with_expiry(Expiry::OnInactivity(Duration::seconds(3600)))
}
