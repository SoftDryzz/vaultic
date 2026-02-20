# Registro de cambios

Todos los cambios notables en este proyecto se documentarán en este archivo.

El formato se basa en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/),
y este proyecto sigue [Versionado Semántico](https://semver.org/lang/es/).

> **[English](../CHANGELOG.md)** | Español

## [Sin publicar]

### Planificado

- Parser dotenv con detección de variables (Fase 3)
- Comandos diff y check (Fase 3)
- Resolución multi-entorno con herencia (Fase 4)
- Audit log con JSON lines (Fase 5)
- Git pre-commit hook (Fase 5)

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
