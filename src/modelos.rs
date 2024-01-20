use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

// Buscador de contenido en YouTube
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Argumentos {
    /// Puerto en el que se iniciará la aplicación web
    #[arg(short, long, default_value_t = 8080)]
    pub port: i64,
}
pub struct Estado {
    pub cliente_http: reqwest::Client,
    pub exp_reg_transcripciones: Mutex<Regex>,
    pub exp_reg_youtube_id: Mutex<Regex>,
}

#[derive(Serialize, Deserialize)]
pub struct Solicitud {
    pub url: String,
    pub busqueda: Option<String>,
    pub piped: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptionTrack {
    pub base_url: String,
    pub language_code: String,
}

#[derive(Serialize, Deserialize)]
pub struct RespuestaTranscripcion {
    pub coincidencias: Vec<Entrada>,
    pub serializacion: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transcripcion {
    #[serde(rename = "text")]
    pub entradas: Vec<Entrada>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entrada {
    #[serde(rename = "start")]
    pub inicio: String,
    #[serde(rename = "dur")]
    pub duracion: String,
    #[serde(rename = "$value")]
    pub texto: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComentariosPiped {
    pub comments: Vec<Comentarios>,
    pub nextpage: Value,
    pub disabled: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comentarios {
    pub author: String,
    pub comment_text: String,
    pub comment_id: String,
    #[serde(skip_serializing)]
    pub replies_page: Value,
    pub commented_time: String,
    pub comments_collection: Option<Vec<Comentarios>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComentariosPaginacion {
    pub siguiente: String,
    pub id_asociado: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ComentariosRespuesta {
    pub coincidencias: Vec<Comentarios>,
    pub total: i32,
    pub serializacion: String,
}
