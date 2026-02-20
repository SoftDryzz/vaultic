# Vaultic

[![CI](https://github.com/SoftDryzz/vaultic/workflows/CI/badge.svg)](https://github.com/SoftDryzz/vaultic/actions)
[![crates.io](https://img.shields.io/crates/v/vaultic.svg)](https://crates.io/crates/vaultic)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](../LICENSE)

> **[English](../README.md)** | Español

**Protege tus secretos. Sincroniza tu equipo. Confía en tus configs.**

Vaultic es una herramienta CLI para gestionar secretos y archivos de configuración de forma segura en equipos de desarrollo. Cifra tus archivos sensibles, los sincroniza vía Git, detecta variables faltantes y audita cada cambio.

## ¿Por qué Vaultic?

- **Cifrado robusto** — age o GPG, tú eliges
- **Detecta problemas** — variables faltantes, configs desincronizadas
- **Multi-entorno** — dev/staging/prod con herencia inteligente
- **Auditoría** — quién cambió qué, cuándo
- **Zero cloud** — todo local + Git, sin dependencias externas
- **Extensible** — diseñado para soportar .env, .toml, .yaml, .json

## Instalación

### Con Cargo (requiere Rust)

```bash
cargo install vaultic
```

### Binarios precompilados

Descarga desde [Releases](https://github.com/SoftDryzz/vaultic/releases) para Windows, Linux o macOS.

## Inicio rápido

```bash
# 1. Inicializa en tu proyecto
cd mi-proyecto
vaultic init

# 2. Cifra tus secretos
vaultic encrypt .env --env dev

# 3. Commitea el archivo cifrado (seguro)
git add .vaultic/
git commit -m "feat: add encrypted secrets"

# 4. Otro dev clona y descifra
vaultic decrypt --env dev
```

## Comandos

| Comando | Descripción |
|---------|-------------|
| `vaultic init` | Inicializa Vaultic en el proyecto actual |
| `vaultic encrypt [archivo]` | Cifra archivos de secretos |
| `vaultic decrypt [archivo]` | Descifra archivos de secretos |
| `vaultic check` | Verifica variables faltantes contra el template |
| `vaultic diff` | Compara archivos de secretos o entornos |
| `vaultic resolve --env <env>` | Genera archivo resuelto con herencia |
| `vaultic keys setup` | Genera o importa una clave |
| `vaultic keys add <clave>` | Añade un recipient |
| `vaultic keys list` | Lista recipients autorizados |
| `vaultic keys remove <clave>` | Elimina un recipient |
| `vaultic log` | Muestra historial de operaciones |
| `vaultic status` | Muestra estado completo del proyecto |
| `vaultic hook install` | Instala git pre-commit hook |

## Seguridad

Los archivos `.enc` cifrados usan criptografía asimétrica. Solo los recipients autorizados pueden descifrarlos con su clave privada. Las claves públicas en el repositorio solo se usan para cifrar y no suponen ningún riesgo.

Consulta [SECURITY.md](../SECURITY.md) para la política de seguridad completa.

## Licencia

Este proyecto está licenciado bajo la [GNU Affero General Public License v3.0](../LICENSE).

Licencias comerciales disponibles para organizaciones que requieran términos alternativos. Contacto: legal@softdryzz.com
