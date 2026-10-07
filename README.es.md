# B4S: aplicación no oficial para auriculares Baseus en Windows, macOS y Linux

[English](README.md) | [Tiếng Việt](README.vi.md) | Español | [简体中文](README.zh-CN.md) | [Português (Brasil)](README.pt-BR.md)

**B4S es una aplicación de escritorio gratuita y de código abierto para controlar auriculares Bluetooth LE de Baseus desde tu PC.**
Cambia la cancelación de ruido (ANC), la transparencia y el modo adaptativo,
ajusta el ecualizador, activa el audio espacial y el modo juego y consulta la
batería sin usar la app de Baseus del teléfono. Funciona con **Baseus Bass BP1
Pro** y **BP1 Ultra**, con perfiles experimentales para **EP10 Ultra, EP10 Pro,
Bowie M4s, Bowie MS1 y Bowie M3s**. Está desarrollada con SolidJS, Tauri y Rust.

[**Descargar el último instalador**](https://github.com/hoan02/b4s/releases/latest) · [Auriculares compatibles](#auriculares-baseus-compatibles) · [Preguntas frecuentes](#preguntas-frecuentes)

| Buscar y conectar | Controles del dispositivo | Más controles | Ajustes |
|---|---|---|---|
| ![B4S buscando auriculares Baseus por Bluetooth LE en un PC](assets/i1.png) | ![B4S mostrando batería, modos de cancelación de ruido y audio espacial de Baseus Bass BP1 Pro](assets/i2.png) | ![Controles de B4S agrupados en sonido, controles y dispositivo](assets/i4.png) | ![Ajustes de B4S con idioma, tema y actualizaciones](assets/i3.png) |

Las capturas muestran la interfaz en vietnamita. Las pantallas de control se generaron con los componentes de la aplicación y datos de ejemplo.

## Novedades de 0.1.3

El catálogo incluye 124 perfiles, con 122 modelos solo de reconocimiento. Los nombres e imágenes usan metadatos públicos; las fotos se guardan en una caché local de 64 MiB para reutilizarlas sin conexión. Los identificadores y los datos guardados del dispositivo/EQ se migran automáticamente.

Los usuarios de 0.1.1/0.1.2 deben instalar 0.1.3 manualmente una vez porque cambió la clave de firma del actualizador.

[Changelog](CHANGELOG.md) · [Model contract](docs/model-identity-presentation.md) · [Image cache](docs/product-image-cache.md)

## Auriculares Baseus compatibles

B4S solo habilita controles para auriculares con un perfil revisado. Que el
nombre coincida nunca activa un control por sí solo.

| Modelo Baseus | Nivel | Qué puedes controlar |
|---|---|---|
| Bass BP1 Pro | Perfil revisado | ANC, transparencia y modo adaptativo, ecualizador y EQ personalizado, audio espacial, modo juego, Bass Boost, localizar auriculares |
| Bass BP1 Ultra | Experimental, probado en Windows | Batería, ANC, modo juego, audio espacial, Bass Boost, LDAC, protección auditiva, gestos. EQ/SoundFit no disponibles |
| Bass EP10 Ultra, Bowie M4s, Bowie MS1 | Experimental, sin probar en hardware | Los mismos controles que BP1 Ultra (adaptador compartido) |
| Bass EP10 Pro, Bowie M3s | Experimental, sin probar en hardware | ANC, audio espacial, modo juego, Bass Boost, gestos, reducción de ruido del viento (EP10 Pro también tiene presets de EQ) |
| 117 modelos Baseus más del catálogo | Solo detección | Nombre e imagen en la lista, sin controles |

Algunos controles (gestos táctiles, detección en el oído, multipunto, reducción
de ruido del viento, L/R adaptativo, restaurar valores) se marcan como
**Experimental** en la aplicación y dependen de *Settings → Experimental mode*.
Las funciones varían según el modelo y el firmware.

### Niveles de compatibilidad

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

## Preguntas frecuentes

**¿Existe una app de Baseus para Windows o PC?**
Baseus publica su app oficial para teléfonos. B4S es una aplicación de escritorio
independiente y no oficial que cubre los controles anteriores en Windows, macOS y
Linux. Hasta ahora Windows es la plataforma usada para las pruebas.

**¿Puedo cambiar la cancelación de ruido (ANC) o el EQ de Baseus desde el ordenador?**
Sí, en los modelos compatibles: conecta por Bluetooth LE y cambia ANC,
transparencia o modo adaptativo y elige un preset de EQ en B4S.

**¿Funciona con mis auriculares Baseus?**
Consulta [Auriculares Baseus compatibles](#auriculares-baseus-compatibles). B4S
reconoce 124 modelos de auriculares Baseus, pero solo los de esa tabla tienen controles.

**¿Por qué B4S no encuentra o no controla mis auriculares en Windows?**
Windows suele listar los mismos auriculares dos veces (salida de audio y entrada
de control BLE). Elige la entrada marcada como **Control**. Consulta la
[guía de solución de problemas](docs/desktop-troubleshooting.md).

**¿Es seguro? ¿Envía mis datos a algún sitio?**
Los controles funcionan localmente por Bluetooth. B4S no necesita cuenta y no
envía datos del dispositivo ni personales a ningún servidor. Lee el aviso en
[Uso seguro y licencia](#uso-seguro-y-licencia).

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
