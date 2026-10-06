# B4S

[English](README.md) | [Tiếng Việt](README.vi.md) | [Español](README.es.md) | [简体中文](README.zh-CN.md) | Português (Brasil)

B4S é um aplicativo independente para desktop que controla alguns fones
Bluetooth LE no Windows, macOS e Linux. Foi desenvolvido com SolidJS, Tauri e Rust.

| Buscar e conectar | Controles do dispositivo | Configurações |
|---|---|---|
| ![Buscar e conectar](assets/i1.png) | ![Bateria, ANC e áudio](assets/i2.png) | ![Configurações](assets/i3.png) |

As capturas mostram a interface em vietnamita.

## Compatibilidade

BP1 Pro tem um perfil de modelo revisado. O BP1 Ultra é reconhecido, mas continua
somente para busca enquanto seu transporte de controle não for verificado.
Outros modelos do catálogo podem ser experimentais ou apenas reconhecidos.
Identificar o nome do dispositivo não significa que os controles foram
verificados.

| Nível | Significado |
|---|---|
| Verificado | Comandos e comportamento foram testados em hardware real. |
| Experimental | Existe um perfil, mas faltam testes do modelo ou firmware. |
| Apenas detecção | O dispositivo é reconhecido; o controle não está habilitado. |

Consulte o [catálogo de modelos](docs/model-catalog.md) e as [notas do protocolo](docs/protocol/overview.md).

## Recursos e desenvolvimento

Quando compatível, o B4S mostra a bateria e controla ANC, transparência, EQ,
áudio espacial, modo jogo e localização dos fones. Em **Settings**, você pode
ativar a reconexão automática para buscar uma vez os últimos fones compatíveis
ao iniciar; ela vem desativada. A busca dura até 12 segundos. Também é possível
iniciar o B4S ao entrar no computador. Fechar a janela oculta o B4S na bandeja
quando disponível; escolha **Quit B4S** no menu da bandeja para sair. Os recursos
variam conforme o modelo e o firmware.

Se não conseguir buscar ou conectar, consulte o [guia de solução de problemas do desktop](docs/desktop-troubleshooting.md).

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
