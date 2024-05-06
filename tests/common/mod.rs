use core::net::SocketAddr;
use gluestick::{db::migrations, db::Database, router};
use once_cell::sync::Lazy;
use tokio::net::TcpListener;
use tokio_rusqlite::Connection;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static INIT_TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("GLUESTICK_TEST_LOG").is_ok() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_env("GLUESTICK_TEST_LOG")
                    .unwrap_or_else(|_| "info".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
});

pub struct TestApp {
    pub address: SocketAddr,
    pub db: Database,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&INIT_TRACING);

    let mut db = Database {
        conn: Connection::open_in_memory()
            .await
            .expect("Failed to establish connection with test database."),
    };

    db.conn
        .call(|conn| {
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
            conn.pragma_update(None, "busy_timeout", "5000")?;
            conn.pragma_update(None, "foreign_keys", "true")?;
            Ok(())
        })
        .await
        .expect("Failed to set pragmas on the test database.");

    migrations()
        .to_latest(&mut db.conn)
        .await
        .expect("Failed to migrate the test database.");

    // Binding to port 0 will cause the OS to scan for an available port which will then be used
    // for the bind. So this effectively runs the test server on a random, open port.
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .await
        .expect("Failed to bind test server to address.");
    let address = listener.local_addr().unwrap();

    let db_clone = db.clone();
    tokio::spawn(async move {
        axum::serve(listener, router(db_clone))
            .await
            .expect("Failed to serve test server.")
    });

    TestApp { address, db }
}