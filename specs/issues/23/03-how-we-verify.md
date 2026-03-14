# Issue 23: Как проверяем

## Acceptance Criteria

- `init` больше не содержит special-case для `__SESSION_NAME__`
- `templates/init/settings.yml` содержит `zellij.session_name: "${REPO}"`
- `zellij.session_name` рендерится в рантайме через общий template path
- для `zellij.session_name` поддерживается только `${REPO}`
- literal значения `zellij.session_name` продолжают работать без миграции
- при неразрешенных placeholder в `zellij.session_name` команда завершается
  понятной ошибкой конфигурации
- README, SSOT, feature docs и ADR-слой синхронизированы с новым контрактом

## Ready Criteria

- зафиксирован тип задачи как `small feature` для `infra/platform`
- согласовано, что `${ISSUE_NUMBER}` и `${BRANCH}` не входят в контракт
  `zellij.session_name`
- в analysis artifacts явно зафиксировано, что нужен новый ADR
- определен единый источник `${REPO}`: `RepoContext.github_repo`

## Invariants

- `zellij.session_name` остается stable semantic name уровня репозитория
- `zellij.session_name` не становится issue-specific runtime value
- `${REPO}` для `zellij.session_name` и `launch_agent.*` означает один и тот же
  canonical repo identifier
- literal значения `zellij.session_name` не требуют миграции
- недопустимые placeholders не должны тихо превращаться в literal session names

## Test Plan

Unit tests:

- bootstrap `settings.yml` больше не содержит `__SESSION_NAME__`
- `init` оставляет в generated config literal `${REPO}`, а не materialized имя
- runtime renderer превращает `${REPO}` в `RepoContext.github_repo`
- literal `zellij.session_name` проходит без изменений
- `${BRANCH}` и другие неподдерживаемые placeholders приводят к явной ошибке
- `launch_agent.*` сохраняет текущий рендеринг без регрессии

Integration tests:

- `ai-teamlead init` создает `settings.yml` с новым bootstrap default
- путь запуска, использующий `zellij.session_name`, работает с literal config
- тот же путь работает с template config `${REPO}`
- при ошибочном placeholder команда падает до запуска `zellij` с читаемым
  сообщением

Manual validation:

- проверить generated `.ai-teamlead/settings.yml` после `init`
- проверить, что новый репозиторий запускается с session name, равным значению
  `${REPO}` после рендера
- проверить, что существующий репозиторий с literal session name не требует
  ручной миграции
- проверить, что документация не содержит старый bootstrap token и хардкод
  `-ai-teamlead` как обязательный default

## Verification Checklist

- шаблон `settings.yml` обновлен
- код `init` очищен от `__SESSION_NAME__`
- runtime path для `zellij.session_name` использует canonical `${REPO}`
- ошибка на неподдерживаемый placeholder покрыта тестом
- literal backward compatibility покрыта тестом
- README и связанные docs обновлены
- новый ADR добавлен и связан с существующими документами

## Happy Path

1. Оператор запускает `ai-teamlead init` в новом репозитории.
2. В `settings.yml` появляется `zellij.session_name: "${REPO}"`.
3. При `poll` или `run` приложение рендерит `${REPO}` из repo context.
4. `zellij` session находится или создается по уже resolved имени.

## Edge Cases

- `zellij.session_name` уже задан literal строкой вроде `teamlead-ai-teamlead`
- `zellij.session_name` содержит несколько вхождений `${REPO}`
- repo name в local directory не совпадает с GitHub repo slug

## Failure Scenarios

- в `zellij.session_name` указан `${BRANCH}` или любой другой неподдерживаемый
  placeholder
- после рендера в строке остался `${...}` из-за опечатки или неполной замены
- документация обновлена частично и вводит владельца репозитория в заблуждение
