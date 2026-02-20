# Registro de cambios

Todos los cambios notables en este proyecto se documentarán en este archivo.

El formato se basa en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/),
y este proyecto sigue [Versionado Semántico](https://semver.org/lang/es/).

> **[English](../CHANGELOG.md)** | Español

## [Sin publicar]

### Planificado

- Backend de cifrado age (Fase 2)
- Backend de cifrado GPG (Fase 2)
- Comandos encrypt/decrypt operativos (Fase 2)
- Gestión de claves: añadir, listar, eliminar recipients (Fase 2)
- Parser dotenv con detección de variables (Fase 3)
- Comandos diff y check (Fase 3)
- Resolución multi-entorno con herencia (Fase 4)
- Audit log con JSON lines (Fase 5)
- Git pre-commit hook (Fase 5)

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
