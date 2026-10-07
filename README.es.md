# B4S

[English](README.md) | [Tiếng Việt](README.vi.md) | Español | [简体中文](README.zh-CN.md) | [Português (Brasil)](README.pt-BR.md)

B4S es una aplicación de escritorio independiente para controlar determinados
auriculares Bluetooth LE en Windows, macOS y Linux. Está desarrollada con
SolidJS, Tauri y Rust.

| Buscar y conectar | Controles del dispositivo | Ajustes |
|---|---|---|
| ![Buscar y conectar](assets/i1.png) | ![Batería, ANC y audio](assets/i2.png) | ![Ajustes](assets/i3.png) |

Las capturas muestran la interfaz en vietnamita.

## Novedades de 0.1.3

El catálogo incluye 124 perfiles, con 122 modelos solo de reconocimiento. Los nombres e imágenes usan metadatos públicos; las fotos se guardan en una caché local de 64 MiB para reutilizarlas sin conexión. Los identificadores y los datos guardados del dispositivo/EQ se migran automáticamente.

Los usuarios de 0.1.1/0.1.2 deben instalar 0.1.3 manualmente una vez porque cambió la clave de firma del actualizador.

[Changelog](CHANGELOG.md) · [Model contract](docs/model-identity-presentation.md) · [Image cache](docs/product-image-cache.md)

## Compatibilidad

BP1 Pro tiene un perfil de modelo revisado. BP1 Ultra admite conexión BLE/789C
y batería, ANC, modo juego, audio espacial, Bass Boost, LDAC, protección
auditiva y gestos experimentales. EQ/SoundFit aún no están disponibles.
Otros modelos del catálogo pueden ser experimentales o solo
reconocibles. Que la aplicación identifique un nombre no significa que sus
controles estén verificados.

| Nivel | Significado |
|---|---|
| Verificado | Los comandos y el comportamiento se probaron en hardware real. |
| Experimental | Hay un perfil, pero faltan pruebas del modelo o firmware. |
| Solo detección | Se reconoce el dispositivo; no se habilita el control. |

Consulta el [catálogo de modelos](docs/model-catalog.md) y las [notas del protocolo](docs/protocol/overview.md).

## Funciones y desarrollo

B4S puede mostrar la batería y controlar ANC, transparencia, EQ, audio espacial,
modo juego y localización cuando el modelo lo admite. En **Settings** puedes
activar la reconexión automática para buscar una vez tus últimos auriculares
compatibles al iniciar; está desactivada por defecto. La búsqueda dura hasta 12
segundos. También puedes activar el inicio de B4S al iniciar sesión. Al cerrar
la ventana, B4S se oculta en la bandeja cuando está disponible; elige **Quit
B4S** en el menú de la bandeja para salir. Las funciones dependen del modelo y
del firmware.

Si no puedes buscar o conectar, consulta la [guía de solución de problemas de escritorio](docs/desktop-troubleshooting.md).

Necesitas Bun 1.4.0, Rust estable, los requisitos de Tauri para tu plataforma y
auriculares Bluetooth para probar el dispositivo:

```sh
bun install --frozen-lockfile
bun run tauri:dev
```

La aplicación se inicia en inglés e incluye vietnamita, chino simplificado,
español y portugués de Brasil. Cambia el idioma en **Settings**; las
traducciones están incluidas y funcionan sin conexión. Consulta la [guía de traducción](docs/translations.md)
para contribuir.

## Documentación

- [Contribuir](CONTRIBUTING.md)
- [Arquitectura](docs/architecture.md)
- [Añadir un modelo](docs/model-catalog.md)
- [Lanzamientos y actualizaciones](docs/release.md)

## Uso seguro y licencia

B4S es software independiente y no está afiliado oficialmente a Baseus ni a
ningún fabricante. Los controles funcionan localmente por Bluetooth. La función
para localizar auriculares puede reproducir un sonido fuerte: quítatelos antes
de activarla. El software se proporciona tal cual, sin garantía de
compatibilidad, funcionamiento continuo ni recuperación del firmware.

No añadas APK oficiales, claves privadas, firmware ni código descompilado con
copyright al repositorio. Licencia: MIT.
