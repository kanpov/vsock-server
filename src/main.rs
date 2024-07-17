use axum::{routing::get, Json, Router};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto,
    service::TowerToHyperService,
};
use serde::Deserialize;
use tokio_vsock::{VsockAddr, VsockListener};
use tower::Service;

#[derive(Deserialize)]
pub struct Name {
    first: String,
    middle: String,
    last: String,
}

async fn sample_route(Json(name): Json<Name>) -> String {
    name.first + name.middle.as_str() + name.last.as_str()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let vsock_addr = VsockAddr::new(1, 8000);
    let mut listener = VsockListener::bind(vsock_addr).unwrap();

    let app = Router::new().route("/", get(sample_route));
    let mut make_service = app.into_make_service();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let tower_service = make_service.call(&socket).await.unwrap();

        tokio::spawn(async move {
            let tokio_io = TokioIo::new(socket);
            let hyper_service = TowerToHyperService::new(tower_service);

            if let Err(err) = auto::Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(tokio_io, hyper_service)
                .await
            {
                eprintln!("Failed to serve connection: {:?}", err);
            }
        });
    }
}
