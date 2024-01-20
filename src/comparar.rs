pub fn existe_coincidencia(cadena: &str, busqueda: &str) -> bool {
    // Escapar html
    let mut cadena_normalizada = String::new();
    match htmlescape::decode_html(&cadena) {
        Err(_) => (),
        Ok(ok) => cadena_normalizada = ok,
    }
    let mut busqueda_normalizada = String::new();
    match htmlescape::decode_html(&busqueda) {
        Err(_) => (),
        Ok(ok) => busqueda_normalizada = ok,
    }

    // Minúsculas
    cadena_normalizada = cadena_normalizada.to_lowercase();
    busqueda_normalizada = busqueda_normalizada.to_lowercase();

    // Unidecode
    cadena_normalizada = unidecode::unidecode(&cadena_normalizada);
    busqueda_normalizada = unidecode::unidecode(&busqueda_normalizada);

    return cadena_normalizada.contains(&busqueda_normalizada);
}
