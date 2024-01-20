# Buscador de contenido en YouTube
Aplicación web para facilitar la búsqueda contenido en las transcripciones y los comentarios de un vídeo de YouTube

Descarga el ejecutable desde el apartado "Releases", ejecutalo y accede a la aplicación a través de [http://localhost:8080](http://localhost:8080)

Si el puerto está ocupado puedes asignar uno distinto con el argumento --port o -port
```bash
./buscador_contenido_youtube --port 8081
```

## Importante
La búsqueda de comentarios se hace a través de la API de Piped y exige un relativo número elevado de peticiones si el número de comentarios es alto. Esa parte exige una mejor implementación: tiempos de espera, gestión de errores (ahora es éxito o error, perdiendo todo lo recuperado si el proceso no llega satisfactoriamente al final), etc. Quizá sea necesario probar con distintas alternativas hasta dar con una opción válida.