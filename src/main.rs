use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
mod cliente;
mod comentarios;
mod comparar;
mod index;
mod modelos;
mod transcripcion;
mod transcripciones;
use clap::Parser;
use regex::Regex;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let argumentos = modelos::Argumentos::parse();
    let cliente_http = cliente::crear_cliente().await;

    let exp_reg_transcripciones = Regex::new(r##"\{"captionTracks":(?<captions>\[.*?\])"##)
        .expect("ha fallado la creación de la expresión regular para las transcripciones");
    let exp_reg_youtube_id = Regex::new(r##"<meta itemprop="identifier" content="(?<id>.*?)">"##)
        .expect("ha fallado la creación de la expresión regular para el id de youtube");

    let estado = Arc::new(modelos::Estado {
        cliente_http,
        exp_reg_transcripciones: Mutex::new(exp_reg_transcripciones),
        exp_reg_youtube_id: Mutex::new(exp_reg_youtube_id),
    });

    let app = Router::new()
        .route("/", get(index::index))
        .route(
            "/transcripciones",
            post(transcripciones::recuperar_transcripciones),
        )
        .route(
            "/transcripcion",
            post(transcripcion::recuperar_transcripcion),
        )
        .route("/comentarios", post(comentarios::buscar_comentarios))
        .with_state(estado);

    let direccion_local = format!("0.0.0.0:{}", argumentos.port);
    let manejador_web = tokio::net::TcpListener::bind(direccion_local)
        .await
        .unwrap();
    println!(
        "Aplicación web iniciada en http://0.0.0.0:{}",
        argumentos.port
    );
    axum::serve(manejador_web, app).await.unwrap();
}
