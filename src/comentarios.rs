use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use regex::Regex;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{comparar, modelos};

pub async fn recuperar_identificador_youtube(
    cliente: &Client,
    url: &str,
    exp_reg_youtube_id: &Mutex<Regex>,
) -> Result<String, String> {
    let r;
    match cliente.get(url).send().await {
        Err(error) => {
            let mensaje_error =
                format!("ha fallado la petición ha YouTube para buscar el identificador: {error}");
            return Err(mensaje_error);
        }
        Ok(ok) => r = ok,
    }

    if r.status() != 200 {
        let mensaje_error = format!("la petición ha YouTube para buscar el identificador ha devuelto un status code incorrecto: {}", r.status());
        return Err(mensaje_error);
    }
    let respuesta;
    match r.text().await {
        Err(error) => {
            let mensaje_error = format!("no se ha podido recuperar la respuesta de la petición a YouTube para buscar el identificador: {error}");
            return Err(mensaje_error);
        }
        Ok(ok) => respuesta = ok,
    }

    let exp_reg = exp_reg_youtube_id.lock().await;

    match exp_reg.captures(&respuesta) {
        None => {
            let mensaje_error = format!("no se ha podido localizar el identificador de YouTube, intentalo con una URL más común");
            return Err(mensaje_error);
        }
        Some(ok) => {
            let id = &ok["id"];
            if id.len() != 11 {
                let mensaje_error = format!("no se ha podido localizar el identificador de 11 caracteres de YouTube, intentalo con una URL más común");
                return Err(mensaje_error);
            }
            return Ok(String::from(id));
        }
    }
}

pub async fn buscar_comentarios(
    State(estado): State<Arc<modelos::Estado>>,
    Json(solicitud): Json<modelos::Solicitud>,
) -> impl IntoResponse {
    if solicitud.piped.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "text/plain")],
            "no se ha facilitado la base url de la API de Piped",
        )
            .into_response();
    }

    if solicitud.busqueda.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "text/plain")],
            "no se ha facilitado el término de búsqueda",
        )
            .into_response();
    }

    let busqueda = solicitud.busqueda.unwrap();
    let base_piped = solicitud.piped.unwrap();

    let cliente = &estado.cliente_http;
    let youtube_id;
    match recuperar_identificador_youtube(cliente, &solicitud.url, &estado.exp_reg_youtube_id).await
    {
        Err(error) => {
            return (
                StatusCode::BAD_GATEWAY,
                [(header::CONTENT_TYPE, "text/plain")],
                error,
            )
                .into_response()
        }
        Ok(ok) => youtube_id = ok,
    }

    //
    let mut paginacion = Vec::new();
    paginacion.push(modelos::ComentariosPaginacion {
        siguiente: String::from(""),
        id_asociado: None,
    });

    let mut comentarios: Vec<modelos::Comentarios> = Vec::new();

    loop {
        let endpoint;

        let cliente_inicializado;
        if paginacion[0].siguiente == "" {
            endpoint = format!("{}/comments/{}", base_piped, youtube_id);
            cliente_inicializado = cliente.get(endpoint);
        } else {
            endpoint = format!("{}/nextpage/comments/{}", base_piped, youtube_id);
            let query_url = vec![("nextpage", &paginacion[0].siguiente)];
            cliente_inicializado = cliente.get(endpoint).query(&query_url);
        }

        let r;
        match cliente_inicializado.send().await {
            Err(error) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    [(header::CONTENT_TYPE, "text/plain")],
                    format!("ha fallado la petición para recuperar comentarios {error}"),
                )
                    .into_response()
            }
            Ok(ok) => r = ok,
        }

        if r.status() != 200 {
            return (
                    StatusCode::BAD_GATEWAY,
                    [(header::CONTENT_TYPE, "text/plain")],
                    format!("ha fallado la petición para recuperar comentarios devolviendo un status code incorrecto {}", r.status()),
                )
                    .into_response();
        }

        let respuesta;
        match r.text().await {
            Err(error) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    [(header::CONTENT_TYPE, "text/plain")],
                    format!("ha fallado la obtención del cuerpo en una petición para recuperar comentarios {error}"),
                )
                    .into_response()
            }
            Ok(ok) => respuesta = ok
        }

        let serializacion_comentarios: Result<modelos::ComentariosPiped, serde_json::Error> =
            serde_json::from_str(&respuesta);

        let c;
        match serializacion_comentarios {
            Err(error) => {
                println!("{}", respuesta);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(header::CONTENT_TYPE, "text/plain")],
                    format!("ha fallado la serialización del cuerpo en una petición para recuperar comentarios {error}"),
                )
                    .into_response()
            }
            Ok(ok) => c = ok
        }

        for v in c.comments {
            if v.replies_page.is_string() {
                let pagina_interes = v.replies_page.as_str().unwrap();

                paginacion.push(modelos::ComentariosPaginacion {
                    siguiente: String::from(pagina_interes),
                    id_asociado: Some(String::from(&v.comment_id)),
                })
            }

            // Tiene un ID asociado, eso significa que es respuesta
            if paginacion[0].id_asociado.is_some() {
                for c in comentarios.iter_mut() {
                    if c.comment_id == paginacion[0].id_asociado.clone().unwrap() {
                        if c.comments_collection.is_none() {
                            c.comments_collection = Some(vec![modelos::Comentarios {
                                author: String::from(&v.author),
                                comment_text: String::from(&v.comment_text),
                                comment_id: String::from(&v.comment_id),
                                replies_page: serde_json::Value::Null,
                                commented_time: String::from(&v.commented_time),
                                comments_collection: None,
                            }])
                        } else {
                            let coleccion_con_comentarios = c.comments_collection.as_mut().unwrap();
                            coleccion_con_comentarios.push(modelos::Comentarios {
                                author: String::from(&v.author),
                                comment_text: String::from(&v.comment_text),
                                comment_id: String::from(&v.comment_id),
                                replies_page: serde_json::Value::Null,
                                commented_time: String::from(&v.commented_time),
                                comments_collection: None,
                            })
                        }
                    }
                }
            } else {
                comentarios.push(modelos::Comentarios {
                    author: v.author,
                    comment_text: v.comment_text,
                    comment_id: v.comment_id,
                    replies_page: v.replies_page,
                    commented_time: v.commented_time,
                    comments_collection: None,
                })
            }
        }

        if c.nextpage.is_string() {
            paginacion.push(modelos::ComentariosPaginacion {
                siguiente: c.nextpage.as_str().unwrap().to_string(),
                id_asociado: None,
            })
        }

        paginacion.remove(0);

        if paginacion.is_empty() {
            break;
        }
    }

    let serializacion_respuesta: Result<String, serde_json::Error> =
        serde_json::to_string(&comentarios);
    let serializacion;
    match serializacion_respuesta {
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "text/plain")],
                format!("ha fallado la serialización de los comentarios recuperados {error}"),
            )
                .into_response()
        }
        Ok(ok) => serializacion = ok,
    }

    let mut coincidencias = Vec::new();
    let mut total: i32 = 0;
    for c in comentarios {
        total += 1;
        if comparar::existe_coincidencia(&c.comment_text, &busqueda) {
            coincidencias.push(c.clone())
        }

        if c.comments_collection.is_some() {
            let respuestas = c.comments_collection.unwrap();
            for r in respuestas {
                total += 1;
                if comparar::existe_coincidencia(&r.comment_text, &busqueda) {
                    coincidencias.push(r)
                }
            }
        }
    }

    return (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain")],
        Json(modelos::ComentariosRespuesta {
            coincidencias,
            serializacion,
            total,
        }),
    )
        .into_response();
}
