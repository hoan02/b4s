# B4S

[English](README.md) | [Tiếng Việt](README.vi.md) | [Español](README.es.md) | [简体中文](README.zh-CN.md) | Português (Brasil)

B4S é um aplicativo independente para desktop que controla alguns fones
Bluetooth LE no Windows, macOS e Linux. Foi desenvolvido com SolidJS, Tauri e Rust.

| Buscar e conectar | Controles do dispositivo | Configurações |
|---|---|---|
| ![Buscar e conectar](assets/i1.png) | ![Bateria, ANC e áudio](assets/i2.png) | ![Configurações](assets/i3.png) |

As capturas mostram a interface em vietnamita.

## Compatibilidade

Baseus Bass BP1 Pro e BP1 Ultra são os dispositivos de hardware verificados para
a família de protocolo BP1. Outros modelos do catálogo podem ser experimentais ou
apenas reconhecidos. Identificar o nome do dispositivo não significa que os
controles foram verificados.

| Nível | Significado |
|---|---|
| Verificado | Comandos e comportamento foram testados em hardware real. |
| Experimental | Existe um perfil, mas faltam testes do modelo ou firmware. |
| Apenas detecção | O dispositivo é reconhecido; o controle não está habilitado. |

Consulte o [catálogo de modelos](docs/model-catalog.md) e as [notas do protocolo](docs/protocol/overview.md).

## Recursos e desenvolvimento

Quando compatível, o B4S mostra a bateria e controla ANC, transparência, EQ,
áudio espacial, modo jogo e localização dos fones. Os recursos variam conforme o
modelo e o firmware.

Requisitos: Node.js 20, Rust stable, dependências do Tauri para sua plataforma e
fones Bluetooth para testar o dispositivo:

```sh
npm ci
npm run tauri:dev
```

O aplicativo inicia em inglês e inclui vietnamita, chinês simplificado,
espanhol e português do Brasil. Altere o idioma em **Settings**; as traduções
estão incluídas e funcionam offline. Consulte o [guia de tradução](docs/translations.md)
para contribuir.

## Documentação

- [Como contribuir](CONTRIBUTING.md)
- [Arquitetura](docs/architecture.md)
- [Como adicionar um modelo](docs/model-catalog.md)
- [Lançamentos e atualizações](docs/release.md)

## Segurança e licença

B4S é independente e não tem afiliação oficial com a Baseus ou qualquer
fabricante. Os controles funcionam localmente por Bluetooth. O recurso para
localizar os fones pode emitir um som alto: retire-os dos ouvidos antes de usar.
O software é fornecido no estado em que se encontra, sem garantia de
compatibilidade, funcionamento contínuo ou recuperação de firmware.

Não inclua APKs oficiais, chaves privadas, firmware ou código descompilado
protegido por direitos autorais no repositório. Licença: MIT.
