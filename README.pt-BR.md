# B4S: aplicativo não oficial para fones Baseus no Windows, macOS e Linux

[English](README.md) | [Tiếng Việt](README.vi.md) | [Español](README.es.md) | [简体中文](README.zh-CN.md) | Português (Brasil)

**O B4S é um aplicativo desktop gratuito e de código aberto para controlar fones Bluetooth LE da Baseus pelo PC.**
Alterne o cancelamento de ruído (ANC), a transparência e o modo adaptativo,
ajuste o equalizador, ative o áudio espacial e o modo jogo e veja a bateria sem
usar o app da Baseus no celular. Funciona com **Baseus Bass BP1 Pro** e **BP1
Ultra**, com perfis experimentais para **EP10 Ultra, EP10 Pro, Bowie M4s, Bowie
MS1 e Bowie M3s**. Foi desenvolvido com SolidJS, Tauri e Rust.

[**Baixar o instalador mais recente**](https://github.com/hoan02/b4s/releases/latest) · [Fones compatíveis](#fones-baseus-compatíveis) · [Perguntas frequentes](#perguntas-frequentes)

<p align="center"><a href="assets/b4s-demo.mp4"><img src="assets/b4s-demo.gif" alt="Demo do B4S: bateria, controle de ruído, áudio espacial, EQ, controles de toque e ajustes de som" width="320"></a></p>

<p align="center"><sub>Prévia do aplicativo com a interface em vietnamita e dados de exemplo. <a href="assets/b4s-demo.mp4">Assistir ao vídeo completo com narração (vietnamita, ~75 s)</a>.</sub></p>

## Novidades da versão 0.1.3

O catálogo inclui 124 perfis, com 122 modelos apenas para reconhecimento. Nomes e imagens usam metadados públicos; as fotos ficam em um cache local de 64 MiB para uso offline. Os identificadores e dados salvos do dispositivo/EQ são migrados automaticamente.

Usuários das versões 0.1.1/0.1.2 precisam instalar a versão 0.1.3 manualmente uma vez porque a chave de assinatura do atualizador mudou.

[Changelog](CHANGELOG.md) · [Model contract](docs/model-identity-presentation.md) · [Image cache](docs/product-image-cache.md)

## Fones Baseus compatíveis

O B4S só habilita controles para fones com perfil revisado. Um nome igual nunca
liga um controle por si só.

| Modelo Baseus | Nível | O que você pode controlar |
|---|---|---|
| Bass BP1 Pro | Perfil revisado | ANC, transparência e modo adaptativo, equalizador e EQ personalizado, áudio espacial, modo jogo, Bass Boost, localizar fones |
| Bass BP1 Ultra | Experimental, testado no Windows | Bateria, ANC, modo jogo, áudio espacial, Bass Boost, LDAC, proteção auditiva, gestos. EQ/SoundFit indisponíveis |
| Bass EP10 Ultra, Bowie M4s, Bowie MS1 | Experimental, não testado em hardware | Os mesmos controles do BP1 Ultra (adaptador compartilhado) |
| Bass EP10 Pro, Bowie M3s | Experimental, não testado em hardware | ANC, áudio espacial, modo jogo, Bass Boost, gestos, redução de ruído de vento (o EP10 Pro também tem presets de EQ) |
| Outros 117 modelos Baseus do catálogo | Apenas detecção | Nome e imagem na lista, sem controles |

Alguns controles (gestos de toque, detecção no ouvido, multiponto, redução de
ruído de vento, L/R adaptativo, restaurar padrões) aparecem como **Experimental**
no aplicativo e dependem de *Settings → Experimental mode*. Os recursos variam
conforme o modelo e o firmware.

### Níveis de suporte

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

Requisitos: Bun 1.4.0, Rust stable, dependências do Tauri para sua plataforma e
fones Bluetooth para testar o dispositivo:

```sh
bun install --frozen-lockfile
bun run tauri:dev
```

O aplicativo inicia em inglês e inclui vietnamita, chinês simplificado,
espanhol e português do Brasil. Altere o idioma em **Settings**; as traduções
estão incluídas e funcionam offline. Consulte o [guia de tradução](docs/translations.md)
para contribuir.

## Perguntas frequentes

**Existe um app da Baseus para Windows ou PC?**
A Baseus publica o app oficial para celulares. O B4S é um aplicativo desktop
independente e não oficial que oferece os controles acima no Windows, macOS e
Linux. Até agora, o Windows é a plataforma usada nos testes.

**Posso mudar o cancelamento de ruído (ANC) ou o EQ da Baseus pelo computador?**
Sim, nos modelos compatíveis: conecte por Bluetooth LE e troque ANC, transparência
ou modo adaptativo e escolha um preset de EQ no B4S.

**Funciona com os meus fones Baseus?**
Veja [Fones Baseus compatíveis](#fones-baseus-compatíveis). O B4S reconhece 124
modelos de fones Baseus, mas apenas os dessa tabela têm controles.

**Por que o B4S não encontra ou não controla meus fones no Windows?**
O Windows costuma listar os mesmos fones duas vezes (saída de áudio e entrada de
controle BLE). Escolha a entrada marcada como **Control**. Veja o
[guia de solução de problemas](docs/desktop-troubleshooting.md).

**É seguro? Meus dados são enviados para algum lugar?**
Os controles rodam localmente por Bluetooth. O B4S não exige conta e não envia
dados do dispositivo nem pessoais a nenhum servidor. Leia o aviso em
[Segurança e licença](#segurança-e-licença).

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
