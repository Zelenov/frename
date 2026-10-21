# frename UI text in Spanish. Same ids, attributes and $arguments as i18n/en/frename.ftl, in the
# same order (checked by the tests in src/i18n.rs).
#
# Glossary (docs/design/localization.md): every string uses these terms; the reader is addressed
# impersonally (infinitives or noun phrases for actions and labels, «se» constructions in
# sentences), never with «tú», and with «usted» only where a sentence cannot do without a subject,
# as in the Spanish Premiere Pro. Spelling and terms follow Spain (vídeo, archivo, coste), as
# Premiere Pro in Spanish does.
# Typography: no space before «:», «;», «!», «?» or «%»; opening «¿» and «¡»; quotes are «…».
# Keys: Ctrl stays Ctrl, Shift is Mayús (as the French file uses Maj), Home is Inicio, Enter Intro.
#   tag / tags                        tag / tags, masculine (never «etiqueta»: that is Premiere's Label)
#   in / out points                   puntos de entrada/salida
#   comment                           comentario
#   subtitles                         subtítulos
#   screenshot / frame                fotograma
#   marker (Premiere)                 marcador
#   range (a marker with a duration)  rango
#   subclip (Premiere)                subclip (as in Spanish Premiere Pro)
#   Description column (Premiere)     columna «Descripción»
#   checked (files)                   seleccionado (not «marcado»: too close to «marcador»)
#   All / Invert (check boxes)        Todos / Invertir
#   video / clip                      vídeo / clip (both masculine)
#   tag panel / tag list              panel de tags / lista de tags (the folder's list of tags)
#   file name                         nombre de archivo
#   a tag name inside a sentence      in «…» quotes: «Commented»
#   untagged (filter)                 Sin tags
#   changed / unchanged / failed      modificado / sin cambios / error
#   Cancel / Stopping…                Cancelar / Deteniendo…
#   log, "see the log"                registro, «ver el registro»
#   System (language list)            Sistema (…)
#   batch action / batch mode         acción por lotes / modo por lotes
#   request (to the AI)               solicitud
#   moments (AI)                      momentos
#   file list / folder                lista de archivos / carpeta (feminine)
#   settings                          Configuración
#   update (a newer frename)          actualización
#   AI description (Describe with AI) descripción con IA (Describir con IA)
#   subtitles generation (Soniox)     transcripción / subtítulos
#   cue (a subtitle line)             subtítulo
#   keyring                           llavero
#   frame rate / drop-frame           velocidad de fotogramas / con eliminación de fotogramas
#
# Plurals: Spanish has the categories one / other here, other as the default. «one» is only 1
# (0 archivos, 1 archivo, 2 archivos). Participles and adjectives agree with their noun in each
# variant (archivo seleccionado / archivos seleccionados, carpeta abierta); where a figure stands
# alone, invariable phrases (sin cambios, sin procesar) avoid guessing the gender.

## Language names: the same in every file, each written in its own language.

language-name-en = English
language-name-ru = Русский
language-name-de = Deutsch
language-name-fr = Français
language-name-es = Español

## Settings window

settings-window-title = Configuración
settings-page-interface = Interfaz
settings-page-saving = Guardado
settings-page-ai = Descripción con IA
settings-close = Cerrar
settings-apply-note = Los cambios se aplican al instante. Tab pasa de una opción a otra; Ctrl+Tab, de una página a otra.
settings-language = Idioma
settings-language-system = Sistema ({ $language })
settings-video = Vídeo
settings-video-autoplay = Reproducir los vídeos automáticamente al abrirlos
settings-subtitle-size = Tamaño de los subtítulos
settings-subtitle-size-small = Pequeño
settings-subtitle-size-medium = Mediano
settings-subtitle-size-large = Grande
settings-subtitle-size-extra-large = Muy grande
settings-timecode = Formato de tiempo
settings-timecode-hint = Cómo se escriben los tiempos bajo el vídeo, en las listas de marcadores y de subtítulos y en los indicadores In / Out, y el tiempo del cabezal de reproducción que copia Ctrl+Mayús+C. Premiere Pro y DaVinci Resolve muestran horas:minutos:segundos:fotogramas a la velocidad de fotogramas del clip; 29,97 y 59,94 fps van con eliminación de fotogramas (drop-frame). Un clip sin una velocidad de fotogramas constante mantiene el formato predeterminado. Los archivos conservan sus propios formatos.
settings-timecode-default = Predeterminado (mm:ss)
settings-timecode-premiere = Premiere Pro (HH:MM:SS:FF)
settings-timecode-resolve = DaVinci Resolve (HH:MM:SS:FF)
settings-tags = Colores de los tags
settings-tags-monochrome = Monocromo
settings-tags-monochrome-hint = Los tags añadidos se muestran en un gris más oscuro; los demás, en uno más claro, sin otros colores.
settings-file-names = Nombres de archivo
settings-tags-space-after = Espacio después de cada tag
settings-tags-space-example = Food. Goat. clip.mp4
settings-tags-space-note = Los archivos conservan su nombre hasta que se renombran o se guardan.
settings-tags-space-add = Añadir el espacio a los nombres de archivo…
settings-tags-space-remove = Quitar el espacio de los nombres de archivo…
settings-comments = Comentarios
settings-comments-in-video = Dentro del archivo de vídeo
settings-comments-in-video-hint = XMP, la columna «Descripción» de Premiere Pro
settings-comments-text-file = En un archivo de texto junto al vídeo
settings-comments-text-file-example = clip.comment.txt
settings-comments-note = Los archivos conservan sus comentarios donde están hasta que se mueven.
settings-comments-move-into-videos = Mover los comentarios a los vídeos…
settings-comments-move-into-text-files = Mover los comentarios a archivos de texto…
settings-commented-tag = Poner un tag a los vídeos con comentario
settings-commented-tag-name = Tag
settings-commented-tag-hint = Se añade al escribir un comentario en un vídeo, p. ej. { $tag }.IMG_0424.MOV, y se quita al borrarlo (las descripciones con IA no cuentan). Por lo demás, se puede cambiar libremente.
settings-commented-tag-off = Los vídeos con comentario no reciben ningún tag.
settings-in-out = Puntos de entrada/salida
settings-in-out-in-video = Dentro del archivo de vídeo
settings-in-out-in-video-hint = Un marcador de subclip en Premiere Pro
settings-in-out-comment = En el comentario, en una línea
settings-in-out-comment-example = In/Out: 00:01:05.250 – 00:02:10.000
settings-in-out-note = Los archivos conservan sus puntos de entrada/salida donde están hasta que se mueven.
settings-in-out-move-into-videos = Mover los puntos de entrada/salida a los vídeos…
settings-in-out-move-into-comments = Mover los puntos de entrada/salida a los comentarios…
settings-markers = Marcadores y rangos
settings-markers-in-video = Dentro del archivo de vídeo
settings-markers-in-video-hint = XMP, se muestran en el clip en Premiere Pro
settings-markers-comment = En el comentario, una línea por marcador
settings-markers-comment-example = 0:41–0:47 — Lion
settings-markers-note = Los archivos conservan sus marcadores donde están hasta que se mueven.
settings-markers-move-into-videos = Mover los marcadores a los vídeos…
settings-markers-copy-into-comment = Copiar los marcadores a los comentarios…
settings-markers-hint = Tanto puntos como rangos, también los momentos que encuentra la IA.
settings-updates = Actualizaciones
settings-version = Versión
settings-old-title = Configuración de un frename anterior
settings-old-hint = Traer la configuración de un frename ejecutado desde una carpeta zip.
settings-old-scheduled = La configuración se importará al reiniciar frename
settings-old-not-found = No hay ningún frename.exe con un frename.db en { $folder }
settings-old-failed = No se pudo importar: { $reason }
settings-old-import-button = Importar desde una carpeta antigua de frename…
settings-ai = Descripción con IA
settings-ai-key-label = Clave de API de Anthropic
settings-ai-key-placeholder = sk-ant-…
settings-ai-key-get = Obtener una clave en console.anthropic.com → API keys.
settings-ai-key-remove-confirm = ¿Quitar la clave de Anthropic guardada?
settings-ai-used-by = La usa «Describir con IA» en el modo por lotes y en los marcadores (✨, Ctrl+F2), cuando se elige un modelo Claude.
settings-ai-openai-key-label = Clave de API de OpenAI
settings-ai-openai-key-placeholder = sk-…
settings-ai-openai-key-get = Obtener una clave en platform.openai.com/api-keys.
settings-ai-openai-key-remove-confirm = ¿Quitar la clave de OpenAI guardada?
settings-ai-openai-used-by = La usa «Describir con IA» en el modo por lotes y en los marcadores (✨, Ctrl+F2), cuando se elige un modelo GPT.
settings-ai-gemini-key-label = Clave de API de Gemini
settings-ai-gemini-key-placeholder = AIza…
settings-ai-gemini-key-get = Obtener una clave en aistudio.google.com/apikey.
settings-ai-gemini-key-remove-confirm = ¿Quitar la clave de Gemini guardada?
settings-ai-gemini-used-by = La usa «Describir con IA» en el modo por lotes y en los marcadores (✨, Ctrl+F2), cuando se elige un modelo Gemini.
settings-ai-model-needs-key = { $model } — primero añadir la clave de { $provider }
settings-ai-model-label = Modelo
settings-ai-language-label = Idioma de la descripción
settings-ai-hint = Haiku, GPT-4.1 mini y Gemini Flash-Lite son los más baratos; Sonnet, Opus, GPT-4.1 y Gemini Flash captan más detalles. Cada modelo necesita la clave de su propia empresa.
settings-ai-moments-label = Momentos
settings-ai-moments-important = Solo lo que destaca
settings-ai-moments-full = Cubrir todo el clip
settings-ai-moments-important-hint = También sugiere dónde poner la entrada y la salida (la pastilla IA junto a IN y OUT)
settings-ai-moments-full-hint = Un momento para cada parte, también las tranquilas; sin sugerencia de entrada y salida
settings-ai-tag-suggestions = Sugerir también tags de entre los de la carpeta
settings-ai-tag-suggestions-hint = En la misma solicitud, la IA indica qué tags de la carpeta encajan con cada clip. Un tag sugerido solo se añade al hacer clic en él.
settings-subtitles = Subtítulos
settings-subtitles-key-label = Clave de API de Soniox
settings-subtitles-key-placeholder = Pegar la clave
settings-subtitles-key-get = Obtener una clave en console.soniox.com. El audio se envía a Soniox.
settings-subtitles-key-remove-confirm = ¿Quitar la clave de Soniox guardada?
settings-subtitles-languages-label = Idiomas
settings-subtitles-languages-locked = Guardar la clave para elegir entre todos los idiomas que conoce Soniox.
settings-subtitles-languages-loading = Obteniendo los idiomas de Soniox…
settings-subtitles-languages-none = Ninguno seleccionado: detección automática.
settings-subtitles-languages-hint = Los idiomas que se hablan en el material, como pistas.
settings-subtitles-cue-length-label = Longitud de los subtítulos
settings-subtitles-cue-short = Corta
settings-subtitles-cue-short-hint = Una línea, hasta 8 s
settings-subtitles-cue-sentence = Una frase
settings-subtitles-hint = Lo usa «Generar subtítulos» en el modo por lotes. La longitud se aplica a los subtítulos nuevos.
settings-key-store-windows = el Administrador de credenciales de Windows
settings-key-store-macos = el Llavero de macOS
settings-key-store-other = el llavero del sistema
settings-key-unavailable = No se pudo abrir el llavero del sistema
settings-key-unavailable-hint = Puede que esté bloqueado o que no exista (como GNOME Keyring o KWallet). La configuración lo vuelve a comprobar cada vez que se abre.
settings-key-remove-confirm-hint = Habrá que pegarla de nuevo.
settings-key-remove-ask = Quitar…
settings-key-remove = Quitar la clave
settings-key-keep = Conservar
settings-key-replace = Reemplazar…
settings-key-hide = Ocultar
settings-key-show = Mostrar
settings-key-save = Guardar la clave
settings-key-get = Obtener una clave
settings-key-billing = Añadir saldo
settings-key-cancel = Cancelar
settings-key-saved-in = Guardada en { $store } en este equipo
settings-key-save-into = «Guardar la clave» la conserva en { $store } en este equipo.

## Batch mode

batch-checked-count = { $count ->
    [one] { $count } archivo seleccionado
   *[other] { $count } archivos seleccionados
}
batch-no-key = Sin clave
batch-group-move = Mover entre ubicaciones
batch-group-fix = Corregir nombres y vídeos
batch-group-paid = Servicios de pago
batch-reason-none-checked = Ningún archivo seleccionado
batch-reason-reading = Leyendo la duración de los clips…
batch-check-all = Seleccionar { $count ->
    [one] { $count } archivo
   *[other] los { $count } archivos
}
batch-job-running = en curso
batch-job-finished = terminado
batch-job-stopped = detenido
batch-progress-files = { $finished } de { $total ->
    [one] { $total } archivo
   *[other] { $total } archivos
}
batch-time-left = quedan unos { $time }
batch-time-estimating = calculando el tiempo…
batch-time-spent = { $time } hasta ahora
batch-runs-in-background = Se puede seguir trabajando mientras se ejecuta
batch-clip-in-work = En uso: { $action }
batch-folder-held = Ocupada: { $action }
batch-indicator-hint = Hacer clic para ver el panel y Cancelar
batch-action-prepare = Preparar clips
batch-action-prepare-hint = Para cada clip seleccionado, en una sola pasada: los subtítulos (Soniox) si no los tiene y, después, la descripción con IA: un resumen, sus momentos como marcadores, el comentario y sugerencias de tags. Un solo precio, un solo límite y un solo progreso para todo.
batch-prepare-step-off = desactivado
batch-prepare-nothing = Nada que hacer con estos clips
batch-prepare-report-cap = Gastado { $spent } de un límite de { $cap }
batch-option-steps = Pasos
batch-prepare-step-subtitles = Crear primero los subtítulos (Soniox)
batch-prepare-step-description = Describir los clips con IA
batch-prepare-cap = Dejar de gastar en
batch-prepare-cap-none = Sin límite (dólares estadounidenses)
batch-prepare-cap-invalid = El límite es una cantidad de dólares, como 5
batch-prepare-left = La última ejecución se detuvo con { $count ->
    [one] { $count } clip pendiente
   *[other] { $count } clips pendientes
}
batch-prepare-check-left = Seleccionarlos
batch-prepare-estimating = Calculando qué hará cada paso…
batch-prepare-plan-subtitles = Subtítulos
batch-prepare-plan-description = Descripción
batch-prepare-plan-none = nada que hacer
batch-prepare-plan-subtitles-line = { $videos ->
    [one] { $videos } vídeo
   *[other] { $videos } vídeos
} que transcribir, { $duration } de audio
batch-prepare-plan-description-line = { $videos ->
    [one] { $videos } vídeo
   *[other] { $videos } vídeos
} que describir
batch-prepare-resume-note = Cada paso omite lo que ya existe, así que una ejecución detenida se reanuda volviendo a ejecutarla: nada se paga dos veces.
batch-prepare-run = Preparar { $clips ->
    [one] { $clips } clip
   *[other] { $clips } clips
} · unos { $dollars }
batch-service-prepare = Soniox + { $provider }
batch-prepare-groups-title = Tomas similares
batch-prepare-groups-hint = Clips que parecen la misma toma: duplicados, reexportaciones, una cámara que no se movió.
batch-prepare-group-clips = { $count } clips
batch-prepare-check-group = Seleccionar estos clips
batch-solo-queued = En cola: { $action }
batch-suggest-in-job = Una descripción de este clip ya está en el trabajo en curso, sin sugerencias de tags. Volver a pedirla cuando termine
batch-suggest-no-tags = Esta carpeta aún no tiene tags que sugerir: añadir algunos primero
batch-solo-gave-up = No se pudo preparar
batch-solo-withdrawn = Retirado: { $action }
batch-solo-in-work = { $action } está en curso. Se puede cancelar desde la barra o desde la página del trabajo
batch-waiting-title = { $count ->
    [one] { $count } ejecución en espera
   *[other] { $count } ejecuciones en espera
}
batch-waiting-remove = Quitar
batch-indicator-waiting = { $count } en espera
file-workspace-describe-clip = Describir este clip con IA
file-workspace-subtitles-clip = Generar subtítulos para este clip
file-workspace-describe-clip-queued = Describir con IA: esperando su turno. Hacer clic para retirarlo
file-workspace-describe-clip-running = Describir con IA: en curso
file-workspace-suggest-tags-clip = Sugerir tags para este clip con IA (Ctrl+F6)
file-workspace-suggest-tags-clip-queued = Sugerir tags: esperando su turno. Hacer clic para retirarlo
file-workspace-suggest-tags-clip-running = Sugerir tags: en curso
file-workspace-subtitles-clip-queued = Generar subtítulos: esperando su turno. Hacer clic para retirarlo
file-workspace-subtitles-clip-running = Generar subtítulos: en curso
batch-run-again = Volver a ejecutar en { $count ->
    [one] { $count } archivo
   *[other] { $count } archivos
}
batch-result-done = Hecho: { $total ->
    [one] { $total } archivo
   *[other] { $total } archivos
}
batch-result-done-detail = { $changed } con cambios, { $unchanged } sin nada que cambiar
batch-result-problems = Hecho con problemas: { $failed } de { $total ->
    [one] { $total } archivo
   *[other] { $total } archivos
} sin procesar
batch-figure-unchanged = sin cambios
batch-figure-not-done = sin procesar
batch-figure-not-reached = sin alcanzar
batch-files-not-done = Archivos sin procesar
batch-table-file = Archivo
batch-table-why = Motivo

batch-run-move-comments = Mover { $count ->
    [one] { $count } comentario
   *[other] { $count } comentarios
}
batch-run-move-in-out = Mover la entrada/salida de { $count ->
    [one] { $count } archivo
   *[other] { $count } archivos
}
batch-run-convert = Convertir { $count ->
    [one] { $count } archivo
   *[other] { $count } archivos
}
batch-run-rotate = Girar { $count ->
    [one] { $count } vídeo
   *[other] { $count } vídeos
}
batch-run-tag = Poner el tag a { $count ->
    [one] { $count } archivo
   *[other] { $count } archivos
}
batch-run-fix-tags = Reordenar los tags en { $count ->
    [one] { $count } nombre
   *[other] { $count } nombres
}
batch-run-rename = Renombrar { $count ->
    [one] { $count } archivo
   *[other] { $count } archivos
}
batch-run-reload = Recargar { $count ->
    [one] { $count } archivo
   *[other] { $count } archivos
}
batch-run-export-markers = Exportar los marcadores de { $count ->
    [one] { $count } archivo
   *[other] { $count } archivos
}
batch-service-soniox = Soniox
batch-change-renames = Renombra archivos
batch-change-videos = Escribe en los vídeos
batch-change-text-files = Escribe archivos de texto junto a los vídeos
batch-change-comments = Escribe en los comentarios
batch-change-subtitles = Escribe archivos de subtítulos junto a los vídeos
batch-change-records = Solo cambia los registros propios de frename
batch-option-direction = Dirección
batch-option-language = Idioma
batch-option-tag-suggestions = Sugerencias de tags
batch-option-described = Vídeos descritos
batch-option-subtitled = Vídeos subtitulados
batch-option-subtitles-write = Archivos que escribir
batch-option-spacing = Espaciado
batch-option-tag = Tag
batch-option-turn = Giro
batch-plan-videos = Vídeos
batch-plan-length = Duración
batch-plan-cost = Coste
batch-plan-time = Tiempo
batch-reason-estimate = Esperando la estimación
batch-reason-subtitles-no-format = Seleccionar al menos un archivo que escribir
batch-set-key = Configurar la clave…
batch-check-key = Comprobar la clave…
batch-subtitles-languages-auto = Detectado en cada vídeo
batch-action-describe-ai-run-waiting = Describir vídeos
batch-action-fix-tags-order = Orden: como en la lista de tags, primero los tags desconocidos.
batch-action-markers-comment-hint-short = Convierte las líneas del comentario con un tiempo en marcadores, o copia los marcadores en el comentario.
batch-action-markers-to-comment-hint = Los marcadores se quedan; volver a ejecutarlo no añade nada dos veces.
batch-action-rotate-hint-short = Gira los clips MP4 y MOV según su indicador de rotación, sin volver a codificarlos.
batch-action-tag-commented-off = El tag de los vídeos con comentario está desactivado
batch-action-tag-commented-choose = Elegir el tag…

batch-title = Acciones por lotes
batch-done-label-changed = modificados
batch-done-label-subtitled = subtitulados
batch-ai-at-least = al menos
batch-cancel = Cancelar
batch-stopping = Deteniendo…
batch-stop-over-budget = Detenido: el siguiente clip podría hacer que se superara el límite de { $cap } ({ $spent } gastados). Subir el límite para hacer el resto.
batch-stopped = Detenido tras { $finished } de { $total ->
    [one] { $total } archivo
   *[other] { $total } archivos
}.
batch-close = Cerrar
batch-failed-subtitles = Sin subtitular:
batch-written-subtitles = Archivos escritos:
batch-table-written = Escritos
batch-merged-markers = Marcadores duplicados fusionados:
batch-table-merged = Fusionados
batch-markers-merged = { $count ->
    [one] { $count } marcador duplicado fusionado
   *[other] { $count } marcadores duplicados fusionados
}
batch-add-credit = Añadir saldo
batch-open-log = Abrir el registro

batch-action-move-comments = Mover comentarios
batch-action-move-comments-hint = Mueve el comentario de cada archivo seleccionado al lugar elegido, con los puntos de entrada/salida que contiene. Los tags y los puntos de entrada/salida guardados en el vídeo se quedan donde están.
batch-action-move-comments-into-videos = De archivos de texto a los vídeos (XMP)
batch-action-move-comments-into-text-files = De los vídeos (XMP) a archivos de texto

batch-action-move-in-out = Puntos de entrada/salida: comentario ⇄ vídeo (XMP)
batch-action-move-in-out-hint = Mueve los puntos de entrada/salida de cada archivo seleccionado al lugar elegido. El resto del comentario se queda donde está.
batch-action-move-in-out-into-videos = De los comentarios a los vídeos (marcador XMP de Adobe)
batch-action-move-in-out-into-comments = De los vídeos (marcador XMP) a los comentarios

batch-action-markers-comment = Marcadores ⇄ comentario
batch-action-markers-comment-to-markers = Líneas del comentario con un tiempo a marcadores
batch-action-markers-to-comment = Marcadores al comentario (una copia: los marcadores se quedan)
batch-action-markers-comment-hint = Una línea como «03:24 — Take 3 — nice light» es un marcador en 3:24 llamado «Take 3» con el comentario «nice light»; «0:41-0:47 — Lion» es un marcador de 0:41 a 0:47. El nombre y el comentario se separan en el primer « — » o « -- », no en un simple « - ». Los momentos de una descripción con IA («0:00–0:14 Street.») se convierten en marcadores blancos y se quedan en la descripción. Volver a ejecutarlo en cualquiera de los dos sentidos no añade nada dos veces.

batch-action-tag-commented = Poner un tag a los vídeos con comentario
batch-action-tag-commented-hint = Añade el tag «{ $tag }» a cada vídeo seleccionado que tenga un comentario propio (las descripciones con IA no cuentan) y lo quita de los que no lo tienen. Los archivos cuyo tag cambia se renombran.
batch-action-tag-commented-hint-off = Añade el tag de los vídeos con comentario a cada vídeo seleccionado que tenga un comentario propio y lo quita de los que no lo tienen. El tag está desactivado en la configuración.
batch-action-fix-tags = Corregir los tags por prioridad
batch-action-fix-tags-hint = Coloca los tags del nombre de cada archivo seleccionado en el orden del panel de tags: cuanto más arriba esté un tag en él, antes aparece en el nombre. Los tags que la carpeta aún no conoce van primero, como en el panel de tags. Los archivos cuyo orden cambia se renombran.

batch-action-respace-tags = Aplicar el espaciado de los tags
batch-action-respace-tags-hint-space = Renombra cada archivo seleccionado para poner un espacio después de cada tag, como indica la configuración: Food. Goat. clip.mp4.
batch-action-respace-tags-hint-no-space = Renombra cada archivo seleccionado para que no quede ningún espacio después de sus tags, como indica la configuración: Food.Goat.clip.mp4.
batch-action-respace-tags-status-space = Espaciado: un espacio después de cada tag
batch-action-respace-tags-status-no-space = Espaciado: sin espacio después de los tags
batch-action-export-markers = Exportar los marcadores a texto
batch-action-export-markers-hint = Escribe los marcadores de cada archivo seleccionado en un archivo de texto junto a él, clip.markers.txt: una línea por marcador en orden temporal, como «0:41–0:47 [red] — Lion crossing — walks left to right». Un color distinto del verde va entre corchetes; el comentario sigue al nombre. «Marcadores ⇄ comentario» vuelve a leer esas líneas como marcadores. Un archivo sin marcadores no recibe ningún archivo de texto, y el archivo de texto de una ejecución anterior se reemplaza.
batch-export-markers-cannot-hold = Este archivo no puede contener marcadores
batch-action-reload-files = Restablecer la caché y recargar
batch-action-reload-files-hint = Vuelve a leer del propio archivo el comentario y los puntos de entrada/salida de cada archivo seleccionado y reemplaza lo que la carpeta recordaba de él. Para usar después de cambiar los archivos en otro programa. Los archivos cuyos valores recordados faltaban o estaban desactualizados cuentan como modificados.

batch-action-describe-ai = Describir con IA
batch-action-describe-ai-run = Describir { $videos } · unos { $dollars }
batch-action-describe-ai-estimating = Calculando… { $known } / { $total }
batch-action-describe-ai-none = No hay vídeos que describir.
batch-action-describe-ai-hint = { batch-runs-in-background }. Cancelar conserva los vídeos ya descritos; volver a ejecutarlo los omite.
batch-action-describe-ai-no-subtitles = Sin subtítulos (solo se describe la imagen): { $videos }.
batch-action-describe-ai-redo = Rehacer los vídeos que ya tienen una descripción con IA
batch-action-describe-ai-hint-panel = Describe qué ocurre en cada vídeo seleccionado, y cuándo: un resumen y segmentos con marcas de tiempo van a la descripción con IA de su comentario; el texto propio se conserva. Los fotogramas y los subtítulos se envían a { $provider }, la empresa del modelo elegido.
batch-ai-change = Cambiar
batch-ai-tag-suggestions-off = Desactivadas
batch-ai-tag-suggestions-no-tags = Activadas, pero esta carpeta aún no tiene tags
batch-ai-tag-suggestions-on = { $tags ->
    [one] Del único tag de la carpeta
   *[other] De los { $tags } tags de la carpeta
}
batch-ai-key-missing = Añadir una clave de API de { $provider } en Configuración
batch-ai-key-unavailable = No se pudo abrir el llavero del sistema: puede que esté bloqueado o que no exista (como GNOME Keyring o KWallet).
## AI description language names: the Settings picker and { $language } above.

ai-language-same-as-subtitles = Igual que los subtítulos
ai-language-english = Inglés
ai-language-russian = Ruso
ai-language-ukrainian = Ucraniano
ai-language-german = Alemán
ai-language-spanish = Español
ai-language-french = Francés
batch-videos-count = { $n ->
    [one] { $n } vídeo
   *[other] { $n } vídeos
}
batch-minutes = { $n } min
batch-ai-minutes-under = < 1 min
batch-ai-duration-under-minute = menos de un minuto
batch-ai-duration-hours = { $h } h
batch-ai-duration-hours-minutes = { $h } h { $m } min
batch-ai-dollars-under = menos de 0,01 $
batch-ai-skip-described = ya descritos
batch-ai-skip-too-long = más de 30 min
batch-ai-skip-unreadable = ilegibles
batch-ai-skipped = Omitidos: { $parts }.
batch-ai-progress-frame = fotograma { $done } de { $total }
batch-ai-progress-waiting = esperando a { $provider }
batch-ai-progress-saving = guardando
batch-ai-progress-retry-rate-limit = nuevo intento en { $seconds } s (límite de solicitudes)
batch-ai-progress-retry-temporary = nuevo intento en { $seconds } s (error temporal)
batch-ai-stop-no-key = Detenido: no hay clave de API de { $provider }. Añadir una en Configuración.
batch-ai-fail-no-key = Sin clave de API
batch-ai-fail-unreadable = No se pudo leer el vídeo
batch-ai-fail-too-long = El clip es demasiado largo para la IA (más de 30 min)
batch-ai-fail-not-saved = No se pudo guardar la descripción
batch-ai-stop-offline = Detenido: sin conexión con { $provider }. Volver a ejecutarlo para describir el resto.

batch-action-generate-subtitles = Generar subtítulos
batch-action-generate-subtitles-install-ffmpeg = para leer .mkv, .m2ts, .avi …, instalar ffmpeg desde ffmpeg.org, añadirlo al PATH y reiniciar frename
batch-action-generate-subtitles-replace = Reemplazar los subtítulos existentes
batch-action-generate-subtitles-replace-hint = Vuelve a transcribir; cuesta lo indicado.
batch-action-generate-subtitles-srt = Subtítulos SRT (clip.srt)
batch-action-generate-subtitles-premiere = Transcripción de Premiere Pro (clip.premiere.json)
batch-action-generate-subtitles-premiere-hint = En Premiere Pro: panel Texto → Transcripción → Importar transcripción estática.
batch-action-generate-subtitles-privacy = El audio de estos vídeos se envía a Soniox y después se elimina allí.
batch-action-generate-subtitles-duration-hint = Tarda unos minutos por cada hora de audio. { batch-runs-in-background }. Cerrar frename detiene la ejecución; los subtítulos terminados se conservan.
batch-action-generate-subtitles-hint = Transcribe el habla de cada vídeo seleccionado con Soniox y guarda el resultado junto a él: subtítulos (clip.srt), donde frename los muestra, y/o una transcripción de Premiere Pro.
batch-subtitles-transcribe = Transcribir
batch-subtitles-transcribe-count = Transcribir { $videos }
batch-subtitles-build-free = Crear { $count } (gratis)
batch-subtitles-nothing = Nada que transcribir
batch-subtitles-estimating = Calculando…
batch-subtitles-key-missing = Añadir una clave de API de Soniox en Configuración
batch-subtitles-key-rejected = Soniox rechazó la clave
batch-subtitles-cost-unknown = coste desconocido
batch-subtitles-typical-price = { $usd } (precio habitual)
batch-subtitles-unknown-length = + { $n } de duración desconocida
batch-subtitles-plan-line = { $videos } que transcribir, { $duration } de audio{ $unknown } · { $cost }
batch-subtitles-already = { $n ->
    [one] { $n } ya tiene subtítulos
   *[other] { $n } ya tienen subtítulos
}
batch-subtitles-rebuilt-free = { $n ->
    [one] { $n } recreado
   *[other] { $n } recreados
} gratis a partir de una transcripción guardada
batch-subtitles-no-speech-before = { $n } sin habla la última vez
batch-subtitles-no-audio = { $n ->
    [one] { $n } no tiene audio
   *[other] { $n } no tienen audio
}
batch-subtitles-shared-name = { $n ->
    [one] { $n } comparte el nombre de sus subtítulos con otro vídeo
   *[other] { $n } comparten el nombre de sus subtítulos con otro vídeo
}
batch-subtitles-unreadable = { $n ->
    [one] { $n } no se puede leer
   *[other] { $n } no se pueden leer
} ({ $how_to })
batch-subtitles-shared-name-reason = comparte el nombre de sus subtítulos con otro vídeo
batch-subtitles-unsupported-reason = formato de audio no compatible
batch-subtitles-progress-transcribing = transcribiendo…
batch-subtitles-progress-extracting = extrayendo el audio { $percent }%
batch-subtitles-progress-extracting-unknown = extrayendo el audio
batch-subtitles-progress-uploading = subiendo { $percent }%
batch-subtitles-progress-uploading-unknown = subiendo
batch-subtitles-progress-transcribing-left = transcribiendo… quedan unos { $time }
batch-subtitles-progress-saving = guardando
batch-subtitles-reason-already = ya tenía subtítulos
batch-subtitles-reason-no-audio = sin audio
batch-subtitles-reason-no-speech = sin habla
batch-subtitles-reason-soniox = Soniox: { $message }
batch-subtitles-reason-unreachable = no se puede conectar con Soniox
batch-subtitles-stop-key-rejected = Detenido: Soniox rechazó la clave. Comprobarla en Configuración → Subtítulos.
batch-subtitles-stop-balance-empty = Detenido: el saldo de Soniox está vacío. Recargarlo en console.soniox.com.
batch-subtitles-stop-budget-used = Detenido: se ha agotado el presupuesto mensual de Soniox. Aumentarlo en console.soniox.com.
batch-subtitles-stop-other = Detenido: Soniox se negó a continuar ({ $error_type }).
batch-subtitles-stop-unreachable = Detenido: no se puede conectar con Soniox. Comprobar la conexión a internet.
batch-subtitles-report-spend = Soniox: al menos { $duration } · { $usd }
batch-subtitles-report-failed-deletes = { $n ->
    [one] { $n } archivo subido no se pudo eliminar
   *[other] { $n } archivos subidos no se pudieron eliminar
} en Soniox (ver el registro)
batch-subtitles-languages-need-key = Guardar una clave de Soniox para elegir entre todos sus idiomas.
batch-subtitles-languages-rejected = Soniox rechazó la clave.
batch-subtitles-languages-failed = No se pudieron obtener los idiomas de Soniox.
batch-subtitles-count = { $n ->
    [one] { $n } subtítulo
   *[other] { $n } subtítulos
}
batch-subtitles-duration-seconds = { $s } s
batch-subtitles-usd-under = menos de 0,01 $
batch-subtitles-usd-about = unos { $amount } $

## File list
folder-has-subtitles = Tiene subtítulos
folder-closed-by-job = Cerrada mientras una acción por lotes la modifica
folder-filter-tip = Mostrar solo…

folder-search-placeholder = Buscar un archivo o un comentario
folder-search-comments-loading = Buscando en los comentarios… quedan { $n }
folder-search-clear = Borrar
folder-all = Todos
folder-invert = Invertir
folder-checked = { $count ->
    [one] { $count } seleccionado
   *[other] { $count } seleccionados
}
folder-outcome-changed = Modificado
folder-outcome-unchanged = Nada que cambiar
folder-outcome-failed = Error, ver el registro
folder-rename-error-empty = El nombre está vacío
folder-rename-error-bad-character = No permitido: \ / : * ? " < > |
folder-rename-error-trailing = No puede terminar en punto ni en espacio
folder-rename-error-exists = Ya existe un archivo con este nombre
folder-markers-not-saved = Marcadores no guardados: el archivo es de solo lectura o está en uso (cerrarlo en Premiere y, después, abrir el archivo y volver a salir de él)
drag-out-not-saved = No se arrastró: no se puede guardar el archivo (solo lectura, o abierto en Premiere)
folder-opening = Abriendo la carpeta…
folder-empty-title = No hay vídeos en esta carpeta
folder-empty-line = frename muestra archivos MP4, MOV, MKV y otros archivos de vídeo.
folder-open-another = Abrir otra carpeta…
folder-no-match = Ningún archivo coincide
folder-show-all = Mostrar todo
folder-locked = Las casillas están bloqueadas mientras se ejecuta { $action }
folder-checked-hidden = { $count ->
    [one] { $count } seleccionado
   *[other] { $count } seleccionados
} · { $hidden ->
    [one] { $hidden } oculto
   *[other] { $hidden } ocultos
}
folder-checked-hidden-tip = { $hidden ->
    [one] { $hidden } archivo seleccionado está oculto por la búsqueda o un filtro; la acción también se ejecuta en él
   *[other] { $hidden } archivos seleccionados están ocultos por la búsqueda o un filtro; la acción también se ejecuta en ellos
}
folder-outcome-working = En curso
folder-outcome-not-reached = Sin alcanzar
folder-window-empty-title = Abrir una carpeta de clips
folder-window-empty-line = O soltar una carpeta en la ventana. Un clic derecho en el botón abre un solo archivo.
folder-window-open = Abrir una carpeta…

## Recent folders (the arrow next to the open button, and the empty screen)

recent-folders-tip = Carpetas recientes
recent-folders-title = Carpetas recientes
recent-folders-none = Aún no se ha abierto ninguna carpeta
recent-folders-not-found = No encontrada
recent-folders-remove-tip = Quitar de la lista
recent-folders-clear = Vaciar la lista
recent-folders-remove = Quitar
recent-folders-keep = Conservar
recent-folders-missing-line = No encontrada. ¿Quitarla?
recent-folders-just-now = Ahora mismo
recent-folders-minutes = hace { $count } min
recent-folders-hours = hace { $count } h
recent-folders-days = { $count ->
    [one] hace { $count } día
   *[other] hace { $count } días
}
recent-folders-weeks = { $count ->
    [one] hace { $count } semana
   *[other] hace { $count } semanas
}
recent-folders-months = { $count ->
    [one] hace { $count } mes
   *[other] hace { $count } meses
}
recent-folders-years = { $count ->
    [one] hace { $count } año
   *[other] hace { $count } años
}

## File menu (right-click on a file, or F11 / Shift+F11 / Ctrl+F11)

file-menu-show-in-explorer = Mostrar en el Explorador
file-menu-show-in-file-manager = Mostrar en el gestor de archivos
file-menu-copy-path = Copiar la ruta completa
file-menu-copy-name = Copiar el nombre del archivo
file-menu-copied = Copiado
file-menu-not-copied = No copiado: el portapapeles no está disponible
file-menu-not-shown = No se pudo abrir el gestor de archivos

## Controls bar under the file list

folder-controls-filter-untagged = Sin tags
folder-controls-filter-subtitles = Subtítulos
folder-controls-filter-comments = Comentarios
folder-controls-filter-markers = Marcadores
folder-controls-scroll = Mostrar el archivo abierto en la lista
folder-controls-open = Abrir una carpeta
folder-controls-batch = Acciones por lotes en los archivos seleccionados
folder-controls-batch-back = Volver al archivo abierto
folder-controls-update-available = Actualización disponible: { $version }
folder-controls-previous = Archivo anterior
folder-controls-next = Archivo siguiente
folder-controls-more = Más
folder-controls-open-file = Clic derecho: abrir un solo archivo

## Video

video-controls-back = Retroceder 10 s
video-controls-play = Reproducir
video-controls-pause = Pausa
video-controls-forward = Avanzar 10 s
video-controls-frame-back = Un fotograma atrás
video-controls-frame-forward = Un fotograma adelante
video-controls-set-in = Establecer el punto de entrada
video-controls-set-out = Establecer el punto de salida
video-controls-screenshot = Guardar este fotograma
video-controls-add-marker = Añadir un marcador
video-controls-add-marker-hold = Mantener pulsado para un rango; volver a pulsar para ponerle nombre
video-controls-cannot-hold-markers = Este archivo no puede contener marcadores
video-controls-add-a-name = Añadir un nombre
video-controls-rotate-left = Girar a la izquierda
video-controls-rotate-right = Girar a la derecha
video-controls-more = Más
media-viewer-loading-slow = Esperando el archivo… (un archivo en la nube puede tardar un poco)
media-viewer-cannot-play = Este clip no se puede reproducir
media-viewer-no-picture = Este archivo no tiene imagen de vídeo
video-timecode-copied = Código de tiempo copiado: { $time }
video-timecode-unreadable = «{ $text }» no es un tiempo de este clip
video-controls-volume-scroll = Volumen — girar la rueda para cambiarlo
rotate-cannot = No se puede girar: { $reason }
rotate-reason-missing = el archivo ya no está
rotate-flag-right = 90° a la derecha
rotate-flag-left = 90° a la izquierda
rotate-flag-half = 180°
rotate-flag-none = ninguna
rotate-now = Rotación: { $flag }
rotate-turned = Girado { $turn } · rotación actual: { $flag }
rotate-failed = No girado: { $reason }
undo-name-taken = Sin cambios: otro archivo ya se llama { $name }
undo-back-on-clip = De vuelta en este clip (se deshizo el movimiento). Ctrl+Z de nuevo deshace su último cambio.
redo-on-clip = De nuevo en este clip (se rehízo el movimiento).
rotate-reason-in-use = el archivo es de solo lectura o está en uso
rotate-reason-format = este formato no tiene indicador de rotación
rotate-reason-damaged = el archivo está dañado
rotate-reason-no-video = el archivo no tiene pista de vídeo
rotate-reason-matrix = el vídeo tiene una matriz de visualización inusual
batch-action-rotate = Girar vídeos
batch-action-rotate-hint = Cambia el indicador de rotación de cada archivo MP4 y MOV seleccionado; la imagen no se vuelve a codificar, y los comentarios, los puntos de entrada/salida y los marcadores se mantienen. Se espera que Premiere Pro muestre el clip girado al importarlo. Los demás formatos no tienen indicador de rotación y fallan. Volver a ejecutarlo gira de nuevo los archivos; «Restablecer» elimina cualquier giro, incluido el que grabó un teléfono, así que el clip vertical de un teléfono se reproduce entonces de lado.
batch-action-rotate-right = 90° a la derecha (sentido horario)
batch-action-rotate-left = 90° a la izquierda (sentido antihorario)
batch-action-rotate-half = 180°
batch-action-rotate-reset = Restablecer: sin rotación (0°)
media-viewer-video-subtitle-list = Lista de subtítulos
media-viewer-video-marker-list = Lista de marcadores
media-viewer-video-markers-hint = Mayús+F1 / Mayús+F3 saltan entre marcadores; Mayús+arrastrar ajusta
media-viewer-video-tab-markers = Marcadores
media-viewer-video-fullscreen = Pantalla completa
media-viewer-video-close-list = Cerrar la lista
media-viewer-video-subtitle-file-unreadable = No se pudo leer este archivo
# Shown over the picture when a clip opens where playback stopped last time; Home or a click goes to the start.
media-viewer-video-resumed = Reanudado en { $time } · Inicio: volver a empezar

## Markers list
markers-in-out = Puntos de entrada/salida

markers-empty = Aún no hay marcadores
markers-add = Añadir un marcador
markers-ai-hint = Marcador de IA: se reemplaza cuando la IA vuelve a describir este clip
markers-keep-color = Mantener el color
markers-done = Hecho
markers-delete = Eliminar el marcador
markers-cannot-hold-hint = Premiere lee los marcadores de los archivos MP4 y MOV.
markers-color-green = Verde
markers-color-red = Rojo
markers-color-orange = Naranja
markers-color-yellow = Amarillo
markers-color-white = Blanco
markers-color-blue = Azul
markers-color-cyan = Cian
markers-color-lavender = Lavanda
markers-color-magenta = Magenta
markers-color-other = Otro color
markers-read-only = solo lectura
markers-name-placeholder = Nombre
markers-ai-describe = Describir con IA: poner nombre al marcador y añadir lo que ocurre
markers-ai-stop = Dejar de describir
markers-ai-describe-unnamed = Describir { $count } sin nombre
markers-ai-describe-unnamed-hint = Poner nombre y describir con IA cada marcador que no tenga nombre, unos pocos a la vez. Una solicitud por marcador; un nombre escrito mientras tanto se conserva.
markers-ai-price-about = unos { $dollars }
markers-ai-describe-unnamed-cost = Una solicitud por marcador: { $each } cada uno, { $total } en total.
markers-ai-confirm = { $count ->
    [one] ¿Describir { $count } marcador? Una solicitud: { $total }.
   *[other] ¿Describir { $count } marcadores? Una solicitud por marcador: { $total } en total.
}
markers-ai-confirm-unpriced = { $count ->
    [one] ¿Describir { $count } marcador? Una solicitud de pago.
   *[other] ¿Describir { $count } marcadores? Una solicitud de pago por marcador.
}
markers-ai-confirm-yes = Describir
markers-ai-confirm-no = Cancelar
markers-ai-waiting = Esperando su turno…
markers-ai-run = Describiendo { $describing } · { $waiting } en espera
markers-ai-run-last = Describiendo { $describing }
markers-ai-stop-all = Detener todo
markers-ai-describing = Describiendo…
markers-ai-done = Marcador descrito
markers-ai-nothing-new = El marcador ya tiene esta descripción
markers-no-marker-notice = No hay ningún marcador aquí
markers-ai-no-key = No hay clave de API de { $provider }: añadir una en Configuración
markers-ai-failed = No descrito: { $reason }
markers-ai-failed-unknown = algo salió mal
markers-read-only-notice = Ese marcador es de solo lectura
markers-ai-gone = El marcador ya no existe: su descripción no se añadió
markers-ai-stopping-for-batch = Deteniendo primero las descripciones de marcadores… Suele tardar unos segundos.
folder-not-saved-marker-ai = No guardado: la descripción del marcador aún estaba leyendo el clip. Volver a intentarlo en un momento.

## File workspace

file-workspace-search-placeholder = Buscar un tag — o simplemente escribir
file-workspace-search-clear = Borrar
file-workspace-comment-placeholder = Comentario...
file-workspace-comment-collapse = Volver a los tags
file-workspace-comment-expand = Ampliar el comentario

## Tag grid

tag-grid-star = Destacar: mantenerlo arriba
tag-grid-unstar = Quitar de destacados
tag-grid-save = Añadir a los tags de la carpeta
tag-grid-delete = Eliminar «{ $tag }» de los tags de la carpeta
tag-grid-create = Crear «{ $tag }»
tag-grid-no-file = Abrir un clip para ponerle tags
tag-grid-no-tags = Aún no hay tags
tag-grid-no-tags-hint = Escribir un nombre y pulsar Intro para crear el primero.
tag-grid-group-unsaved = Fuera de los tags de la carpeta
tag-grid-group-folder = Tags de la carpeta
tag-grid-more = +{ $count } más

## Order strip

sync-panel-locked = Reordenar abajo reordena la carpeta
sync-panel-unlocked = Reordenar abajo solo cambia este clip
sync-panel-unlock = Desbloquear
sync-panel-lock = Bloquear
sync-panel-differs = El orden difiere del de la carpeta
sync-panel-use-for-folder = Usar para la carpeta
sync-panel-sort-like-folder = Ordenar como la carpeta

## File name card

file-name-panel-no-tags = Este clip no tiene tags
file-name-panel-untag = Quitar el tag
file-name-panel-clear-in = Borrar el punto de entrada
file-name-panel-clear-out = Borrar el punto de salida
file-name-panel-suggested-in-out = IA
file-name-panel-apply-suggested-in-out = Poner la entrada y la salida que sugiere la IA (redondeadas a segundos enteros)
file-name-panel-suggested-tags = Tags de IA
file-name-panel-add-suggested-tag = Añadir el tag «{ $tag }» (seguridad del { $percent }%)
file-name-panel-add-suggested-tag-unsure = Añadir el tag «{ $tag }»
file-name-panel-add-all-suggested-tags = Añadir todos
file-name-panel-add-all-suggested-tags-tip = Añadir todos los tags que sugiere la IA
file-name-panel-tag-ideas = Ideas: { $ideas }
file-name-panel-tag-ideas-tip = Tags que la IA detectó y que esta carpeta no tiene. No se añaden.
file-name-panel-saved = Todos los cambios guardados
file-name-panel-saved-to-recovery = Guardado en la recuperación; se aplica al salir del clip

## Updates (Settings)

updates-check = Buscar actualizaciones
updates-not-installed = Las actualizaciones funcionan en la versión instalada
updates-from-store = Las actualizaciones llegan desde Microsoft Store
updates-from-releases = Versiones nuevas: descargarlas desde la página de versiones
updates-open-releases = Abrir la página de versiones
updates-checking = Comprobando…
updates-downloading-named = Descargando { $version }… { $percent }%
updates-downloading = Descargando… { $percent }%
updates-restarting = Reiniciando…
updates-check-failed = No se pudieron buscar actualizaciones: { $reason }
updates-update-failed = No se pudo actualizar: { $reason }
updates-close-other-window = Cerrar primero la otra ventana de frename
updates-reason-github = GitHub no respondió, volver a intentarlo más tarde
updates-reason-no-connection = sin conexión
updates-reason-read-only = la carpeta es de solo lectura
updates-reason-not-installed = las actualizaciones funcionan en la versión instalada
updates-reason-download-stopped = la descarga se detuvo inesperadamente
updates-version-available = La versión { $version } está disponible
updates-up-to-date = frename está actualizado
updates-update-and-restart = Actualizar y reiniciar
updates-wait-for-batch = Esperar a que termine la acción por lotes
updates-current-version = frename { $version }
updates-check-on-start = Buscar actualizaciones al iniciar frename

recovery-restored = frename se cerró inesperadamente. Se restauró el trabajo sin guardar en { $clip } ({ $what }).
recovery-kept = frename se cerró inesperadamente. El trabajo sin guardar en { $clip } ({ $what }) no se aplicó: { $why }. Hay una copia legible para volver a escribirlo en { $folder }.
recovery-unreadable = frename se cerró inesperadamente y no se pudo leer un archivo de recuperación. Se conserva en { $folder }.
recovery-why-gone = el clip ya no está o se renombró
recovery-why-changed = el clip cambió después de las ediciones
recovery-why-not-written = ya existe un archivo con ese nombre, o el clip es de solo lectura o está en uso
recovery-tags = { $count ->
    [one] { $count } tag
   *[other] { $count } tags
}
recovery-markers = { $count ->
    [one] { $count } marcador
   *[other] { $count } marcadores
}
recovery-comment = comentario
recovery-in-out = puntos de entrada/salida
recovery-edits = ediciones
folder-frame-saved = Fotograma guardado
folder-not-saved = No guardado: ya existe un archivo con ese nombre, o es de solo lectura o está en uso
markers-deleted-notice = Marcador eliminado
markers-not-saved-notice = Marcadores no guardados: el archivo es de solo lectura o está en uso
markers-comment-not-saved-notice = Marcadores no guardados: el archivo de comentario es de solo lectura o está en uso
markers-copy = Copiar
markers-copy-tip = Copiar los marcadores como texto, una línea por marcador
markers-export = Exportar…
markers-export-tip = Guardar los marcadores en un archivo de texto
markers-exported = Marcadores guardados en { $name }
markers-not-exported = Marcadores no guardados: no se pudo escribir el archivo
