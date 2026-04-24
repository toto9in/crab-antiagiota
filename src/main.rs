use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    crab_antiagiota::seed::run().await?;

    let state = crab_antiagiota::state::AppState::from_env().await?;
    let app = crab_antiagiota::api::router(state);
    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("Falha ao vincular o listener TCP");

    axum::serve(listener, app)
        .await
        .expect("Servidor crab-antiagiota falhou :-(");

    Ok(())
}
