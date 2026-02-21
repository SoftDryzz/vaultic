# Vaultic — Fases de Desarrollo

Resumen de cada fase de desarrollo, su alcance y estado actual.
Para la especificación arquitectónica detallada, consulta la documentación interna del proyecto.

> **[English](phases.md)** | Español

---

## Fase 1 — Fundación ✅

Establece el esqueleto del proyecto y los límites arquitectónicos.

- **Arquitectura hexagonal** estructurada: `core/` (dominio), `adapters/` (implementaciones), `cli/` (presentación), `config/`
- **Capa de dominio** definida: modelos, traits (puertos), firmas de servicios y manejo de errores tipado
- **Parseo CLI** con clap: los 10 comandos registrados con flags globales (`--cipher`, `--env`, `--verbose`)
- **Pipelines CI/CD** configurados: format + lint + test en tres plataformas; workflow de release para binarios y crates.io
- **Metadatos del proyecto**: README con badges, licencia AGPL-3.0, `.gitignore`

---

## Fase 2 — Cifrado ✅

Implementa el motor de cifrado principal con soporte dual de backends.

- **Backend age** (`AgeBackend`): cifrado/descifrado usando X25519 + ChaCha20-Poly1305 con salida ASCII-armored
- **Backend GPG** (`GpgBackend`): integración shell con GPG del sistema, sin dependencias C
- **Strategy pattern** operativo: selección de backend vía flag `--cipher age|gpg`, el mismo servicio orquesta ambos
- **Gestión de claves**: `vaultic keys setup/add/list/remove` — onboarding interactivo + gestión de recipients
- **`vaultic init`** crea la estructura del directorio `.vaultic/` con detección y generación interactiva de claves
- **27 tests**: 15 unitarios (backends + key store) + 12 de integración (flujos CLI completos)

---

## Fase 3 — Diff y Check ✅

Añade capacidades de detección de variables y comparación de archivos.

- **Parser dotenv** (`DotenvParser`): parseo y serialización de archivos `.env` preservando comentarios, líneas vacías y orden con enum `Line` (`Entry`/`Comment`/`Blank`)
- **Comando check**: compara `.env` local contra `.env.template` — reporta variables faltantes, extra y con valores vacíos con conteos resumidos
- **Comando diff**: compara dos archivos de secretos mostrando claves añadidas, eliminadas y modificadas en tabla formateada
- **Output con colores**: tablas formateadas e indicadores de estado para resultados de diff/check
- **38 tests**: 27 unitarios (dotenv parser + diff service + check service) + 11 de integración (comandos check y diff)

---

## Fase 4 — Multi-entorno y Herencia ✅

Habilita gestión de entornos por capas con resolución inteligente.

- **Resolver de entornos** (`EnvResolver`): merge multi-nivel (base → shared → dev) con semántica overlay-wins y 13 tests unitarios
- **Entornos por configuración**: `AppConfig::load()` lee definiciones de entornos y cadenas de herencia desde `config.toml`
- **`vaultic resolve --env <env>`**: descifra capas en memoria, mergea de raíz a hoja, escribe `.env` resuelto
- **Diff entre entornos**: `vaultic diff --env dev --env prod` descifra y resuelve ambos entornos antes de comparar
- **Detección de herencia circular**: error con diagnóstico claro cuando se encuentran ciclos (ej. `dev → staging → dev`)
- **Descifrado en memoria**: `decrypt_to_bytes` evita archivos temporales durante la resolución
- **Flag `--env` repetible**: `Vec<String>` permite sintaxis `--env dev --env prod`
- **25 tests**: 13 unitarios (merge del resolver, cadena, ciclos) + 6 de integración (resolve, env-diff) + 6 tests de truncate existentes

---

## Fase 5 — Auditoría y Pulido ✅

Completa el conjunto de funcionalidades con audit log, reporte de estado y pulido de UX.

- **Audit logger** (`JsonAuditLogger`): JSON lines append-only en `.vaultic/audit.log` con consultas filtradas por autor y fecha
- **Cableado del audit**: todos los comandos registran entradas de auditoría vía módulo compartido `audit_helpers` con logging no bloqueante y resolución de identidad git
- **`vaultic log`** con filtros: `--author` (nombre/email, case-insensitive), `--since` (ISO 8601), `--last N`
- **`vaultic status`**: dashboard completo del proyecto — config, recipients, entornos cifrados con tamaño, estado local (.env, template, gitignore), conteo de entradas audit
- **Git pre-commit hook**: `vaultic hook install/uninstall` — bloquea archivos `.env` en texto plano antes de commitear, instalación segura con detección de hooks ajenos
- **Mensajes de error descriptivos**: cada error incluye causa, contexto y siguiente paso sugerido
- **30 nuevos tests**: 16 unitarios (9 audit logger + 7 git hook) + 14 integración (audit, log, status, hook)
- Eliminado `#![allow(dead_code)]` global — anotaciones específicas en superficie API reservada

---

## Milestone: Estabilidad ✅

Corrige crashes, conecta los flags CLI declarados, cierra gaps de funcionalidad y endurece la validación de entrada.

- **Corrección de bugs**: panic de `truncate_key` en caracteres no-ASCII, flags `--all`/`--key` faltantes, acciones de auditoría de hooks, columna de autor en log
- **Flags CLI conectados**: `--quiet`/`--verbose` en todos los comandos, `--config` para directorio vaultic personalizado
- **Paridad GPG**: el backend GPG ahora funciona en resolve, diff, detección en init y keys setup
- **Gestión de claves**: `keys setup` ofrece importar clave age existente y opciones de keyring GPG; `keys add` valida formato de clave antes de almacenar
- **Integridad del audit**: `state_hash` SHA-256 registrado para operaciones encrypt/decrypt
- **Dashboard de estado**: sección "Your key" mostrando ubicación de clave privada, clave pública y estado en lista de recipients
- **Validación de entrada**: nombres de entorno restringidos a `[a-zA-Z0-9_-]`, nombre de archivo de audit log validado contra separadores de ruta — previene path traversal desde CLI y archivos de configuración comprometidos

---

## Milestone: Pulido ✅

Limpia dependencias sin uso, mejora los diagnósticos de error y añade refinamientos de UX.

- **Limpieza de dependencias**: eliminado crate `similar` sin uso; `indicatif` ahora usado para spinners en encrypt/decrypt
- **Calidad de mensajes de error**: las 5 variantes de error objetivo ahora siguen el patrón causa + contexto + solución; `EnvironmentNotFound` lista los entornos disponibles
- **Ayuda enriquecida**: ayuda detallada por comando con descripciones y ejemplos de uso vía clap `long_about` + `after_help`
- **Compatibilidad dotenv**: soporte para sintaxis `export KEY=value` en el parser para archivos `.env` estilo shell

---

## Milestone: Release 🔲

Validación final y publicación de v1.0.0.

- **Bump de versión**: actualizar `Cargo.toml`, CHANGELOG y referencias en README
- **Verificación CI**: pasar en Linux, macOS, Windows — fmt, clippy, test
- **Publicación**: tag `v1.0.0`, lanzar workflow de release, verificar binarios y crates.io

---

## Leyenda

| Símbolo | Significado |
|---------|-------------|
| ✅ | Completado |
| 🔲 | Planificado |
