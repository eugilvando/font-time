# Font Time — planejamento para substituir o FontBase no Windows

Referência de 18 de novembro de 2026. Este documento reúne as decisões sobre ativação, a comparação com o código do Font Time 0.2.3 e os ajustes visíveis na captura do usuário. Os valores da captura representam o perfil pessoal de referência. Preferências já salvas no Font Time devem ser respeitadas.

## Objetivo principal

Usar como origem a pasta de fontes que o usuário já mantém no OneDrive e utiliza com o FontBase. Selecioná-la uma vez, consultar o acervo e suas subpastas no local e ativar manualmente as poucas fontes necessárias a cada trabalho. Preservar os arquivos originais e guardar o índice e as preferências do Font Time separadamente. Fontes adicionadas à pasta depois também entram apenas para consulta.

## 1. Ativação de fontes

| Ação | Font Time hoje | Comportamento planejado |
| --- | --- | --- |
| Adicionar arquivos | Arrastar fontes para a janela aciona a instalação. O catálogo do Google também tem uma ação de instalar. | Adicionar à biblioteca permite pesquisar e visualizar a fonte. Uma fonte nova entra inativa. Se já estiver disponível no Windows, preservar e mostrar seu estado real. Distinguir “Adicionar” de “Ativar” em cada ponto de entrada. |
| Monitorar uma pasta | A leitura é recursiva e carrega automaticamente fontes da pasta na sessão do Windows. | Indexar arquivos existentes e novos para prévia. Ativar somente quando o usuário pedir. Manter os originais no lugar. |
| Indicar a ativação | Lista e grade usam um círculo preenchido com a cor do texto e uma marca de seleção. | Usar verde com marca de seleção para fontes ativas e contorno para inativas. Manter identificação e proteção das fontes bloqueadas. O indicador deve refletir o resultado real da operação. |
| Fechar a janela para a bandeja | O processo continua e as fontes da sessão continuam disponíveis. | Preservar esse comportamento, independentemente do interruptor de desativação ao sair. |
| Encerrar o processo | O aplicativo descarrega as fontes temporárias das pastas monitoradas. As fontes instaladas têm outro ciclo de vida. | Com **Desativar fontes ao sair do Font Time** desligado por padrão, as ativações manuais continuam disponíveis após sair por completo. Ligado, o app desfaz apenas as ativações que ele próprio gerencia. Preservar as fontes instaladas por Windows, FontBase e outros aplicativos. |

O tutorial do FontBase confirma que as fontes entram inativas. A versão 2.25.0, de 26 de janeiro de 2026, passou a permitir manter fontes ativas com o processo encerrado. Para o comportamento ao sair, a nota mais recente atualiza o tutorial. [Ativação no FontBase](https://fontba.se/learn/font-activation) · [Histórico oficial](https://fontba.se/updates)

## 2. Preferências da captura

| Configuração | Referência pessoal | Situação e proposta para o Font Time |
| --- | --- | --- |
| Pasta raiz | Biblioteca do FontBase no OneDrive | Criar um seletor para a pasta da biblioteca do Font Time. Mostrar o caminho e o botão **Alterar**. A troca aponta para outro local, preserva a pasta anterior e deixa a cópia dos arquivos a cargo de uma ação explícita. Pastas monitoradas continuam sendo referências aos originais. |
| Tamanho padrão da prévia | 42 | O controle de tamanho já existe e começa em 30 numa configuração nova. Oferecer 42 para reproduzir o perfil da captura. |
| Velocidade de rolagem | Normal | Incluir a preferência para a lista de fontes, mantendo a opção Normal. |
| Agrupar fontes em famílias | Ligado | O Font Time já agrupa por família. Permitir alternar entre famílias agrupadas e estilos individuais. |
| Restaurar agrupamentos personalizados | Botão Restaurar | Implementar junto da edição manual de famílias. Restaurar os nomes lidos dos arquivos e desfazer somente os agrupamentos personalizados. Preservar arquivos, favoritos e coleções. |
| Textos de amostra | Cinco frases, primeira como padrão | O Font Time já salva uma frase padrão. Planejar uma lista editável e ordenável. A primeira frase alimenta a prévia inicial e a lista pode conter amostras em português. |
| Ocultar na bandeja ao fechar | Ligado | Já existe. Identificar claramente as ações **Fechar janela** e **Sair do Font Time**. |
| Iniciar com o Windows | Desligado | Já existe. Manter independente da persistência das fontes. |
| Iniciar oculto | Ligado | Hoje, a inicialização automática já pede abertura oculta. Expor essa preferência separadamente. Para este perfil, propor abertura oculta no início automático e janela visível quando o usuário abrir o app pelo atalho. |
| Atualizar automaticamente | Ligado | O Font Time tem busca por novas versões e instalação acionada pelo usuário. A instalação automática é uma etapa distinta e depende de um canal de versões assinado do próprio projeto. |
| Desativar fontes ao sair | Desligado | Incluir em **Configurações → Ativação de fontes**, conforme a regra da seção 1. |
| Tema | Preferência definida na conversa | Exibir **Sistema**, **Claro** e **Escuro**. Propor Sistema na primeira abertura e acompanhar mudanças do Windows enquanto a janela estiver aberta. O código atual resolve o tema inicialmente, mas o seletor só oferece Claro e Escuro. |

O FontBase distingue importação para a biblioteca de monitoramento no local. Seu histórico também registra a adoção do tema do sistema por padrão. [Adicionar arquivos e pastas](https://fontba.se/learn/adding-fonts) · [Histórico oficial](https://fontba.se/updates)

## 3. Ordem de implementação proposta

1. **Pasta existente do OneDrive e ativação sob demanda.** Entregar primeiro a seleção e o monitoramento dessa pasta. Separar catalogação, prévia e ativação no Windows. Registrar quais ativações pertencem ao Font Time e tratar duplicatas, reinício do aplicativo e arquivos indisponíveis. Entregar o indicador verde, a preferência ao sair e os três temas nesta etapa.
2. **Configurações e uso da biblioteca existente.** Oferecer a troca da pasta nas preferências, os controles de exibição, as amostras e a inicialização oculta independente. Fazer o primeiro ensaio com uma pasta de fontes de teste antes de apontar para a biblioteca de trabalho. Monitorar a pasta usada pelo FontBase permite ler os arquivos. Importar as coleções e os favoritos daquele aplicativo exigiria validar o formato e o mapeamento dos dados.
3. **Recursos adicionais conforme o uso.** O Font Time já oferece busca, filtros de ativas e inativas, favoritos, coleções com ativação em lote, comparação e mapa de caracteres. Avaliar depois a exportação de coleções em ZIP e os controles de eixos de fontes variáveis se fizerem parte do fluxo de trabalho. [Coleções no FontBase](https://fontba.se/learn/collections) · [Fontes variáveis](https://fontba.se/learn/variable-fonts)

## 4. Como verificar a substituição

- Usar uma pasta de teste no OneDrive. Comparar a lista de fontes ativas do Windows antes e depois de apontar o Font Time para a pasta. A simples catalogação deve preservar o conjunto original. Com um arquivo disponível somente online, identificar sua disponibilidade e evitar baixar todo o acervo para iniciar a consulta.
- Adicionar uma pasta com duas fontes inéditas. As duas exibem a prévia no Font Time e continuam ausentes no seletor de fontes de outro programa. Adicionar um terceiro arquivo à pasta produz o mesmo resultado.
- Ativar uma delas. Confirmar o indicador verde e abrir outro programa para verificar que somente a fonte escolhida passou a estar disponível.
- Com a desativação ao sair desligada, encerrar pelo menu **Sair**, confirmar o fim do processo e abrir um novo programa para verificar a disponibilidade da fonte. Reabrir o Font Time e conferir o estado indicado.
- Com a opção ligada, primeiro fechar a janela para a bandeja, depois sair pelo menu. Somente a segunda ação deve desfazer as ativações gerenciadas pelo Font Time. Incluir no teste uma fonte que já estava instalada antes e uma ativada por outro gerenciador.
- Trocar o tema do Windows com o Font Time aberto. **Sistema** acompanha a mudança na janela e no menu da bandeja. **Claro** e **Escuro** mantêm a escolha explícita.
- Trocar a pasta raiz entre duas pastas de teste e verificar que os arquivos originais continuam intactos. Alterar a primeira amostra de texto, reiniciar o app e conferir a prévia e o tamanho salvos.

## 5. Opções avançadas das capturas

Esta avaliação descreve possibilidades para o Font Time. A prioridade é preservar a ativação manual e permitir que o usuário leve sua organização para outro computador.

| Opção da captura | Viabilidade e proposta |
| --- | --- |
| Tema | Primeira etapa. Expor Sistema, Claro e Escuro, conforme a seção 2. |
| Importar e exportar coleções | Primeira etapa de organização. O Font Time já guarda os nomes das coleções e as famílias associadas. Propor um arquivo de intercâmbio próprio, com versão, que preserve esses vínculos. Na importação, mostrar fontes ausentes, permitir mesclar e manter a ativação atual. Esse arquivo pode ser levado a outro computador sem incluir os arquivos de fonte. A leitura direta do formato FontBase depende de validar uma exportação de exemplo. |
| Sincronizar coleções | Segunda etapa, se houver uso em dois computadores. O OneDrive que o usuário já utiliza é uma opção de transporte. A sincronização precisa identificar coleções e famílias sem depender do caminho absoluto, conciliar edições simultâneas e lembrar exclusões. Um backup manual no OneDrive já é possível como desenho da primeira etapa. Sincronizar organização não deve ativar fontes no outro computador. |
| Ocultar tags do sistema | Viável localmente. Gerar etiquetas a partir das categorias e dos metadados que o Font Time já lê, como serifada, peso e itálico. Permitir ocultar as sugestões automáticas e manter visíveis as etiquetas criadas pelo usuário. |
| Ocultar tags da comunidade | Depende de uma fonte de dados coletiva. Propor primeiro etiquetas pessoais pesquisáveis. Adiar o controle de etiquetas comunitárias até existir um catálogo compartilhado com cobertura suficiente. |
| Ativar fontes ausentes ao abrir um documento Adobe | Etapa experimental, opcional e desligada por padrão. Um complemento próprio identifica a fonte e o estilo pedidos e consulta a biblioteca local. Começar por um aplicativo e validar com documentos reais. InDesign é candidato porque expõe eventos de documento. Illustrator e Photoshop exigem validação separada. A simples importação de uma pasta continua manual para ativação. |
| Desativar ao fechar o documento Adobe | Depende da integração anterior. Manter um registro do motivo da ativação e dos documentos que usam a fonte. Ao fechar o último documento, desfazer somente a ativação feita por essa integração, se a opção estiver ligada. Uma fonte ativada manualmente continua disponível. Essa opção é independente de desativar ao sair do Font Time. |

**Critérios de verificação.** Exportar e reimportar uma coleção com uma fonte ausente deve preservar a associação e todas as ativações anteriores. Ocultar as tags automáticas deve preservar as etiquetas pessoais. Na futura integração Adobe, abrir dois documentos que compartilham uma fonte e fechar apenas um deve manter a fonte disponível para o outro.

O FontBase documenta coleções, etiquetas compartilhadas e integração com os aplicativos da Adobe. A viabilidade acima considera a implementação própria do Font Time. [Coleções](https://fontba.se/learn/collections) · [Histórico de recursos](https://fontba.se/updates) · [Recursos Awesome](https://fontba.se/awesome) · [Eventos de documento do InDesign](https://developer.adobe.com/indesign/uxp/dom/api/d/document-event/)
