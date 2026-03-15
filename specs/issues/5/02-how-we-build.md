# Issue 5: Как строим

Статус: approved
Последнее обновление: 2026-03-14
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-14T19:14:28+03:00

## Approach

Implementation stage нужно проектировать как отдельный flow-слой, а не как
продолжение analysis prompt теми же сущностями.

Базовый подход:

- добавить отдельный системный SSOT `docs/issue-implementation-flow.md`;
- добавить отдельную feature-спеку для implementation stage;
- сохранить `run <issue>` как единый канонический issue-level entrypoint;
- расширить `run` до stage-aware dispatcher, который по текущему статусу issue
  определяет analysis flow или implementation flow;
- использовать approved analysis artifacts как входной контракт реализации;
- отделить implementation branch/worktree/session lifecycle от analysis
  lifecycle;
- переиспользовать уже принятый pattern versioned launcher + internal
  finalization command, но расширить его на implementation stage;
- завершать implementation stage через формализованные outcomes, а не через
  неявный “агент закодил что-то и остановился”.

Такой подход удерживает границы ответственности:

- analysis flow остается про спецификацию и план;
- implementation flow отвечает за код, тесты, commit/push/PR и переход в review;
- `run` отвечает за stage dispatch и выбор нужного orchestration path;
- runtime и launcher знают, какой stage они обслуживают;
- GitHub Project status остается source of truth по lifecycle issue.

## Affected Areas

- `docs/issue-analysis-flow.md`
  нужно явно зафиксировать, что `run` после `Ready for Implementation`
  маршрутизирует issue в отдельный implementation flow, а не ломается и не
  требует отдельной CLI-команды;
- новый `docs/issue-implementation-flow.md`
  должен стать SSOT для lifecycle, allowed statuses, outcomes и human gates
  implementation stage;
- новая feature-спека implementation stage
  должна покрыть продуктовую, архитектурную и verification-оси;
- `README.md`
  должен обновиться как repo-level summary, когда feature будет принята;
- `src/app.rs` или эквивалентный issue-level dispatcher
  должен научиться stage-aware маршрутизации внутри `run`;
- `./.ai-teamlead/settings.yml`
  потребуется расширить stage-specific naming и launcher templates для
  implementation workspace;
- `./.ai-teamlead/flows/`
  потребуется дополнить project-local entrypoint и staged prompts для
  implementation;
- `./.ai-teamlead/launch-agent.sh`
  нельзя неявно расширять до multi-stage script без нового контракта:
  нужен отдельный implementation launcher или stage-aware wrapper с явными
  границами;
- `.git/.ai-teamlead/`
  runtime state должен поддержать implementation session-binding без конфликта
  с текущим analysis invariant `issue <-> session_uuid`;
- CLI и GitHub adapter layer
  должны научиться claim/re-entry/finalization для implementation statuses,
  draft PR и CI/review quality gates.

## Interfaces And Data

### Входной контракт

Implementation flow принимает issue только если одновременно выполняются
следующие условия:

- GitHub issue state = `open`;
- issue находится в default GitHub Project;
- project status = `Ready for Implementation`;
- approved analysis artifacts доступны как канонический источник для
  `specs/issues/${ISSUE_NUMBER}/`.

Верхнеуровневый вход при этом остается единым:

- оператор вызывает `run <issue>`;
- `run` читает текущий project status;
- stage dispatcher выбирает analysis или implementation path;
- выбранный flow работает со своим собственным SSOT и stage-specific runtime.

Минимальный безопасный контракт входа:

- implementation flow работает не по произвольному тексту issue, а по
  versioned SDD-комплекту;
- analysis artifacts считаются immutable input для текущего coding stage;
- если approved SDD недоступен, implementation flow должен останавливаться с
  явным blocker, а не продолжать работу по догадкам.

### Статусная модель implementation stage

Минимальный lifecycle после `Ready for Implementation`:

- `Ready for Implementation`
  issue готова к запуску implementation stage;
- `Implementation In Progress`
  агент реализует код и проходит локальные проверки;
- `Waiting for CI`
  изменения запушены, draft PR создан, stage ждет результат обязательных CI
  checks;
- `Waiting for Code Review`
  обязательные quality gates пройдены, PR готов к human review;
- `Implementation Blocked`
  stage остановлен из-за технического или продуктового блокера.

Разрешенные переходы в рамках нового flow:

- `Ready for Implementation` -> `Implementation In Progress`
- `Implementation In Progress` -> `Waiting for CI`
- `Implementation In Progress` -> `Implementation Blocked`
- `Waiting for CI` -> `Waiting for Code Review`
- `Waiting for CI` -> `Implementation In Progress`
- `Waiting for Code Review` -> `Implementation In Progress`
- `Implementation Blocked` -> `Implementation In Progress`

Эта модель отделяет:

- coding work;
- асинхронное ожидание CI;
- human review gate;
- блокеры.

### Branch, worktree и PR contract

Для implementation stage нужен отдельный workspace contract.

Минимальный вариант:

- отдельная implementation branch, по умолчанию
  `implementation/issue-${ISSUE_NUMBER}`;
- отдельный implementation worktree root, configurable через `settings.yml`;
- base branch для implementation worktree — default branch репозитория;
- implementation PR создается из implementation branch в default branch;
- analysis branch не используется как coding branch и не становится скрытой
  базой для implementation PR.

Это решение намеренно разделяет:

- analysis artifacts как входной контракт;
- implementation branch как носитель кодовых изменений.

Иначе plan review и code review смешиваются в одну историю ветки и PR.

### Session-binding и runtime

Текущий analysis runtime фиксирует invariant:

- одна issue в анализе связана ровно с одним `session_uuid`.

Для implementation stage нужен отдельный stage-scoped runtime contract.

Минимальное требование:

- implementation session-binding не должен перезаписывать analysis binding;
- у issue может быть не более одного активного implementation session-binding;
- stage dispatcher должен явно понимать, какой binding искать: analysis или
  implementation;
- finalization implementation stage должна знать issue number, branch, PR и
  target project status;
- runtime layout должен различать stage хотя бы по директории хранения или по
  явному полю `stage`.

Практический путь для MVP:

- либо сделать отдельные stage-specific runtime директории;
- либо обобщить runtime schema до stage-aware модели.

Оба варианта требуют отдельного ADR, потому что затрагивают уже принятый
контракт session-binding.

### Finalization contract

Implementation stage должен использовать тот же инженерный принцип, что и
analysis:

- агент сообщает outcome одной CLI-командой;
- сама CLI-команда инкапсулирует commit, push, PR и status transition logic;
- prompt не содержит ручных `git commit`, `git push`, `gh pr create`,
  `gh pr checks` как обязательный бизнес-путь.

Минимальные outcomes implementation stage:

- `ready-for-ci`
  локальные проверки пройдены, изменения запушены, draft PR создан;
- `ready-for-review`
  обязательные CI checks зеленые, issue можно переводить в
  `Waiting for Code Review`;
- `blocked`
  stage не может продолжаться;
- `needs-rework`
  CI или review вернули issue обратно в `Implementation In Progress`.

Точный CLI surface и vocabulary нужно зафиксировать новым ADR, чтобы не
ломать уже принятый analysis-only контракт из ADR-0020.

## External Interfaces

- GitHub Project
  хранит source of truth по implementation statuses;
- GitHub Pull Requests
  являются обязательным результатом implementation stage;
- GitHub checks / CI
  формируют quality gate между `Waiting for CI` и `Waiting for Code Review`;
- Git
  создает и переиспользует implementation branch/worktree;
- Zellij
  дает stable execution context для implementation session;
- агент (`codex` / другой configured agent)
  запускается через project-local implementation launcher;
- test runner проекта
  исполняет обязательные локальные проверки до push/finalization.

## Architecture Notes

### Единый `run`, но отдельные stage-flow

Пользовательский контракт должен оставаться простым:

- оператор всегда вызывает `run <issue>`;
- `run` сам определяет текущую стадию issue;
- дальше `run` запускает нужный flow и применяет правильные переходы.

Следовательно, нужен не новый top-level entrypoint, а stage-aware dispatcher
внутри `run` и обновление текущего SSOT, который сегодня описывает `run` как
analysis-only path.

### Analysis artifacts как immutable input

Implementation flow не должен перепридумывать план по issue text или по
истории чата.

Правильный порядок:

1. human review принимает analysis artifacts;
2. implementation flow читает versioned SDD;
3. coding decisions вносятся в код и при необходимости в follow-up docs/ADR,
   но не переписывают silently исходный approved plan.

### Separate launcher contract

У implementation stage другие обязанности, чем у analysis launcher:

- подготовить coding workspace;
- загрузить implementation prompt;
- запускать проверки;
- поддерживать PR/CI lifecycle.

Это делает отдельный launcher contract предпочтительным даже если часть shell
логики будет переиспользована и даже если оба launcher path выбираются из одного
`run`.

### Approval metadata как часть versioned контракта

Документы не должны переходить из `draft` в `approved` неявно.

Минимальный контракт:

- пока issue в `Waiting for Plan Review`, SDD-комплект имеет
  `Статус согласования: pending human review`;
- когда человек подтверждает план и issue переводится в
  `Ready for Implementation`, система обновляет status metadata;
- package README и входной `04-implementation-plan.md` обязаны хранить
  `Approved By` и `Approved At`;
- при необходимости тот же metadata block может дублироваться в остальных
  документах комплекта для локальной читаемости.

## ADR Impact

Для реализации этой issue нужны как минимум новые ADR по следующим решениям:

1. Как approved analysis artifacts становятся каноническим входом для
   implementation stage.
2. Как `run` маршрутизирует issue между analysis и implementation stage.
3. Как устроен stage-scoped runtime/session-binding для implementation flow.
4. Какой internal CLI finalization contract используется для commit/push/PR/CI
   transitions implementation stage.

Без этих ADR implementation stage рискует получить скрытые соглашения в коде.

## Risks

- если approved analysis artifacts не попадут в стабильный источник,
  implementation flow будет зависеть от локальных worktree или unmerged branch;
- попытка переиспользовать analysis runtime без stage separation сломает
  текущий invariant `issue <-> session_uuid`;
- ожидание CI внутри agent session может сделать flow долгим и чувствительным к
  flaky checks;
- слишком ранняя универсализация launcher под все стадии может привести к
  “god-script” вместо inspectable contracts;
- если naming implementation branch/worktree не будет configurable,
  инструмент потеряет repo-agnostic переносимость.

## Alternatives Considered

### Расширить существующий `issue-analysis-flow` до multi-stage prompt

Отклонено.

Это размоет границу между спецификацией и реализацией, усложнит статусы и
сломает текущий SSOT, где `run` и `issue-analysis-flow` описаны как analysis-
only path.

### Отдельная top-level команда вместо единого `run`

Отклонено.

Это расходится с ожидаемым пользовательским контрактом: оператор хочет всегда
вызывать `run <issue>` и делегировать системе определение текущей стадии.

### Реализовывать код прямо из analysis branch

Отклонено.

Это смешивает plan review и code review в одной ветке, делает analysis
artifacts частью coding lifecycle по умолчанию и ухудшает понятность PR-model.

### Полагаться только на локальные тесты без отдельного CI stage

Отклонено.

Issue прямо требует quality gates для implementation stage. Без явного места
для CI нельзя различать “код написан” и “изменение готово к human review”.

## Migration Or Rollout Notes

Rollout лучше делать поэтапно:

1. сначала зафиксировать SSOT, feature-docs и ADR для implementation stage;
2. затем добавить statuses и config contract;
3. потом внедрить launcher/runtime/finalization path;
4. только после этого подключать реальный implementation agent и CI gating.

Это позволяет сохранить работоспособность текущего analysis MVP и не вводить
частично задокументированный hybrid flow.
