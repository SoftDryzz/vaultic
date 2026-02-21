# Política de Seguridad

> **[English](../SECURITY.md)** | Español

## Modelo de Cifrado

Vaultic utiliza criptografía asimétrica (pares de clave pública/privada):

- **age**: Acuerdo de clave X25519 + ChaCha20-Poly1305 (predeterminado, recomendado)
- **GPG**: Depende de la configuración del usuario (RSA/ECC)

Cada archivo se cifra para N destinatarios. Solo quienes posean la clave privada correspondiente pueden descifrar.

## Qué Es Seguro Publicar

| Archivo | ¿Seguro en repo público? | Motivo |
|---------|--------------------------|--------|
| `*.env.enc` | Sí | Cifrado, ilegible sin clave privada |
| `recipients.txt` | Sí | Solo claves públicas (para cifrar) |
| `config.toml` | Sí | Metadatos de configuración, sin secretos |
| `audit.log` | Sí | Solo metadatos de operaciones, sin valores |
| `.env` | **NUNCA** | Secretos en texto plano |
| `keys.txt` / claves privadas | **NUNCA** | Material de clave privada |

## Versiones Soportadas

| Versión | Soportada |
|---------|-----------|
| 0.x.x (actual) | Sí |

## Reportar una Vulnerabilidad

Si descubres una vulnerabilidad de seguridad en Vaultic, repórtala de forma responsable.

**NO** abras un issue público para vulnerabilidades de seguridad.

**Email:** security@softdryzz.com

Confirmaremos la recepción en un plazo de 48 horas y proporcionaremos una evaluación inicial en un máximo de 5 días laborables.

## Respuesta ante Incidentes

### Filtración de archivo `.env` en texto plano

1. **Rota TODOS los secretos inmediatamente** (API keys, contraseñas, tokens)
2. Elimina el archivo del historial de Git usando `git filter-branch` o [BFG Repo-Cleaner](https://rtyley.github.io/bfg-repo-cleaner/)
3. Re-cifra con los nuevos valores: `vaultic encrypt --env <env>`
4. Audita los logs de acceso por uso no autorizado

### Clave privada comprometida

1. Elimina el destinatario: `vaultic keys remove <clave>`
2. Genera una nueva clave: `vaultic keys setup`
3. Re-cifra todos los entornos: `vaultic encrypt --all`
4. Rota cualquier secreto accesible con la clave comprometida
5. Los archivos cifrados anteriores en el historial de Git siguen en riesgo — rota los secretos afectados

### Salida de un miembro del equipo

1. Elimina su clave pública: `vaultic keys remove <clave>`
2. Re-cifra todos los entornos: `vaultic encrypt --all` (asegura que los nuevos cifrados excluyan la clave eliminada)
3. Rota los secretos sensibles (API keys de producción, contraseñas de base de datos, claves de firma)

## Principios de Diseño de Seguridad

- **Sin texto plano en disco durante la resolución**: `vaultic resolve` descifra las capas en memoria y escribe solo el resultado final combinado
- **Sin llamadas de red**: Vaultic v1 opera completamente offline — sin telemetría, sin dependencias cloud
- **Sin valores secretos en logs**: el audit log registra operaciones y metadatos, nunca valores de variables
- **Cifrado siempre asimétrico**: los secretos se cifran para destinatarios específicos, nunca con contraseñas simétricas
- **Verificación de integridad**: las operaciones de cifrado y descifrado registran un hash SHA-256 del archivo resultante en el audit log, permitiendo detección de manipulación
- **Validación de claves de recipients**: las claves públicas se validan al añadirlas (formato Bech32 para age, formato fingerprint para GPG) para prevenir errores tipográficos
