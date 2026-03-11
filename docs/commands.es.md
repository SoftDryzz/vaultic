# Referencia de Comandos

> **[English](commands.md)** | Español

Referencia completa de todos los comandos de Vaultic CLI con ejemplos y explicaciones.

## Tabla de Contenidos

- [Flags Globales](#flags-globales)
- [init](#vaultic-init)
- [encrypt](#vaultic-encrypt)
- [decrypt](#vaultic-decrypt)
- [check](#vaultic-check)
- [template sync](#vaultic-template-sync)
- [validate](#vaultic-validate)
- [diff](#vaultic-diff)
- [resolve](#vaultic-resolve)
- [keys setup](#vaultic-keys-setup)
- [keys add](#vaultic-keys-add)
- [keys list](#vaultic-keys-list)
- [keys remove](#vaultic-keys-remove)
- [log](#vaultic-log)
- [status](#vaultic-status)
- [hook install / uninstall](#vaultic-hook)
- [Flujos Comunes](#flujos-comunes)

---

## Flags Globales

Estos flags funcionan con cualquier comando:

| Flag | Corto | Default | Descripción |
|------|-------|---------|-------------|
| `--cipher <age\|gpg>` | — | `age` | Backend de cifrado |
| `--env <nombre>` | — | `dev` | Entorno objetivo (repetible para diff) |
| `--config <ruta>` | — | `.vaultic/` | Ruta a directorio vaultic personalizado |
| `--verbose` | `-v` | off | Salida detallada |
| `--quiet` | `-q` | off | Solo errores |

---

## `vaultic init`

Inicializa Vaultic en un proyecto nuevo. Crea el directorio `.vaultic/` con archivos de configuración y opcionalmente genera tu clave de cifrado.

```
vaultic init
```

**Qué hace:**

1. Crea el directorio `.vaultic/`
2. Genera `config.toml` con entornos por defecto (base, dev, staging, prod)
3. Crea `recipients.txt` vacío
4. Crea `.env.template`
5. Añade `.env` a `.gitignore`
6. Busca claves age/GPG existentes en tu sistema
7. Si no encuentra ninguna, pregunta si quieres generar una
8. Registra la operación en el audit log

**Detección interactiva de claves:**

- Si respondes **Y**: genera una clave age en `~/.config/age/keys.txt` y añade tu clave pública a `recipients.txt`
- Si respondes **N**: salta la generación — puedes ejecutar `vaultic keys setup` después

**Ejemplo:**

```
$ vaultic init

🔐 Vaultic — Initializing project
  ✓ Created .vaultic/
  ✓ Generated config.toml with defaults
  ✓ Created .env.template
  ✓ Added .env to .gitignore

🔑 Key configuration
  No age key found. Generate one now? [Y/n]: Y

  ✓ Private key saved to: ~/.config/age/keys.txt
  ✓ Public key: age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p
  ✓ Public key added to .vaultic/recipients.txt
  ✓ Project ready.
```

**Errores:**

| Error | Causa | Solución |
|-------|-------|----------|
| "already initialized" | `.vaultic/` ya existe | El proyecto ya está configurado |

---

## `vaultic encrypt`

Cifra un archivo en texto plano para que pueda commitearse a Git de forma segura.

```
vaultic encrypt [ARCHIVO] [--env <nombre>] [--all] [--cipher <age|gpg>]
```

| Opción | Default | Descripción |
|--------|---------|-------------|
| `ARCHIVO` | `.env` | Archivo fuente a cifrar |
| `--env <nombre>` | `dev` | Etiqueta de entorno para el archivo cifrado |
| `--all` | off | Re-cifra todos los entornos (ignora ARCHIVO y --env) |

**Qué hace:**

1. Lee tu archivo en texto plano (ej: `.env`)
2. Lo cifra con las claves públicas de todos los recipients en `recipients.txt`
3. Guarda el resultado en `.vaultic/{env}.env.enc`
4. El archivo original NO se modifica ni se elimina

**El flag `--env`** es una etiqueta que nombra el archivo cifrado. Distintos entornos tienen distintos secretos:

```bash
vaultic encrypt .env --env dev       # → .vaultic/dev.env.enc
vaultic encrypt .env --env staging   # → .vaultic/staging.env.enc
vaultic encrypt .env --env prod      # → .vaultic/prod.env.enc
```

**El flag `--all`** re-cifra cada entorno definido en `config.toml`. Es esencial después de añadir o eliminar un miembro del equipo:

```bash
# Después de añadir un nuevo recipient
vaultic keys add age1x9ynm5k...
vaultic encrypt --all    # Re-cifra todos los entornos para que el nuevo miembro pueda descifrar
```

Cómo funciona `--all`: descifra cada archivo `.enc` en memoria (sin texto plano en disco) y lo re-cifra con la lista actual de recipients.

**Ejemplo:**

```
$ vaultic encrypt .env --env dev

  Source: .env
  ⏳ Encrypting dev with age for 3 recipient(s)...
  ✓ Encrypted with age for 3 recipient(s)
  ✓ Saved to .vaultic/dev.env.enc

  Commit .vaultic/dev.env.enc to the repo.
```

**Errores:**

| Error | Causa | Solución |
|-------|-------|----------|
| "not initialized" | Falta `.vaultic/` | Ejecuta `vaultic init` primero |
| "No recipients" | `recipients.txt` vacío | Ejecuta `vaultic keys add <clave>` |
| "Unknown cipher" | Valor `--cipher` inválido | Usa `age` o `gpg` |

---

## `vaultic decrypt`

Descifra un archivo cifrado para restaurar tu `.env` local.

```
vaultic decrypt [ARCHIVO] [--env <nombre>] [--key <ruta>] [-o <ruta>] [--cipher <age|gpg>]
```

| Opción | Corto | Default | Descripción |
|--------|-------|---------|-------------|
| `ARCHIVO` | — | `.vaultic/{env}.env.enc` | Archivo cifrado a descifrar |
| `--env <nombre>` | — | `dev` | Entorno a descifrar |
| `--key <ruta>` | — | `~/.config/age/keys.txt` | Ruta a tu clave privada |
| `--output <ruta>` | `-o` | `.env` | Dónde escribir el archivo descifrado |

**Qué hace:**

1. Lee el archivo cifrado (`.vaultic/dev.env.enc`)
2. Lo descifra usando tu clave privada
3. Escribe el texto plano en la ruta de salida (por defecto: `.env`)
4. Muestra cuántas variables se descifraron

**El flag `--key`** permite usar una clave privada desde una ubicación personalizada:

```bash
vaultic decrypt --env dev --key /ruta/a/mi-clave.txt
```

**El flag `-o`** permite escribir la salida descifrada a una ruta personalizada:

```bash
vaultic decrypt --env dev -o backend/.env     # Descifra a un subdirectorio
vaultic decrypt --env prod -o deploy/.env     # Descifra prod a la carpeta deploy
```

**Ejemplo:**

```
$ vaultic decrypt --env dev

  Source: .vaultic/dev.env.enc
  ⏳ Decrypting dev with age...
  ✓ Decrypted .vaultic/dev.env.enc
  ✓ Generated .env with 23 variables

  Run 'vaultic check' to verify no variables are missing.
```

**Errores:**

| Error | Causa | Solución |
|-------|-------|----------|
| "not found" | Archivo cifrado no existe | Verifica el nombre con `vaultic status` o haz `git pull` |
| "No private key found" | Archivo de clave no existe | Ejecuta `vaultic keys setup` o usa `--key <ruta>` |
| "no matching key found" | Tu clave no está en la lista de recipients | Pide a un admin que ejecute `vaultic keys add <tu_clave>` |

---

## `vaultic check`

Compara tu `.env` local contra `.env.template` para detectar variables faltantes o extra.

```
vaultic check
```

Sin flags — siempre compara `.env` vs `.env.template` en la raíz del proyecto.

**Qué reporta:**

- **Variables faltantes**: existen en el template pero no en tu `.env`
- **Variables extra**: existen en tu `.env` pero no en el template
- **Valores vacíos**: variables sin valor asignado

**Ejemplo:**

```
$ vaultic check

  🔍 vaultic check
  ⚠ Missing variables (2):
      • REDIS_CLUSTER_URL
      • FEATURE_FLAG_V2

  ⚠ Extra variables not in template (1):
      • OLD_API_KEY

  21/23 variables present, 2 issue(s) found
```

Si todo está sincronizado:

```
$ vaultic check

  ✓ 23/23 variables present — all good
```

---

## `vaultic template sync`

Genera automáticamente `.env.template` desde todos los entornos cifrados. Descifra cada entorno en memoria, recoge la unión de todas las claves, elimina todos los valores, y escribe el resultado.

```bash
# Sincroniza .env.template desde todos los entornos cifrados
vaultic template sync

# Escribe en una ruta personalizada
vaultic template sync -o custom.template
```

**Qué hace:**
1. Descifra cada archivo `.env.enc` en memoria (requiere tu clave privada)
2. Recoge cada clave de cada entorno (unión)
3. Elimina todos los valores (cadenas vacías)
4. Escribe el resultado en `.env.template` (o ruta personalizada)

Esto mantiene tu template siempre sincronizado con los secretos reales — sin mantenimiento manual. El archivo de salida es seguro para commitear.

**Opciones:**

| Flag | Descripción |
|------|-------------|
| `-o, --output <ruta>` | Escribe en una ruta personalizada en lugar de `.env.template` |

---

## `vaultic validate`

Valida tu `.env` local contra reglas de formato definidas en `.vaultic/config.toml`.

```bash
# Valida .env
vaultic validate

# Valida un archivo específico
vaultic validate -f prod.env
```

**Ejemplo de salida:**
```
🔍 vaultic validate
  File: .env
  Rules: 5 defined

  ✗ STRIPE_KEY — does not match pattern "^sk_live_.*"
  ✓ API_KEY — ok
  ✓ DATABASE_URL — ok
  ✓ DEBUG — ok
  ✓ PORT — ok

  4/5 rules passed
```

**Configurar reglas** en `.vaultic/config.toml`:

```toml
[validation]
DATABASE_URL = { type = "url", required = true }
PORT = { type = "integer", min = 1024, max = 65535 }
API_KEY = { type = "string", min_length = 32 }
DEBUG = { type = "boolean" }
STRIPE_KEY = { pattern = "^sk_live_.*" }
```

**Campos de reglas soportados:**

| Campo | Descripción | Ejemplo |
|-------|-------------|---------|
| `type` | Tipo de valor: `url`, `integer`, `boolean`, `string` | `type = "url"` |
| `required` | La clave debe estar presente y no vacía | `required = true` |
| `min` / `max` | Límites numéricos (tipo integer) | `min = 1024, max = 65535` |
| `min_length` / `max_length` | Límites de longitud de cadena | `min_length = 32` |
| `pattern` | Patrón regex que el valor debe cumplir | `pattern = "^sk_live_.*"` |

Todos los campos son opcionales y combinables. Si una clave no es requerida y está ausente, se omite silenciosamente.

**Compatible con CI:** sale con código 1 en caso de fallo, ideal para pipelines CI.

**Opciones:**

| Flag | Descripción |
|------|-------------|
| `-f, --file <ruta>` | Archivo a validar (default: `.env`) |

---

## `vaultic diff`

Compara dos archivos de secretos o dos entornos resueltos lado a lado.

**Modo archivo** — compara dos archivos en texto plano:

```
vaultic diff <archivo1> <archivo2>
```

**Modo entorno** — compara dos entornos resueltos (descifra y aplica herencia):

```
vaultic diff --env <nombre1> --env <nombre2>
```

**Qué muestra:**

| Color | Significado |
|-------|-------------|
| Verde | Añadido — existe en el segundo pero no en el primero |
| Rojo | Eliminado — existe en el primero pero no en el segundo |
| Amarillo | Modificado — misma clave, distintos valores |

**Ejemplo:**

```
$ vaultic diff --env dev --env prod

  Comparing environments: dev vs prod

  Variable            │ dev           │ prod
  ────────────────────┼───────────────┼──────────────
  DATABASE_URL        │ localhost     │ rds.aws.com
  DEBUG               │ true          │ ✗ (missing)
  REDIS_CLUSTER       │ ✗ (missing)   │ redis.prod

  ✓ 1 added, 1 removed, 1 modified
```

Esto es útil para detectar desfases de configuración entre entornos — por ejemplo, una variable que existe en dev pero se olvidó en prod.

---

## `vaultic resolve`

Genera un archivo `.env` final fusionando capas de entorno (base + overlay).

```
vaultic resolve --env <nombre> [-o <ruta>] [--cipher <age|gpg>]
```

| Opción | Corto | Default | Descripción |
|--------|-------|---------|-------------|
| `--env <nombre>` | — | desde config | Entorno a resolver |
| `--output <ruta>` | `-o` | `.env` | Dónde escribir el archivo resuelto |

**Cómo funciona la herencia:**

Tu `config.toml` define cadenas de herencia:

```toml
[environments]
base = "base.env"
dev = { file = "dev.env", inherits = "base" }
staging = { file = "staging.env", inherits = "base" }
prod = { file = "prod.env", inherits = "base" }
```

Cuando ejecutas `vaultic resolve --env prod`:

1. Descifra `base.env.enc` → obtiene variables base
2. Descifra `prod.env.enc` → obtiene variables de prod
3. Fusiona: prod sobreescribe base donde las claves coinciden
4. Escribe el resultado final en `.env`

Todo el descifrado ocurre en memoria — sin archivos de texto plano intermedios en disco.

**Ejemplo:**

```
$ vaultic resolve --env prod

  Resolving environment: prod
  ✓ Inheritance chain: base → prod
  ✓ Resolved 42 variables from 2 layer(s)
  ✓ Written to .env

$ vaultic resolve --env staging -o deploy/.env
  # Resuelve staging y escribe en deploy/.env
```

**Errores:**

| Error | Causa | Solución |
|-------|-------|----------|
| "Environment not found" | Nombre no está en `config.toml` | Verifica la ortografía o añádelo al config |
| "Circular inheritance" | ej: dev → staging → dev | Corrige la cadena en `config.toml` |

---

## `vaultic keys setup`

Generación o importación interactiva de claves para nuevos usuarios.

```
vaultic keys setup
```

**Presenta un menú interactivo:**

1. **Generar nueva clave age** (recomendado) — crea un par de claves en `~/.config/age/keys.txt`
2. **Importar clave age existente desde archivo** — copia tu clave a la ubicación estándar
3. **Usar clave GPG existente** — si GPG está disponible en tu sistema

Después del setup, muestra tu clave pública e instrucciones para el admin del proyecto:

```
$ vaultic keys setup

  ✓ Key generated
  ✓ Public key: age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p

  📋 Siguiente paso:
     Envía tu clave PÚBLICA al admin del proyecto.
     El admin ejecutará: vaultic keys add age1ql3z7hjy...ac8p
     Después podrás descifrar con: vaultic decrypt --env dev
```

**¿Es seguro compartir la clave pública?** Sí. La clave pública solo puede cifrar datos para ti — no puede descifrar nada. Piensa en ella como un candado abierto: cualquiera puede cerrarlo, pero solo tú tienes la llave para abrirlo.

---

## `vaultic keys add`

Añade la clave pública de un recipient a la lista de autorizados.

```
vaultic keys add <CLAVE>
```

**Formatos de clave aceptados:**

| Formato | Ejemplo |
|---------|---------|
| Clave pública age | `age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p` |
| Email GPG | `user@example.com` |
| Fingerprint GPG | `ABCDEF1234567890...` |

**Después de añadir una clave, debes re-cifrar** para que el nuevo miembro pueda descifrar:

```bash
vaultic keys add age1x9ynm5k...
vaultic encrypt --all
git add .vaultic/ && git commit -m "chore: add new team member"
```

**Errores:**

| Error | Causa | Solución |
|-------|-------|----------|
| "already exists" | Clave ya está en `recipients.txt` | No se necesita acción |
| "Invalid age public key" | Clave malformada | Verifica que empiece con `age1` |

---

## `vaultic keys list`

Lista todos los recipients autorizados.

```
vaultic keys list
```

**Ejemplo:**

```
$ vaultic keys list

  📋 Authorized recipients (3)
  • age1ql3z7hjy...ac8p
  • age1x9ynm5k...7f2p
  • age1htr8gqn...9d3k  # team-lead
```

Las etiquetas después de `#` son comentarios opcionales en `recipients.txt`.

---

## `vaultic keys remove`

Elimina un recipient de la lista de autorizados.

```
vaultic keys remove <CLAVE>
```

**Después de eliminar una clave, debes re-cifrar** para revocar el acceso:

```bash
vaultic keys remove age1x9ynm5k...
vaultic encrypt --all
git add .vaultic/ && git commit -m "chore: remove departed member"
```

Los archivos cifrados anteriores en el historial de Git siguen siendo descifrables con la clave eliminada — rota los secretos sensibles después de eliminar un miembro.

---

## `vaultic log`

Muestra el historial de operaciones (audit log).

```
vaultic log [--author <nombre>] [--since <fecha>] [--last <n>]
```

| Opción | Formato | Descripción |
|--------|---------|-------------|
| `--author <nombre>` | texto libre | Filtrar por nombre de autor (git user.name) |
| `--since <fecha>` | `YYYY-MM-DD` | Mostrar entradas desde esta fecha |
| `--last <n>` | entero | Mostrar solo las últimas N entradas |

**Ejemplo:**

```
$ vaultic log --last 5

  Date/Time            │ Author   │ Action   │ Detail
  ─────────────────────┼──────────┼──────────┼─────────────────
  2026-02-23 14:30:00  │ Cristo   │ encrypt  │ dev.env.enc
  2026-02-23 10:15:00  │ María    │ decrypt  │ prod → .env
  2026-02-22 18:45:00  │ Cristo   │ check    │ 23/23 present
  2026-02-22 16:20:00  │ Alex     │ key add  │ age1x9y...
  2026-02-22 09:00:00  │ Cristo   │ init     │ —

  Showing 5 entries

$ vaultic log --author "Cristo" --since 2026-02-22
  # Muestra solo las entradas de Cristo desde el 22 de febrero
```

El audit log nunca contiene valores secretos — solo metadatos de operaciones (acción, archivos, timestamps).

**Errores:**

| Error | Causa | Solución |
|-------|-------|----------|
| "Invalid date format" | Valor de `--since` no es `YYYY-MM-DD` | Usa formato ISO 8601 |

---

## `vaultic status`

Muestra una vista completa de la configuración y estado del proyecto.

```
vaultic status
```

**Ejemplo:**

```
$ vaultic status

  🔐 Vaultic v1.1.0
  Cipher: age
  Config: .vaultic/config.toml

  Recipients (3):
  • age1ql3z7hjy...ac8p
  • age1x9ynm5k...7f2p
  • age1htr8gqn...9d3k

  Encrypted environments:
  ✓ base.env.enc
  ✓ dev.env.enc
  ✓ staging.env.enc
  ✓ prod.env.enc
  ✗ testing (not encrypted)
```

---

## `vaultic hook`

Instala o desinstala un hook pre-commit de Git que bloquea commits accidentales de archivos `.env` en texto plano.

**Instalar:**

```
vaultic hook install
```

**Desinstalar:**

```
vaultic hook uninstall
```

**Qué hace el hook:**

Cuando ejecutas `git commit`, el hook escanea los archivos staged. Si encuentra un archivo `.env` en texto plano, bloquea el commit:

```
🚨 Vaultic pre-commit hook

  Plaintext .env file detected in staged files!
  Encrypt first: vaultic encrypt
  Or bypass (not recommended): git commit --no-verify
```

**Errores:**

| Error | Causa | Solución |
|-------|-------|----------|
| "Not a git repository" | No hay directorio `.git/` | Ejecuta `git init` primero |
| "not installed by Vaultic" | Hook existente de otra herramienta | Elimínalo manualmente o conserva tu hook actual |

---

## Flujos Comunes

### Configuración inicial (proyecto nuevo)

```bash
vaultic init                           # Crear .vaultic/ y generar clave
echo "DATABASE_URL=localhost" > .env   # Crear tu .env
vaultic encrypt --env dev              # Cifrarlo
git add .vaultic/ .env.template        # Commitear archivo cifrado + template
git push
```

### Unirte a un proyecto existente

```bash
git clone <repo> && cd <proyecto>
vaultic keys setup                     # Generar tu clave
# Envía tu clave PÚBLICA al admin
# El admin ejecuta: vaultic keys add <tu_clave> && vaultic encrypt --all
vaultic decrypt --env dev              # Descifrar tu .env local
vaultic check                          # Verificar que no falta nada
```

### Después de cambiar secretos

```bash
# Edita .env con los nuevos valores
vaultic encrypt --env dev              # Re-cifrar
git add .vaultic/dev.env.enc
git commit -m "chore: update dev secrets"
```

### Añadir un miembro al equipo

```bash
vaultic keys add <su_clave_publica>    # Añadir su clave
vaultic encrypt --all                  # Re-cifrar todos los entornos
git add .vaultic/
git commit -m "chore: add new team member"
```

### Eliminar un miembro del equipo

```bash
vaultic keys remove <su_clave_publica> # Eliminar su clave
vaultic encrypt --all                  # Re-cifrar sin él
# Rotar secretos sensibles (API keys, contraseñas)
git add .vaultic/
git commit -m "chore: revoke departed member access"
```

### Comparar entornos antes de deploy

```bash
vaultic diff --env staging --env prod  # Ver qué difiere
vaultic resolve --env prod -o .env     # Obtener la config resuelta de prod
```
