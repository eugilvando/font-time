# Font Time 0.5.0 · Plano de produto e validação

Revisado em 19 de novembro de 2026. A versão pública de referência é a 0.4.1. A 0.5.0 tem alterações locais em andamento. O desenvolvimento está pausado para revisão deste plano.

## 1. Fluxo principal

A biblioteca de trabalho fica no OneDrive e já contém as fontes usadas com o FontBase. O fluxo diário é abrir o acervo, encontrar uma família ou estilo, avaliar a amostra e ativar apenas o necessário para o projeto. O Font Time deve preservar originais, preferências, coleções e o estado de ativação ao atualizar a instalação existente.

## 2. Biblioteca, barra superior e filtros

**Decisão de interface. Os filtros abrem a partir de um ícone na barra superior.** A barra lateral serve à navegação. A imagem anterior, que mostrava as categorias permanentemente na lateral e os estados sempre expostos, precisa ser substituída como referência de layout.

| Local | Conteúdo e comportamento |
| --- | --- |
| Barra superior | Busca, controles de amostra, ícone **Filtros** e alternância entre lista e grade. O ícone tem nome acessível e indicação quando há critérios aplicados. |
| Painel do ícone | Filtros rápidos, categorias, etiquetas e propriedades tipográficas. Oferecer os quatro intervalos da busca avançada a partir do mesmo painel. A busca textual pode ser combinada com esses critérios. |
| Barra lateral | Navegação com ícone e rótulo em cada entrada. Biblioteca (Todas as fontes), Fontes do sistema, Favoritas, Recentes, Ativas e Inativas. Google Fonts deve ser identificável em Provedores. Incluir Filtros inteligentes, Coleções e Pastas. Usar setas nas entradas expansíveis e contagens onde já forem relevantes. |
| Resultados | Na Biblioteca, as categorias abrangem todo o acervo indexado. Em uma pasta ou coleção, o resultado respeita a origem aberta. A visualização por estilo conta e filtra estilos. A visualização agrupada identifica famílias com ativação parcial. |
| Interação proposta | Clique no ícone abre o painel junto ao botão. Aplicação imediata, ação **Limpar filtros** e fechamento por Escape ou clique fora. Fechar o painel ou mudar entre lista e grade conserva os critérios. |

Exemplo para verificar o desenho: abrir Biblioteca, tocar em Filtros, escolher Sem serifa e Inativas e fechar o painel. A lista deve incluir o Regular inativo mesmo quando o Bold da mesma família estiver ativo, na visualização por estilo. Limpar os filtros recupera o acervo.

### Ícones e navegação lateral

A referência mostra folhas sobrepostas em Todas as fontes, uma fonte ou arquivo em Fontes do sistema, estrela em Favoritas, relógio em Recentes, marca do provedor em Google Fonts e pasta com símbolo de configuração em Filtros inteligentes. Preservar os rótulos ao lado dos ícones. Representar Ativas e Inativas com os indicadores de estado do Font Time. Manter espessura, tamanho, alinhamento, recuo de subníveis e contraste consistentes nos três temas e nas escalas da interface.

**Filtros inteligentes** aparece como uma entrada expansível na lateral. Como proposta de funcionamento, essa entrada pode reunir combinações reutilizáveis. A referência confirma sua posição e seu ícone. O modo de criar, editar e salvar combinações ainda precisa ser definido. Para filtrar a pesquisa em andamento, o acesso continua pelo ícone na barra superior.

### Conteúdo do painel conforme os prints

| Grupo | Requisito |
| --- | --- |
| Filtros rápidos | Favoritas, Variáveis, Ativas, Inativas, Coloridas e Duplicadas. Usar botões compactos com texto e estado selecionado. “Coloridas” identifica fontes com glifos coloridos. “Duplicadas” serve para localizar e inspecionar ocorrências e seus caminhos. |
| Categorias | Sem serifa, Com serifa, Slab serif, Manuscritas, Display, Blackletter (góticas), Monoespaçadas, Símbolos e Sem classificação. Conservar a cobertura atual de outros sistemas de escrita. A indicação de categoria depende de dados disponíveis no acervo. |
| Propriedades | Controles visuais com exemplos de letras e números, como no segundo print. Incluir descrição acessível. Os pictogramas adicionais da imagem não têm legenda suficiente para fixar seu significado. Os quatro atributos com significado confirmado estão descritos abaixo. |
| Busca avançada | **Contraste dos traços**, **peso**, **proporção** e **altura-x**. Cada atributo tem uma distribuição das fontes em forma de histograma, seleção de intervalo com limites visíveis, exemplos tipográficos e ação para restaurar o intervalo. Os valores numéricos do print ilustram uma seleção, sem estabelecer padrões para a biblioteca. |

Proposta de interação para validar: combinar grupos de filtros por interseção e categorias selecionadas por união. Ativas e Inativas funcionam como alternativas de estado. Manter a busca textual disponível quando o painel estiver aberto. Um indicador no ícone informa se há filtros aplicados. A referência usa azul para a seleção dos controles; a cor de seleção no Font Time é uma decisão de tema. Verde continua identificando ativação.

### Etiquetas nos resultados

Na lista, cada família mostra indicador de ativação, nome, formato, quantidade de estilos e a amostra tipográfica. Logo abaixo, mostrar as etiquetas em pílulas legíveis, como na referência de Bodoni, Caslon e Museo Slab. Buscar por etiqueta precisa encontrar fontes inativas. Proposta para validar: clicar em uma pílula aplica a etiqueta como filtro e a mantém acessível no painel superior. A ficha e o menu de contexto oferecem edição das etiquetas pessoais. Distinguir etiquetas pessoais, metadados e etiquetas públicas. A referência menciona contribuição comunitária; publicar e moderar contribuições exige um serviço próprio ou um provedor que aceite contribuições, além do catálogo público lido hoje.

### Dados e critérios de aceite da busca

- Documentar como identificar variáveis, glifos coloridos e duplicatas. Proposta inicial para duplicatas: agrupar cópias idênticas por conteúdo, listar seus caminhos e distinguir arquivos homônimos ou versões diferentes.
- Indexar as propriedades por estilo e guardar o resultado. Peso e altura-x podem ter metadados utilizáveis. Contraste e proporção exigem definir uma medição comparável entre fontes. Validar as quatro métricas com exemplos conhecidos antes de apresentar os intervalos como funcionais.
- Distinguir “sem dados” de um valor numérico zero. Exibir contagens coerentes com o escopo da biblioteca, a seleção de estados e a granularidade por família ou estilo.
- Ensaiar com uma fonte variável, uma colorida, duas cópias idênticas em pastas distintas, duas versões com o mesmo nome e uma fonte sem a métrica de altura-x. Conferir o resultado ao aplicar e limpar cada filtro.
- Conferir se pílulas longas, histogramas e limites dos intervalos continuam legíveis em janelas menores e entre 90% e 150% de escala.

### Referências visuais do planejamento

[Proposta visual da biblioteca com os ícones laterais e o painel de filtros aberto](referencias-0.5.0/proposta-biblioteca.png)

- [Etiquetas nas linhas de resultados](referencias-0.5.0/etiquetas.png)
- [Ícones e destinos da barra lateral](referencias-0.5.0/barra-lateral.png)
- [Painel compacto de busca, filtros e categorias](referencias-0.5.0/painel-filtros.png)
- [Busca avançada e distribuições das quatro propriedades](referencias-0.5.0/busca-avancada.png)
- [Ações do menu de contexto sobre a seleção](referencias-0.5.0/menu-contexto.png)
- [Ficha em abas e navegação de glifos](referencias-0.5.0/glifos.png)

## 3. Ficha da fonte

Cabeçalho comum com nome da família, seletor de estilo e estado de ativação. Abrir pelo Bold deve selecionar Bold. A escolha acompanha a troca de abas. As cinco abas seguem a ordem da referência.

| Aba | Conteúdo |
| --- | --- |
| **Estilos** | Amostra dos estilos encontrados, nome, peso e controle individual de ativação. A ação que afeta toda a família tem identificação própria. |
| **Licenças** | Texto de licença, endereço e créditos presentes no arquivo do estilo. Quando faltarem dados, indicar que a fonte não fornece essa informação. |
| **Cascata** | A mesma frase em vários tamanhos usando o estilo escolhido. Incluir amostras de caracteres e eixos ajustáveis quando o arquivo for variável. |
| **Glifos** | Pesquisa por caractere, nome disponível ou código Unicode, contagem filtrada e total, blocos recolhíveis e grade de símbolos. Selecionar um caractere copia seu texto. Para glifos sem Unicode com contorno disponível, identificar a ação de copiar o desenho em SVG. Testar também coleções com múltiplas faces. |
| **Detalhes** | Grupo da família, etiquetas pessoais editáveis, atributos e etiquetas do Google Fonts separados, metadados do estilo e caminhos para localizar os arquivos. Campos com tamanho legível, etiquetas removíveis e listas longas recolhíveis. |

## 4. Menu do botão direito

**Ativação direta pelo menu.** Clique direito sobre uma fonte abre as ações junto ao cursor na lista e na grade. Escolher **Ativar** ou **Desativar** executa a ação ali mesmo e atualiza o indicador e as contagens. Abrir ou dispensar o menu conserva o estado atual. Ao clicar em um item da seleção múltipla, o menu usa essa seleção. Ao clicar fora dela, usa o item apontado. Oferecer navegação por teclado e acomodar o menu nos cantos da janela.

1. **Ativar** e **Desativar** são as primeiras ações do menu, acima de um separador. Para um estilo selecionado, a ação vale apenas para esse estilo. Na visão agrupada, indicar quando a ação afeta a família inteira. Na seleção múltipla, **Ativar (n)** e **Desativar (n)** mostram as quantidades elegíveis para cada ação. Por exemplo, oito itens com cinco inativos e três ativos oferecem Ativar (5) e Desativar (3), se todos puderem ser alterados.
2. Após o primeiro separador, **Ver família** abre a ficha da fonte apontada e conserva o estilo de origem.
3. **Gerenciar etiquetas** abre o editor de etiquetas. **Editar grupo** permite escolher, criar ou remover a associação ao grupo.
4. Após outro separador, **Excluir do disco** apresenta os caminhos elegíveis antes de enviar os arquivos confirmados à lixeira. Identificar o efeito sobre a pasta sincronizada do OneDrive. Cancelar preserva todos os arquivos. Fontes do sistema e externas ficam protegidas.

## 5. Ativação, cores e escala

- Ativar ou desativar Regular, Bold ou Italic separadamente quando o formato permitir. Para estilos inseparáveis do mesmo arquivo, explicar a limitação no controle. Conferir o estado real ao reabrir o app.
- Usar indicador verde preenchido para ativo, contorno para inativo e indicação própria para parcial ou protegido. Aplicar o mesmo significado à lista, grade, árvore de pastas e ficha. Manter texto e amostras inativas legíveis nos temas Claro, Escuro e Sistema.
- Disponibilizar **Escala da interface** nas configurações. Faixa proposta no protótipo local: 90%, 100%, 110%, 125% e 150%. A preferência afeta menus, controles, filtros e abas. O tamanho da amostra tipográfica continua com seu próprio controle.
- Fechar a janela para a bandeja e encerrar o processo são ações distintas. Com **Desativar fontes ao sair** desligado, as ativações manuais persistem após sair. Com a preferência ligada, o Font Time desfaz as ativações que gerencia.

### Indicadores na árvore de pastas

Os ícones de navegação identificam seções. Cada pasta e subpasta conserva seu indicador circular de ativação ao lado do nome, a seta de expansão quando houver filhos e a contagem à direita. O fundo de seleção identifica a pasta aberta independentemente do estado das fontes.

| Estado do conteúdo | Indicador da pasta |
| --- | --- |
| Todas as fontes ativas | Círculo verde preenchido |
| Parte das fontes ativa | Círculo com contorno verde e centro vazio |
| Nenhuma fonte ativa | Círculo com contorno neutro e centro vazio |

**Preferência definida:** verde de ativação mais vivo e saturado. **Amostras propostas:** `#009A32` no tema claro e `#00E54D` no escuro. O tema Sistema acompanha a variante em uso. Conferir os círculos no tamanho real da barra lateral, inclusive sobre o fundo da pasta selecionada. Validar transições entre os três estados ao ativar e desativar estilos em subpastas.

[Referência enviada](referencias-0.5.0/estado-das-pastas.png) · [Amostra em tema claro e escuro](referencias-0.5.0/amostra-verde-pastas.svg)

## 6. OneDrive, organização e preferências

- Selecionar a pasta principal e monitorar as subpastas no local. Exibir caminhos e contagens na árvore. Manter o aviso sobre monitoramento e os controles de gerenciamento em Configurações.
- Usar essa pasta como destino para importações por arquivo e downloads do Google Fonts. Antes de escolher uma pasta principal, usar a biblioteca interna. Se a pasta escolhida ficar indisponível, informar o problema e permitir nova tentativa.
- Catalogar e visualizar sem ativação automática. Arquivos já presentes, nomes repetidos, arquivos apenas online e a troca de pasta precisam preservar os originais.
- Preservar favoritos, recentes, coleções, comparação de famílias, amostras editáveis, velocidade de rolagem, agrupamento e restauração dos grupos. Manter os idiomas português do Brasil, inglês e espanhol.
- Coleções usam o formato próprio do Font Time e podem ser exportadas em JSON ou em ZIP com fontes. A sincronização via OneDrive compartilha a organização. A ativação permanece local em cada computador.
- A integração opcional via Adobe Bridge requer teste com documentos reais do InDesign, Illustrator e Photoshop. As etiquetas públicas disponíveis vêm do Google Fonts. O provedor FontMarket continua em aberto.
- Preservar a identidade Font Time na janela, barra de tarefas e instaladores. A tela Sobre explica a origem do projeto e mantém a atribuição solicitada. O GitHub deve listar funcionalidades, imagens e downloads correspondentes à versão entregue.

## 7. Situação ao interromper o desenvolvimento

| Situação | Evidência e trabalho restante |
| --- | --- |
| Versão distribuída | 0.4.1. O envio do código da 0.5.0 ao GitHub e a geração dos novos instaladores estão pendentes. |
| Alterações locais já exercitadas | Teste isolado no Windows ativou e desativou Bold preservando Regular. Outro teste cobriu importação repetida e colisão de nomes. Testes de interface em dados de demonstração cobriram seleção múltipla, menu, grupos, etiquetas, filtros, pesquisa de glifos, tema escuro e escala a 125%. |
| Filtros na barra superior | O código ainda usa categorias laterais e controle de estado separado. O painel do ícone, os novos filtros rápidos, as categorias ampliadas, as pílulas nas linhas e a busca avançada com histogramas precisam de implementação e teste. |
| Ficha em abas | A estrutura das cinco abas foi iniciada depois dos testes acima. A leitura de licença, os rótulos nos três idiomas e a validação da navegação e do estilo entre abas estão pendentes. |
| Verificação final | Falta validar o conjunto na instalação do Windows. Sincronização em dois computadores, documentos Adobe reais e uso instalado em macOS e Ubuntu continuam com testes próprios pendentes. |

## 8. Ordem de retomada e critérios de entrega

1. **Validar o desenho.** Preparar uma imagem com o painel aberto pelo ícone superior e outra vista da ficha com as cinco abas e Glifos selecionada. Usar o plano revisado como referência de implementação.
2. **Concluir a navegação.** Definir e validar os dados de categorização e das quatro métricas, implementar o painel de filtros e os histogramas, incluir as etiquetas nos resultados, finalizar as abas, a leitura dos metadados e as traduções. Revisar o comportamento de seleção, os indicadores e a escala.
3. **Testar com uma biblioteca controlada.** Cobrir estados mistos na mesma família, lista e grade, busca combinada, clique direito em seleção múltipla, exclusão cancelada e confirmada em arquivos descartáveis, importação duplicada, indisponibilidade da pasta e persistência ao encerrar. Conferir glifos latinos, símbolos monetários, fonte com mais de 4.000 glifos, estilo alternativo e informação de licença ausente.
4. **Testar apresentação e atualização.** Conferir os três temas e idiomas, janela pequena, escala de 90% a 150%, teclado e retorno de foco. Validar a atualização da instalação Windows preservando biblioteca e preferências.
5. **Publicar.** Atualizar documentação, imagens e Git. Gerar instalador Windows, ZIP portátil, DMG universal para Mac e DEB para Ubuntu 24.04 AMD64. Verificar arquivos, hashes, assinatura do atualizador Windows e links públicos. Informar o estágio dos testes práticos de cada plataforma.

O levantamento inicial permanece disponível em [Planejamento de referência do FontBase](planejamento-fontbase.md). Para as decisões de interface da 0.5.0, usar este documento.
