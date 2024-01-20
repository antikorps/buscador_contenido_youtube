use crate::{
    comparar,
    modelos::{self, RespuestaTranscripcion},
};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

pub async fn recuperar_transcripcion(
    State(estado): State<Arc<modelos::Estado>>,
    Json(solicitud): Json<modelos::Solicitud>,
) -> impl IntoResponse {
    if solicitud.busqueda.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "text/plain")],
            format!("se debe especificar un término de búsqueda"),
        )
            .into_response();
    }

    let busqueda = solicitud.busqueda.unwrap();

    let cliente = &estado.cliente_http;

    let r;
    match cliente.get(solicitud.url).send().await {
        Err(error) => {
            return (
                StatusCode::BAD_GATEWAY,
                [(header::CONTENT_TYPE, "text/plain")],
                format!(
                    "ha fallado la petición a YouTube para buscar la transcripción en el idioma indicado: {error}"
                ),
            )
                .into_response()
        }
        Ok(ok) => r = ok,
    }

    if r.status() != 200 {
        return (
            StatusCode::BAD_GATEWAY,
            [(header::CONTENT_TYPE, "text/plain")],
            format!(
                "la petición a YouTube para la transcripción en el idioma indicado ha devuelto un status code incorrecto: {}", r.status()
            ),
        )
            .into_response();
    }

    let respuesta;
    match r.text().await {
        Err(error) => {
            return (
                StatusCode::BAD_GATEWAY,
                [(header::CONTENT_TYPE, "text/plain")],
                format!("ha fallado la recuperación del cuerpo de la respuesta de YouTube para buscar la transcripción en el idioma indicado: {error}"),
            ).into_response()
        }
        Ok(ok) => respuesta = ok,
    }

    let transcripcion;
    let deserializacion_error: Result<modelos::Transcripcion, serde_xml_rs::Error> =
        serde_xml_rs::from_str(&respuesta);
    match deserializacion_error {
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "text/plain")],
                format!("no se ha podido deserializar la transcripción solicitada: {error}"),
            )
                .into_response()
        }
        Ok(ok) => transcripcion = ok,
    }

    let transcripcion_serializada;
    match serde_json::to_string(&transcripcion) {
        Err(error) => {
            let mensaje_error = error.to_string();
            let transcripcion_serializada_json = json!({
                "error": true,
                "mensaje_error": mensaje_error,
            });
            match serde_json::to_string(&transcripcion_serializada_json) {
                Err(error) => {
                    let mensaje_error = error.to_string().replace(r###"""###, "");
                    transcripcion_serializada = format!(
                        r###"{{"error": "imposible serializar json {}"}}"###,
                        mensaje_error
                    )
                }
                Ok(ok) => transcripcion_serializada = ok,
            }
        }
        Ok(ok) => transcripcion_serializada = ok,
    }

    let mut coincidencias = Vec::new();
    for v in transcripcion.entradas {
        if comparar::existe_coincidencia(&v.texto, &busqueda) {
            coincidencias.push(v);
        }
    }

    return (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(RespuestaTranscripcion {
            coincidencias,
            serializacion: transcripcion_serializada,
        }),
    )
        .into_response();
}
