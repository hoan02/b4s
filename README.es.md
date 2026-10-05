# B4S

[English](README.md) | [Tiếng Việt](README.vi.md) | Español | [简体中文](README.zh-CN.md) | [Português (Brasil)](README.pt-BR.md)

B4S es una aplicación de escritorio independiente para controlar determinados
auriculares Bluetooth LE en Windows, macOS y Linux. Está desarrollada con
SolidJS, Tauri y Rust.

| Buscar y conectar | Controles del dispositivo | Ajustes |
|---|---|---|
| ![Buscar y conectar](assets/i1.png) | ![Batería, ANC y audio](assets/i2.png) | ![Ajustes](assets/i3.png) |

Las capturas muestran la interfaz en vietnamita.

## Compatibilidad

Baseus Bass BP1 Pro y BP1 Ultra son los dispositivos de hardware verificados
para la familia de protocolo BP1. Otros modelos del catálogo pueden ser
experimentales o solo reconocibles. Que la aplicación identifique un nombre no
significa que sus controles estén verificados.

| Nivel | Significado |
|---|---|
| Verificado | Los comandos y el comportamiento se probaron en hardware real. |
| Experimental | Hay un perfil, pero faltan pruebas del modelo o firmware. |
| Solo detección | Se reconoce el dispositivo; no se habilita el control. |

Consulta el [catálogo de modelos](docs/model-catalog.md) y las [notas del protocolo](docs/protocol/overview.md).

## Funciones y desarrollo

B4S puede mostrar la batería y controlar ANC, transparencia, EQ, audio espacial,
modo juego y localización cuando el modelo lo admite. Las funciones dependen del
modelo y del firmware.

Necesitas Node.js 20, Rust estable, los requisitos de Tauri para tu plataforma y
auriculares Bluetooth para probar el dispositivo:

```sh
npm ci
npm run tauri:dev
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
