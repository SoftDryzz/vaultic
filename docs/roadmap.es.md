# Roadmap

> **[English](roadmap.md)** | Español

Planes futuros para Vaultic, organizados por versión. Cada versión tiene un alcance claro y puede publicarse de forma independiente.

Versión actual: **v1.1.0**

---

## v1.2.0 — Notificaciones de Actualización

Saber cuándo hay una nueva versión disponible sin tener que comprobarlo manualmente.

- **Check de versión en background**: en la primera ejecución del día, un hilo en background (no bloqueante) consulta la última versión en crates.io. El resultado se cachea durante 24 horas. En la siguiente ejecución, si hay una versión más nueva, se muestra una línea al final del output:
  ```
  Update available: v1.1.0 → v1.2.0 — run 'vaultic self-update'
  ```
- **`vaultic self-update`**: descarga e instala la última versión. Detecta el método de instalación:
  - Si se instaló con `cargo install` → ejecuta `cargo install vaultic`
  - Si se instaló con binario precompilado → descarga de GitHub Releases y reemplaza el binario
- **Ubicación del cache**: `~/.vaultic/update_cache.json` (global, no por proyecto)
- **Zero impacto en rendimiento**: el check nunca bloquea la ejecución del comando

---

## v1.3.0 — Validación y Salud de Secretos

Detectar errores de configuración antes de que lleguen a producción.

- **`vaultic validate`**: verifica secretos contra reglas de formato definidas en `config.toml`:
  ```toml
  [validation]
  DATABASE_URL = { type = "url", required = true }
  PORT = { type = "integer", min = 1024, max = 65535 }
  API_KEY = { type = "string", min_length = 32 }
  DEBUG = { type = "boolean" }
  ```
  Output:
  ```
  ✓ DATABASE_URL — URL válida
  ✗ PORT — se esperaba integer, recibido "abc"
  ✗ API_KEY — demasiado corto (12 chars, mínimo 32)
  ✓ DEBUG — boolean válido
  ```
- **Seguimiento de antigüedad**: registrar cuándo se modificó cada secreto por última vez. Avisar si un secreto no se ha rotado en un número configurable de días:
  ```
  ⚠ DB_PASSWORD última rotación hace 120 días (política: 90 días)
  ```
- **`vaultic template sync`**: auto-generar `.env.template` desde las claves de tus entornos cifrados, sin exponer valores. Mantiene el template siempre sincronizado.

---

## v1.4.0 — Integración Docker y CI/CD

Integración fluida con contenedores y pipelines de despliegue.

- **`vaultic docker-env --env dev`**: genera un archivo `.env` listo para `docker-compose up`. Resuelve herencia y escribe en la ruta que espera docker-compose.
  ```yaml
  # docker-compose.yml
  env_file:
    - .env  # Generado por: vaultic docker-env --env dev
  ```
- **`vaultic ci export --env dev`**: exporta secretos en formatos compatibles con sistemas CI:
  - GitHub Actions: `KEY=value >> $GITHUB_ENV`
  - GitLab CI: `export KEY=value`
  - Genérico: líneas `KEY=value`
- **`vaultic ci mask`**: genera comandos `::add-mask::` para GitHub Actions para evitar que los secretos aparezcan en logs
- **GitHub Action pre-construida** (`softdryzz/vaultic-action@v1`):
  ```yaml
  - uses: softdryzz/vaultic-action@v1
    with:
      env: dev
      key: ${{ secrets.VAULTIC_AGE_KEY }}
  ```

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
