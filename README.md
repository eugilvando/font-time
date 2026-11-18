<picture>
  <source media="(prefers-color-scheme: dark)" srcset="public/logo-white.svg">
  <img src="public/logo-dark.svg" width="230" alt="Font Time">
</picture>

# Font Time

Gerenciador de fontes para Windows 10 e 11. Digite um texto para visualizar as famílias instaladas, compare até quatro fontes e organize suas escolhas em coleções. O aplicativo também oferece ativação de fontes, pastas monitoradas e acesso ao catálogo do Google Fonts.

## Executar o projeto

Com Node.js 22 ou superior, Rust estável, ferramentas de compilação C++ da Microsoft e WebView2 instalados, execute na pasta do projeto.

~~~sh
npm ci
npm run dev
~~~

Use `npm run build` para gerar `src-tauri/target/release/font-time.exe`. O comando `npm run instalador` gera o instalador para Windows.

Os dados do Font Time ficam em `%APPDATA%\com.eugilvando.fonttime`. As atualizações automáticas dependem da configuração de uma chave pública própria em `src-tauri/tauri.conf.json`.

## Origem e licença

Font Time deriva do [Tuna Fonts de AtunFrito](https://github.com/atunfrito/tuna-fonts). O código de origem mantém os créditos e a [licença MIT](LICENSE). As licenças das fontes e dos demais componentes estão em [avisos de terceiros](THIRD-PARTY-NOTICES.md).
