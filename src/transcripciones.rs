use crate::modelos;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

pub async fn recuperar_transcripciones(
    State(estado): State<Arc<modelos::Estado>>,
    Json(solicitud): Json<modelos::Solicitud>,
) -> impl IntoResponse {
    let cliente = &estado.cliente_http;

    let r;
    match cliente.get(solicitud.url).send().await {
        Err(error) => {
            return (
                StatusCode::BAD_GATEWAY,
                [(header::CONTENT_TYPE, "text/plain")],
                format!(
                    "ha fallado la petición a YouTube para buscar las transcripciones: {error}"
                ),
            )
                .into_response()
        }
        Ok(ok) => {
            r = ok;
        }
    }

    if r.status() != 200 {
        return (
            StatusCode::BAD_GATEWAY,
            [(header::CONTENT_TYPE, "text/plain")],
            format!("se ha obtenido un status code incorrecto al intentar recuperar las transcripciones: {}", r.status()),
        ).into_response();
    }

    let respuesta;
    match r.text().await {
        Err(error) => {
            return (
                StatusCode::BAD_GATEWAY,
                [(header::CONTENT_TYPE, "text/plain")],
                format!("ha fallado la recuperación del cuerpo de la respuesta de YouTube para buscar las transcripciones: {error}"),
            ).into_response()
        }
        Ok(ok) => {
            respuesta = ok;
        }
    }

    let exp_regular = estado.exp_reg_transcripciones.lock().await;

    match exp_regular.captures(&respuesta) {
        None => {
            return (
                StatusCode::BAD_GATEWAY,
                [(header::CONTENT_TYPE, "text/plain")],
                "no se ha encontrado ningún enlace con transcripciones",
            )
                .into_response()
        }
        Some(capturas) => {
            let transcripciones = &capturas["captions"];

            let deserializacion: Result<Vec<modelos::CaptionTrack>, serde_json::Error> =
                serde_json::from_str(&transcripciones);
            match deserializacion {
                Err(error) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        [(header::CONTENT_TYPE, "text/plain")],
                        format!(
                            "no se ha podido deserializar las transcripciones encontradas: {error}"
                        ),
                    )
                        .into_response()
                }
                Ok(ok) => {
                    if ok.len() == 0 {
                        return (
                            StatusCode::BAD_GATEWAY,
                            [(header::CONTENT_TYPE, "text/plain")],
                            "la URL de youtube no ha devuelto ningún enlace con transcripciones",
                        )
                            .into_response();
                    }
                    return (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, "application/json")],
                        Json(ok),
                    )
                        .into_response();
                }
            }
        }
    }
}
