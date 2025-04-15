# Ovejas Project
Proyecto para la memoria de título de Ingeniería Civil Electrónica. Este repositorio sirve como implementación de referencia al documento adjunto.

## Proyecto de ejemplo (python\_example\_project/)
Proyecto de ejemplo para ser usado con la herramienta de línea de comandos.

### Requisitos
* [Poetry](https://python-poetry.org/docs/#installation)

### Instalación
En el directorio `python_example_project/` es necesario ejecutar:

```bash
poetry install
```

## Servidor (server/)
Proyecto que envía los estados objetivo desde la herramienta de línea de comandos a los agentes.

### Requisitos
* [Diesel CLI](https://diesel.rs/guides/getting-started.html#installing-diesel-cli)
* OpenSSL en Linux

### Instalación
En el directorio `server/` es necesario ejecutar:

```bash
export DATABASE_URL='<nombre de la base de datos>.db' # Base de datos de sqlite
diesel database --setup migrations
cargo build
cargo run
```

### Variables de entorno
El servidor recibe las siguientes variables de entorno:
* `DATABASE_URL`: URL de la base de datos SQLite (Obligatoria)
* `PORT`: Puerto del servidor (Opcional; 9734 por defecto)
* `ADDRESS`: Dirección del servidor (Opcional; 127.0.0.1 por defecto)

Las variables de entorno se pueden pasar mediante un archivo `.env` o mediante un archivo `config.yaml` en el directorio desde que se ejecute el servidor.

## CLI (cli/)
Herramienta por interfaz de línea de comandos para levantar o bajar la infraestructura definida en un proyecto de Python.

### Requisitos
* Python 3.12 (recomendado usar pyenv)
* OpenSSL en Linux

### Instalación
En el directorio `cli/` es necesario ejecutar:
```bash
cargo install --path .
```

## Shared (shared/)
Biblioteca compartida por el servidor y el agente para la serialización/deserialización de los datos.

## Agente (device/)
Proyecto que funciona como agente en el dispositivo y recibe las actualizaciones de infraestructura desde el servidor.

### Requisitos
* OpenSSL en Linux

### Instalación
En el directorio `device/` es necesario ejecutar:

```bash
diesel generate run
cargo build
sudo -E ./target/debug/device
```

## Infraestructura (infra/)
Proyecto de OpenTofu que levanta un agente en un servicio de nube
