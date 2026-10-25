# frename UI 텍스트, 원본 언어.
# i18n/ 아래의 다른 모든 파일은 이 파일과 똑같은 id, 속성, $인수를 가집니다 (src/i18n.rs의 테스트가 확인).
# 키: <기능>-<요소>[-<세부>], 기능은 src/features/ 아래의 폴더 이름입니다.

## 언어 이름: 모든 파일에서 동일하며, 각각 해당 언어로 표기합니다.

language-name-en = English
language-name-ru = Русский
language-name-de = Deutsch
language-name-fr = Français
language-name-es = Español
language-name-it = Italiano
language-name-pt-BR = Português (Brasil)
language-name-uk = Українська
language-name-sr-Cyrl = Српски (ћирилица)
language-name-sr-Latn = Srpski (latinica)
language-name-ja = 日本語
language-name-ko = 한국어
language-name-zh-Hans = 简体中文
language-name-zh-Hant = 繁體中文

## 설정 창

settings-window-title = 설정
settings-page-interface = 인터페이스
settings-page-saving = 저장
settings-page-ai = AI로 설명
settings-close = 닫기
settings-apply-note = 변경 사항은 바로 적용됩니다. Tab은 옵션 사이, Ctrl+Tab은 페이지 사이를 이동합니다.
settings-language = 언어
settings-language-system = 시스템 ({ $language })
settings-video = 비디오
settings-video-autoplay = 열 때 비디오 자동 재생
settings-subtitle-size = 자막 크기
settings-subtitle-size-small = 작게
settings-subtitle-size-medium = 보통
settings-subtitle-size-large = 크게
settings-subtitle-size-extra-large = 아주 크게
settings-timecode = 시간 표시
settings-timecode-hint = 비디오 아래, 마커 및 자막 목록, 인/아웃 칩에 표시되는 시간과 Ctrl+Shift+C로 복사하는 재생 헤드 시간의 표기 방식입니다. Premiere Pro와 DaVinci Resolve는 클립의 프레임 레이트에 맞춰 시:분:초:프레임으로 표시하며, 29.97fps와 59.94fps는 드롭 프레임입니다. 프레임 레이트가 일정하지 않은 클립은 기본 방식을 유지합니다. 파일에 기록되는 형식은 바뀌지 않습니다.
settings-timecode-default = 기본 (mm:ss)
settings-timecode-premiere = Premiere Pro (HH:MM:SS:FF)
settings-timecode-resolve = DaVinci Resolve (HH:MM:SS:FF)
settings-tags = 태그 색상
settings-tags-monochrome = 단색
settings-tags-monochrome-hint = 직접 추가한 태그는 더 진한 회색, 나머지는 더 연한 회색으로 표시하며 다른 색은 쓰지 않습니다.
settings-file-names = 파일 이름
settings-tags-space-after = 각 태그 뒤에 공백
settings-tags-space-example = Food. Goat. clip.mp4
settings-tags-space-note = 파일 이름은 이름을 바꾸거나 저장할 때까지 그대로입니다.
settings-tags-space-add = 파일 이름에 공백 추가…
settings-tags-space-remove = 파일 이름에서 공백 제거…
settings-comments = 코멘트
settings-comments-in-video = 비디오 파일 안
settings-comments-in-video-hint = XMP, Premiere Pro의 설명 열
settings-comments-text-file = 비디오 옆의 텍스트 파일
settings-comments-text-file-example = clip.comment.txt
settings-comments-note = 코멘트는 옮기기 전까지 현재 위치에 그대로 있습니다.
settings-comments-move-into-videos = 코멘트를 비디오로 이동…
settings-comments-move-into-text-files = 코멘트를 텍스트 파일로 이동…
settings-commented-tag = 코멘트가 있는 비디오에 태그 달기
settings-commented-tag-name = 태그
settings-commented-tag-hint = 비디오에 코멘트를 쓰면 켜지고(예: { $tag }.IMG_0424.MOV) 지우면 꺼집니다(AI 설명은 해당 없음). 그 외에는 직접 바꿀 수 있습니다.
settings-commented-tag-off = 코멘트가 있는 비디오에 태그를 달지 않습니다.
settings-in-out = 인/아웃 포인트
settings-in-out-in-video = 비디오 파일 안
settings-in-out-in-video-hint = Premiere Pro의 서브클립 마커
settings-in-out-comment = 코멘트 안, 한 줄로
settings-in-out-comment-example = In/Out: 00:01:05.250 – 00:02:10.000
settings-in-out-note = 인/아웃 포인트는 옮기기 전까지 현재 위치에 그대로 있습니다.
settings-in-out-move-into-videos = 인/아웃 포인트를 비디오로 이동…
settings-in-out-move-into-comments = 인/아웃 포인트를 코멘트로 이동…
settings-markers = 마커와 구간
settings-markers-in-video = 비디오 파일 안
settings-markers-in-video-hint = XMP, Premiere Pro의 클립에 표시됨
settings-markers-comment = 코멘트 안, 한 줄에 하나씩
settings-markers-comment-example = 0:41–0:47 — Lion
settings-markers-note = 마커는 옮기기 전까지 현재 위치에 그대로 있습니다.
settings-markers-move-into-videos = 마커를 비디오로 이동…
settings-markers-copy-into-comment = 마커를 코멘트로 복사…
settings-markers-hint = 지점과 구간 모두, AI가 찾은 순간도 포함합니다.
settings-updates = 업데이트
settings-version = 버전
settings-release-notes = 릴리스 노트
settings-old-title = 이전 frename의 설정
settings-old-hint = zip 폴더에서 실행하던 frename의 설정을 가져옵니다.
settings-old-scheduled = frename을 다시 시작하면 설정을 가져옵니다
settings-old-not-found = { $folder }에 frename.db가 있는 frename.exe가 없습니다
settings-old-failed = 가져올 수 없습니다: { $reason }
settings-old-import-button = 이전 frename 폴더에서 가져오기…
settings-ai = AI로 설명
settings-ai-key-label = Anthropic API 키
settings-ai-key-placeholder = sk-ant-…
settings-ai-key-get = console.anthropic.com → API keys에서 키를 받으세요.
settings-ai-key-remove-confirm = 저장된 Anthropic 키를 삭제할까요?
settings-ai-used-by = Claude 모델을 선택하면 일괄 작업과 마커(✨, Ctrl+F2)의 AI로 설명에서 사용합니다.
settings-ai-openai-key-label = OpenAI API 키
settings-ai-openai-key-placeholder = sk-…
settings-ai-openai-key-get = platform.openai.com/api-keys에서 키를 받으세요.
settings-ai-openai-key-remove-confirm = 저장된 OpenAI 키를 삭제할까요?
settings-ai-openai-used-by = GPT 모델을 선택하면 일괄 작업과 마커(✨, Ctrl+F2)의 AI로 설명에서 사용합니다.
settings-ai-gemini-key-label = Gemini API 키
settings-ai-gemini-key-placeholder = AIza…
settings-ai-gemini-key-get = aistudio.google.com/apikey에서 키를 받으세요.
settings-ai-gemini-key-remove-confirm = 저장된 Gemini 키를 삭제할까요?
settings-ai-gemini-used-by = Gemini 모델을 선택하면 일괄 작업과 마커(✨, Ctrl+F2)의 AI로 설명에서 사용합니다.
settings-ai-model-needs-key = { $model } — 먼저 { $provider } 키를 추가하세요
settings-ai-model-label = 모델
settings-ai-language-label = 설명 언어
settings-ai-hint = Haiku, GPT-4.1 mini, Gemini Flash-Lite가 가장 저렴하고, Sonnet, Opus, GPT-4.1, Gemini Flash는 더 많은 것을 파악합니다. 모델에는 해당 회사의 키가 필요합니다.
settings-ai-moments-label = 순간
settings-ai-moments-important = 눈에 띄는 것만
settings-ai-moments-full = 클립 전체
settings-ai-moments-important-hint = 인/아웃을 지정할 위치도 제안합니다 (IN과 OUT 옆의 AI 배지)
settings-ai-moments-full-hint = 조용한 부분까지 모든 구간에 순간을 만들며, 인/아웃 제안은 없습니다
settings-ai-tag-suggestions = 폴더의 태그에서 태그도 제안
settings-ai-tag-suggestions-hint = 같은 요청에서 AI가 폴더의 태그 중 각 클립에 맞는 것을 알려 줍니다. 제안된 태그는 클릭할 때만 추가됩니다.
settings-subtitles = 자막
settings-subtitles-key-label = Soniox API 키
settings-subtitles-key-placeholder = 키 붙여넣기
settings-subtitles-key-get = console.soniox.com에서 키를 받으세요. 오디오가 Soniox로 전송됩니다.
settings-subtitles-key-remove-confirm = 저장된 Soniox 키를 삭제할까요?
settings-subtitles-languages-label = 언어
settings-subtitles-languages-locked = 키를 저장하면 Soniox가 지원하는 모든 언어에서 고를 수 있습니다.
settings-subtitles-languages-loading = Soniox에서 언어를 가져오는 중…
settings-subtitles-languages-none = 선택 없음: 자동으로 감지합니다.
settings-subtitles-languages-hint = 영상에서 사용된 언어를 힌트로 지정합니다.
settings-subtitles-cue-length-label = 자막 길이
settings-subtitles-cue-short = 짧게
settings-subtitles-cue-short-hint = 한 줄, 최대 8초
settings-subtitles-cue-sentence = 한 문장
settings-subtitles-hint = 일괄 작업의 자막 생성에서 사용합니다. 자막 길이는 새 자막에 적용됩니다.
settings-key-store-windows = Windows 자격 증명 관리자
settings-key-store-macos = macOS 키체인
settings-key-store-other = 시스템 키링
settings-key-unavailable = 시스템 키링을 열 수 없습니다
settings-key-unavailable-hint = 잠겨 있거나 키링이 없을 수 있습니다(GNOME Keyring, KWallet 등). 설정은 열 때마다 다시 확인합니다.
settings-key-remove-confirm-hint = 다시 붙여넣어야 합니다.
settings-key-remove-ask = 삭제…
settings-key-remove = 키 삭제
settings-key-keep = 유지
settings-key-replace = 교체…
settings-key-hide = 숨기기
settings-key-show = 보기
settings-key-save = 키 저장
settings-key-get = 키 받기
settings-key-billing = 크레딧 충전
settings-key-cancel = 취소
settings-key-saved-in = 이 컴퓨터의 { $store }에 저장됨
settings-key-save-into = 키 저장을 누르면 이 컴퓨터의 { $store }에 보관합니다.

## 일괄 작업

batch-checked-count = { $count ->
   *[other] { $count }개 파일 선택됨
}
batch-no-key = 키 없음
batch-group-move = 위치 간 이동
batch-group-fix = 이름과 비디오 수정
batch-group-paid = 유료 서비스
batch-reason-none-checked = 선택한 파일이 없습니다
batch-reason-reading = 클립 길이 읽는 중…
batch-check-all = { $count ->
   *[other] { $count }개 파일 모두 선택
}
batch-job-running = 실행 중
batch-job-finished = 완료
batch-job-stopped = 중지됨
batch-progress-files = { $finished } / { $total ->
   *[other] { $total }개 파일
}
batch-time-left = 약 { $time } 남음
batch-time-estimating = 시간 계산 중…
batch-time-spent = 지금까지 { $time }
batch-runs-in-background = 실행 중에도 계속 작업할 수 있습니다
batch-clip-in-work = 사용 중: { $action }
batch-folder-held = 작업 중: { $action }
batch-indicator-hint = 클릭하면 패널과 취소 버튼이 나타납니다
batch-action-prepare = 클립 준비
batch-action-prepare-hint = 선택한 각 클립에 대해 한 번에 처리합니다. 자막이 없으면 자막(Soniox)을 만들고, 이어서 AI로 설명합니다. 요약, 마커로 만든 순간, 코멘트, 태그 제안이 포함됩니다. 비용, 상한, 진행률은 전체에 대해 하나입니다.
batch-prepare-step-off = 꺼짐
batch-prepare-nothing = 이 클립들은 할 일이 없습니다
batch-prepare-report-cap = 상한 { $cap } 중 { $spent } 사용
batch-option-steps = 단계
batch-prepare-step-subtitles = 먼저 자막 만들기 (Soniox)
batch-prepare-step-description = AI로 클립 설명
batch-prepare-cap = 지출 중단 한도
batch-prepare-cap-none = 상한 없음 (미국 달러)
batch-prepare-cap-invalid = 상한은 5와 같은 달러 금액이어야 합니다
batch-prepare-left = 지난 실행이 중지되어 { $count ->
   *[other] 클립 { $count }개
}가 남았습니다
batch-prepare-check-left = 선택하기
batch-prepare-estimating = 각 단계에서 할 일을 계산하는 중…
batch-prepare-plan-subtitles = 자막
batch-prepare-plan-description = 설명
batch-prepare-plan-none = 할 일 없음
batch-prepare-plan-subtitles-line = { $videos ->
   *[other] 비디오 { $videos }개
}를 텍스트로 변환, 오디오 { $duration }
batch-prepare-plan-description-line = { $videos ->
   *[other] 비디오 { $videos }개
}를 설명
batch-prepare-resume-note = 각 단계는 이미 있는 것을 건너뛰므로, 중지된 작업은 다시 실행하면 이어집니다. 비용이 두 번 나가지 않습니다.
batch-prepare-run = { $clips ->
   *[other] 클립 { $clips }개
} 준비 · 약 { $dollars }
batch-service-prepare = Soniox + { $provider }
batch-prepare-groups-title = 비슷한 영상
batch-prepare-groups-hint = 같은 샷으로 보이는 클립: 중복, 다시 내보낸 파일, 움직이지 않은 카메라.
batch-prepare-group-clips = 클립 { $count }개
batch-prepare-check-group = 이 클립들 선택
batch-solo-queued = 대기 중: { $action }
batch-suggest-in-job = 이 클립의 설명이 이미 실행 중인 작업에 있으며 태그 제안은 없습니다. 작업이 끝난 뒤 다시 요청하세요
batch-suggest-no-tags = 이 폴더에는 제안할 태그가 아직 없습니다. 먼저 추가하세요
batch-solo-gave-up = 준비하지 못했습니다
batch-solo-withdrawn = 철회됨: { $action }
batch-solo-in-work = { $action } 작업 중입니다. 상단 막대나 작업 페이지에서 취소하세요
batch-waiting-title = { $count ->
   *[other] 대기 중인 작업 { $count }개
}
batch-waiting-remove = 제거
batch-indicator-waiting = { $count }개 대기 중
file-workspace-describe-clip = 이 클립을 AI로 설명
file-workspace-subtitles-clip = 이 클립의 자막 생성
file-workspace-describe-clip-queued = AI로 설명: 순서를 기다리는 중. 클릭하면 철회합니다
file-workspace-describe-clip-running = AI로 설명: 작업 중
file-workspace-suggest-tags-clip = AI로 이 클립의 태그 제안 (Ctrl+F6)
file-workspace-suggest-tags-clip-queued = 태그 제안: 순서를 기다리는 중. 클릭하면 철회합니다
file-workspace-suggest-tags-clip-running = 태그 제안: 작업 중
file-workspace-subtitles-clip-queued = 자막 생성: 순서를 기다리는 중. 클릭하면 철회합니다
file-workspace-subtitles-clip-running = 자막 생성: 작업 중
batch-run-again = { $count ->
   *[other] { $count }개 파일에 다시 실행
}
batch-result-done = 완료: { $total ->
   *[other] { $total }개 파일
}
batch-result-done-detail = { $changed }개 변경됨, { $unchanged }개는 바꿀 것이 없음
batch-result-problems = 문제와 함께 완료: { $total ->
   *[other] { $total }개 파일
} 중 { $failed }개 실패
batch-figure-unchanged = 변경 없음
batch-figure-not-done = 실패
batch-figure-not-reached = 미처리
batch-files-not-done = 처리하지 못한 파일
batch-table-file = 파일
batch-table-why = 이유

batch-run-move-comments = { $count ->
   *[other] 코멘트 { $count }개 이동
}
batch-run-move-in-out = { $count ->
   *[other] { $count }개 파일의 인/아웃 이동
}
batch-run-convert = { $count ->
   *[other] { $count }개 파일 변환
}
batch-run-rotate = { $count ->
   *[other] 비디오 { $count }개 회전
}
batch-run-tag = { $count ->
   *[other] { $count }개 파일에 태그 달기
}
batch-run-fix-tags = { $count ->
   *[other] 이름 { $count }개의 태그 순서 변경
}
batch-run-rename = { $count ->
   *[other] { $count }개 파일 이름 바꾸기
}
batch-run-reload = { $count ->
   *[other] { $count }개 파일 다시 불러오기
}
batch-run-export-markers = { $count ->
   *[other] { $count }개 파일의 마커 내보내기
}
batch-service-soniox = Soniox
batch-change-renames = 파일 이름을 바꿉니다
batch-change-videos = 비디오에 기록합니다
batch-change-text-files = 비디오 옆에 텍스트 파일을 만듭니다
batch-change-comments = 코멘트에 기록합니다
batch-change-subtitles = 비디오 옆에 자막 파일을 만듭니다
batch-change-records = frename 자체 기록만 변경합니다
batch-option-direction = 방향
batch-option-language = 언어
batch-option-tag-suggestions = 태그 제안
batch-option-described = 설명된 비디오
batch-option-subtitled = 자막이 있는 비디오
batch-option-subtitles-write = 만들 파일
batch-option-spacing = 공백
batch-option-tag = 태그
batch-option-turn = 회전
batch-plan-videos = 비디오
batch-plan-length = 길이
batch-plan-cost = 비용
batch-plan-time = 시간
batch-reason-estimate = 예상 비용 계산 대기 중
batch-reason-subtitles-no-format = 만들 파일을 하나 이상 선택하세요
batch-set-key = 키 설정…
batch-check-key = 키 확인…
batch-subtitles-languages-auto = 비디오마다 감지
batch-action-describe-ai-run-waiting = 비디오 설명
batch-action-fix-tags-order = 순서: 태그 목록과 같게, 알 수 없는 태그가 먼저.
batch-action-markers-comment-hint-short = 시간이 있는 코멘트 줄을 마커로 바꾸거나, 마커를 코멘트로 복사합니다.
batch-action-markers-to-comment-hint = 마커는 그대로 남으며, 다시 실행해도 중복되지 않습니다.
batch-action-rotate-hint-short = MP4와 MOV 클립을 재인코딩 없이 회전 플래그로 돌립니다.
batch-action-tag-commented-off = 코멘트가 있는 비디오 태그 달기가 꺼져 있습니다
batch-action-tag-commented-choose = 태그 선택…

batch-title = 일괄 작업
batch-done-label-changed = 변경됨
batch-done-label-subtitled = 자막 생성됨
batch-ai-at-least = 최소
batch-cancel = 취소
batch-stopping = 중지하는 중…
batch-stop-over-budget = 중지됨: 다음 클립을 처리하면 상한 { $cap }을(를) 넘을 수 있습니다({ $spent } 사용). 나머지를 처리하려면 상한을 올리세요.
batch-stopped = { $total ->
   *[other] { $total }개 파일
} 중 { $finished }개 처리 후 중지됨.
batch-close = 닫기
batch-failed-subtitles = 자막을 만들지 못함:
batch-written-subtitles = 만든 파일:
batch-table-written = 생성됨
batch-merged-markers = 병합된 중복 마커:
batch-table-merged = 병합됨
batch-markers-merged = { $count ->
   *[other] 중복 마커 { $count }개 병합됨
}
batch-add-credit = 크레딧 충전
batch-open-log = 로그 열기

batch-action-move-comments = 코멘트 이동
batch-action-move-comments-hint = 선택한 각 파일의 코멘트를 인/아웃 포인트와 함께 선택한 위치로 옮깁니다. 비디오에 저장된 태그와 인/아웃 포인트는 그대로 남습니다.
batch-action-move-comments-into-videos = 텍스트 파일에서 비디오로 (XMP)
batch-action-move-comments-into-text-files = 비디오(XMP)에서 텍스트 파일로

batch-action-move-in-out = 인/아웃 포인트: 코멘트 ⇄ 비디오 (XMP)
batch-action-move-in-out-hint = 선택한 각 파일의 인/아웃 포인트를 선택한 위치로 옮깁니다. 나머지 코멘트는 그대로 남습니다.
batch-action-move-in-out-into-videos = 코멘트에서 비디오로 (Adobe XMP 마커)
batch-action-move-in-out-into-comments = 비디오(XMP 마커)에서 코멘트로

batch-action-markers-comment = 마커 ⇄ 코멘트
batch-action-markers-comment-to-markers = 시간이 있는 코멘트 줄을 마커로
batch-action-markers-to-comment = 마커를 코멘트로 (복사이며 마커는 그대로 남음)
batch-action-markers-comment-hint = "03:24 — Take 3 — nice light" 같은 줄은 3:24 위치에 이름이 "Take 3", 코멘트가 "nice light"인 마커가 되고, "0:41-0:47 — Lion"은 0:41부터 0:47까지의 마커가 됩니다. 이름과 코멘트는 처음 나오는 " — " 또는 " -- "에서 나누며, 일반 " - "에서는 나누지 않습니다. AI 설명의 순간("0:00–0:14 Street.")은 흰색 마커가 되고 설명에도 남습니다. 어느 방향이든 다시 실행해도 중복되지 않습니다.

batch-action-tag-commented = 코멘트가 있는 비디오에 태그 달기
batch-action-tag-commented-hint = 직접 쓴 코멘트가 있는 선택한 각 비디오에 "{ $tag }" 태그를 달고(AI 설명은 해당 없음), 코멘트가 없는 비디오에서는 태그를 제거합니다. 태그가 바뀐 파일은 이름이 변경됩니다.
batch-action-tag-commented-hint-off = 직접 쓴 코멘트가 있는 선택한 각 비디오에 코멘트 표시용 태그를 달고, 코멘트가 없는 비디오에서는 제거합니다. 이 태그는 설정에서 꺼져 있습니다.
batch-action-fix-tags = 우선순위대로 태그 정리
batch-action-fix-tags-hint = 선택한 각 파일 이름의 태그를 태그 패널의 순서대로 정렬합니다. 패널에서 위에 있는 태그일수록 이름에서 앞에 옵니다. 폴더에 아직 없는 태그는 태그 패널과 마찬가지로 맨 앞에 옵니다. 순서가 바뀐 파일은 이름이 변경됩니다.

batch-action-respace-tags = 태그 공백 적용
batch-action-respace-tags-hint-space = 설정에 따라 선택한 각 파일의 이름을 바꿔 태그마다 뒤에 공백을 넣습니다: Food. Goat. clip.mp4.
batch-action-respace-tags-hint-no-space = 설정에 따라 선택한 각 파일의 이름을 바꿔 태그 뒤 공백을 없앱니다: Food.Goat.clip.mp4.
batch-action-respace-tags-status-space = 공백: 각 태그 뒤에 공백
batch-action-respace-tags-status-no-space = 공백: 태그 뒤에 공백 없음
batch-action-export-markers = 마커를 텍스트로 내보내기
batch-action-export-markers-hint = 선택한 각 파일의 마커를 옆에 텍스트 파일(clip.markers.txt)로 저장합니다. 마커마다 한 줄이며 시간순이고, 예: "0:41–0:47 [red] — Lion crossing — walks left to right". 초록색이 아닌 색은 대괄호 안에 쓰고, 이름 뒤에 코멘트가 옵니다. "마커 ⇄ 코멘트"는 이런 줄을 다시 마커로 읽습니다. 마커가 없는 파일에는 텍스트 파일을 만들지 않으며, 이전 실행의 텍스트 파일은 덮어씁니다.
batch-export-markers-cannot-hold = 이 파일에는 마커를 저장할 수 없습니다
batch-action-reload-files = 캐시 초기화 후 다시 불러오기
batch-action-reload-files-hint = 선택한 각 파일의 코멘트와 인/아웃 포인트를 파일에서 다시 읽어, 폴더에 기억된 값을 대체합니다. 다른 프로그램에서 파일을 변경한 뒤에 사용하세요. 기억된 값이 없었거나 오래된 파일은 변경된 것으로 집계됩니다.

batch-action-describe-ai = AI로 설명
batch-action-describe-ai-run = { $videos } 설명 · 약 { $dollars }
batch-action-describe-ai-estimating = 계산 중… { $known } / { $total }
batch-action-describe-ai-none = 설명할 비디오가 없습니다.
batch-action-describe-ai-hint = { batch-runs-in-background }. 취소해도 이미 설명된 비디오는 유지되며, 다시 실행하면 건너뜁니다.
batch-action-describe-ai-no-subtitles = 자막 없음(화면만 설명): { $videos }.
batch-action-describe-ai-redo = 이미 AI 설명이 있는 비디오 다시 설명
batch-action-describe-ai-hint-panel = 선택한 각 비디오에서 무슨 일이 언제 일어나는지 설명합니다. 요약과 시간 구간별 내용이 코멘트의 AI 설명에 들어가며, 직접 쓴 텍스트는 유지됩니다. 프레임과 자막은 선택한 모델의 회사인 { $provider }(으)로 전송됩니다.
batch-ai-change = 변경
batch-ai-tag-suggestions-off = 끔
batch-ai-tag-suggestions-no-tags = 켜짐, 하지만 이 폴더에는 아직 태그가 없습니다
batch-ai-tag-suggestions-on = { $tags ->
   *[other] 폴더의 태그 { $tags }개에서
}
batch-ai-key-missing = 설정에서 { $provider } API 키를 설정하세요
batch-ai-key-unavailable = 시스템 키링을 열 수 없습니다. 잠겨 있거나 키링이 없을 수 있습니다(GNOME Keyring, KWallet 등).
## AI 설명 언어 이름: 설정의 선택 목록과 위의 { $language }.

ai-language-same-as-subtitles = 자막과 같게
ai-language-english = 영어
ai-language-russian = 러시아어
ai-language-ukrainian = 우크라이나어
ai-language-german = 독일어
ai-language-spanish = 스페인어
ai-language-french = 프랑스어
batch-videos-count = { $n ->
   *[other] 비디오 { $n }개
}
batch-minutes = { $n }분
batch-ai-minutes-under = 1분 미만
batch-ai-duration-under-minute = 1분 미만
batch-ai-duration-hours = { $h }시간
batch-ai-duration-hours-minutes = { $h }시간 { $m }분
batch-ai-dollars-under = $0.01 미만
batch-ai-skip-described = 이미 설명됨
batch-ai-skip-too-long = 30분 초과
batch-ai-skip-unreadable = 읽을 수 없음
batch-ai-skipped = 건너뜀: { $parts }.
batch-ai-progress-frame = 프레임 { $done } / { $total }
batch-ai-progress-waiting = { $provider } 응답 대기 중
batch-ai-progress-saving = 저장 중
batch-ai-progress-retry-rate-limit = { $seconds }초 후 재시도 (요청 한도)
batch-ai-progress-retry-temporary = { $seconds }초 후 재시도 (일시적 오류)
batch-ai-stop-no-key = 중지됨: { $provider } API 키가 없습니다. 설정에서 지정하세요.
batch-ai-fail-no-key = API 키 없음
batch-ai-fail-unreadable = 비디오를 읽을 수 없습니다
batch-ai-fail-too-long = 클립이 AI에 비해 너무 깁니다 (30분 초과)
batch-ai-fail-not-saved = 설명을 저장하지 못했습니다
batch-ai-stop-offline = 중지됨: { $provider }에 연결할 수 없습니다. 나머지를 설명하려면 다시 실행하세요.

batch-action-generate-subtitles = 자막 생성
batch-action-generate-subtitles-install-ffmpeg = .mkv, .m2ts, .avi 등을 읽으려면 ffmpeg.org에서 ffmpeg를 설치하고 PATH에 추가한 뒤 frename을 다시 시작하세요
batch-action-generate-subtitles-replace = 기존 자막 교체
batch-action-generate-subtitles-replace-hint = 다시 텍스트로 변환하며, 표시된 비용이 듭니다.
batch-action-generate-subtitles-srt = SRT 자막 (clip.srt)
batch-action-generate-subtitles-premiere = Premiere Pro 스크립트 (clip.premiere.json)
batch-action-generate-subtitles-premiere-hint = Premiere Pro에서: 텍스트 패널 → 스크립트 → 정적 스크립트 가져오기.
batch-action-generate-subtitles-privacy = 이 비디오들의 오디오는 Soniox로 전송되고, 이후 그곳에서 삭제됩니다.
batch-action-generate-subtitles-duration-hint = 오디오 1시간당 몇 분이 걸립니다. { batch-runs-in-background }. frename을 닫으면 작업이 중지되며, 완료된 자막은 유지됩니다.
batch-action-generate-subtitles-hint = 선택한 각 비디오의 음성을 Soniox로 텍스트 변환하여 옆에 저장합니다. frename이 표시하는 자막(clip.srt)과 Premiere Pro 스크립트 중 하나 또는 둘 다 만듭니다.
batch-subtitles-transcribe = 텍스트 변환
batch-subtitles-transcribe-count = { $videos } 텍스트 변환
batch-subtitles-build-free = { $count }개 만들기 (무료)
batch-subtitles-nothing = 텍스트로 변환할 것이 없습니다
batch-subtitles-estimating = 계산 중…
batch-subtitles-key-missing = 설정에서 Soniox API 키를 지정하세요
batch-subtitles-key-rejected = Soniox가 키를 거부했습니다
batch-subtitles-cost-unknown = 비용 알 수 없음
batch-subtitles-typical-price = { $usd } (일반적인 가격)
batch-subtitles-unknown-length = + 길이를 알 수 없는 { $n }개
batch-subtitles-plan-line = { $videos }를 텍스트로 변환, 오디오 { $duration }{ $unknown } · { $cost }
batch-subtitles-already = { $n ->
   *[other] { $n }개는 이미 자막이 있음
}
batch-subtitles-rebuilt-free = { $n }개는 저장된 변환 결과로 무료 재생성
batch-subtitles-no-speech-before = { $n }개는 지난번에 음성이 없었음
batch-subtitles-no-audio = { $n ->
   *[other] { $n }개는 오디오가 없음
}
batch-subtitles-shared-name = { $n ->
   *[other] { $n }개는 다른 비디오와 자막 이름이 같음
}
batch-subtitles-unreadable = { $n }개는 읽을 수 없음 ({ $how_to })
batch-subtitles-shared-name-reason = 다른 비디오와 자막 이름이 같음
batch-subtitles-unsupported-reason = 지원하지 않는 오디오 형식
batch-subtitles-progress-transcribing = 텍스트 변환 중…
batch-subtitles-progress-extracting = 오디오 추출 중 { $percent } %
batch-subtitles-progress-extracting-unknown = 오디오 추출 중
batch-subtitles-progress-uploading = 업로드 중 { $percent } %
batch-subtitles-progress-uploading-unknown = 업로드 중
batch-subtitles-progress-transcribing-left = 텍스트 변환 중… 약 { $time } 남음
batch-subtitles-progress-saving = 저장 중
batch-subtitles-reason-already = 이미 자막이 있음
batch-subtitles-reason-no-audio = 오디오 없음
batch-subtitles-reason-no-speech = 음성 없음
batch-subtitles-reason-soniox = Soniox: { $message }
batch-subtitles-reason-unreachable = Soniox에 연결할 수 없음
batch-subtitles-stop-key-rejected = 중지됨: Soniox가 키를 거부했습니다. 설정 → 자막에서 확인하세요.
batch-subtitles-stop-balance-empty = 중지됨: Soniox 잔액이 없습니다. console.soniox.com에서 충전하세요.
batch-subtitles-stop-budget-used = 중지됨: Soniox 월 예산을 모두 사용했습니다. console.soniox.com에서 늘리세요.
batch-subtitles-stop-other = 중지됨: Soniox가 계속하기를 거부했습니다 ({ $error_type }).
batch-subtitles-stop-unreachable = 중지됨: Soniox에 연결할 수 없습니다. 인터넷 연결을 확인하세요.
batch-subtitles-report-spend = Soniox: 최소 { $duration } · { $usd }
batch-subtitles-report-failed-deletes = Soniox에서 { $n ->
   *[other] 업로드 { $n }개
}를 삭제하지 못했습니다 (로그 참조)
batch-subtitles-languages-need-key = Soniox 키를 저장하면 모든 언어에서 고를 수 있습니다.
batch-subtitles-languages-rejected = Soniox가 키를 거부했습니다.
batch-subtitles-languages-failed = Soniox에서 언어를 가져오지 못했습니다.
batch-subtitles-count = { $n ->
   *[other] 자막 { $n }개
}
batch-subtitles-duration-seconds = { $s }초
batch-subtitles-usd-under = $0.01 미만
batch-subtitles-usd-about = 약 ${ $amount }

## 파일 목록
folder-has-subtitles = 자막 있음
folder-closed-by-job = 일괄 작업이 변경하는 동안 닫혀 있습니다
folder-filter-tip = 필터…

folder-search-placeholder = 파일 또는 코멘트 찾기
folder-search-comments-loading = 코멘트 검색 중… { $n }개 남음
folder-search-clear = 지우기
folder-all = 전체
folder-invert = 반전
folder-checked = { $count }개 선택됨
folder-outcome-changed = 변경됨
folder-outcome-unchanged = 바꿀 것 없음
folder-outcome-failed = 실패, 로그 참조
folder-rename-error-empty = 이름이 비어 있습니다
folder-rename-error-bad-character = 사용할 수 없음: \ / : * ? " < > |
folder-rename-error-trailing = 마침표나 공백으로 끝날 수 없습니다
folder-rename-error-exists = 같은 이름의 파일이 있습니다
folder-markers-not-saved = 마커를 저장하지 못했습니다: 파일이 읽기 전용이거나 사용 중입니다 (Premiere에서 닫은 뒤 파일을 열었다가 나가세요)
drag-out-not-saved = 끌어내지 못했습니다: 파일을 저장할 수 없습니다 (읽기 전용이거나 Premiere에서 열려 있음)
folder-opening = 폴더 여는 중…
folder-empty-title = 이 폴더에 비디오가 없습니다
folder-empty-line = frename은 MP4, MOV, MKV 등의 비디오 파일을 표시합니다.
folder-open-another = 다른 폴더 열기…
folder-no-match = 일치하는 파일이 없습니다
folder-show-all = 모두 보기
folder-locked = { $action } 실행 중에는 선택이 잠깁니다
folder-checked-hidden = { $count }개 선택됨 · { $hidden }개 숨김
folder-checked-hidden-tip = { $hidden ->
   *[other] 선택한 파일 { $hidden }개가 검색이나 필터로 숨겨져 있으며, 작업은 이 파일에도 실행됩니다
}
folder-outcome-working = 작업 중
folder-outcome-not-reached = 미처리
folder-window-empty-title = 클립 폴더 열기
folder-window-empty-line = 또는 폴더를 창에 놓으세요. 버튼을 오른쪽 클릭하면 파일 하나를 엽니다.
folder-window-open = 폴더 열기…

## 최근 폴더 (열기 버튼 옆 화살표와 빈 화면)

recent-folders-tip = 최근 폴더
recent-folders-title = 최근 폴더
recent-folders-none = 아직 연 폴더가 없습니다
recent-folders-not-found = 찾을 수 없음
recent-folders-remove-tip = 목록에서 제거
recent-folders-clear = 목록 지우기
recent-folders-remove = 제거
recent-folders-keep = 유지
recent-folders-missing-line = 찾을 수 없습니다. 제거할까요?
recent-folders-just-now = 방금
recent-folders-minutes = { $count }분 전
recent-folders-hours = { $count }시간 전
recent-folders-days = { $count ->
   *[other] { $count }일 전
}
recent-folders-weeks = { $count ->
   *[other] { $count }주 전
}
recent-folders-months = { $count ->
   *[other] { $count }개월 전
}
recent-folders-years = { $count ->
   *[other] { $count }년 전
}

## 파일 메뉴 (파일에서 오른쪽 클릭, 또는 F11 / Shift+F11 / Ctrl+F11)

file-menu-show-in-explorer = 탐색기에서 보기
file-menu-show-in-file-manager = 파일 관리자에서 보기
file-menu-copy-path = 전체 경로 복사
file-menu-copy-name = 파일 이름 복사
file-menu-copied = 복사됨
file-menu-not-copied = 복사하지 못했습니다: 클립보드를 사용할 수 없습니다
file-menu-not-shown = 파일 관리자를 열 수 없습니다

## 파일 목록 아래의 컨트롤 바

folder-controls-filter-untagged = 태그 없음
folder-controls-filter-subtitles = 자막
folder-controls-filter-comments = 코멘트
folder-controls-filter-markers = 마커
folder-controls-scroll = 열린 파일을 목록에서 보기
folder-controls-open = 폴더 열기
folder-controls-batch = 선택한 파일에 일괄 작업
folder-controls-batch-back = 열린 파일로 돌아가기
folder-controls-update-available = 업데이트 가능: { $version }
folder-controls-whats-new = { $version }의 새로운 기능
folder-controls-previous = 이전 파일
folder-controls-next = 다음 파일
folder-controls-more = 더 보기
folder-controls-open-file = 오른쪽 클릭: 파일 하나 열기

## 비디오

video-controls-back = 10초 뒤로
video-controls-play = 재생
video-controls-pause = 일시 정지
video-controls-forward = 10초 앞으로
video-controls-frame-back = 한 프레임 뒤로
video-controls-frame-forward = 한 프레임 앞으로
video-controls-set-in = 인 포인트 지정
video-controls-set-out = 아웃 포인트 지정
video-controls-screenshot = 이 프레임 저장
video-controls-add-marker = 마커 추가
video-controls-add-marker-hold = 길게 누르면 구간, 다시 누르면 이름 입력
video-controls-cannot-hold-markers = 이 파일에는 마커를 저장할 수 없습니다
video-controls-add-a-name = 이름 추가
video-controls-rotate-left = 왼쪽으로 회전
video-controls-rotate-right = 오른쪽으로 회전
video-controls-more = 더 보기
media-viewer-loading-slow = 파일을 기다리는 중… (클라우드 파일은 시간이 걸릴 수 있습니다)
media-viewer-cannot-play = 이 클립은 재생할 수 없습니다
media-viewer-no-picture = 이 파일에는 비디오 화면이 없습니다
video-timecode-copied = 타임코드 복사됨: { $time }
video-timecode-unreadable = “{ $text }”은(는) 이 클립의 시간이 아닙니다
video-controls-volume-scroll = 볼륨 — 스크롤하여 조절
rotate-cannot = 회전할 수 없습니다: { $reason }
rotate-reason-missing = 파일이 더 이상 없습니다
rotate-flag-right = 오른쪽 90°
rotate-flag-left = 왼쪽 90°
rotate-flag-half = 180°
rotate-flag-none = 없음
rotate-now = 회전: { $flag }
rotate-turned = { $turn } 회전함 · 현재 회전 { $flag }
rotate-failed = 회전하지 못했습니다: { $reason }
undo-name-taken = 변경하지 않음: 다른 파일의 이름이 이미 { $name }입니다
undo-back-on-clip = 이 클립으로 돌아왔습니다 (이동을 취소함). Ctrl+Z를 다시 누르면 마지막 변경을 취소합니다.
redo-on-clip = 다시 이 클립으로 왔습니다 (이동을 다시 실행함).
rotate-reason-in-use = 파일이 읽기 전용이거나 사용 중입니다
rotate-reason-format = 이 형식에는 회전 플래그가 없습니다
rotate-reason-damaged = 파일이 손상되었습니다
rotate-reason-no-video = 파일에 비디오 트랙이 없습니다
rotate-reason-matrix = 비디오의 디스플레이 매트릭스가 특이합니다
batch-action-rotate = 비디오 회전
batch-action-rotate-hint = 선택한 각 MP4와 MOV 파일의 회전 플래그를 변경합니다. 화면은 재인코딩하지 않으며, 코멘트, 인/아웃 포인트, 마커는 유지됩니다. Premiere Pro가 가져올 때 클립을 회전된 상태로 표시할 것으로 예상됩니다. 다른 형식에는 회전 플래그가 없어 실패합니다. 다시 실행하면 파일이 다시 회전하며, “초기화”는 휴대폰이 기록한 회전을 포함해 모든 회전을 제거하므로 휴대폰의 세로 클립은 옆으로 누워 재생됩니다.
batch-action-rotate-right = 오른쪽 90° (시계 방향)
batch-action-rotate-left = 왼쪽 90° (시계 반대 방향)
batch-action-rotate-half = 180°
batch-action-rotate-reset = 초기화: 회전 없음 (0°)
media-viewer-video-subtitle-list = 자막 목록
media-viewer-video-marker-list = 마커 목록
media-viewer-video-markers-hint = Shift+F1 / Shift+F3으로 마커 사이 이동, Shift+드래그로 맞춤
media-viewer-video-tab-markers = 마커
media-viewer-video-fullscreen = 전체 화면
media-viewer-video-close-list = 목록 닫기
media-viewer-video-subtitle-file-unreadable = 이 파일을 읽을 수 없습니다
# Shown over the picture when a clip opens where playback stopped last time; Home or a click goes to the start.
media-viewer-video-resumed = { $time }부터 이어서 재생 · Home: 처음부터

## 마커 목록
markers-in-out = 인/아웃 포인트

markers-empty = 아직 마커가 없습니다
markers-add = 마커 추가
markers-ai-hint = AI 마커: AI가 이 클립을 다시 설명하면 교체됩니다
markers-keep-color = 색상 유지
markers-done = 완료
markers-delete = 마커 삭제
markers-cannot-hold-hint = Premiere는 MP4와 MOV 파일에서 마커를 읽습니다.
markers-color-green = 초록
markers-color-red = 빨강
markers-color-orange = 주황
markers-color-yellow = 노랑
markers-color-white = 흰색
markers-color-blue = 파랑
markers-color-cyan = 청록
markers-color-lavender = 연보라
markers-color-magenta = 자홍
markers-color-other = 다른 색
markers-read-only = 읽기 전용
markers-name-placeholder = 이름
markers-ai-describe = AI로 설명: 마커 이름을 짓고 일어나는 일을 추가
markers-ai-stop = 설명 중지
markers-ai-describe-unnamed = 이름 없는 { $count }개 설명
markers-ai-describe-unnamed-hint = 이름이 없는 모든 마커에 AI로 이름을 짓고 설명을 추가합니다. 몇 개씩 나눠서 처리하며, 요청은 마커마다 한 번입니다. 그동안 직접 입력한 이름은 유지됩니다.
markers-ai-price-about = 약 { $dollars }
markers-ai-describe-unnamed-cost = 요청은 마커마다 한 번: 마커당 { $each }, 전체 { $total }.
markers-ai-confirm = { $count ->
   *[other] 마커 { $count }개를 설명할까요? 요청은 마커마다 한 번이며 전체 { $total }입니다.
}
markers-ai-confirm-unpriced = { $count ->
   *[other] 마커 { $count }개를 설명할까요? 유료 요청이 마커마다 한 번 발생합니다.
}
markers-ai-confirm-yes = 설명
markers-ai-confirm-no = 취소
markers-ai-waiting = 순서를 기다리는 중…
markers-ai-run = 설명 중 { $describing } · 대기 중 { $waiting }
markers-ai-run-last = 설명 중 { $describing }
markers-ai-stop-all = 모두 중지
markers-ai-describing = 설명하는 중…
markers-ai-done = 마커를 설명했습니다
markers-ai-nothing-new = 마커에 이미 같은 설명이 있습니다
markers-no-marker-notice = 여기에는 마커가 없습니다
markers-ai-no-key = { $provider } API 키가 없습니다. 설정에서 지정하세요
markers-ai-failed = 설명하지 못했습니다: { $reason }
markers-ai-failed-unknown = 문제가 발생했습니다
markers-read-only-notice = 이 마커는 읽기 전용입니다
markers-ai-gone = 마커가 없어져 설명을 추가하지 않았습니다
markers-ai-stopping-for-batch = 먼저 마커 설명을 중지하는 중… 보통 몇 초 걸립니다.
folder-not-saved-marker-ai = 저장하지 못했습니다: 마커 설명이 아직 클립을 읽고 있었습니다. 잠시 후 다시 시도하세요.

## 파일 작업 영역

file-workspace-search-placeholder = 태그 찾기 — 또는 바로 입력
file-workspace-search-clear = 지우기
file-workspace-comment-placeholder = 코멘트...
file-workspace-comment-collapse = 태그로 돌아가기
file-workspace-comment-expand = 코멘트 펼치기

## 태그 격자

tag-grid-star = 즐겨찾기: 맨 위에 유지
tag-grid-unstar = 즐겨찾기 해제
tag-grid-save = 폴더 태그에 추가
tag-grid-delete = 폴더 태그에서 “{ $tag }” 삭제
tag-grid-create = “{ $tag }” 만들기
tag-grid-no-file = 클립을 열어 태그를 다세요
tag-grid-no-tags = 아직 태그가 없습니다
tag-grid-no-tags-hint = 이름을 입력하고 Enter를 누르면 첫 태그가 만들어집니다.
tag-grid-group-unsaved = 폴더 태그에 없음
tag-grid-group-folder = 폴더 태그
tag-grid-more = +{ $count }개 더

## 순서 막대

sync-panel-locked = 아래에서 순서를 바꾸면 폴더 전체의 순서가 바뀝니다
sync-panel-unlocked = 아래에서 순서를 바꾸면 이 클립만 바뀝니다
sync-panel-unlock = 잠금 해제
sync-panel-lock = 잠금
sync-panel-differs = 폴더와 순서가 다릅니다
sync-panel-use-for-folder = 폴더에 적용
sync-panel-sort-like-folder = 폴더와 같게 정렬

## 파일 이름 카드

file-name-panel-no-tags = 이 클립에는 태그가 없습니다
file-name-panel-untag = 태그 제거
file-name-panel-clear-in = 인 포인트 지우기
file-name-panel-clear-out = 아웃 포인트 지우기
file-name-panel-suggested-in-out = AI
file-name-panel-apply-suggested-in-out = 인과 아웃을 AI가 제안한 값으로 지정 (초 단위로 반올림)
file-name-panel-suggested-tags = AI 태그
file-name-panel-add-suggested-tag = 태그 "{ $tag }" 추가 ({ $percent }% 확신)
file-name-panel-add-suggested-tag-unsure = 태그 "{ $tag }" 추가
file-name-panel-add-all-suggested-tags = 모두 추가
file-name-panel-add-all-suggested-tags-tip = AI가 제안한 태그를 모두 추가
file-name-panel-tag-ideas = 아이디어: { $ideas }
file-name-panel-tag-ideas-tip = AI가 발견했지만 이 폴더에 없는 태그입니다. 추가되지는 않습니다.
file-name-panel-saved = 모든 변경 사항 저장됨
file-name-panel-saved-to-recovery = 복구용으로 저장됨, 클립을 나갈 때 적용됩니다

## 업데이트 (설정)

updates-check = 업데이트 확인
updates-not-installed = 업데이트는 설치된 버전에서 작동합니다
updates-from-store = 업데이트는 Microsoft Store에서 받습니다
updates-from-releases = 새 버전은 릴리스 페이지에서 다운로드하세요
updates-open-releases = 릴리스 페이지 열기
updates-checking = 확인하는 중…
updates-downloading-named = { $version } 다운로드 중… { $percent }%
updates-downloading = 다운로드 중… { $percent }%
updates-restarting = 다시 시작하는 중…
updates-check-failed = 업데이트를 확인할 수 없습니다: { $reason }
updates-update-failed = 업데이트할 수 없습니다: { $reason }
updates-close-other-window = 먼저 다른 frename 창을 닫으세요
updates-reason-github = GitHub가 응답하지 않습니다. 나중에 다시 시도하세요
updates-reason-no-connection = 연결 없음
updates-reason-read-only = 폴더가 읽기 전용입니다
updates-reason-not-installed = 업데이트는 설치된 버전에서 작동합니다
updates-reason-download-stopped = 다운로드가 예기치 않게 중단되었습니다
updates-version-available = { $version } 버전을 사용할 수 있습니다
updates-up-to-date = frename이 최신 버전입니다
updates-update-and-restart = 업데이트 후 다시 시작
updates-wait-for-batch = 일괄 작업이 끝날 때까지 기다리세요
updates-current-version = frename { $version }
updates-check-on-start = frename을 시작할 때 업데이트 확인
updates-notes-show = 릴리스 노트
updates-notes-hide = 릴리스 노트 숨기기
updates-notes-all-releases = GitHub의 모든 릴리스
updates-notes-history = 모든 버전, 최신순
updates-notes-update = { $version }에서 달라지는 점
updates-notes-whats-new = { $version }의 새로운 기능
updates-notes-loading = 릴리스 노트를 불러오는 중…
updates-notes-unavailable = 최신 버전의 노트를 불러올 수 없습니다.
updates-notes-more = 더 보기

recovery-restored = frename이 예기치 않게 종료되었습니다. { $clip }의 저장하지 않은 작업({ $what })을 복원했습니다.
recovery-kept = frename이 예기치 않게 종료되었습니다. { $clip }의 저장하지 않은 작업({ $what })을 적용하지 못했습니다: { $why }. 다시 입력할 수 있는 읽기용 사본이 { $folder }에 있습니다.
recovery-unreadable = frename이 예기치 않게 종료되었고 복구 파일을 읽을 수 없습니다. 파일은 { $folder }에 보관되어 있습니다.
recovery-why-gone = 클립이 없어졌거나 이름이 바뀌었습니다
recovery-why-changed = 편집한 뒤 클립이 변경되었습니다
recovery-why-not-written = 같은 이름의 파일이 있거나, 클립이 읽기 전용이거나 사용 중입니다
recovery-tags = { $count ->
   *[other] 태그 { $count }개
}
recovery-markers = { $count ->
   *[other] 마커 { $count }개
}
recovery-comment = 코멘트
recovery-in-out = 인/아웃 포인트
recovery-edits = 편집 내용
folder-frame-saved = 프레임 저장됨
folder-not-saved = 저장하지 못했습니다: 같은 이름의 파일이 이미 있거나, 읽기 전용이거나 사용 중입니다
markers-deleted-notice = 마커를 삭제했습니다
markers-not-saved-notice = 마커를 저장하지 못했습니다: 파일이 읽기 전용이거나 사용 중입니다
markers-comment-not-saved-notice = 마커를 저장하지 못했습니다: 코멘트 파일이 읽기 전용이거나 사용 중입니다
markers-copy = 복사
markers-copy-tip = 마커를 한 줄에 하나씩 텍스트로 복사
markers-export = 내보내기…
markers-export-tip = 마커를 텍스트 파일로 저장
markers-exported = 마커를 { $name }에 저장했습니다
markers-not-exported = 마커를 저장하지 못했습니다: 파일에 쓸 수 없습니다

batch-option-notes = AI용 메모
batch-ai-notes-info = 이 폴더의 영상에 대한 직접 쓴 메모로, 여기서 보내는 모든 AI 요청(설명, 태그 제안, 마커 설명)과 함께 전송됩니다. 폴더의 .frename 파일에 저장됩니다.
batch-ai-notes-placeholder = 예: 가장 자주 나오는 남자는 Eugene입니다. 광고 간판을 표시하세요. 차가 보이면 tea 태그를 제안하세요.
batch-ai-notes-hint = 프레임과 함께 { $provider }(으)로 전송됩니다. 이름은 이 메모에서만 가져오며, frename은 얼굴을 인식하지 않습니다. 이미 설명된 비디오는 건너뜁니다. 새 메모로 다시 설명하려면 “다시 설명”을 선택하세요.
batch-ai-notes-too-long = 메모가 { $chars }자이며 한도는 { $max }자입니다. 실행하려면 줄이세요.
batch-ai-notes-too-long-reason = AI용 메모가 너무 깁니다
batch-option-notes-once = 이번 실행에만 쓰는 추가 줄
batch-ai-notes-once-placeholder = 예: 이 클립: 빨간 옷을 입은 여자는 Ana입니다
file-workspace-ai-notes-on = 메모 켜짐
file-workspace-ai-notes-tip = AI용 메모는 이 폴더의 모든 AI 요청과 함께 전송됩니다. AI로 설명(일괄 작업)에서 편집하세요: { $notes }
batch-ai-notes-no-subtitles = 메모가 대사에 관한 내용인 것 같은데 { $videos }에 자막이 없습니다. AI가 대사를 읽을 수 있도록 먼저 자막을 생성하세요.
