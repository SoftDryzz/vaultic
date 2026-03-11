# Artículo LinkedIn — Vaultic

> Copiar desde "---INICIO---" hasta "---FIN---" para pegar en LinkedIn.

---INICIO---

En casi todos los equipos en los que he trabajado, los secretos de desarrollo se comparten por Slack, email o WhatsApp.

Contraseñas de base de datos, API keys, tokens de acceso — en texto plano, sin control de versiones, sin auditoría.

Decidí construir una herramienta para resolver esto de forma definitiva.

—

El problema es más común de lo que parece:

• Alguien actualiza una variable de entorno y no avisa. Otro dev pierde horas debuggeando.
• No hay forma de saber qué variables faltan al clonar un proyecto.
• Gestionar configs diferentes por entorno (dev/staging/prod) es caótico.
• No hay trazabilidad: si se filtra un secreto, no sabes cuándo ni quién lo cambió por última vez.

Y las soluciones actuales o dependen de la nube, o son demasiado complejas para equipos pequeños.

—

Así que construí Vaultic: una CLI escrita en Rust que cifra, sincroniza y audita secretos de equipo a través de Git.

Sin dependencias cloud. Sin servicios externos. Todo local + Git.

El flujo es simple:

vaultic init                    → Inicializa en tu proyecto
vaultic encrypt .env --env dev  → Cifra tus secretos
git push                        → El archivo cifrado es seguro de commitear
vaultic decrypt --env dev       → Otro dev descifra con su clave

Cada miembro tiene su propio par de claves. El cifrado es asimétrico — como HTTPS, pero para tus archivos .env.

Además de cifrar y descifrar, Vaultic:

• Detecta variables faltantes comparando contra un template
• Compara entornos entre sí (dev vs prod)
• Soporta herencia de entornos (base → dev, base → prod)
• Registra cada operación en un audit log (quién, qué, cuándo)
• Bloquea commits de archivos en plano con un pre-commit hook

—

Algunas decisiones técnicas que tomé y por qué:

Rust — Necesitaba un binario único sin dependencias. Un "cargo install vaultic" o descargar el binario y listo. Sin runtimes, sin intérpretes. Y la seguridad de memoria importa cuando manejas secretos.

Arquitectura hexagonal — El core de negocio no sabe si el cifrado es age o GPG, ni si el parser lee .env o YAML. Traits como puertos, adaptadores como implementaciones. Cambiar o añadir un backend es un archivo nuevo, no un refactor.

age sobre GPG como default — GPG es potente pero complejo. age (del creador de filippo.io) es moderno, auditable y cabe en una especificación de una página. Vaultic soporta ambos, pero recomienda age.

—

Ya lo usamos en nuestro equipo en proyectos reales con varios entornos y miembros.

Vaultic es open source y está publicado en crates.io. Disponible para Linux, macOS y Windows.

Si tu equipo gestiona secretos por Slack, email o documentos compartidos, dale una oportunidad:

cargo install vaultic

GitHub: https://github.com/SoftDryzz/vaultic
crates.io: https://crates.io/crates/vaultic
lib.rs: https://lib.rs/crates/vaultic

Y si te resulta útil, una estrella en el repo ayuda a que llegue a más equipos.

#OpenSource #Rust #DevSecOps #CLI #Seguridad #Ciberseguridad #DesarrolloDeSoftware #DevOps #Encryption #DotEnv

---FIN---
