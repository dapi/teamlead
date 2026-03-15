# Feature 0005: Что строим

## Проблема

Главная проблема не в отсутствии еще одного test runner-а как такового, а в
том, что проект пока не тестирует project-local prompts и agent flow реальным
агентом поверх реального LLM.

Текущие требования к integration tests покрывают в первую очередь launcher,
runtime и `zellij`-контракт, но не дают канонического способа проверить, что
настоящий агент корректно проходит versioned flow, используя те же host-level
настройки и параметры подключения к LLM, с которыми запускается сам test suite.

Нужна каноническая платформа, которая:

- запускается одной локальной командой
- создает полностью независимый sandbox для `ai-teamlead`
- прогоняет реальный launcher path, включая `zellij`
- умеет запускать настоящего агента (`codex` по умолчанию, `claude` как
  дополнительный профиль) с host-level настройками и параметрами подключения к
  LLM, включая account/session auth по подписке, а не только прямые API-вызовы
- не обращается к реальному GitHub во время тестового прогона, а использует
  контролируемый `gh` stub
- не затрагивает host `zellij` session пользователя

Без такой платформы проверка agent flow остается смесью ручных действий,
ад-хок shell-скриптов и частичных integration tests с test doubles.

## Пользователь

Основной пользователь:

- владелец репозитория, который меняет flow, launcher, prompt или runtime
  orchestration и хочет быстро проверить реальный сценарий локально

Вторичные пользователи:

- разработчик `ai-teamlead`, который хочет воспроизводимые integration tests в
  CI
- владелец подключенного репозитория, который хочет убедиться, что его
  project-local `.ai-teamlead/`, `.claude/` и `.codex/` работают в изоляции

## Результат

Полезным результатом считается платформа, в которой пользователь запускает один
entrypoint и получает:

- отдельный disposable sandbox
- воспроизводимый прогон одного или нескольких versioned сценариев
- реальный запуск `ai-teamlead`, `launch-agent.sh` и agent flow
- выбор режима `stub` или `live`
- `codex` как default live path для локального тестирования
- `claude` как дополнительный live-profile, в том числе для Claude Code с
  моделью класса Sonnet
- лог вызовов `gh` stub, позволяющий проверить обращения к GitHub-слою
- экспорт артефактов прогона: логи, sandbox metadata, runtime-state, итоговый
  статус и диагностические файлы

## Scope

В первую версию входят:

- единый CLI entrypoint для запуска integration scenarios локально
- Docker-based headless sandbox как основной и канонический runtime
- изолированный запуск `zellij` только внутри sandbox
- snapshot текущего репозитория в disposable workspace
- запуск реального `ai-teamlead` CLI внутри sandbox
- запуск `launch-agent.sh` и project-local flow prompt без bypass path
- два режима агента:
  - `stub` для детерминированных сценариев и CI
  - `live` для запуска реального `codex` по умолчанию и `claude` как
    дополнительного профиля
- repo-local описание сценариев и expected assertions
- явный bridge для host env vars, host config files и host account/session auth
  artifacts, нужных агенту для доступа к LLM; bridge использует те же
  host-level значения, с которыми запущен test suite
- собственный `gh` stub внутри sandbox с versioned fixtures и логированием всех
  вызовов
- export test artifacts наружу из sandbox

## Вне scope

В первую версию не входят:

- универсальный browser E2E для произвольных UI
- автоматический прогон live LLM tests на каждый коммит в CI
- скрытый доступ sandbox ко всему `$HOME` пользователя
- поддержка произвольных container runtime без отдельного контракта
- сравнение качества reasoning модели по содержанию generated plan
- полная эмуляция GitHub SaaS без выделенного fake/stub слоя

## Ограничения и предпосылки

- host `zellij` пользователя считается off-limits; любые `zellij`-related
  проверки выполняются только в headless sandbox
- project-local repo assets остаются источником истины для flow и launcher
- sandbox не должен менять рабочее дерево пользователя
- live-режим использует локальные настройки и credentials пользователя, но
  только через явный allowlist env vars, mounts и account/session artifacts
- под "локальными настройками" в этой feature имеются в виду именно host-level
  настройки, credentials, account/session auth и параметры подключения, с
  которыми запущен сам test suite
- live-режим должен уметь работать с `codex` / `claude`, если доступ к модели
  обеспечивается через подписочный account login агента, без обязательного
  отдельного API key, если такой auth path поддерживается самим агентом
- доступ sandbox к GitHub должен идти только через `gh` stub; прямой доступ к
  реальному GitHub API и GitHub credentials в тестовом прогоне запрещен
- `stub` и `live` должны использовать один и тот же orchestration path; нельзя
  делать отдельный shortcut только для тестов
- результат теста должен быть inspectable без повторного входа в sandbox

## Связанные документы

- [README.md](./README.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
