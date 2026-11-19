<picture>
  <source media="(prefers-color-scheme: dark)" srcset="public/logo-white.svg">
  <img src="public/logo-dark.svg" width="230" alt="Font Time">
</picture>

# Font Time

O Font Time é um gerenciador de fontes para quem guarda muitas famílias tipográficas e usa apenas algumas em cada projeto. Adicione a pasta onde sua biblioteca já está, veja como um texto fica em cada família e ative as fontes que quiser usar em outros aplicativos. Os arquivos originais continuam na pasta que você escolheu, inclusive quando ela fica no OneDrive.

## Baixar o Font Time

| Sistema | Download da versão 0.4.0 | Como usar |
| --- | --- | --- |
| Windows 10 e 11 · 64 bits | [Instalador (.exe)](https://github.com/eugilvando/font-time/releases/download/v0.4.0/font-time_0.4.0_x64-setup.exe) | Execute para instalar ou atualizar. |
| Windows 10 e 11 · 64 bits | [Edição portátil (.zip)](https://github.com/eugilvando/font-time/releases/download/v0.4.0/font-time_0.4.0_x64-portable.zip) | Extraia o ZIP e abra `font-time.exe`. |
| macOS · Apple Silicon e Intel | [Instalador universal (.dmg)](https://github.com/eugilvando/font-time/releases/download/v0.4.0/font-time_0.4.0_macos-universal.dmg) | Abra a imagem e arraste o Font Time para Aplicativos. |
| Ubuntu 24.04 LTS · AMD64/x86-64 | [Pacote Debian (.deb)](https://github.com/eugilvando/font-time/releases/download/v0.4.0/font-time_0.4.0_ubuntu-24.04_amd64.deb) | Na pasta do download, execute `sudo apt install ./font-time_0.4.0_ubuntu-24.04_amd64.deb`. |

**macOS e Ubuntu são edições experimentais.** Os pacotes passaram pela conferência de formato e arquitetura. Os testes de instalação e uso nesses sistemas estão pendentes. A edição macOS ainda não tem notarização da Apple.

Os arquivos SHA-256 para conferir os downloads e as novidades de cada versão ficam na [página de releases](https://github.com/eugilvando/font-time/releases).

## Usar uma pasta como biblioteca

1. Na barra lateral, clique em **Adicionar pasta** e selecione a pasta principal das fontes. Para usar o OneDrive, escolha a pasta sincronizada no computador e mantenha disponíveis localmente os arquivos que quiser visualizar ou ativar.
2. Expanda **Pastas** para navegar pelas subpastas encontradas. Digite uma frase no campo **Digite um texto** para ver a mesma amostra em várias famílias.
3. Clique no círculo ao lado de uma família para ativá-la. O indicador verde mostra que ela está ativa. A seção **Ativas** reúne as famílias disponíveis para uso.
4. Quando terminar o projeto, desative as fontes que já não usa. Por padrão, as fontes ativadas pelo Font Time continuam disponíveis ao fechar o aplicativo. Você pode mudar esse comportamento em **Configurações**.

## Funcionalidades

| Recurso | O que você pode fazer |
| --- | --- |
| Biblioteca e pastas monitoradas | Navegar por subpastas, acompanhar alterações e localizar fontes nos formatos TTF, OTF, TTC e OTC. |
| Busca e filtros | Pesquisar por nome e consultar todas, recentes, favoritas, ativas ou inativas. As categorias tipográficas abrangem toda a biblioteca. |
| Prévia em lista ou grade | Escolher o texto e o tamanho da amostra. A lista ajuda a avaliar uma frase inteira, e a grade facilita reconhecer o desenho das letras. |
| Amostras e comparação | Abrir uma família para explorar seus estilos e comparar até quatro famílias lado a lado. |
| Favoritas e coleções | Marcar famílias para reencontrá-las e reunir escolhas por projeto. |
| Google Fonts | Explorar o catálogo e baixar famílias pelo aplicativo. |
| Aparência e idioma | Escolher tema claro, escuro ou do sistema e usar a interface em português do Brasil, inglês ou espanhol. |

## Novidades da versão 0.4.0

- **Personalizar a biblioteca.** Escolha o tamanho das amostras, a velocidade de rolagem e as frases disponíveis na prévia. A primeira linha é o texto padrão. Abra uma família para atribuir um grupo e etiquetas, pesquisar por esses termos e experimentar os eixos disponíveis em uma fonte variável.
- **Transferir coleções.** Em Configurações, exporte o JSON do Font Time e importe-o em outro computador. A importação mostra as famílias ausentes antes de mesclar as coleções. Escolha ZIP para arquivar também os arquivos de fonte disponíveis das famílias selecionadas nas coleções. Observe as licenças das fontes ao compartilhar esses arquivos.
- **Sincronizar pelo OneDrive.** Selecione nas configurações a mesma pasta sincronizada nos dois computadores. Com os aplicativos abertos, o Font Time reconcilia mudanças de nome, vínculos de famílias e exclusões. As fontes ativas continuam sob controle de cada computador.
- **Escolher etiquetas.** A busca encontra as etiquetas pessoais, as classificações dos metadados e, quando habilitadas, as [etiquetas públicas do Google Fonts](https://github.com/google/fonts/tree/main/tags).
- **Ativação por documento Adobe (experimental).** Em Configurações, salve o complemento na pasta de scripts de inicialização indicada pelo Adobe Bridge. Reinicie o Bridge, habilite o script e mantenha o Bridge e o Font Time abertos. A integração lê os nomes de fontes informados pelo InDesign, Photoshop e Illustrator. O reconhecimento depende dos dados que o documento disponibiliza para cada estilo. Use **Manter ativa manualmente** na ficha de uma família para continuar a usá-la após fechar o documento. A validação nos aplicativos Adobe está pendente.

## Imagens do aplicativo

Capturas do Font Time 0.3.3 no Windows, com a interface em português do Brasil.

### Biblioteca em lista

A mesma frase aparece em cada família. Use a barra lateral para filtrar fontes ativas, inativas e favoritas.

![Biblioteca do Font Time em lista, com prévias de texto, filtros e categorias](docs/font-time-biblioteca.png)

### Biblioteca em grade

A amostra curta de cada família permite observar rapidamente diferenças de peso, proporção e estilo.

![Font Time em grade, com uma amostra Aa e o nome de cada família](docs/font-time-grade.png)

## Executar o projeto no Windows

Com Node.js 22 ou superior, Rust estável, ferramentas de compilação C++ da Microsoft e WebView2 instalados, execute na pasta do projeto.

~~~sh
npm ci
npm run dev
~~~

Use `npm run build` para gerar `src-tauri/target/release/font-time.exe`. O comando `npm run instalador` gera o instalador para Windows.

Os dados do Font Time ficam em `%APPDATA%\com.eugilvando.fonttime`. O instalador Windows da versão 0.4.0 incorpora a chave pública do projeto para conferir a assinatura das próximas atualizações. Quem usa a versão 0.3.3 atualiza manualmente uma vez.

## Origem e licença

Font Time deriva do [Tuna Fonts de AtunFrito](https://github.com/atunfrito/tuna-fonts). O código de origem mantém os créditos e a [licença MIT](LICENSE). As licenças das fontes e dos demais componentes estão em [avisos de terceiros](THIRD-PARTY-NOTICES.md).
