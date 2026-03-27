# Roadmap

> **[English](roadmap.md)** | Español

Planes futuros para Vaultic, organizados por versión. Cada versión tiene un alcance claro y puede publicarse de forma independiente.

Versión actual: **v1.4.0**

---

## ~~v1.2.0 — Notificaciones de Actualización~~ ✅ Publicada

- `vaultic update`: comprueba e instala la última versión con verificación SHA256 + minisign
- Check pasivo de versión en cada comando (caché 24h, suprimido en modo `--quiet`)
- Auto-descubrimiento de template: `vaultic check` busca `.env.template`, `.env.example`, `.env.sample`, `env.template`
- Override de template por entorno y global en `config.toml`
- Campo `format_version` para compatibilidad hacia atrás entre versiones
- SHA256SUMS + firmas minisign en GitHub Releases

---

## ~~v1.3.0 — Validación y Salud de Secretos~~ ✅ Publicada

- `vaultic template sync`: genera automáticamente `.env.template` desde todos los entornos cifrados (unión de claves, valores eliminados). Usa `-o <ruta>` para ubicación personalizada.
- `vaultic validate`: valida secretos contra reglas de formato en `config.toml` — soporta `type` (url/integer/boolean/string), `min`/`max`, `min_length`/`max_length`, `required` y `pattern` (regex). Las reglas son combinables. Sale con código distinto de cero en caso de fallo (compatible con CI). Usa `-f <archivo>` para un archivo específico.
- Seguimiento de antigüedad en `vaultic status`: muestra cuándo se cifró cada entorno por última vez y señala los que exceden la política de rotación (`rotation_days` en config)
- Nueva sección `[validation]` y opción `rotation_days` en `config.toml`

---

## ~~v1.4.0 — Integración Docker y CI/CD~~ ✅ Publicada

- Variable de entorno `VAULTIC_AGE_KEY`: usa una clave privada desde env en lugar de archivo — esencial para CI/CD
- Flag `--stdout` en `decrypt` y `resolve`: salida cruda a stdout para piping (sin mensajes de UI)
- `vaultic ci export`: exporta secretos en formatos específicos de CI (`--format github|gitlab|generic`, `--mask` para GitHub Actions)
- Comprobación de seguridad `.dockerignore` en `vaultic status`: avisa cuando hay archivos Docker presentes pero `.env` no está en `.dockerignore`
- `vaultic validate` ahora sale con código 2 para fallos de validación (distinguible de otros errores en CI)

---

## v1.5.0 — Parsers Multi-formato

Soporte para archivos de configuración más allá de `.env`.

- **Nuevos parsers**: TOML, YAML, JSON — implementando el trait `ConfigParser` existente
- **Auto-detección**: formato determinado por extensión de archivo (`.toml`, `.yml`, `.yaml`, `.json`)
- **Preservación round-trip**: comentarios y orden de claves mantenidos después de cifrar/descifrar
- **Proyectos mixtos**: diferentes entornos pueden usar diferentes formatos
  ```toml
  [environments]
  dev = { file = "dev.env", inherits = "base" }
  k8s = { file = "k8s-secrets.yaml", inherits = "base", format = "yaml" }
  ```

---

## v2.0.0 — Control de Acceso

Permisos basados en roles para seguridad del equipo. Cambio breaking: nuevas secciones en config.toml.

- **Roles**: `admin` y `member` definidos en `config.toml`:
  ```toml
  [permissions]
  admin_keys = ["age1ql3z7hjy...ac8p"]  # Solo estos pueden gestionar claves/config

  [permissions.environments]
  dev = "all"           # Todos pueden cifrar/descifrar dev
  staging = "all"       # Todos pueden cifrar/descifrar staging
  prod = ["age1ql3z7hjy...ac8p", "age1x9ynm5k...7f2p"]  # Solo claves listadas
  ```
- **Comandos solo admin**: `keys add`, `keys remove`, `encrypt --all`, editar config
- **Acceso por entorno**: un developer puede descifrar `dev` pero no `prod` a menos que esté explícitamente autorizado
- **Enforcement local**: no necesita servidor — permisos verificados contra `config.toml` y tu clave local

---

## v2.1.0 — Importar y Exportar

Migrar desde otras herramientas y exportar a cualquier formato.

- **Importar**:
  ```bash
  vaultic import --from dotenv-vault .env.vault   # Desde dotenv-vault
  vaultic import --from sops secrets.yaml          # Desde Mozilla SOPS
  vaultic import --from 1password "vault/item"     # Desde 1Password CLI
  vaultic import --from aws-ssm /app/prod/         # Desde AWS Parameter Store
  ```
- **Exportar**:
  ```bash
  vaultic export --env dev --format json           # Salida JSON
  vaultic export --env dev --format yaml           # Salida YAML
  vaultic export --env dev --format toml           # Salida TOML
  vaultic export --env dev --format shell          # export KEY=value
  ```
- **Guías de migración**: documentación paso a paso para cambiar desde cada competidor

---

## v2.2.0 — Notificaciones y Webhooks

Mantener al equipo informado sobre cambios en secretos.

- **`vaultic notify setup`**: configurar una URL webhook para notificaciones:
  ```toml
  [notifications]
  webhook_url = "https://hooks.slack.com/services/T.../B.../xxx"
  events = ["encrypt", "key_add", "key_remove"]
  ```
- **Plataformas soportadas**: Slack, Discord, Microsoft Teams, HTTP POST genérico
- **Eventos personalizables**: elige qué operaciones disparan una notificación
- **Payload**: acción, autor, entorno, timestamp (nunca incluye valores secretos)

---

## v3.0.0 — Sincronización con Servidor

Servidor central opcional para sincronización de secretos en tiempo real. Cambio breaking: runtime asíncrono.

- **Servidor central**: un servidor Rust ligero que almacena archivos cifrados y gestiona acceso del equipo
- **`vaultic sync push --env dev`**: subir entorno cifrado al servidor
- **`vaultic sync pull --env dev`**: descargar última versión del servidor
- **`vaultic sync status`**: mostrar estado de sincronización de todos los entornos
- **`vaultic team`**: gestionar miembros del equipo y permisos vía servidor
- **Resolución de conflictos**: detectar cuándo local y remoto divergen, ofrecer estrategias de merge
- **Offline-first**: sin configuración de servidor, Vaultic funciona de forma idéntica a v1.x/v2.x — basado en Git puro
- **Self-hostable**: despliega tu propio servidor o usa una instancia gestionada

---

## Política de Versiones

| Tipo de bump | Cuándo |
|-------------|--------|
| Patch (1.1.x) | Corrección de bugs, docs |
| Minor (1.x.0) | Nuevas features, compatible hacia atrás |
| Major (x.0.0) | Cambios breaking en config.toml o comportamiento CLI |

Todas las releases v1.x mantienen compatibilidad total con proyectos v1.0.0.
