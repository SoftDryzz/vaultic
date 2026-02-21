# Registro de cambios

Todos los cambios notables en este proyecto se documentarán en este archivo.

El formato se basa en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/),
y este proyecto sigue [Versionado Semántico](https://semver.org/lang/es/).

> **[English](../CHANGELOG.md)** | Español

## [Sin publicar]

### Milestone: Estabilidad

#### Añadido

- `vaultic encrypt --all`: re-cifra todos los entornos para los recipients actuales (rotación de claves, cambios de recipients)
- `vaultic decrypt --key <ruta>`: especifica una ubicación de clave privada personalizada
- Flags `--quiet` / `--verbose`: suprime la salida no esencial o muestra información detallada en todos los comandos
- Flag `--config <ruta>`: usa un directorio vaultic personalizado en lugar del `.vaultic/` por defecto
- Soporte GPG en `decrypt_in_memory`: `vaultic resolve --cipher gpg` y `vaultic diff --cipher gpg` ahora funcionan correctamente
- `vaultic keys setup`: importar clave age existente desde archivo (opción 2), usar clave GPG existente del keyring (opción 3, cuando GPG está disponible)
- Validación de clave pública en `vaultic keys add`: valida claves age como `x25519::Recipient`, acepta fingerprints GPG e identificadores email
- SHA-256 `state_hash` en audit log: las operaciones encrypt y decrypt ahora registran el hash del archivo resultante para verificación de integridad
- Sección "Your key" en `vaultic status`: muestra ubicación de clave privada, clave pública y si estás en la lista de recipients
- Detección de keyring GPG durante `vaultic init`: cuando no existe clave age pero GPG está disponible, ofrece elegir entre age y GPG
- Validación de entrada: nombres de entorno restringidos a `[a-zA-Z0-9_-]` para prevenir path traversal; nombre de archivo de audit log validado contra separadores de ruta

#### Corregido

- `truncate_key` ya no produce panic con caracteres no-ASCII (ej. identidades GPG con nombres como "María")
- `vaultic log` ahora muestra la columna de autor según la documentación
- Los comandos hook ahora registran acciones de auditoría `HookInstall`/`HookUninstall` en lugar de `Init`

### Milestone: Pulido

#### Añadido

- Spinners para operaciones encrypt/decrypt usando `indicatif` para feedback visual
- Ayuda enriquecida: `--help` detallado con descripciones y ejemplos de uso para todos los comandos
- Parser dotenv: soporte para sintaxis `export KEY=value` de archivos `.env` estilo shell
- Mensajes de error descriptivos: todas las variantes de error siguen el patrón "causa + contexto + solución"
- Los errores `EnvironmentNotFound` ahora listan los entornos disponibles desde la configuración

#### Cambiado

- Eliminado crate `similar` sin uso (limpieza de dependencias)

## [0.5.0-alpha] - 2026-02-21

### Añadido

- `JsonAuditLogger`: logger append-only en formato JSON lines con consultas filtradas por autor y fecha
- Cableado del audit: todos los comandos (init, encrypt, decrypt, keys, resolve, check, diff) registran entradas de auditoría
- Módulo `audit_helpers`: resolución de identidad git compartida y logging de auditoría no bloqueante
- `vaultic log`: muestra historial de auditoría con filtros `--author`, `--since` y `--last N`
- `vaultic status`: dashboard completo del proyecto mostrando config, recipients, entornos cifrados, estado local y estado del audit
- `vaultic hook install/uninstall`: hook pre-commit de git que bloquea archivos `.env` en texto plano
- Adapter `git_hook`: instalación/desinstalación segura con detección de hooks ajenos mediante comentarios marcadores
- Eliminado `#![allow(dead_code)]` global — todos los items sin uso tienen anotaciones específicas
- SECURITY.md: modelo de cifrado, respuesta ante incidentes, reporte de vulnerabilidades (inglés + español)
- CONTRIBUTING.md: acuerdo de contribución para licencia dual, guía de desarrollo (inglés + español)
- COMMERCIAL.md: FAQ de licencia dual para organizaciones (inglés + español)
- 16 nuevos tests unitarios (9 audit logger, 7 git hook)
- 14 nuevos tests de integración (audit, log, status, hook)

## [0.4.0-alpha] - 2026-02-20

### Añadido

- `EnvResolver`: herencia multi-nivel de entornos con lógica de merge (overlay gana sobre base) y detección de dependencias circulares
- `AppConfig::load()`: lectura y parseo de `.vaultic/config.toml` con definiciones de entornos
- `vaultic resolve --env <env>`: resuelve la cadena completa de herencia, descifra capas en memoria y escribe `.env` mergeado
- `vaultic diff --env dev --env prod`: compara dos entornos resueltos lado a lado
- `decrypt_to_bytes` en `EncryptionService`: descifrado en memoria sin escritura a disco
- Flag `--env` repetible: soporta múltiples valores para comparación de entornos
- 13 tests unitarios nuevos (merge del resolver, construcción de cadena, detección de ciclos)
- 6 tests de integración (comando resolve, diff entre entornos)

## [0.3.0-alpha] - 2026-02-20

### Añadido

- Parser dotenv (`DotenvParser`): parseo y serialización de archivos `.env` preservando comentarios, líneas vacías y orden
- Enum `Line` en el modelo (`Entry`/`Comment`/`Blank`) para round-trips sin pérdida de formato
- `DiffService`: comparación de dos archivos de secretos detectando variables añadidas, eliminadas y modificadas
- `CheckService`: validación de `.env` local contra `.env.template` reportando variables faltantes, extra y con valores vacíos
- `vaultic check`: comando CLI con output con colores para validación de template
- `vaultic diff <archivo1> <archivo2>`: comando CLI con tabla formateada mostrando diferencias de variables
- 27 tests unitarios (dotenv parser, diff service, check service)
- 11 tests de integración (comandos check y diff con rutas de error)

## [0.2.0-alpha] - 2026-02-20

### Añadido

- Backend de cifrado age (`AgeBackend`): X25519 + ChaCha20-Poly1305 con salida ASCII-armored
- Backend de cifrado GPG (`GpgBackend`): integración shell con GPG del sistema
- Key store basado en archivo (`FileKeyStore`): gestión de recipients vía `.vaultic/recipients.txt`
- `EncryptionService`: orquesta backend de cifrado + key store para cifrado/descifrado de archivos
- `KeyService`: gestiona claves de recipients a través del key store
- `vaultic init`: setup interactivo del proyecto con detección y generación de claves
- `vaultic encrypt`: cifra archivos para todos los recipients autorizados
- `vaultic decrypt`: descifra archivos usando la clave privada local
- `vaultic keys setup`: generación interactiva de claves para nuevos usuarios
- `vaultic keys add/list/remove`: gestión de recipients autorizados
- 15 tests unitarios (backend age, backend gpg, file key store)
- 12 tests de integración (init, encrypt, decrypt, keys, rutas de error)

## [0.1.0-alpha] - 2026-02-19

### Añadido

- Arquitectura hexagonal: capas `core/`, `adapters/`, `cli/`, `config/`
- Modelos de dominio: `SecretFile`, `SecretEntry`, `Environment`, `KeyIdentity`, `AuditEntry`, `DiffResult`
- Traits del core (puertos): `CipherBackend`, `ConfigParser`, `KeyStore`, `AuditLogger`
- Firmas de servicios: `EncryptionService`, `DiffService`, `CheckService`, `EnvResolver`, `KeyService`
- Manejo de errores tipado con enum `VaulticError` (11 variantes)
- Parseo CLI completo con clap: 10 comandos + flags globales
- Helpers de output con colores (`success`, `warning`, `error`, `header`)
- Pipeline CI: fmt + clippy + test en Linux, macOS, Windows
- Pipeline de release: build multiplataforma + publicación en crates.io
- Licencia AGPL-3.0
- README con badges, instalación, inicio rápido y referencia de comandos

[Sin publicar]: https://github.com/SoftDryzz/vaultic/compare/v0.1.0-alpha...HEAD
[0.1.0-alpha]: https://github.com/SoftDryzz/vaultic/releases/tag/v0.1.0-alpha
