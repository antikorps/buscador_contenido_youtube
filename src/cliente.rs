use std::time::Duration;

pub async fn crear_cliente() -> reqwest::Client {
    let mut cabeceras = reqwest::header::HeaderMap::new();
    // Piped API devuelve 403 con la mayoría de user agent más comunes ¿?, con el de curl parece funcionar
    cabeceras.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("curl/7.81.0"),
    );

    let constructor = reqwest::ClientBuilder::new()
        .default_headers(cabeceras)
        .timeout(Duration::from_secs(7));

    let cliente = constructor
        .build()
        .expect("no se ha podido crear el cliente http");
    return cliente;
}
