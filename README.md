# Gestor de Tareas — Rust + Axum

API REST para gestionar tareas, desarrollada con Rust, Axum y PostgreSQL.

## Tecnologías

- Rust (edition 2024)
- Axum: enrutamiento HTTP
- Tokio: ejecución asíncrona
- PostgreSQL: persistencia de datos
- SQLx: consultas y migraciones
- Serde: serialización JSON
- Tower HTTP: configuración CORS

## Funcionalidades

- Crear, consultar, actualizar y eliminar tareas.
- Validar títulos, estados y prioridades.
- Consultar una tarea mediante su identificador para actualizarla.
- Responder con códigos HTTP apropiados.
- Ejecutar migraciones automáticamente al iniciar.
- Pruebas unitarias de validación.

## Requisitos

- Rust y Cargo
- PostgreSQL
- Una base de datos y un usuario con permisos sobre ella

## Configuración

1. Crea una base de datos PostgreSQL llamada `gestor_tareas_rust`.
2. Configura el usuario de la aplicación con permisos sobre esa base de datos.
3. Crea el archivo `.env` a partir de `.env.example`.
4. Ajusta `DATABASE_URL` con tus credenciales locales.

No publiques el archivo `.env` ni credenciales reales.

## Ejecutar

```bash
cargo check
cargo test
cargo run
```

La API escucha en `http://127.0.0.1:8101`.

- Estado general: `GET /`
- Salud: `GET /api/health`
- Listar tareas: `GET /api/tasks`
- Crear tarea: `POST /api/tasks`
- Actualizar tarea: `PUT /api/tasks/{id}`
- Eliminar tarea: `DELETE /api/tasks/{id}`

## Ejemplos HTTP

Crear una tarea:

```bash
curl -i -X POST http://127.0.0.1:8101/api/tasks \
  -H 'Content-Type: application/json' \
  -d '{
    "title": "Aprender Rust",
    "description": "Practicar Axum y PostgreSQL",
    "status": "pending",
    "priority": "high",
    "due_date": "2026-10-25"
  }'
```

Listar tareas:

```bash
curl http://127.0.0.1:8101/api/tasks
```

Actualizar una tarea:

```bash
curl -i -X PUT http://127.0.0.1:8101/api/tasks/1 \
  -H 'Content-Type: application/json' \
  -d '{
    "title": "Aprender Rust y Axum",
    "description": "CRUD con PostgreSQL",
    "status": "in_progress",
    "priority": "urgent",
    "due_date": "2026-10-20"
  }'
```

Eliminar una tarea:

```bash
curl -i -X DELETE http://127.0.0.1:8101/api/tasks/1
```

## Valores permitidos

Estados:
- `pending`
- `in_progress`
- `completed`

Prioridades:
- `low`
- `medium`
- `high`
- `urgent`

Los títulos deben contener entre 1 y 150 caracteres, sin contar espacios externos.

## Pruebas

Ejecutar las pruebas unitarias:

```bash
cargo test
```

Las pruebas actuales cubren la validación de títulos, estados y prioridades.

## Estructura

```text
backend-rust/
├── migrations/
│   └── 0001_create_tasks.sql
├── src/
│   └── main.rs
├── .env.example
├── .gitignore
├── Cargo.toml
├── Cargo.lock
└── README.md
```

## Seguridad y alcance

Este proyecto es una API de portafolio en desarrollo. Antes de desplegarlo públicamente, configura CORS para los orígenes autorizados, añade autenticación y autorización si se requieren usuarios, y revisa la configuración de producción.
