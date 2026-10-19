# frename UI text in French. Same ids, attributes and $arguments as i18n/en/frename.ftl, in the
# same order (checked by the tests in src/i18n.rs).
#
# Glossary (docs/design/localization.md): every string uses these terms; the reader is addressed
# impersonally (infinitive or noun phrases for actions and labels), and with «vous» or «votre» only where a
# sentence needs a subject or a possessive, as in the French Premiere Pro and most French video software.
# Typography: a no-break space (U+00A0) before « : » and inside «guillemets», a narrow no-break
# space (U+202F) before «;», «!», «?» and «%».
#   tag / tags                        tag / tags (never «étiquette»: that is Premiere's Label)
#   in / out points                   points d’entrée/de sortie
#   comment                           commentaire
#   subtitles                         sous-titres
#   screenshot                        capture d’écran
#   marker (Premiere)                 marqueur
#   range (a marker with a duration)  plage
#   subclip (Premiere)                sous-clip (as in French Premiere Pro)
#   Description column (Premiere)     colonne «Description»
#   checked (files)                   coché
#   All / Invert (check boxes)        Tous / Inverser
#   video / clip                      vidéo / clip
#   tag panel / tag list              panneau des tags / liste des tags (the folder's list of tags)
#   file name                         nom de fichier
#   a tag name inside a sentence      in «…» quotes: «Commented»
#   untagged (filter)                 Sans tag
#   changed / unchanged / failed      modifié / inchangé / échec
#   Cancel / Stopping…                Annuler / Arrêt en cours…
#   log, "see the log"                journal, «voir le journal»
#   System (language list)            Système (…)
#   batch action / batch mode         action par lot / mode par lot
#   request (to the AI)               requête
#   moments (AI)                      moments
#   file list / folder                liste des fichiers / dossier
#   settings                          Paramètres
#   update (a newer frename)          mise à jour
#   AI description (Describe with AI) description par IA
#   subtitles generation (Soniox)     transcription / sous-titres
#   cue (a subtitle line)             sous-titre
#   keyring                           trousseau
#
# Plurals: French has the categories one / other, other as the default. «one» covers 0 and 1
# (0 fichier, 1 fichier, 2 fichiers), so a «one» variant always prints its number.

## Language names: the same in every file, each written in its own language.

language-name-en = English
language-name-ru = Русский
language-name-de = Deutsch
language-name-fr = Français

## Settings window

settings-window-title = Paramètres
settings-page-interface = Interface
settings-page-saving = Enregistrement
settings-page-ai = Description par IA
settings-close = Fermer
settings-apply-note = Les modifications s’appliquent immédiatement. La touche Tab passe d’une option à l’autre, Ctrl+Tab d’une page à l’autre.
settings-language = Langue
settings-language-system = Système ({ $language })
settings-video = Vidéo
settings-video-autoplay = Lire automatiquement les vidéos à l’ouverture
settings-subtitle-size = Taille des sous-titres
settings-subtitle-size-small = Petite
settings-subtitle-size-medium = Moyenne
settings-subtitle-size-large = Grande
settings-subtitle-size-extra-large = Très grande
settings-tags = Couleurs des tags
settings-tags-monochrome = Monochrome
settings-tags-monochrome-hint = Les tags ajoutés s’affichent en gris plus foncé, les autres en gris plus clair, sans autre couleur.
settings-file-names = Noms de fichier
settings-tags-space-after = Espace après chaque tag
settings-tags-space-example = Food. Goat. clip.mp4
settings-tags-space-note = Les fichiers gardent leur nom tant qu’ils ne sont pas renommés ou enregistrés.
settings-tags-space-add = Ajouter l’espace aux noms de fichier…
settings-tags-space-remove = Supprimer l’espace des noms de fichier…
settings-comments = Commentaires
settings-comments-in-video = Dans le fichier vidéo
settings-comments-in-video-hint = XMP, colonne « Description » dans Premiere Pro
settings-comments-text-file = Dans un fichier texte à côté de la vidéo
settings-comments-text-file-example = clip.comment.txt
settings-comments-note = Les fichiers gardent leurs commentaires là où ils sont tant qu’ils ne sont pas déplacés.
settings-comments-move-into-videos = Déplacer les commentaires dans les vidéos…
settings-comments-move-into-text-files = Déplacer les commentaires dans des fichiers texte…
settings-commented-tag = Tagger les vidéos commentées
settings-commented-tag-name = Tag
settings-commented-tag-hint = Coché quand un commentaire est écrit sur une vidéo, p. ex. { $tag }.IMG_0424.MOV, et décoché quand il est effacé (les descriptions par IA ne comptent pas). Sinon, il reste à modifier librement.
settings-commented-tag-off = Les vidéos commentées ne reçoivent aucun tag.
settings-in-out = Points d’entrée/de sortie
settings-in-out-in-video = Dans le fichier vidéo
settings-in-out-in-video-hint = Un marqueur de sous-clip dans Premiere Pro
settings-in-out-comment = Dans le commentaire, sur une ligne
settings-in-out-comment-example = In/Out: 00:01:05.250 – 00:02:10.000
settings-in-out-note = Les fichiers gardent leurs points d’entrée/de sortie là où ils sont tant qu’ils ne sont pas déplacés.
settings-in-out-move-into-videos = Déplacer les points d’entrée/de sortie dans les vidéos…
settings-in-out-move-into-comments = Déplacer les points d’entrée/de sortie dans les commentaires…
settings-markers = Marqueurs et plages
settings-markers-in-video = Dans le fichier vidéo
settings-markers-in-video-hint = XMP, affiché sur le clip dans Premiere Pro
settings-markers-comment = Dans le commentaire, une ligne chacune
settings-markers-comment-example = 0:41–0:47 — Lion
settings-markers-note = Les fichiers gardent leurs marqueurs là où ils sont tant qu’ils ne sont pas déplacés.
settings-markers-move-into-videos = Déplacer les marqueurs dans les vidéos…
settings-markers-copy-into-comment = Copier les marqueurs dans les commentaires…
settings-markers-hint = Marqueurs ponctuels et plages, y compris les moments trouvés par l’IA.
settings-updates = Mises à jour
settings-version = Version
settings-old-title = Paramètres d’un ancien frename
settings-old-hint = Récupérer les paramètres d’un frename lancé depuis un dossier zip.
settings-old-scheduled = Les paramètres seront importés au redémarrage de frename
settings-old-not-found = Aucun frename.exe avec un frename.db dans { $folder }
settings-old-failed = Importation impossible : { $reason }
settings-old-import-button = Importer depuis un ancien dossier frename…
settings-ai = Description par IA
settings-ai-key-label = Clé d’API Anthropic
settings-ai-key-placeholder = sk-ant-…
settings-ai-key-get = Obtenir une clé sur console.anthropic.com → API keys.
settings-ai-key-remove-confirm = Supprimer la clé Anthropic enregistrée ?
settings-ai-used-by = Utilisée par « Décrire par IA » en mode par lot et sur les marqueurs (✨, Ctrl+F2), quand un modèle Claude est choisi.
settings-ai-openai-key-label = Clé d’API OpenAI
settings-ai-openai-key-placeholder = sk-…
settings-ai-openai-key-get = Obtenir une clé sur platform.openai.com/api-keys.
settings-ai-openai-key-remove-confirm = Supprimer la clé OpenAI enregistrée ?
settings-ai-openai-used-by = Utilisée par « Décrire par IA » en mode par lot et sur les marqueurs (✨, Ctrl+F2), quand un modèle GPT est choisi.
settings-ai-gemini-key-label = Clé API Gemini
settings-ai-gemini-key-placeholder = AIza…
settings-ai-gemini-key-get = Obtenir une clé sur aistudio.google.com/apikey.
settings-ai-gemini-key-remove-confirm = Supprimer la clé Gemini enregistrée ?
settings-ai-gemini-used-by = Utilisée par « Décrire par IA » en mode par lot et sur les marqueurs (✨, Ctrl+F2), quand un modèle Gemini est choisi.
settings-ai-model-needs-key = { $model } — ajouter d’abord une clé { $provider }
settings-ai-model-label = Modèle
settings-ai-language-label = Langue de la description
settings-ai-hint = Haiku, GPT-4.1 mini et Gemini Flash-Lite sont les moins chers ; Sonnet, Opus, GPT-4.1 et Gemini Flash remarquent davantage de choses. Un modèle a besoin de la clé de son propre éditeur.
settings-ai-moments-label = Moments
settings-ai-moments-important = Seulement ce qui ressort
settings-ai-moments-full = Couvrir tout le clip
settings-ai-moments-important-hint = Propose aussi où placer l’entrée et la sortie (la pastille IA à côté de IN et OUT)
settings-ai-moments-full-hint = Un moment pour chaque partie, même les calmes ; pas de proposition d’entrée et de sortie
settings-ai-tag-suggestions = Proposer aussi des tags parmi ceux du dossier
settings-ai-tag-suggestions-hint = Dans la même requête, l’IA indique lesquels des tags du dossier conviennent à chaque clip. Un tag proposé n’est ajouté que si on clique dessus.
settings-subtitles = Sous-titres
settings-subtitles-key-label = Clé d’API Soniox
settings-subtitles-key-placeholder = Coller la clé
settings-subtitles-key-get = Obtenir une clé sur console.soniox.com. L’audio est envoyé à Soniox.
settings-subtitles-key-remove-confirm = Supprimer la clé Soniox enregistrée ?
settings-subtitles-languages-label = Langues
settings-subtitles-languages-locked = Enregistrer la clé pour choisir parmi toutes les langues connues de Soniox.
settings-subtitles-languages-loading = Récupération des langues auprès de Soniox…
settings-subtitles-languages-none = Aucune cochée : détection automatique.
settings-subtitles-languages-hint = Les langues parlées dans les images, comme indications.
settings-subtitles-cue-length-label = Longueur des sous-titres
settings-subtitles-cue-short = Court
settings-subtitles-cue-short-hint = Une ligne, jusqu’à 8 s
settings-subtitles-cue-sentence = Une phrase
settings-subtitles-hint = Utilisée par « Générer des sous-titres » en mode par lot. La longueur s’applique aux nouveaux sous-titres.
settings-key-store-windows = le Gestionnaire d’identification Windows
settings-key-store-macos = le Trousseau d’accès macOS
settings-key-store-other = le trousseau système
settings-key-unavailable = Le trousseau système n’a pas pu être ouvert
settings-key-unavailable-hint = Il est peut-être verrouillé, ou il n’y en a pas (comme GNOME Keyring ou KWallet). Les paramètres revérifient à chaque ouverture.
settings-key-remove-confirm-hint = Il faudra la coller de nouveau.
settings-key-remove-ask = Supprimer…
settings-key-remove = Supprimer la clé
settings-key-keep = Conserver
settings-key-replace = Remplacer…
settings-key-hide = Masquer
settings-key-show = Afficher
settings-key-save = Enregistrer la clé
settings-key-get = Obtenir une clé
settings-key-billing = Ajouter du crédit
settings-key-cancel = Annuler
settings-key-saved-in = Enregistrée dans { $store } sur cet ordinateur
settings-key-save-into = « Enregistrer la clé » la conserve dans { $store } sur cet ordinateur.

## Batch mode

batch-checked-count = { $count ->
    [one] { $count } fichier coché
   *[other] { $count } fichiers cochés
}
batch-no-key = Pas de clé
batch-group-move = Déplacer entre emplacements
batch-group-fix = Corriger les noms et les vidéos
batch-group-paid = Services payants
batch-reason-none-checked = Aucun fichier coché
batch-reason-reading = Lecture des durées des clips…
batch-check-all = Cocher { $count ->
    [one] { $count } fichier
   *[other] { $count } fichiers
}
batch-job-running = en cours
batch-job-finished = terminé
batch-job-stopped = arrêté
batch-progress-files = { $finished } sur { $total ->
    [one] { $total } fichier
   *[other] { $total } fichiers
}
batch-time-left = environ { $time } restantes
batch-time-estimating = estimation du temps…
batch-time-spent = { $time } écoulées
batch-runs-in-background = Il est possible de continuer à travailler pendant l’exécution
batch-clip-in-work = Utilisé : { $action }
batch-folder-held = Occupé : { $action }
batch-indicator-hint = Cliquer pour ouvrir le panneau et annuler
batch-action-prepare = Préparer les clips
batch-action-prepare-hint = Pour chaque clip coché, en un seul passage : les sous-titres (Soniox) s’il n’en a pas, puis la description par IA : un résumé, ses moments sous forme de marqueurs, le commentaire et des propositions de tags. Un seul prix, un seul plafond et une seule progression pour l’ensemble.
batch-prepare-step-off = désactivée
batch-prepare-nothing = Rien à faire pour ces clips
batch-prepare-report-cap = { $spent } dépensés sur un plafond de { $cap }
batch-option-steps = Étapes
batch-prepare-step-subtitles = Créer d’abord les sous-titres (Soniox)
batch-prepare-step-description = Décrire les clips par IA
batch-prepare-cap = Arrêter les dépenses à
batch-prepare-cap-none = Aucun plafond (dollars US)
batch-prepare-cap-invalid = Le plafond est un nombre de dollars, par exemple 5
batch-prepare-left = La dernière exécution s’est arrêtée avec { $count ->
    [one] { $count } clip restant
   *[other] { $count } clips restants
}
batch-prepare-check-left = Les cocher
batch-prepare-estimating = Calcul de ce que fera chaque étape…
batch-prepare-plan-subtitles = Sous-titres
batch-prepare-plan-description = Description
batch-prepare-plan-none = rien à faire
batch-prepare-plan-subtitles-line = { $videos ->
    [one] { $videos } vidéo
   *[other] { $videos } vidéos
} à transcrire, { $duration } d’audio
batch-prepare-plan-description-line = { $videos ->
    [one] { $videos } vidéo
   *[other] { $videos } vidéos
} à décrire
batch-prepare-resume-note = Chaque étape saute ce qui existe déjà, donc une exécution arrêtée se reprend en la relançant : rien n’est payé deux fois.
batch-prepare-run = Préparer { $clips ->
    [one] { $clips } clip
   *[other] { $clips } clips
} · environ { $dollars }
batch-service-prepare = Soniox + { $provider }
batch-prepare-groups-title = Plans similaires
batch-prepare-groups-hint = Des clips qui ressemblent au même plan : doublons, réexportations, caméra qui n’a pas bougé.
batch-prepare-group-clips = { $count } clips
batch-prepare-check-group = Cocher ces clips
batch-solo-queued = En attente : { $action }
batch-suggest-in-job = Une description de ce clip figure déjà dans la tâche en cours, sans propositions de tags. Redemander à la fin de la tâche
batch-suggest-no-tags = Ce dossier n’a pas encore de tags à proposer : en ajouter d’abord
batch-solo-gave-up = Préparation impossible
batch-solo-withdrawn = Retiré : { $action }
batch-solo-in-work = { $action } est en cours. L’annuler depuis le bandeau ou la page de la tâche
batch-waiting-title = { $count ->
    [one] { $count } exécution en attente
   *[other] { $count } exécutions en attente
}
batch-waiting-remove = Retirer
batch-indicator-waiting = { $count } en attente
file-workspace-describe-clip = Décrire ce clip par IA
file-workspace-subtitles-clip = Générer des sous-titres pour ce clip
file-workspace-describe-clip-queued = Décrire par IA : en attente de son tour. Cliquer pour retirer
file-workspace-describe-clip-running = Décrire par IA : en cours
file-workspace-suggest-tags-clip = Proposer des tags pour ce clip par IA (Ctrl+F6)
file-workspace-suggest-tags-clip-queued = Proposer des tags : en attente de son tour. Cliquer pour retirer
file-workspace-suggest-tags-clip-running = Proposer des tags : en cours
file-workspace-subtitles-clip-queued = Générer des sous-titres : en attente de son tour. Cliquer pour retirer
file-workspace-subtitles-clip-running = Générer des sous-titres : en cours
batch-run-again = Relancer sur { $count ->
    [one] { $count } fichier
   *[other] { $count } fichiers
}
batch-result-done = Terminé : { $total ->
    [one] { $total } fichier
   *[other] { $total } fichiers
}
batch-result-done-detail = { $changed ->
    [one] { $changed } modifié
   *[other] { $changed } modifiés
}, { $unchanged } sans rien à modifier
batch-result-problems = Terminé avec des problèmes : { $failed } sur { $total ->
    [one] { $total } fichier
   *[other] { $total } fichiers
} non traités
batch-figure-unchanged = inchangés
batch-figure-not-done = non traités
batch-figure-not-reached = non atteints
batch-files-not-done = Fichiers non traités
batch-table-file = Fichier
batch-table-why = Raison

batch-run-move-comments = Déplacer { $count ->
    [one] { $count } commentaire
   *[other] { $count } commentaires
}
batch-run-move-in-out = Déplacer l’entrée/la sortie de { $count ->
    [one] { $count } fichier
   *[other] { $count } fichiers
}
batch-run-convert = Convertir { $count ->
    [one] { $count } fichier
   *[other] { $count } fichiers
}
batch-run-rotate = Faire pivoter { $count ->
    [one] { $count } vidéo
   *[other] { $count } vidéos
}
batch-run-tag = Tagger { $count ->
    [one] { $count } fichier
   *[other] { $count } fichiers
}
batch-run-fix-tags = Réordonner les tags dans { $count ->
    [one] { $count } nom
   *[other] { $count } noms
}
batch-run-rename = Renommer { $count ->
    [one] { $count } fichier
   *[other] { $count } fichiers
}
batch-run-reload = Recharger { $count ->
    [one] { $count } fichier
   *[other] { $count } fichiers
}
batch-run-export-markers = Exporter les marqueurs de { $count ->
    [one] { $count } fichier
   *[other] { $count } fichiers
}
batch-service-soniox = Soniox
batch-change-renames = Renomme des fichiers
batch-change-videos = Écrit dans les vidéos
batch-change-text-files = Écrit des fichiers texte à côté des vidéos
batch-change-comments = Écrit dans les commentaires
batch-change-subtitles = Écrit des fichiers de sous-titres à côté des vidéos
batch-change-records = Ne modifie que les enregistrements propres à frename
batch-option-direction = Sens
batch-option-language = Langue
batch-option-tag-suggestions = Propositions de tags
batch-option-described = Vidéos décrites
batch-option-subtitled = Vidéos sous-titrées
batch-option-subtitles-write = Fichiers à écrire
batch-option-spacing = Espacement
batch-option-tag = Tag
batch-option-turn = Rotation
batch-plan-videos = Vidéos
batch-plan-length = Durée
batch-plan-cost = Coût
batch-plan-time = Temps
batch-reason-estimate = En attente de l’estimation
batch-reason-subtitles-no-format = Cocher au moins un fichier à écrire
batch-set-key = Définir la clé…
batch-check-key = Vérifier la clé…
batch-subtitles-languages-auto = Détectée dans chaque vidéo
batch-action-describe-ai-run-waiting = Décrire les vidéos
batch-action-fix-tags-order = Ordre : comme dans la liste des tags, tags inconnus d’abord.
batch-action-markers-comment-hint-short = Transforme les lignes de commentaire avec un temps en marqueurs, ou copie les marqueurs dans le commentaire.
batch-action-markers-to-comment-hint = Les marqueurs restent ; relancer l’action n’ajoute rien en double.
batch-action-rotate-hint-short = Fait pivoter les clips MP4 et MOV grâce à leur indicateur de rotation, sans réencodage.
batch-action-tag-commented-off = Le tag des vidéos commentées est désactivé
batch-action-tag-commented-choose = Choisir le tag…

batch-title = Actions par lot
batch-done-label-changed = modifiés
batch-done-label-subtitled = sous-titrés
batch-ai-at-least = au moins
batch-cancel = Annuler
batch-stopping = Arrêt en cours…
batch-stop-over-budget = Arrêté : le clip suivant pourrait faire dépasser le plafond de { $cap } ({ $spent } dépensés). Relever le plafond pour faire le reste.
batch-stopped = Arrêté après { $finished } sur { $total ->
    [one] { $total } fichier
   *[other] { $total } fichiers
}.
batch-close = Fermer
batch-failed-subtitles = Non sous-titrés :
batch-written-subtitles = Fichiers écrits :
batch-table-written = Écrits
batch-merged-markers = Marqueurs en double fusionnés :
batch-table-merged = Fusionnés
batch-markers-merged = { $count ->
    [one] { $count } marqueur en double fusionné
   *[other] { $count } marqueurs en double fusionnés
}
batch-add-credit = Ajouter du crédit
batch-open-log = Ouvrir le journal

batch-action-move-comments = Déplacer les commentaires
batch-action-move-comments-hint = Déplace le commentaire de chaque fichier coché vers l’emplacement choisi, avec les points d’entrée/de sortie qu’il contient. Les tags et les points d’entrée/de sortie conservés dans la vidéo restent où ils sont.
batch-action-move-comments-into-videos = Des fichiers texte vers les vidéos (XMP)
batch-action-move-comments-into-text-files = Des vidéos (XMP) vers des fichiers texte

batch-action-move-in-out = Points d’entrée/de sortie : commentaire ⇄ vidéo (XMP)
batch-action-move-in-out-hint = Déplace les points d’entrée/de sortie de chaque fichier coché vers l’emplacement choisi. Le reste du commentaire reste où il est.
batch-action-move-in-out-into-videos = Des commentaires vers les vidéos (marqueur XMP Adobe)
batch-action-move-in-out-into-comments = Des vidéos (marqueur XMP) vers les commentaires

batch-action-markers-comment = Marqueurs ⇄ commentaire
batch-action-markers-comment-to-markers = Lignes de commentaire avec un temps vers des marqueurs
batch-action-markers-to-comment = Marqueurs vers le commentaire (une copie : les marqueurs restent)
batch-action-markers-comment-hint = Une ligne comme « 03:24 — Take 3 — nice light » est un marqueur à 3:24 nommé « Take 3 » avec le commentaire « nice light » ; « 0:41-0:47 — Lion » est un marqueur de 0:41 à 0:47. Le nom et le commentaire sont séparés au premier «  —  » ou «  --  », pas à un simple «  -  ». Les moments d’une description par IA (« 0:00–0:14 Street. ») deviennent des marqueurs blancs et restent dans la description. Relancer l’action dans un sens ou dans l’autre n’ajoute rien en double.

batch-action-tag-commented = Tagger les vidéos commentées
batch-action-tag-commented-hint = Ajoute le tag « { $tag } » à chaque vidéo cochée ayant un commentaire de votre part (les descriptions par IA ne comptent pas) et le retire de celles qui n’en ont pas. Les fichiers dont le tag change sont renommés.
batch-action-tag-commented-hint-off = Ajoute le tag des vidéos commentées à chaque vidéo cochée ayant un commentaire de votre part et le retire de celles qui n’en ont pas. Le tag est désactivé dans les paramètres.
batch-action-fix-tags = Corriger les tags par priorité
batch-action-fix-tags-hint = Place les tags du nom de chaque fichier coché dans l’ordre du panneau des tags : plus un tag est haut dans le panneau, plus il vient tôt dans le nom. Les tags que le dossier ne connaît pas encore viennent en premier, comme dans le panneau des tags. Les fichiers dont l’ordre change sont renommés.

batch-action-respace-tags = Appliquer l’espacement des tags
batch-action-respace-tags-hint-space = Renomme chaque fichier coché pour mettre une espace après chaque tag, comme défini dans les paramètres : Food. Goat. clip.mp4.
batch-action-respace-tags-hint-no-space = Renomme chaque fichier coché pour ne laisser aucune espace après ses tags, comme défini dans les paramètres : Food.Goat.clip.mp4.
batch-action-respace-tags-status-space = Espacement : une espace après chaque tag
batch-action-respace-tags-status-no-space = Espacement : aucune espace après les tags
batch-action-export-markers = Exporter les marqueurs en texte
batch-action-export-markers-hint = Écrit les marqueurs de chaque fichier coché dans un fichier texte à côté, clip.markers.txt : une ligne par marqueur dans l’ordre du temps, comme « 0:41–0:47 [red] — Lion crossing — walks left to right ». Une couleur autre que le vert est entre crochets ; le commentaire suit le nom. « Marqueurs ⇄ commentaire » relit ces lignes comme des marqueurs. Un fichier sans marqueur n’obtient pas de fichier texte, et le fichier texte d’une exécution précédente est remplacé.
batch-export-markers-cannot-hold = Ce fichier ne peut pas contenir de marqueurs
batch-action-reload-files = Réinitialiser le cache et recharger
batch-action-reload-files-hint = Relit depuis le fichier lui-même le commentaire et les points d’entrée/de sortie de chaque fichier coché et remplace ce que le dossier en avait mémorisé. À utiliser après une modification des fichiers dans un autre programme. Les fichiers dont les valeurs mémorisées manquaient ou étaient périmées comptent comme modifiés.

batch-action-describe-ai = Décrire par IA
batch-action-describe-ai-run = Décrire { $videos } · environ { $dollars }
batch-action-describe-ai-estimating = Estimation… { $known } / { $total }
batch-action-describe-ai-none = Aucune vidéo à décrire.
batch-action-describe-ai-hint = { batch-runs-in-background }. Annuler conserve les vidéos déjà décrites ; relancer les saute.
batch-action-describe-ai-no-subtitles = Sans sous-titres (seule l’image est décrite) : { $videos }.
batch-action-describe-ai-redo = Refaire les vidéos qui ont déjà une description par IA
batch-action-describe-ai-hint-panel = Décrit ce qui se passe dans chaque vidéo cochée, et quand : un résumé et des segments horodatés vont dans la description par IA de son commentaire ; votre propre texte est conservé. Les images et les sous-titres sont envoyés à { $provider }, l’éditeur du modèle choisi.
batch-ai-change = Modifier
batch-ai-tag-suggestions-off = Désactivées
batch-ai-tag-suggestions-no-tags = Activées, mais ce dossier n’a pas encore de tags
batch-ai-tag-suggestions-on = { $tags ->
    [one] Parmi { $tags } tag du dossier
   *[other] Parmi les { $tags } tags du dossier
}
batch-ai-key-missing = Définir une clé d’API { $provider } dans les paramètres
batch-ai-key-unavailable = Le trousseau système n’a pas pu être ouvert : il est peut-être verrouillé, ou il n’y en a pas (comme GNOME Keyring ou KWallet).
## AI description language names: the Settings picker and { $language } above.

ai-language-same-as-subtitles = Comme les sous-titres
ai-language-english = Anglais
ai-language-russian = Russe
ai-language-ukrainian = Ukrainien
ai-language-german = Allemand
ai-language-spanish = Espagnol
ai-language-french = Français
batch-videos-count = { $n ->
    [one] { $n } vidéo
   *[other] { $n } vidéos
}
batch-minutes = { $n } min
batch-ai-minutes-under = < 1 min
batch-ai-duration-under-minute = moins d’une minute
batch-ai-duration-hours = { $h } h
batch-ai-duration-hours-minutes = { $h } h { $m } min
batch-ai-dollars-under = moins de 0,01 $
batch-ai-skip-described = déjà décrites
batch-ai-skip-too-long = plus de 30 min
batch-ai-skip-unreadable = illisibles
batch-ai-skipped = Ignorées : { $parts }.
batch-ai-progress-frame = image { $done } sur { $total }
batch-ai-progress-waiting = en attente de { $provider }
batch-ai-progress-saving = enregistrement
batch-ai-progress-retry-rate-limit = nouvel essai dans { $seconds } s (limite de débit)
batch-ai-progress-retry-temporary = nouvel essai dans { $seconds } s (échec temporaire)
batch-ai-stop-no-key = Arrêté : pas de clé d’API { $provider }. En définir une dans les paramètres.
batch-ai-fail-no-key = Pas de clé d’API
batch-ai-fail-unreadable = La vidéo n’a pas pu être lue
batch-ai-fail-too-long = Le clip est trop long pour l’IA (plus de 30 min)
batch-ai-fail-not-saved = La description n’a pas pu être enregistrée
batch-ai-stop-offline = Arrêté : pas de connexion à { $provider }. Relancer pour décrire le reste.

batch-action-generate-subtitles = Générer des sous-titres
batch-action-generate-subtitles-install-ffmpeg = pour lire les .mkv, .m2ts, .avi …, installer ffmpeg depuis ffmpeg.org, l’ajouter au PATH, redémarrer frename
batch-action-generate-subtitles-replace = Remplacer les sous-titres existants
batch-action-generate-subtitles-replace-hint = Transcrit de nouveau ; coût indiqué.
batch-action-generate-subtitles-srt = Sous-titres SRT (clip.srt)
batch-action-generate-subtitles-premiere = Transcription Premiere Pro (clip.premiere.json)
batch-action-generate-subtitles-premiere-hint = Dans Premiere Pro : panneau Texte → Transcription → Importer une transcription statique.
batch-action-generate-subtitles-privacy = L’audio de ces vidéos est envoyé à Soniox, puis supprimé là-bas.
batch-action-generate-subtitles-duration-hint = Compter quelques minutes par heure d’audio. { batch-runs-in-background }. Fermer frename arrête l’exécution ; les sous-titres terminés sont conservés.
batch-action-generate-subtitles-hint = Transcrit la parole de chaque vidéo cochée avec Soniox et enregistre le résultat à côté : des sous-titres (clip.srt), là où frename les affiche, et/ou une transcription Premiere Pro.
batch-subtitles-transcribe = Transcrire
batch-subtitles-transcribe-count = Transcrire { $videos }
batch-subtitles-build-free = Créer { $count } (gratuit)
batch-subtitles-nothing = Rien à transcrire
batch-subtitles-estimating = Estimation…
batch-subtitles-key-missing = Définir une clé d’API Soniox dans les paramètres
batch-subtitles-key-rejected = Soniox a refusé la clé
batch-subtitles-cost-unknown = coût inconnu
batch-subtitles-typical-price = { $usd } (prix habituel)
batch-subtitles-unknown-length = + { $n } de durée inconnue
batch-subtitles-plan-line = { $videos } à transcrire, { $duration } d’audio{ $unknown } · { $cost }
batch-subtitles-already = { $n ->
    [one] { $n } a déjà des sous-titres
   *[other] { $n } ont déjà des sous-titres
}
batch-subtitles-rebuilt-free = { $n ->
    [one] { $n } recréé
   *[other] { $n } recréés
} gratuitement à partir d’une transcription enregistrée
batch-subtitles-no-speech-before = { $n } sans parole la dernière fois
batch-subtitles-no-audio = { $n ->
    [one] { $n } n’a pas d’audio
   *[other] { $n } n’ont pas d’audio
}
batch-subtitles-shared-name = { $n ->
    [one] { $n } partage le nom de ses sous-titres avec une autre vidéo
   *[other] { $n } partagent le nom de leurs sous-titres avec une autre vidéo
}
batch-subtitles-unreadable = { $n ->
    [one] { $n } illisible
   *[other] { $n } illisibles
} ({ $how_to })
batch-subtitles-shared-name-reason = partage le nom de ses sous-titres avec une autre vidéo
batch-subtitles-unsupported-reason = format audio non pris en charge
batch-subtitles-progress-transcribing = transcription…
batch-subtitles-progress-extracting = extraction de l’audio { $percent } %
batch-subtitles-progress-extracting-unknown = extraction de l’audio
batch-subtitles-progress-uploading = envoi { $percent } %
batch-subtitles-progress-uploading-unknown = envoi
batch-subtitles-progress-transcribing-left = transcription… environ { $time } restantes
batch-subtitles-progress-saving = enregistrement
batch-subtitles-reason-already = avait déjà des sous-titres
batch-subtitles-reason-no-audio = pas d’audio
batch-subtitles-reason-no-speech = pas de parole
batch-subtitles-reason-soniox = Soniox : { $message }
batch-subtitles-reason-unreachable = Soniox est injoignable
batch-subtitles-stop-key-rejected = Arrêté : Soniox a refusé la clé. La vérifier dans Paramètres → Sous-titres.
batch-subtitles-stop-balance-empty = Arrêté : le solde Soniox est vide. Le recharger sur console.soniox.com.
batch-subtitles-stop-budget-used = Arrêté : le budget mensuel Soniox est épuisé. Le relever sur console.soniox.com.
batch-subtitles-stop-other = Arrêté : Soniox a refusé de continuer ({ $error_type }).
batch-subtitles-stop-unreachable = Arrêté : Soniox est injoignable. Vérifier la connexion Internet.
batch-subtitles-report-spend = Soniox : au moins { $duration } · { $usd }
batch-subtitles-report-failed-deletes = { $n ->
    [one] { $n } envoi n’a pas pu être supprimé
   *[other] { $n } envois n’ont pas pu être supprimés
} sur Soniox (voir le journal)
batch-subtitles-languages-need-key = Enregistrer une clé Soniox pour choisir parmi toutes ses langues.
batch-subtitles-languages-rejected = Soniox a refusé la clé.
batch-subtitles-languages-failed = Impossible de récupérer les langues auprès de Soniox.
batch-subtitles-count = { $n ->
    [one] { $n } sous-titre
   *[other] { $n } sous-titres
}
batch-subtitles-duration-seconds = { $s } s
batch-subtitles-usd-under = moins de 0,01 $
batch-subtitles-usd-about = environ { $amount } $

## File list
folder-has-subtitles = A des sous-titres
folder-closed-by-job = Fermé pendant qu’une action par lot le modifie
folder-filter-tip = Afficher seulement…

folder-search-placeholder = Rechercher un fichier ou un commentaire
folder-search-comments-loading = Recherche dans les commentaires… { $n } restants
folder-search-clear = Effacer
folder-all = Tous
folder-invert = Inverser
folder-checked = { $count ->
    [one] { $count } coché
   *[other] { $count } cochés
}
folder-outcome-changed = Modifié
folder-outcome-unchanged = Rien à modifier
folder-outcome-failed = Échec, voir le journal
folder-rename-error-empty = Le nom est vide
folder-rename-error-bad-character = Interdit : \ / : * ? " < > |
folder-rename-error-trailing = Ne peut pas se terminer par un point ou une espace
folder-rename-error-exists = Un fichier portant ce nom existe
folder-markers-not-saved = Marqueurs non enregistrés : le fichier est en lecture seule ou utilisé (le fermer dans Premiere, puis ouvrir le fichier et le quitter de nouveau)
drag-out-not-saved = Glisser impossible : le fichier ne peut pas être enregistré (lecture seule, ou ouvert dans Premiere)
folder-opening = Ouverture du dossier…
folder-empty-title = Aucune vidéo dans ce dossier
folder-empty-line = frename affiche les fichiers MP4, MOV, MKV et autres fichiers vidéo.
folder-open-another = Ouvrir un autre dossier…
folder-no-match = Aucun fichier ne correspond
folder-show-all = Tout afficher
folder-locked = Les cases sont verrouillées pendant l’exécution de { $action }
folder-checked-hidden = { $count ->
    [one] { $count } coché
   *[other] { $count } cochés
} · { $hidden ->
    [one] { $hidden } masqué
   *[other] { $hidden } masqués
}
folder-checked-hidden-tip = { $hidden ->
    [one] { $hidden } fichier coché est masqué par la recherche ou un filtre ; l’action s’exécute aussi sur lui
   *[other] { $hidden } fichiers cochés sont masqués par la recherche ou un filtre ; l’action s’exécute aussi sur eux
}
folder-outcome-working = En cours
folder-outcome-not-reached = Non atteint
folder-window-empty-title = Ouvrir un dossier de clips
folder-window-empty-line = Ou déposer un dossier sur la fenêtre. Un clic droit sur le bouton ouvre un seul fichier.
folder-window-open = Ouvrir un dossier…

## Recent folders (the arrow next to the open button, and the empty screen)

recent-folders-tip = Dossiers récents
recent-folders-title = Dossiers récents
recent-folders-none = Aucun dossier ouvert pour l’instant
recent-folders-not-found = Introuvable
recent-folders-remove-tip = Retirer de la liste
recent-folders-clear = Vider la liste
recent-folders-remove = Retirer
recent-folders-keep = Conserver
recent-folders-missing-line = Introuvable. Le retirer ?
recent-folders-just-now = À l’instant
recent-folders-minutes = il y a { $count } min
recent-folders-hours = il y a { $count } h
recent-folders-days = { $count ->
    [one] il y a { $count } jour
   *[other] il y a { $count } jours
}
recent-folders-weeks = { $count ->
    [one] il y a { $count } semaine
   *[other] il y a { $count } semaines
}
recent-folders-months = { $count ->
    [one] il y a { $count } mois
   *[other] il y a { $count } mois
}
recent-folders-years = { $count ->
    [one] il y a { $count } an
   *[other] il y a { $count } ans
}

## File menu (right-click on a file, or F11 / Shift+F11 / Ctrl+F11)

file-menu-show-in-explorer = Afficher dans l’Explorateur
file-menu-show-in-file-manager = Afficher dans le gestionnaire de fichiers
file-menu-copy-path = Copier le chemin complet
file-menu-copy-name = Copier le nom du fichier
file-menu-copied = Copié
file-menu-not-copied = Non copié : le presse-papiers n’est pas disponible
file-menu-not-shown = Impossible d’ouvrir le gestionnaire de fichiers

## Controls bar under the file list

folder-controls-filter-untagged = Sans tag
folder-controls-filter-subtitles = Sous-titres
folder-controls-filter-comments = Commentaires
folder-controls-filter-markers = Marqueurs
folder-controls-scroll = Afficher le fichier ouvert dans la liste
folder-controls-open = Ouvrir un dossier
folder-controls-batch = Actions par lot sur les fichiers cochés
folder-controls-batch-back = Retour au fichier ouvert
folder-controls-update-available = Mise à jour disponible : { $version }
folder-controls-previous = Fichier précédent
folder-controls-next = Fichier suivant
folder-controls-more = Plus
folder-controls-open-file = Clic droit : ouvrir un seul fichier

## Video

video-controls-back = Reculer de 10 s
video-controls-play = Lecture
video-controls-pause = Pause
video-controls-forward = Avancer de 10 s
video-controls-frame-back = Image précédente
video-controls-frame-forward = Image suivante
video-controls-set-in = Définir le point d’entrée
video-controls-set-out = Définir le point de sortie
video-controls-screenshot = Enregistrer cette image
video-controls-add-marker = Ajouter un marqueur
video-controls-add-marker-hold = Maintenir pour une plage ; appuyer de nouveau pour la nommer
video-controls-cannot-hold-markers = Ce fichier ne peut pas contenir de marqueurs
video-controls-add-a-name = Ajouter un nom
video-controls-rotate-left = Pivoter à gauche
video-controls-rotate-right = Pivoter à droite
video-controls-more = Plus
media-viewer-loading-slow = En attente du fichier… (un fichier dans le cloud peut prendre du temps)
media-viewer-cannot-play = Ce clip ne peut pas être lu
media-viewer-no-picture = Ce fichier n’a pas d’image vidéo
video-controls-volume-scroll = Volume — faire défiler pour modifier
rotate-cannot = Rotation impossible : { $reason }
rotate-reason-missing = le fichier n’est plus là
rotate-flag-right = 90° à droite
rotate-flag-left = 90° à gauche
rotate-flag-half = 180°
rotate-flag-none = aucune
rotate-now = Rotation : { $flag }
rotate-turned = Pivoté de { $turn } · rotation actuelle : { $flag }
rotate-failed = Non pivoté : { $reason }
undo-name-taken = Non modifié : un autre fichier s’appelle déjà { $name }
undo-back-on-clip = De retour sur ce clip (le déplacement est annulé). Ctrl+Z de nouveau annule sa dernière modification.
redo-on-clip = De nouveau sur ce clip (le déplacement est rétabli).
rotate-reason-in-use = le fichier est en lecture seule ou utilisé
rotate-reason-format = ce format n’a pas d’indicateur de rotation
rotate-reason-damaged = le fichier est endommagé
rotate-reason-no-video = le fichier n’a pas de piste vidéo
rotate-reason-matrix = la vidéo a une matrice d’affichage inhabituelle
batch-action-rotate = Faire pivoter les vidéos
batch-action-rotate-hint = Modifie l’indicateur de rotation de chaque fichier MP4 et MOV coché ; l’image n’est pas réencodée, et les commentaires, points d’entrée/de sortie et marqueurs restent. Premiere Pro devrait afficher le clip pivoté à l’importation. Les autres formats n’ont pas d’indicateur de rotation et échouent. Relancer l’action fait pivoter de nouveau les fichiers ; « Réinitialiser » supprime toute rotation, y compris celle enregistrée par un téléphone, si bien qu’un clip portrait de téléphone se lit alors sur le côté.
batch-action-rotate-right = 90° à droite (sens horaire)
batch-action-rotate-left = 90° à gauche (sens antihoraire)
batch-action-rotate-half = 180°
batch-action-rotate-reset = Réinitialiser : aucune rotation (0°)
media-viewer-video-subtitle-list = Liste des sous-titres
media-viewer-video-marker-list = Liste des marqueurs
media-viewer-video-markers-hint = Maj+F1 / Maj+F3 passent d’un marqueur à l’autre ; Maj+glisser aimante
media-viewer-video-tab-markers = Marqueurs
media-viewer-video-fullscreen = Plein écran
media-viewer-video-close-list = Fermer la liste
media-viewer-video-subtitle-file-unreadable = Impossible de lire ce fichier
# Shown over the picture when a clip opens where playback stopped last time; Home or a click goes to the start.
media-viewer-video-resumed = Reprise à { $time } · Origine : recommencer

## Markers list
markers-in-out = Points d’entrée/de sortie

markers-empty = Aucun marqueur pour l’instant
markers-add = Ajouter un marqueur
markers-ai-hint = Marqueur IA : remplacé quand l’IA décrit de nouveau ce clip
markers-keep-color = Garder la couleur
markers-done = Terminé
markers-delete = Supprimer le marqueur
markers-cannot-hold-hint = Premiere lit les marqueurs des fichiers MP4 et MOV.
markers-color-green = Vert
markers-color-red = Rouge
markers-color-orange = Orange
markers-color-yellow = Jaune
markers-color-white = Blanc
markers-color-blue = Bleu
markers-color-cyan = Cyan
markers-color-lavender = Lavande
markers-color-magenta = Magenta
markers-color-other = Autre couleur
markers-read-only = lecture seule
markers-name-placeholder = Nom
markers-ai-describe = Décrire par IA : nommer le marqueur et ajouter ce qui se passe
markers-ai-stop = Arrêter la description
markers-ai-describe-unnamed = Décrire { $count } sans nom
markers-ai-describe-unnamed-hint = Nomme et décrit par IA chaque marqueur sans nom, quelques-uns à la fois. Une requête chacune ; un nom saisi entre-temps est conservé.
markers-ai-price-about = environ { $dollars }
markers-ai-describe-unnamed-cost = Une requête chacune : { $each } par marqueur, { $total } pour tous.
markers-ai-confirm = { $count ->
    [one] Décrire { $count } marqueur ? Une requête : { $total }.
   *[other] Décrire { $count } marqueurs ? Une requête chacune : { $total } au total.
}
markers-ai-confirm-unpriced = { $count ->
    [one] Décrire { $count } marqueur ? Une requête payante.
   *[other] Décrire { $count } marqueurs ? Une requête payante chacune.
}
markers-ai-confirm-yes = Décrire
markers-ai-confirm-no = Annuler
markers-ai-waiting = En attente de son tour…
markers-ai-run = Description de { $describing } · { $waiting } en attente
markers-ai-run-last = Description de { $describing }
markers-ai-stop-all = Tout arrêter
markers-ai-describing = Description en cours…
markers-ai-done = Marqueur décrit
markers-ai-nothing-new = Le marqueur a déjà cette description
markers-no-marker-notice = Aucun marqueur ici
markers-ai-no-key = Pas de clé d’API { $provider } : en définir une dans les paramètres
markers-ai-failed = Non décrit : { $reason }
markers-ai-failed-unknown = une erreur s’est produite
markers-read-only-notice = Ce marqueur est en lecture seule
markers-ai-gone = Le marqueur n’existe plus : sa description n’a pas été ajoutée
markers-ai-stopping-for-batch = Arrêt préalable des descriptions de marqueurs… Cela prend en général quelques secondes.
folder-not-saved-marker-ai = Non enregistré : la description du marqueur lisait encore le clip. Réessayer dans un instant.

## File workspace

file-workspace-search-placeholder = Rechercher un tag — ou simplement taper
file-workspace-search-clear = Effacer
file-workspace-comment-placeholder = Commentaire...
file-workspace-comment-collapse = Retour aux tags
file-workspace-comment-expand = Agrandir le commentaire

## Tag grid

tag-grid-star = Étoile : garder en haut
tag-grid-unstar = Retirer l’étoile
tag-grid-save = Ajouter aux tags du dossier
tag-grid-delete = Supprimer « { $tag } » des tags du dossier
tag-grid-create = Créer « { $tag } »
tag-grid-no-file = Ouvrir un clip pour le tagger
tag-grid-no-tags = Aucun tag pour l’instant
tag-grid-no-tags-hint = Taper un nom et appuyer sur Entrée pour créer le premier.
tag-grid-group-unsaved = Absents des tags du dossier
tag-grid-group-folder = Tags du dossier
tag-grid-more = +{ $count } de plus

## Order strip

sync-panel-locked = Réordonner ci-dessous réordonne le dossier
sync-panel-unlocked = Réordonner ci-dessous ne change que ce clip
sync-panel-unlock = Déverrouiller
sync-panel-lock = Verrouiller
sync-panel-differs = L’ordre diffère de celui du dossier
sync-panel-use-for-folder = Utiliser pour le dossier
sync-panel-sort-like-folder = Trier comme le dossier

## File name card

file-name-panel-no-tags = Aucun tag sur ce clip
file-name-panel-untag = Retirer le tag
file-name-panel-clear-in = Effacer le point d’entrée
file-name-panel-clear-out = Effacer le point de sortie
file-name-panel-suggested-in-out = IA
file-name-panel-apply-suggested-in-out = Définir l’entrée et la sortie selon la proposition de l’IA (arrondies à la seconde)
file-name-panel-suggested-tags = Tags IA
file-name-panel-add-suggested-tag = Ajouter le tag « { $tag } » (sûr à { $percent } %)
file-name-panel-add-suggested-tag-unsure = Ajouter le tag « { $tag } »
file-name-panel-add-all-suggested-tags = Tout ajouter
file-name-panel-add-all-suggested-tags-tip = Ajouter tous les tags proposés par l’IA
file-name-panel-tag-ideas = Idées : { $ideas }
file-name-panel-tag-ideas-tip = Des tags remarqués par l’IA que ce dossier n’a pas. Ils ne sont pas ajoutés.
file-name-panel-saved = Toutes les modifications sont enregistrées
file-name-panel-saved-to-recovery = Enregistré dans la récupération, appliqué en quittant le clip

## Updates (Settings)

updates-check = Rechercher des mises à jour
updates-not-installed = Les mises à jour fonctionnent dans la version installée
updates-from-store = Les mises à jour viennent du Microsoft Store
updates-from-releases = Nouvelles versions : à télécharger depuis la page des versions
updates-open-releases = Ouvrir la page des versions
updates-checking = Vérification…
updates-downloading-named = Téléchargement de { $version }… { $percent } %
updates-downloading = Téléchargement… { $percent } %
updates-restarting = Redémarrage…
updates-check-failed = Impossible de rechercher des mises à jour : { $reason }
updates-update-failed = Mise à jour impossible : { $reason }
updates-close-other-window = Fermer d’abord l’autre fenêtre frename
updates-reason-github = GitHub n’a pas répondu, réessayer plus tard
updates-reason-no-connection = pas de connexion
updates-reason-read-only = le dossier est en lecture seule
updates-reason-not-installed = les mises à jour fonctionnent dans la version installée
updates-reason-download-stopped = le téléchargement s’est arrêté de façon inattendue
updates-version-available = La version { $version } est disponible
updates-up-to-date = frename est à jour
updates-update-and-restart = Mettre à jour et redémarrer
updates-wait-for-batch = Attendre la fin de l’action par lot
updates-current-version = frename { $version }
updates-check-on-start = Rechercher des mises à jour au démarrage de frename

recovery-restored = frename s’est fermé de façon inattendue. Votre travail non enregistré sur { $clip } ({ $what }) a été restauré.
recovery-kept = frename s’est fermé de façon inattendue. Votre travail non enregistré sur { $clip } ({ $what }) n’a pas été appliqué : { $why }. Une copie lisible à retaper se trouve dans { $folder }.
recovery-unreadable = frename s’est fermé de façon inattendue, et un fichier de récupération n’a pas pu être lu. Il est conservé dans { $folder }.
recovery-why-gone = le clip a disparu ou a été renommé
recovery-why-changed = le clip a changé après vos modifications
recovery-why-not-written = un fichier portant ce nom existe, ou le clip est en lecture seule ou utilisé
recovery-tags = { $count ->
    [one] { $count } tag
   *[other] { $count } tags
}
recovery-markers = { $count ->
    [one] { $count } marqueur
   *[other] { $count } marqueurs
}
recovery-comment = commentaire
recovery-in-out = points d’entrée/de sortie
recovery-edits = modifications
folder-frame-saved = Image enregistrée
folder-not-saved = Non enregistré : un fichier portant ce nom existe déjà, ou il est en lecture seule ou utilisé
markers-deleted-notice = Marqueur supprimé
markers-not-saved-notice = Marqueurs non enregistrés : le fichier est en lecture seule ou utilisé
markers-comment-not-saved-notice = Marqueurs non enregistrés : le fichier de commentaire est en lecture seule ou utilisé
markers-copy = Copier
markers-copy-tip = Copier les marqueurs en texte, une ligne chacune
markers-export = Exporter…
markers-export-tip = Enregistrer les marqueurs dans un fichier texte
markers-exported = Marqueurs enregistrés dans { $name }
markers-not-exported = Marqueurs non enregistrés : le fichier n’a pas pu être écrit
