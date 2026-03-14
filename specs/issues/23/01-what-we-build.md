# Issue 23: Что строим

## Problem

Сейчас project-local config использует две разные модели подстановки:

- `zellij.session_name` заполняется в `init` через bootstrap token
  `__SESSION_NAME__`
- `launch_agent.*` рендерится в рантайме через `${...}` templates

Из-за этого:

- в коде появился special-case и хардкод `-ai-teamlead`
- `zellij.session_name` живет вне общего config contract
- `init` и runtime используют разные источники repo identity
- владельцу репозитория сложнее понимать, какие поля являются literal, а какие
  template-capable

## Who Is It For

- владелец репозитория, который настраивает `./.ai-teamlead/settings.yml`
- разработчик, который поддерживает config/runtime contract в `ai-teamlead`
- оператор, который ожидает предсказуемое имя `zellij` session для запуска
  issue-analysis flow

## Outcome

Нужен единый и более простой контракт, в котором:

- `templates/init/settings.yml` содержит `zellij.session_name: "${REPO}"`
- `init` больше не делает отдельный special-case рендеринг `__SESSION_NAME__`
- `zellij.session_name` рендерится в рантайме через тот же template path, что и
  другие launcher templates
- для `zellij.session_name` допустим только `${REPO}`
- literal значения `zellij.session_name` продолжают работать без миграции
- неразрешенные `${...}` в `zellij.session_name` считаются ошибкой
  конфигурации

## Scope

В текущую задачу входит:

- унификация versioned config contract для `zellij.session_name`
- выравнивание canonical repo identifier с `launch_agent.*`
- удаление bootstrap token `__SESSION_NAME__` и hardcoded suffix из `init`
- добавление явной ошибки при неразрешенных placeholder в
  `zellij.session_name`
- синхронизация README, feature docs, SSOT и ADR-слоя с новым контрактом

## Non-Goals

В текущую задачу не входит:

- поддержка `${ISSUE_NUMBER}`, `${BRANCH}` или других placeholders в
  `zellij.session_name`
- изменение семантики `zellij.tab_name`
- расширение набора placeholders для `launch_agent.*`
- автоматическая миграция уже существующих `settings.yml`
- redesign всей template-системы за пределами `zellij.session_name`

## Constraints And Assumptions

- `zellij.session_name` должен оставаться stable semantic name уровня
  репозитория, а не issue-specific runtime value
- canonical значение `${REPO}` должно браться из того же repo context, что уже
  используется для `launch_agent.*`, то есть из GitHub remote slug
- обратная совместимость для literal значений обязательна
- изменение config contract должно быть сначала зафиксировано в документации, а
  затем в коде

## User Story

Как владелец подключенного репозитория, я хочу видеть один и тот же template
contract для launcher-настроек, чтобы понимать конфиг без скрытых bootstrap
исключений и не зависеть от repo-specific хардкода в Rust-коде.

## Use Cases

1. Новый репозиторий запускает `ai-teamlead init` и получает
   `zellij.session_name: "${REPO}"` в `settings.yml`.
2. Существующий репозиторий хранит literal `zellij.session_name` и продолжает
   работать без изменений.
3. Репозиторий по ошибке указывает `zellij.session_name: "${BRANCH}"` и получает
   явную ошибку конфигурации вместо буквального имени session.
