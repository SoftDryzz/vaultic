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

## Fase 2 — Cifrado 🔲

Implementa el motor de cifrado principal con soporte dual de backends.

- **Backend age** (`AgeBackend`): cifrado/descifrado usando X25519 + ChaCha20-Poly1305
- **Backend GPG** (`GpgBackend`): cifrado/descifrado usando el keyring GPG del sistema
- **Strategy pattern** operativo: selección de backend vía flag `--cipher age|gpg`
- **Gestión de claves**: `vaultic keys add`, `keys list`, `keys remove` — gestión de recipients autorizados
- **`vaultic init`** crea la estructura del directorio `.vaultic/` con `config.toml` y `recipients.txt`

---

## Fase 3 — Diff y Check 🔲

Añade capacidades de detección de variables y comparación de archivos.

- **Parser dotenv** (`DotenvParser`): parseo y serialización de archivos `.env` preservando comentarios y orden
- **Comando check**: compara `.env` local contra `.env.template` — reporta variables faltantes, extra y vacías
- **Comando diff**: compara dos archivos de secretos mostrando claves añadidas, eliminadas y modificadas
- **Output con colores**: tablas formateadas e indicadores de estado para resultados de diff/check
- **Tests de integración** para todos los escenarios de comparación

---

## Fase 4 — Multi-entorno y Herencia 🔲

Habilita gestión de entornos por capas con resolución inteligente.

- **Resolver de entornos** (`EnvResolver`): merge de `base.env` + `{env}.env` con semántica de sobreescritura
- **Entornos por configuración**: lectura de definiciones de entornos y cadenas de herencia desde `config.toml`
- **`vaultic resolve --env <env>`**: genera el archivo final mergeado para un entorno dado
- **Diff entre entornos**: `vaultic diff --env dev --env prod` compara las salidas resueltas
- **Detección de herencia circular**: error con diagnóstico claro cuando se encuentran ciclos

---

## Fase 5 — Auditoría y Pulido 🔲

Completa el conjunto de funcionalidades con audit log, reporte de estado y pulido de UX.

- **Audit logger** (`JsonAuditLogger`): registra cada operación como JSON lines en `.vaultic/audit.log`
- **`vaultic log`** con filtros: `--author`, `--since`, `--last N`
- **`vaultic status`**: vista general completa del proyecto — claves, entornos, estado de sincronización, conteo de variables
- **Git pre-commit hook**: `vaultic hook install` — bloquea secretos en texto plano antes de commitear
- **Mensajes de error descriptivos**: cada error incluye causa, contexto y siguiente paso sugerido

---

## Leyenda

| Símbolo | Significado |
|---------|-------------|
| ✅ | Completado |
| 🔲 | Planificado |
