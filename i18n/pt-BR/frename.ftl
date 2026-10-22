# frename UI text in Brazilian Portuguese. Same ids, attributes and $arguments as
# i18n/en/frename.ftl, in the same order (checked by the tests in src/i18n.rs).
#
# Glossary (docs/design/localization.md): every string uses these terms; the reader is addressed
# impersonally (the infinitive or the imperative for actions and labels, «se» constructions or
# the impersonal third person in sentences), never with «tu», and with «você» only where a sentence
# cannot do without a subject. Spelling and terms follow Brazil (arquivo, vídeo, pasta,
# configurações), as Premiere Pro in pt-BR does.
# Typography: no space before «:», «;», «!», «?»; a space before «%» is not used (50%); quotes are
# “…”; decimal comma and dollars as «US$» (US$ 0,01, 29,97 fps).
# Keys: Ctrl stays Ctrl, Shift stays Shift, Home stays Home, Enter stays Enter (as on Brazilian
# keyboards and in Premiere Pro pt-BR).
#   tag / tags                        tag / tags, feminine (never «rótulo»: that is Premiere's Label)
#   in / out points                   pontos de entrada/saída
#   comment                           comentário
#   subtitles                         legendas
#   screenshot / frame                quadro
#   marker (Premiere)                 marcador
#   range (a marker with a duration)  intervalo
#   subclip (Premiere)                subclipe
#   Description column (Premiere)     coluna “Descrição”
#   checked (files)                   selecionado (not «marcado»: too close to «marcador»)
#   All / Invert (check boxes)        Todos / Inverter
#   video / clip / file               vídeo / clipe / arquivo (all masculine)
#   tag panel / tag list              painel de tags / lista de tags (the folder's list of tags)
#   file name                         nome do arquivo
#   a tag name inside a sentence      in “…” quotes: “Commented”
#   untagged (filter)                 Sem tags
#   changed / unchanged / failed      alterado / nada a alterar / falhou
#   Cancel / Stopping…                Cancelar / Interrompendo…
#   log, "see the log"                registro, «ver o registro»
#   System (language list)            Sistema (…)
#   batch action / batch mode         ação em lote / modo em lote
#   request (to the AI)               solicitação
#   moments (AI)                      momentos
#   file list / folder                lista de arquivos / pasta (feminine)
#   settings                          Configurações
#   update (a newer frename)          atualização
#   AI description (Describe with AI) descrição com IA (Descrever com IA)
#   subtitles generation (Soniox)     transcrição / legendas
#   cue (a subtitle line)             legenda
#   keyring                           chaveiro (Chaveiro do macOS, Gerenciador de Credenciais do Windows)
#   timecode / frame rate / drop-frame  timecode / taxa de quadros / drop-frame
#   playhead                          indicador de reprodução
#
# Plurals: Portuguese has the categories one / other here, other as the default. «one» is 1 (in
# CLDR also 0, which reads naturally: 0 arquivo, 1 arquivo, 2 arquivos). Participles and
# adjectives agree with their noun in each variant (arquivo selecionado / arquivos selecionados,
# pasta aberta); where a figure stands alone, invariable phrases (sem alterações, não concluído)
# avoid guessing the gender.

## Language names: the same in every file, each written in its own language.

language-name-en = English
language-name-ru = Русский
language-name-de = Deutsch
language-name-fr = Français
language-name-es = Español
language-name-it = Italiano
language-name-pt-BR = Português (Brasil)
language-name-uk = Українська

## Settings window

settings-window-title = Configurações
settings-page-interface = Interface
settings-page-saving = Salvamento
settings-page-ai = Descrever com IA
settings-close = Fechar
settings-apply-note = As alterações são aplicadas na hora. Tab alterna entre as opções, Ctrl+Tab entre as páginas.
settings-language = Idioma
settings-language-system = Sistema ({ $language })
settings-video = Vídeo
settings-video-autoplay = Reproduzir os vídeos automaticamente ao abrir
settings-subtitle-size = Tamanho das legendas
settings-subtitle-size-small = Pequeno
settings-subtitle-size-medium = Médio
settings-subtitle-size-large = Grande
settings-subtitle-size-extra-large = Muito grande
settings-timecode = Exibição do tempo
settings-timecode-hint = Como os tempos são escritos sob o vídeo, nas listas de marcadores e de legendas e nos indicadores In / Out, e o tempo do indicador de reprodução que Ctrl+Shift+C copia. O Premiere Pro e o DaVinci Resolve mostram horas:minutos:segundos:quadros na taxa de quadros do clipe; 29,97 e 59,94 fps são drop-frame. Um clipe sem taxa de quadros constante mantém o estilo padrão. Os arquivos mantêm os próprios formatos.
settings-timecode-default = Padrão (mm:ss)
settings-timecode-premiere = Premiere Pro (HH:MM:SS:FF)
settings-timecode-resolve = DaVinci Resolve (HH:MM:SS:FF)
settings-tags = Cores das tags
settings-tags-monochrome = Monocromático
settings-tags-monochrome-hint = As tags adicionadas aparecem em um cinza mais escuro, as outras em um mais claro, sem outras cores.
settings-file-names = Nomes de arquivo
settings-tags-space-after = Espaço depois de cada tag
settings-tags-space-example = Food. Goat. clip.mp4
settings-tags-space-note = Os arquivos mantêm os nomes até serem renomeados ou salvos.
settings-tags-space-add = Adicionar o espaço aos nomes de arquivo…
settings-tags-space-remove = Remover o espaço dos nomes de arquivo…
settings-comments = Comentários
settings-comments-in-video = Dentro do arquivo de vídeo
settings-comments-in-video-hint = XMP, a coluna “Descrição” do Premiere Pro
settings-comments-text-file = Em um arquivo de texto ao lado do vídeo
settings-comments-text-file-example = clip.comment.txt
settings-comments-note = Os arquivos mantêm os comentários onde estão até serem movidos.
settings-comments-move-into-videos = Mover os comentários para os vídeos…
settings-comments-move-into-text-files = Mover os comentários para arquivos de texto…
settings-commented-tag = Adicionar uma tag aos vídeos com comentário
settings-commented-tag-name = Tag
settings-commented-tag-hint = É adicionada ao escrever um comentário em um vídeo, p. ex. { $tag }.IMG_0424.MOV, e removida ao apagá-lo (as descrições com IA não contam). No mais, pode ser alterada livremente.
settings-commented-tag-off = Os vídeos com comentário não recebem tag.
settings-in-out = Pontos de entrada/saída
settings-in-out-in-video = Dentro do arquivo de vídeo
settings-in-out-in-video-hint = Um marcador de subclipe no Premiere Pro
settings-in-out-comment = No comentário, em uma linha
settings-in-out-comment-example = In/Out: 00:01:05.250 – 00:02:10.000
settings-in-out-note = Os arquivos mantêm os pontos de entrada/saída onde estão até serem movidos.
settings-in-out-move-into-videos = Mover os pontos de entrada/saída para os vídeos…
settings-in-out-move-into-comments = Mover os pontos de entrada/saída para os comentários…
settings-markers = Marcadores e intervalos
settings-markers-in-video = Dentro do arquivo de vídeo
settings-markers-in-video-hint = XMP, exibidos no clipe no Premiere Pro
settings-markers-comment = No comentário, uma linha por marcador
settings-markers-comment-example = 0:41–0:47 — Lion
settings-markers-note = Os arquivos mantêm os marcadores onde estão até serem movidos.
settings-markers-move-into-videos = Mover os marcadores para os vídeos…
settings-markers-copy-into-comment = Copiar os marcadores para os comentários…
settings-markers-hint = Tanto pontos quanto intervalos, inclusive os momentos encontrados pela IA.
settings-updates = Atualizações
settings-version = Versão
settings-old-title = Configurações de um frename antigo
settings-old-hint = Traz as configurações de um frename executado a partir de uma pasta zip.
settings-old-scheduled = As configurações serão importadas quando o frename for reiniciado
settings-old-not-found = Nenhum frename.exe com um frename.db em { $folder }
settings-old-failed = Não foi possível importar: { $reason }
settings-old-import-button = Importar de uma pasta de frename antigo…
settings-ai = Descrever com IA
settings-ai-key-label = Chave de API da Anthropic
settings-ai-key-placeholder = sk-ant-…
settings-ai-key-get = Obtenha uma chave em console.anthropic.com → API keys.
settings-ai-key-remove-confirm = Remover a chave da Anthropic salva?
settings-ai-used-by = Usada por Descrever com IA no modo em lote e nos marcadores (✨, Ctrl+F2), quando um modelo Claude é escolhido.
settings-ai-openai-key-label = Chave de API da OpenAI
settings-ai-openai-key-placeholder = sk-…
settings-ai-openai-key-get = Obtenha uma chave em platform.openai.com/api-keys.
settings-ai-openai-key-remove-confirm = Remover a chave da OpenAI salva?
settings-ai-openai-used-by = Usada por Descrever com IA no modo em lote e nos marcadores (✨, Ctrl+F2), quando um modelo GPT é escolhido.
settings-ai-gemini-key-label = Chave de API do Gemini
settings-ai-gemini-key-placeholder = AIza…
settings-ai-gemini-key-get = Obtenha uma chave em aistudio.google.com/apikey.
settings-ai-gemini-key-remove-confirm = Remover a chave do Gemini salva?
settings-ai-gemini-used-by = Usada por Descrever com IA no modo em lote e nos marcadores (✨, Ctrl+F2), quando um modelo Gemini é escolhido.
settings-ai-model-needs-key = { $model } — adicione antes a chave de { $provider }
settings-ai-model-label = Modelo
settings-ai-language-label = Idioma da descrição
settings-ai-hint = Haiku, GPT-4.1 mini e Gemini Flash-Lite são os mais baratos; Sonnet, Opus, GPT-4.1 e Gemini Flash percebem mais detalhes. Cada modelo precisa da chave da própria empresa.
settings-ai-moments-label = Momentos
settings-ai-moments-important = Só o que se destaca
settings-ai-moments-full = Cobrir o clipe inteiro
settings-ai-moments-important-hint = Também sugere onde definir In e Out (o selo IA ao lado de IN e OUT)
settings-ai-moments-full-hint = Um momento para cada trecho, inclusive os calmos; sem sugestão de In e Out
settings-ai-tag-suggestions = Sugerir também tags a partir das tags da pasta
settings-ai-tag-suggestions-hint = Na mesma solicitação, a IA diz quais tags da pasta combinam com cada clipe. Uma tag sugerida só é adicionada quando você clica nela.
settings-subtitles = Legendas
settings-subtitles-key-label = Chave de API da Soniox
settings-subtitles-key-placeholder = Cole a chave
settings-subtitles-key-get = Obtenha uma chave em console.soniox.com. O áudio é enviado à Soniox.
settings-subtitles-key-remove-confirm = Remover a chave da Soniox salva?
settings-subtitles-languages-label = Idiomas
settings-subtitles-languages-locked = Salve a chave para escolher entre todos os idiomas que a Soniox conhece.
settings-subtitles-languages-loading = Obtendo os idiomas da Soniox…
settings-subtitles-languages-none = Nenhum selecionado: detecção automática.
settings-subtitles-languages-hint = Os idiomas falados no material, como dicas.
settings-subtitles-cue-length-label = Tamanho das legendas
settings-subtitles-cue-short = Curto
settings-subtitles-cue-short-hint = Uma linha, até 8 s
settings-subtitles-cue-sentence = Uma frase
settings-subtitles-hint = Usado por Gerar legendas no modo em lote. O tamanho das legendas vale para as legendas novas.
settings-key-store-windows = Gerenciador de Credenciais do Windows
settings-key-store-macos = Chaveiro do macOS
settings-key-store-other = Chaveiro do sistema
settings-key-unavailable = Não foi possível abrir o chaveiro do sistema
settings-key-unavailable-hint = Ele pode estar bloqueado, ou não existir (como o GNOME Keyring ou o KWallet). As Configurações verificam de novo a cada abertura.
settings-key-remove-confirm-hint = Será preciso colá-la de novo.
settings-key-remove-ask = Remover…
settings-key-remove = Remover chave
settings-key-keep = Manter
settings-key-replace = Substituir…
settings-key-hide = Ocultar
settings-key-show = Mostrar
settings-key-save = Salvar chave
settings-key-get = Obter uma chave
settings-key-billing = Adicionar crédito
settings-key-cancel = Cancelar
settings-key-saved-in = Salva neste computador ({ $store })
settings-key-save-into = “Salvar chave” a guarda neste computador ({ $store }).

## Batch mode

batch-checked-count = { $count ->
    [one] { $count } arquivo selecionado
   *[other] { $count } arquivos selecionados
}
batch-no-key = Sem chave
batch-group-move = Mover entre locais
batch-group-fix = Corrigir nomes e vídeos
batch-group-paid = Serviços pagos
batch-reason-none-checked = Nenhum arquivo selecionado
batch-reason-reading = Lendo a duração dos clipes…
batch-check-all = Selecionar { $count ->
    [one] { $count } arquivo
   *[other] todos os { $count } arquivos
}
batch-job-running = em execução
batch-job-finished = concluído
batch-job-stopped = interrompido
batch-progress-files = { $finished } de { $total ->
    [one] { $total } arquivo
   *[other] { $total } arquivos
}
batch-time-left = faltam cerca de { $time }
batch-time-estimating = estimando o tempo…
batch-time-spent = { $time } até agora
batch-runs-in-background = Você pode continuar trabalhando enquanto isso roda
batch-clip-in-work = Em uso: { $action }
batch-folder-held = Ocupada: { $action }
batch-indicator-hint = Clique para abrir o painel e Cancelar
batch-action-prepare = Preparar clipes
batch-action-prepare-hint = Para cada clipe selecionado, de uma só vez: legendas (Soniox) se ele não tiver, depois a descrição com IA: um resumo, seus momentos como marcadores, o comentário e sugestões de tags. Um preço, um limite e um progresso para tudo.
batch-prepare-step-off = desativada
batch-prepare-nothing = Nada a fazer para estes clipes
batch-prepare-report-cap = Gasto { $spent } do limite de { $cap }
batch-option-steps = Etapas
batch-prepare-step-subtitles = Criar as legendas primeiro (Soniox)
batch-prepare-step-description = Descrever os clipes com IA
batch-prepare-cap = Parar de gastar em
batch-prepare-cap-none = Sem limite (dólares americanos)
batch-prepare-cap-invalid = O limite é um número de dólares, como 5
batch-prepare-left = A última execução parou com { $count ->
    [one] { $count } clipe restante
   *[other] { $count } clipes restantes
}
batch-prepare-check-left = Selecioná-los
batch-prepare-estimating = Calculando o que cada etapa vai fazer…
batch-prepare-plan-subtitles = Legendas
batch-prepare-plan-description = Descrição
batch-prepare-plan-none = nada a fazer
batch-prepare-plan-subtitles-line = { $videos ->
    [one] { $videos } vídeo
   *[other] { $videos } vídeos
} para transcrever, { $duration } de áudio
batch-prepare-plan-description-line = { $videos ->
    [one] { $videos } vídeo
   *[other] { $videos } vídeos
} para descrever
batch-prepare-resume-note = Cada etapa pula o que já existe, então uma execução interrompida é retomada executando-a de novo: nada é pago duas vezes.
batch-prepare-run = Preparar { $clips ->
    [one] { $clips } clipe
   *[other] { $clips } clipes
} · cerca de { $dollars }
batch-service-prepare = Soniox + { $provider }
batch-prepare-groups-title = Material parecido
batch-prepare-groups-hint = Clipes que parecem a mesma cena: duplicatas, reexportações, uma câmera que não se mexeu.
batch-prepare-group-clips = { $count } clipes
batch-prepare-check-group = Selecionar estes clipes
batch-solo-queued = Na fila: { $action }
batch-suggest-in-job = Já há uma descrição deste clipe na tarefa em andamento, sem sugestões de tags. Peça de novo quando ela terminar
batch-suggest-no-tags = Ainda não há tags nesta pasta para sugerir: adicione algumas antes
batch-solo-gave-up = Não foi possível se preparar
batch-solo-withdrawn = Retirado: { $action }
batch-solo-in-work = { $action } está em andamento. Cancele pela faixa ou pela página da tarefa
batch-waiting-title = { $count ->
    [one] { $count } execução na espera
   *[other] { $count } execuções na espera
}
batch-waiting-remove = Remover
batch-indicator-waiting = { $count } na espera
file-workspace-describe-clip = Descrever este clipe com IA
file-workspace-subtitles-clip = Gerar legendas para este clipe
file-workspace-describe-clip-queued = Descrever com IA: aguardando a vez. Clique para retirar
file-workspace-describe-clip-running = Descrever com IA: em andamento
file-workspace-suggest-tags-clip = Sugerir tags para este clipe com IA (Ctrl+F6)
file-workspace-suggest-tags-clip-queued = Sugerir tags: aguardando a vez. Clique para retirar
file-workspace-suggest-tags-clip-running = Sugerir tags: em andamento
file-workspace-subtitles-clip-queued = Gerar legendas: aguardando a vez. Clique para retirar
file-workspace-subtitles-clip-running = Gerar legendas: em andamento
batch-run-again = Executar de novo em { $count ->
    [one] { $count } arquivo
   *[other] { $count } arquivos
}
batch-result-done = Concluído: { $total ->
    [one] { $total } arquivo
   *[other] { $total } arquivos
}
batch-result-done-detail = Alterados: { $changed }, sem alterações: { $unchanged }
batch-result-problems = Concluído com problemas: { $failed } de { $total ->
    [one] { $total } arquivo não concluído
   *[other] { $total } arquivos não concluídos
}
batch-figure-unchanged = sem alterações
batch-figure-not-done = não concluídos
batch-figure-not-reached = não alcançados
batch-files-not-done = Arquivos não concluídos
batch-table-file = Arquivo
batch-table-why = Motivo

batch-run-move-comments = Mover { $count ->
    [one] { $count } comentário
   *[other] { $count } comentários
}
batch-run-move-in-out = Mover entrada/saída de { $count ->
    [one] { $count } arquivo
   *[other] { $count } arquivos
}
batch-run-convert = Converter { $count ->
    [one] { $count } arquivo
   *[other] { $count } arquivos
}
batch-run-rotate = Girar { $count ->
    [one] { $count } vídeo
   *[other] { $count } vídeos
}
batch-run-tag = Aplicar tag a { $count ->
    [one] { $count } arquivo
   *[other] { $count } arquivos
}
batch-run-fix-tags = Reordenar tags em { $count ->
    [one] { $count } nome
   *[other] { $count } nomes
}
batch-run-rename = Renomear { $count ->
    [one] { $count } arquivo
   *[other] { $count } arquivos
}
batch-run-reload = Recarregar { $count ->
    [one] { $count } arquivo
   *[other] { $count } arquivos
}
batch-run-export-markers = Exportar marcadores de { $count ->
    [one] { $count } arquivo
   *[other] { $count } arquivos
}
batch-service-soniox = Soniox
batch-change-renames = Renomeia arquivos
batch-change-videos = Grava nos vídeos
batch-change-text-files = Grava arquivos de texto ao lado dos vídeos
batch-change-comments = Grava nos comentários
batch-change-subtitles = Grava arquivos de legenda ao lado dos vídeos
batch-change-records = Altera só os registros do próprio frename
batch-option-direction = Direção
batch-option-language = Idioma
batch-option-tag-suggestions = Sugestões de tags
batch-option-described = Vídeos descritos
batch-option-subtitled = Vídeos com legendas
batch-option-subtitles-write = Arquivos a gravar
batch-option-spacing = Espaçamento
batch-option-tag = Tag
batch-option-turn = Giro
batch-plan-videos = Vídeos
batch-plan-length = Duração
batch-plan-cost = Custo
batch-plan-time = Tempo
batch-reason-estimate = Aguardando a estimativa
batch-reason-subtitles-no-format = Selecione pelo menos um arquivo a gravar
batch-set-key = Definir a chave…
batch-check-key = Verificar a chave…
batch-subtitles-languages-auto = Detectado em cada vídeo
batch-action-describe-ai-run-waiting = Descrever vídeos
batch-action-fix-tags-order = Ordem: a da lista de tags, tags desconhecidas primeiro.
batch-action-markers-comment-hint-short = Transforma linhas de comentário com um tempo em marcadores, ou copia os marcadores para o comentário.
batch-action-markers-to-comment-hint = Os marcadores permanecem; executar de novo não adiciona nada duas vezes.
batch-action-rotate-hint-short = Gira clipes MP4 e MOV pelo sinalizador de rotação, sem recodificar.
batch-action-tag-commented-off = A aplicação de tag a vídeos comentados está desativada
batch-action-tag-commented-choose = Escolher a tag…

batch-title = Ações em lote
batch-done-label-changed = alterados
batch-done-label-subtitled = com legendas
batch-ai-at-least = no mínimo
batch-cancel = Cancelar
batch-stopping = Interrompendo…
batch-stop-over-budget = Interrompido: o próximo clipe poderia levar a execução além do limite de { $cap } ({ $spent } gastos). Aumente o limite para fazer o resto.
batch-stopped = Interrompido após { $finished } de { $total ->
    [one] { $total } arquivo
   *[other] { $total } arquivos
}.
batch-close = Fechar
batch-failed-subtitles = Sem legendas:
batch-written-subtitles = Arquivos gravados:
batch-table-written = Gravado
batch-merged-markers = Marcadores duplicados mesclados:
batch-table-merged = Mesclado
batch-markers-merged = { $count ->
    [one] { $count } marcador duplicado mesclado
   *[other] { $count } marcadores duplicados mesclados
}
batch-add-credit = Adicionar crédito
batch-open-log = Abrir o registro

batch-action-move-comments = Mover comentários
batch-action-move-comments-hint = Move o comentário de cada arquivo selecionado para o local escolhido, com os pontos de entrada/saída que ele contém. As tags e os pontos de entrada/saída guardados no vídeo permanecem onde estão.
batch-action-move-comments-into-videos = De arquivos de texto para os vídeos (XMP)
batch-action-move-comments-into-text-files = Dos vídeos (XMP) para arquivos de texto

batch-action-move-in-out = Pontos de entrada/saída: comentário ⇄ vídeo (XMP)
batch-action-move-in-out-hint = Move os pontos de entrada/saída de cada arquivo selecionado para o local escolhido. O resto do comentário permanece onde está.
batch-action-move-in-out-into-videos = Dos comentários para os vídeos (marcador XMP da Adobe)
batch-action-move-in-out-into-comments = Dos vídeos (marcador XMP) para os comentários

batch-action-markers-comment = Marcadores ⇄ comentário
batch-action-markers-comment-to-markers = Linhas de comentário com um tempo para marcadores
batch-action-markers-to-comment = Marcadores para o comentário (uma cópia: os marcadores permanecem)
batch-action-markers-comment-hint = Uma linha como "03:24 — Take 3 — luz bonita" é um marcador em 3:24 chamado "Take 3" com o comentário "luz bonita"; "0:41-0:47 — Lion" é um marcador de 0:41 a 0:47. O nome e o comentário são separados no primeiro " — " ou " -- ", e não em um " - " simples. Os momentos de uma descrição com IA ("0:00–0:14 Rua.") viram marcadores brancos e permanecem na descrição. Executar em qualquer direção de novo não adiciona nada duas vezes.

batch-action-tag-commented = Aplicar tag a vídeos comentados
batch-action-tag-commented-hint = Adiciona a tag "{ $tag }" a cada vídeo selecionado que tenha um comentário seu (as descrições com IA não contam) e a remove dos que não têm. Os arquivos cuja tag muda são renomeados.
batch-action-tag-commented-hint-off = Adiciona a tag de vídeos com comentário a cada vídeo selecionado que tenha um comentário seu e a remove dos que não têm. A tag está desativada nas configurações.
batch-action-fix-tags = Corrigir tags por prioridade
batch-action-fix-tags-hint = Coloca as tags no nome de cada arquivo selecionado na ordem do painel de tags: quanto mais alta a tag ali, mais cedo ela aparece no nome. As tags que a pasta ainda não conhece vêm primeiro, como no painel de tags. Os arquivos cuja ordem muda são renomeados.

batch-action-respace-tags = Aplicar espaçamento das tags
batch-action-respace-tags-hint-space = Renomeia cada arquivo selecionado para pôr um espaço depois de cada tag, conforme as configurações: Food. Goat. clip.mp4.
batch-action-respace-tags-hint-no-space = Renomeia cada arquivo selecionado para não ter espaço depois das tags, conforme as configurações: Food.Goat.clip.mp4.
batch-action-respace-tags-status-space = Espaçamento: um espaço depois de cada tag
batch-action-respace-tags-status-no-space = Espaçamento: sem espaço depois das tags
batch-action-export-markers = Exportar marcadores para texto
batch-action-export-markers-hint = Grava os marcadores de cada arquivo selecionado como um arquivo de texto ao lado dele, clip.markers.txt: uma linha por marcador em ordem de tempo, como "0:41–0:47 [red] — Lion crossing — walks left to right". Uma cor diferente de verde fica entre colchetes; o comentário vem depois do nome. "Marcadores ⇄ comentário" lê essas linhas de volta como marcadores. Um arquivo sem marcadores não recebe arquivo de texto, e um arquivo de texto de uma execução anterior é substituído.
batch-export-markers-cannot-hold = Este arquivo não pode conter marcadores
batch-action-reload-files = Limpar o cache e recarregar
batch-action-reload-files-hint = Lê de novo, do próprio arquivo, o comentário e os pontos de entrada/saída de cada arquivo selecionado e substitui o que a pasta tinha memorizado. Use depois que os arquivos forem alterados em outro programa. Os arquivos cujos valores memorizados faltavam ou estavam desatualizados contam como alterados.

batch-action-describe-ai = Descrever com IA
batch-action-describe-ai-run = Descrever { $videos } · cerca de { $dollars }
batch-action-describe-ai-estimating = Estimando… { $known } / { $total }
batch-action-describe-ai-none = Nenhum vídeo para descrever.
batch-action-describe-ai-hint = { batch-runs-in-background }. Cancelar mantém os vídeos já descritos; executar de novo os pula.
batch-action-describe-ai-no-subtitles = Sem legendas (só a imagem é descrita): { $videos }.
batch-action-describe-ai-redo = Refazer vídeos que já têm uma descrição com IA
batch-action-describe-ai-hint-panel = Descreve o que acontece em cada vídeo selecionado, e quando: um resumo e trechos com intervalos de tempo vão para a descrição com IA do comentário; o seu próprio texto é mantido. Os quadros e as legendas são enviados a { $provider }, a empresa do modelo escolhido.
batch-ai-change = Alterar
batch-ai-tag-suggestions-off = Desativadas
batch-ai-tag-suggestions-no-tags = Ativadas, mas esta pasta ainda não tem tags
batch-ai-tag-suggestions-on = { $tags ->
    [one] A partir da única tag da pasta
   *[other] A partir das { $tags } tags da pasta
}
batch-ai-key-missing = Defina uma chave de API de { $provider } nas Configurações
batch-ai-key-unavailable = Não foi possível abrir o chaveiro do sistema: ele pode estar bloqueado, ou não existir (como o GNOME Keyring ou o KWallet).
## AI description language names: the Settings picker and { $language } above.

ai-language-same-as-subtitles = Igual às legendas
ai-language-english = Inglês
ai-language-russian = Russo
ai-language-ukrainian = Ucraniano
ai-language-german = Alemão
ai-language-spanish = Espanhol
ai-language-french = Francês
batch-videos-count = { $n ->
    [one] { $n } vídeo
   *[other] { $n } vídeos
}
batch-minutes = { $n } min
batch-ai-minutes-under = < 1 min
batch-ai-duration-under-minute = menos de um minuto
batch-ai-duration-hours = { $h } h
batch-ai-duration-hours-minutes = { $h } h { $m } min
batch-ai-dollars-under = menos de US$ 0,01
batch-ai-skip-described = já descritos
batch-ai-skip-too-long = mais de 30 min
batch-ai-skip-unreadable = ilegíveis
batch-ai-skipped = Pulados: { $parts }.
batch-ai-progress-frame = quadro { $done } de { $total }
batch-ai-progress-waiting = aguardando { $provider }
batch-ai-progress-saving = salvando
batch-ai-progress-retry-rate-limit = nova tentativa em { $seconds } s (limite de requisições)
batch-ai-progress-retry-temporary = nova tentativa em { $seconds } s (falha temporária)
batch-ai-stop-no-key = Interrompido: sem chave de API de { $provider }. Defina uma nas Configurações.
batch-ai-fail-no-key = Sem chave de API
batch-ai-fail-unreadable = Não foi possível ler o vídeo
batch-ai-fail-too-long = O clipe é longo demais para a IA (mais de 30 min)
batch-ai-fail-not-saved = Não foi possível salvar a descrição
batch-ai-stop-offline = Interrompido: sem conexão com { $provider }. Execute de novo para descrever o restante.

batch-action-generate-subtitles = Gerar legendas
batch-action-generate-subtitles-install-ffmpeg = para ler .mkv, .m2ts, .avi …, instale o ffmpeg de ffmpeg.org, adicione-o ao PATH e reinicie o frename
batch-action-generate-subtitles-replace = Substituir legendas existentes
batch-action-generate-subtitles-replace-hint = Transcreve de novo; custa o valor indicado.
batch-action-generate-subtitles-srt = Legendas SRT (clip.srt)
batch-action-generate-subtitles-premiere = Transcrição do Premiere Pro (clip.premiere.json)
batch-action-generate-subtitles-premiere-hint = No Premiere Pro: painel Texto → Transcrição → Importar transcrição estática.
batch-action-generate-subtitles-privacy = O áudio destes vídeos é enviado à Soniox e apagado lá em seguida.
batch-action-generate-subtitles-duration-hint = Leva alguns minutos por hora de áudio. { batch-runs-in-background }. Fechar o frename interrompe a execução; as legendas concluídas são mantidas.
batch-action-generate-subtitles-hint = Transcreve a fala de cada vídeo selecionado com a Soniox e salva o resultado ao lado dele: legendas (clip.srt), onde o frename as mostra, e/ou uma transcrição do Premiere Pro.
batch-subtitles-transcribe = Transcrever
batch-subtitles-transcribe-count = Transcrever { $videos }
batch-subtitles-build-free = Criar { $count } (grátis)
batch-subtitles-nothing = Nada a transcrever
batch-subtitles-estimating = Estimando…
batch-subtitles-key-missing = Defina uma chave de API da Soniox nas Configurações
batch-subtitles-key-rejected = A Soniox rejeitou a chave
batch-subtitles-cost-unknown = custo desconhecido
batch-subtitles-typical-price = { $usd } (preço típico)
batch-subtitles-unknown-length = + { $n } de duração desconhecida
batch-subtitles-plan-line = { $videos } para transcrever, { $duration } de áudio{ $unknown } · { $cost }
batch-subtitles-already = { $n ->
    [one] { $n } já tem legendas
   *[other] { $n } já têm legendas
}
batch-subtitles-rebuilt-free = Recriados de graça a partir de uma transcrição salva: { $n }
batch-subtitles-no-speech-before = Sem fala da última vez: { $n }
batch-subtitles-no-audio = { $n ->
    [one] { $n } não tem áudio
   *[other] { $n } não têm áudio
}
batch-subtitles-shared-name = { $n ->
    [one] { $n } compartilha o nome da legenda com outro vídeo
   *[other] { $n } compartilham o nome da legenda com outro vídeo
}
batch-subtitles-unreadable = Ilegíveis: { $n } ({ $how_to })
batch-subtitles-shared-name-reason = compartilha o nome da legenda com outro vídeo
batch-subtitles-unsupported-reason = formato de áudio não suportado
batch-subtitles-progress-transcribing = transcrevendo…
batch-subtitles-progress-extracting = extraindo o áudio { $percent }%
batch-subtitles-progress-extracting-unknown = extraindo o áudio
batch-subtitles-progress-uploading = enviando { $percent }%
batch-subtitles-progress-uploading-unknown = enviando
batch-subtitles-progress-transcribing-left = transcrevendo… faltam cerca de { $time }
batch-subtitles-progress-saving = salvando
batch-subtitles-reason-already = já tinha legendas
batch-subtitles-reason-no-audio = sem áudio
batch-subtitles-reason-no-speech = sem fala
batch-subtitles-reason-soniox = Soniox: { $message }
batch-subtitles-reason-unreachable = não foi possível acessar a Soniox
batch-subtitles-stop-key-rejected = Interrompido: a Soniox rejeitou a chave. Verifique em Configurações → Legendas.
batch-subtitles-stop-balance-empty = Interrompido: o saldo da Soniox acabou. Recarregue em console.soniox.com.
batch-subtitles-stop-budget-used = Interrompido: o orçamento mensal da Soniox foi usado. Aumente-o em console.soniox.com.
batch-subtitles-stop-other = Interrompido: a Soniox se recusou a continuar ({ $error_type }).
batch-subtitles-stop-unreachable = Interrompido: não foi possível acessar a Soniox. Verifique a conexão com a internet.
batch-subtitles-report-spend = Soniox: no mínimo { $duration } · { $usd }
batch-subtitles-report-failed-deletes = { $n ->
    [one] { $n } envio não pôde ser apagado na Soniox (veja o registro)
   *[other] { $n } envios não puderam ser apagados na Soniox (veja o registro)
}
batch-subtitles-languages-need-key = Salve uma chave da Soniox para escolher entre todos os idiomas dela.
batch-subtitles-languages-rejected = A Soniox rejeitou a chave.
batch-subtitles-languages-failed = Não foi possível obter os idiomas da Soniox.
batch-subtitles-count = { $n ->
    [one] { $n } legenda
   *[other] { $n } legendas
}
batch-subtitles-duration-seconds = { $s } s
batch-subtitles-usd-under = menos de US$ 0,01
batch-subtitles-usd-about = cerca de US$ { $amount }

## File list
folder-has-subtitles = Tem legendas
folder-closed-by-job = Fechado enquanto uma ação em lote o altera
folder-filter-tip = Mostrar só…

folder-search-placeholder = Buscar um arquivo ou um comentário
folder-search-comments-loading = Buscando nos comentários… restantes: { $n }
folder-search-clear = Limpar
folder-all = Todos
folder-invert = Inverter
folder-checked = { $count ->
    [one] { $count } selecionado
   *[other] { $count } selecionados
}
folder-outcome-changed = Alterado
folder-outcome-unchanged = Nada a alterar
folder-outcome-failed = Falhou, veja o registro
folder-rename-error-empty = O nome está vazio
folder-rename-error-bad-character = Não permitido: \ / : * ? " < > |
folder-rename-error-trailing = Não pode terminar com ponto ou espaço
folder-rename-error-exists = Já existe um arquivo com este nome
folder-markers-not-saved = Marcadores não salvos: o arquivo é somente leitura ou está em uso (feche-o no Premiere, depois abra o arquivo e saia dele de novo)
drag-out-not-saved = Não arrastado: não foi possível salvar o arquivo (somente leitura, ou aberto no Premiere)
folder-opening = Abrindo a pasta…
folder-empty-title = Nenhum vídeo nesta pasta
folder-empty-line = O frename mostra arquivos MP4, MOV, MKV e outros arquivos de vídeo.
folder-open-another = Abrir outra pasta…
folder-no-match = Nenhum arquivo corresponde
folder-show-all = Mostrar todos
folder-locked = As seleções ficam bloqueadas enquanto { $action } é executada
folder-checked-hidden = { $count ->
    [one] { $count } selecionado
   *[other] { $count } selecionados
} · { $hidden ->
    [one] { $hidden } oculto
   *[other] { $hidden } ocultos
}
folder-checked-hidden-tip = { $hidden ->
    [one] { $hidden } arquivo selecionado está oculto pela busca ou por um filtro; a ação também é executada nele
   *[other] { $hidden } arquivos selecionados estão ocultos pela busca ou por um filtro; a ação também é executada neles
}
folder-outcome-working = Em andamento
folder-outcome-not-reached = Não alcançado
folder-window-empty-title = Abra uma pasta de clipes
folder-window-empty-line = Ou solte uma pasta na janela. Um clique com o botão direito no botão abre um único arquivo.
folder-window-open = Abrir uma pasta…

## Recent folders (the arrow next to the open button, and the empty screen)

recent-folders-tip = Pastas recentes
recent-folders-title = Pastas recentes
recent-folders-none = Nenhuma pasta aberta ainda
recent-folders-not-found = Não encontrada
recent-folders-remove-tip = Remover da lista
recent-folders-clear = Limpar lista
recent-folders-remove = Remover
recent-folders-keep = Manter
recent-folders-missing-line = Não encontrada. Remover?
recent-folders-just-now = Agora há pouco
recent-folders-minutes = há { $count } min
recent-folders-hours = há { $count } h
recent-folders-days = { $count ->
    [one] há { $count } dia
   *[other] há { $count } dias
}
recent-folders-weeks = { $count ->
    [one] há { $count } semana
   *[other] há { $count } semanas
}
recent-folders-months = { $count ->
    [one] há { $count } mês
   *[other] há { $count } meses
}
recent-folders-years = { $count ->
    [one] há { $count } ano
   *[other] há { $count } anos
}

## File menu (right-click on a file, or F11 / Shift+F11 / Ctrl+F11)

file-menu-show-in-explorer = Mostrar no Explorer
file-menu-show-in-file-manager = Mostrar no gerenciador de arquivos
file-menu-copy-path = Copiar o caminho completo
file-menu-copy-name = Copiar o nome do arquivo
file-menu-copied = Copiado
file-menu-not-copied = Não copiado: a área de transferência não está disponível
file-menu-not-shown = Não foi possível abrir o gerenciador de arquivos

## Controls bar under the file list

folder-controls-filter-untagged = Sem tags
folder-controls-filter-subtitles = Legendas
folder-controls-filter-comments = Comentários
folder-controls-filter-markers = Marcadores
folder-controls-scroll = Mostrar o arquivo aberto na lista
folder-controls-open = Abrir uma pasta
folder-controls-batch = Ações em lote nos arquivos selecionados
folder-controls-batch-back = Voltar ao arquivo aberto
folder-controls-update-available = Atualização disponível: { $version }
folder-controls-previous = Arquivo anterior
folder-controls-next = Próximo arquivo
folder-controls-more = Mais
folder-controls-open-file = Botão direito: abrir um arquivo

## Video

video-controls-back = Voltar 10 s
video-controls-play = Reproduzir
video-controls-pause = Pausar
video-controls-forward = Avançar 10 s
video-controls-frame-back = Um quadro para trás
video-controls-frame-forward = Um quadro para a frente
video-controls-set-in = Definir o ponto de entrada
video-controls-set-out = Definir o ponto de saída
video-controls-screenshot = Salvar este quadro
video-controls-add-marker = Adicionar um marcador
video-controls-add-marker-hold = Mantenha pressionado para um intervalo; pressione de novo para nomeá-lo
video-controls-cannot-hold-markers = Este arquivo não pode conter marcadores
video-controls-add-a-name = Adicionar um nome
video-controls-rotate-left = Girar para a esquerda
video-controls-rotate-right = Girar para a direita
video-controls-more = Mais
media-viewer-loading-slow = Aguardando o arquivo… (um arquivo na nuvem pode demorar)
media-viewer-cannot-play = Este clipe não pode ser reproduzido
media-viewer-no-picture = Este arquivo não tem imagem de vídeo
video-timecode-copied = Timecode copiado: { $time }
video-timecode-unreadable = “{ $text }” não é um tempo neste clipe
video-controls-volume-scroll = Volume — role para alterar
rotate-cannot = Não é possível girar: { $reason }
rotate-reason-missing = o arquivo não está mais lá
rotate-flag-right = 90° à direita
rotate-flag-left = 90° à esquerda
rotate-flag-half = 180°
rotate-flag-none = nenhuma
rotate-now = Rotação: { $flag }
rotate-turned = Girado { $turn } · rotação agora { $flag }
rotate-failed = Não girado: { $reason }
undo-name-taken = Não alterado: outro arquivo já se chama { $name }
undo-back-on-clip = De volta a este clipe (a movimentação foi desfeita). Ctrl+Z de novo desfaz a última alteração dele.
redo-on-clip = Neste clipe de novo (a movimentação foi refeita).
rotate-reason-in-use = o arquivo é somente leitura ou está em uso
rotate-reason-format = este formato não tem sinalizador de rotação
rotate-reason-damaged = o arquivo está danificado
rotate-reason-no-video = o arquivo não tem faixa de vídeo
rotate-reason-matrix = o vídeo tem uma matriz de exibição incomum
batch-action-rotate = Girar vídeos
batch-action-rotate-hint = Altera o sinalizador de rotação de cada arquivo MP4 e MOV selecionado; a imagem não é recodificada, e os comentários, os pontos de entrada/saída e os marcadores permanecem. O Premiere Pro deve mostrar o clipe girado ao importá-lo. Outros formatos não têm sinalizador de rotação e falham. Executar de novo gira os arquivos de novo; “Redefinir” remove qualquer giro, inclusive um gravado pelo celular, então o clipe em retrato de um celular passa a ser reproduzido deitado.
batch-action-rotate-right = 90° à direita (sentido horário)
batch-action-rotate-left = 90° à esquerda (sentido anti-horário)
batch-action-rotate-half = 180°
batch-action-rotate-reset = Redefinir: sem rotação (0°)
media-viewer-video-subtitle-list = Lista de legendas
media-viewer-video-marker-list = Lista de marcadores
media-viewer-video-markers-hint = Shift+F1 / Shift+F3 saltam entre marcadores; Shift+arrastar encaixa
media-viewer-video-tab-markers = Marcadores
media-viewer-video-fullscreen = Tela cheia
media-viewer-video-close-list = Fechar a lista
media-viewer-video-subtitle-file-unreadable = Não foi possível ler este arquivo
# Shown over the picture when a clip opens where playback stopped last time; Home or a click goes to the start.
media-viewer-video-resumed = Retomado em { $time } · Home: recomeçar

## Markers list
markers-in-out = Pontos de entrada/saída

markers-empty = Ainda não há marcadores
markers-add = Adicionar um marcador
markers-ai-hint = Marcador da IA: substituído quando a IA descrever este clipe de novo
markers-keep-color = Manter a cor
markers-done = Concluir
markers-delete = Excluir o marcador
markers-cannot-hold-hint = O Premiere lê marcadores de arquivos MP4 e MOV.
markers-color-green = Verde
markers-color-red = Vermelho
markers-color-orange = Laranja
markers-color-yellow = Amarelo
markers-color-white = Branco
markers-color-blue = Azul
markers-color-cyan = Ciano
markers-color-lavender = Lavanda
markers-color-magenta = Magenta
markers-color-other = Outra cor
markers-read-only = somente leitura
markers-name-placeholder = Nome
markers-ai-describe = Descrever com IA: nomear o marcador e acrescentar o que acontece
markers-ai-stop = Parar de descrever
markers-ai-describe-unnamed = Descrever { $count } sem nome
markers-ai-describe-unnamed-hint = Nomeia e descreve com IA todos os marcadores sem nome, alguns por vez. Uma solicitação cada; um nome digitado nesse meio-tempo é mantido.
markers-ai-price-about = cerca de { $dollars }
markers-ai-describe-unnamed-cost = Uma solicitação cada: { $each } por marcador, { $total } no total.
markers-ai-confirm = { $count ->
    [one] Descrever { $count } marcador? Uma solicitação: { $total }.
   *[other] Descrever { $count } marcadores? Uma solicitação cada: { $total } no total.
}
markers-ai-confirm-unpriced = { $count ->
    [one] Descrever { $count } marcador? Uma solicitação paga.
   *[other] Descrever { $count } marcadores? Uma solicitação paga cada.
}
markers-ai-confirm-yes = Descrever
markers-ai-confirm-no = Cancelar
markers-ai-waiting = Aguardando a vez…
markers-ai-run = Descrevendo { $describing } · { $waiting } na espera
markers-ai-run-last = Descrevendo { $describing }
markers-ai-stop-all = Parar tudo
markers-ai-describing = Descrevendo…
markers-ai-done = Marcador descrito
markers-ai-nothing-new = O marcador já tem esta descrição
markers-no-marker-notice = Nenhum marcador aqui
markers-ai-no-key = Sem chave de API de { $provider }: defina uma nas Configurações
markers-ai-failed = Não descrito: { $reason }
markers-ai-failed-unknown = algo deu errado
markers-read-only-notice = Esse marcador é somente leitura
markers-ai-gone = O marcador não existe mais: a descrição não foi adicionada
markers-ai-stopping-for-batch = Interrompendo antes as descrições de marcadores… Isso costuma levar alguns segundos.
folder-not-saved-marker-ai = Não salvo: a descrição do marcador ainda estava lendo o clipe. Tente de novo daqui a pouco.

## File workspace

file-workspace-search-placeholder = Buscar uma tag — ou só digitar
file-workspace-search-clear = Limpar
file-workspace-comment-placeholder = Comentário...
file-workspace-comment-collapse = Voltar às tags
file-workspace-comment-expand = Expandir o comentário

## Tag grid

tag-grid-star = Favoritar: manter no topo
tag-grid-unstar = Desfavoritar
tag-grid-save = Adicionar às tags da pasta
tag-grid-delete = Excluir “{ $tag }” das tags da pasta
tag-grid-create = Criar “{ $tag }”
tag-grid-no-file = Abra um clipe para aplicar tags
tag-grid-no-tags = Ainda não há tags
tag-grid-no-tags-hint = Digite um nome e pressione Enter para criar a primeira.
tag-grid-group-unsaved = Fora das tags da pasta
tag-grid-group-folder = Tags da pasta
tag-grid-more = +{ $count } mais

## Order strip

sync-panel-locked = Reordenar abaixo reordena a pasta
sync-panel-unlocked = Reordenar abaixo altera só este clipe
sync-panel-unlock = Desbloquear
sync-panel-lock = Bloquear
sync-panel-differs = A ordem difere da pasta
sync-panel-use-for-folder = Usar para a pasta
sync-panel-sort-like-folder = Ordenar como a pasta

## File name card

file-name-panel-no-tags = Nenhuma tag neste clipe
file-name-panel-untag = Remover tag
file-name-panel-clear-in = Limpar o ponto de entrada
file-name-panel-clear-out = Limpar o ponto de saída
file-name-panel-suggested-in-out = IA
file-name-panel-apply-suggested-in-out = Definir In e Out como a IA sugere (arredondados para segundos inteiros)
file-name-panel-suggested-tags = Tags da IA
file-name-panel-add-suggested-tag = Adicionar a tag "{ $tag }" ({ $percent }% de certeza)
file-name-panel-add-suggested-tag-unsure = Adicionar a tag "{ $tag }"
file-name-panel-add-all-suggested-tags = Adicionar todas
file-name-panel-add-all-suggested-tags-tip = Adicionar todas as tags que a IA sugere
file-name-panel-tag-ideas = Ideias: { $ideas }
file-name-panel-tag-ideas-tip = Tags que a IA percebeu e que esta pasta não tem. Elas não são adicionadas.
file-name-panel-saved = Todas as alterações salvas
file-name-panel-saved-to-recovery = Salvo na recuperação, aplicado ao sair do clipe

## Updates (Settings)

updates-check = Verificar atualizações
updates-not-installed = As atualizações funcionam na versão instalada
updates-from-store = As atualizações vêm da Microsoft Store
updates-from-releases = Novas versões: baixe-as na página de lançamentos
updates-open-releases = Abrir a página de lançamentos
updates-checking = Verificando…
updates-downloading-named = Baixando { $version }… { $percent }%
updates-downloading = Baixando… { $percent }%
updates-restarting = Reiniciando…
updates-check-failed = Não foi possível verificar as atualizações: { $reason }
updates-update-failed = Não foi possível atualizar: { $reason }
updates-close-other-window = Feche antes a outra janela do frename
updates-reason-github = o GitHub não respondeu, tente mais tarde
updates-reason-no-connection = sem conexão
updates-reason-read-only = a pasta é somente leitura
updates-reason-not-installed = as atualizações funcionam na versão instalada
updates-reason-download-stopped = o download parou inesperadamente
updates-version-available = A versão { $version } está disponível
updates-up-to-date = O frename está atualizado
updates-update-and-restart = Atualizar e reiniciar
updates-wait-for-batch = Aguarde o fim do lote
updates-current-version = frename { $version }
updates-check-on-start = Verificar atualizações quando o frename iniciar

recovery-restored = O frename foi fechado inesperadamente. O trabalho não salvo em { $clip } ({ $what }) foi restaurado.
recovery-kept = O frename foi fechado inesperadamente. O trabalho não salvo em { $clip } ({ $what }) não foi aplicado: { $why }. Uma cópia legível para redigitar está em { $folder }.
recovery-unreadable = O frename foi fechado inesperadamente, e um arquivo de recuperação não pôde ser lido. Ele foi mantido em { $folder }.
recovery-why-gone = o clipe não existe mais ou foi renomeado
recovery-why-changed = o clipe mudou depois das suas edições
recovery-why-not-written = já existe um arquivo com esse nome, ou o clipe é somente leitura ou está em uso
recovery-tags = { $count ->
    [one] { $count } tag
   *[other] { $count } tags
}
recovery-markers = { $count ->
    [one] { $count } marcador
   *[other] { $count } marcadores
}
recovery-comment = comentário
recovery-in-out = pontos de entrada/saída
recovery-edits = edições
folder-frame-saved = Quadro salvo
folder-not-saved = Não salvo: já existe um arquivo com esse nome, ou ele é somente leitura ou está em uso
markers-deleted-notice = Marcador excluído
markers-not-saved-notice = Marcadores não salvos: o arquivo é somente leitura ou está em uso
markers-comment-not-saved-notice = Marcadores não salvos: o arquivo de comentário é somente leitura ou está em uso
markers-copy = Copiar
markers-copy-tip = Copiar os marcadores como texto, uma linha cada
markers-export = Exportar…
markers-export-tip = Salvar os marcadores como um arquivo de texto
markers-exported = Marcadores salvos em { $name }
markers-not-exported = Marcadores não salvos: não foi possível gravar o arquivo
