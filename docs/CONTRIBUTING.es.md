# Contribuir a Vaultic

> **[English](../CONTRIBUTING.md)** | Español

Gracias por tu interés en contribuir a Vaultic. Esta guía te ayudará a empezar.

## Licencia y Acuerdo de Contribución

Vaultic se distribuye bajo un modelo de **licencia dual**:

- **Open source**: [GNU Affero General Public License v3.0](../LICENSE)
- **Comercial**: Disponible para organizaciones que necesiten términos alternativos

Al enviar un pull request, aceptas que:

1. Tu contribución se licencia bajo **AGPLv3**, consistente con la licencia del proyecto
2. Otorgas a **SoftDryzz** una licencia perpetua, irrevocable, mundial y libre de regalías para usar, modificar y sublicenciar tu contribución bajo cualquier licencia — incluyendo la licencia comercial ofrecida junto con AGPLv3
3. Tienes el derecho legal de realizar la contribución (es tu trabajo original o tienes autorización)

Este acuerdo es necesario para mantener el modelo de licencia dual. Sin él, las contribuciones externas no podrían incluirse en la versión comercial.

## Primeros Pasos

### Requisitos

- [Rust](https://www.rust-lang.org/tools/install) (stable, última versión)
- [age](https://github.com/FiloSottile/age) (para tests de cifrado)
- Git

### Configuración

```bash
git clone https://github.com/SoftDryzz/vaultic.git
cd vaultic
cargo build
cargo test
```

### Verificar que Todo Funciona

```bash
cargo fmt --check            # Formato de código
cargo clippy -- -D warnings  # Linting
cargo test --all-features    # Todos los tests
```

Los tres deben pasar antes de enviar un PR.

## Arquitectura del Proyecto

Vaultic sigue una **arquitectura hexagonal** (Clean Architecture adaptada para Rust):

```
src/
├── cli/          # Capa de presentación (comandos CLI, formato de salida)
├── core/         # Capa de dominio (lógica pura, cero deps externas)
│   ├── traits/   # Puertos (interfaces): CipherBackend, ConfigParser, etc.
│   ├── models/   # Entidades: SecretFile, Environment, DiffResult, etc.
│   ├── services/ # Casos de uso: EncryptionService, DiffService, etc.
│   └── errors.rs # Tipos de error del dominio
├── adapters/     # Implementaciones: AgeBackend, DotenvParser, FileKeyStore
└── config/       # Configuración de la aplicación (parseo de config.toml)
```

**Regla clave**: `core/` nunca importa de `adapters/` ni de `cli/`. Las dependencias fluyen hacia adentro.

## Estándares de Código

1. **Rust idiomático**: `Result<T>` para errores, `Option<T>` para ausencia, iteradores sobre bucles manuales, pattern matching exhaustivo
2. **Sin `unwrap()` en producción**: solo en tests. Usar `?` para propagar errores
3. **Sin `clone()` innecesario**: preferir referencias y lifetimes
4. **Documentación**: `///` doc comments en todos los traits, structs y funciones públicas
5. **Tests**: mínimo un test por función pública en `core/`
6. **Módulos**: si un archivo supera ~150 líneas, considerar dividir
7. **Mensajes de error**: cada variante de error debe dar contexto suficiente para diagnosticar sin debug

## Proceso de Pull Request

1. **Haz fork** del repositorio y crea una rama desde `master`
2. **Escribe tu código** siguiendo los estándares anteriores
3. **Añade tests** para cualquier funcionalidad nueva
4. **Ejecuta la verificación completa**:
   ```bash
   cargo fmt --check && cargo clippy -- -D warnings && cargo test
   ```
5. **Escribe un mensaje de commit claro** describiendo el cambio
6. **Abre un PR** contra `master` con:
   - Un título conciso (menos de 70 caracteres)
   - Descripción de qué cambia y por qué
   - Cualquier contexto o trade-off relevante

## Qué Contribuir

### Buenas Primeras Contribuciones

- Corrección de bugs con pasos claros de reproducción
- Mejoras en cobertura de tests
- Correcciones o mejoras en documentación
- Mejoras en mensajes de error (más contexto, pasos siguientes más claros)

### Contribuciones Mayores

Para funcionalidades o cambios arquitectónicos, **abre un issue primero** para discutir el enfoque. Esto evita esfuerzo desperdiciado si la dirección no se alinea con la hoja de ruta del proyecto.

### Áreas de Interés

- Implementaciones de parsers para nuevos formatos (TOML, YAML, JSON) a través del trait `ConfigParser`
- Mejora en diagnósticos de errores y mensajes orientados al usuario
- Mejoras específicas de plataforma (Windows, macOS, Linux)

## Reportar Incidencias

- **Bugs**: Incluye versión de Vaultic, SO, pasos para reproducir, comportamiento esperado vs real
- **Funcionalidades**: Describe el caso de uso, no solo la solución
- **Seguridad**: Consulta [SECURITY.md](SECURITY.es.md) — NO abras issues públicos para vulnerabilidades

## ¿Preguntas?

Abre un [GitHub Discussion](https://github.com/SoftDryzz/vaultic/discussions) o escribe a legal@softdryzz.com para consultas sobre licencias.
