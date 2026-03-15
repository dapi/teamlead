# Issue 11: Как строим

Статус: approved
Последнее обновление: 2026-03-15
Approved By: dapi
Approved At: 2026-03-15T11:52:35+03:00

## Approach

Решение строится в два слоя:

1. отделить определение причины и remediation-плана от форматирования
   сообщения;
2. отделить project snapshot lookup от repo-level issue lookup и project
   mutation-path.

Практически это означает:

- `run_manual_run` больше не должен трактовать отсутствие item в
  `ProjectSnapshot` как единственно возможный случай `not in project`;
- orchestration сначала получает минимальный факт об issue на уровне
  репозитория: существует ли она и открыта ли она;
- затем, если issue открыта, проверяется наличие item в target project и
  допустимость project status для stage-aware `run`;
- если причина проблемы допускает безопасное автоисправление, orchestration
  исполняет remediation-план и продолжает `run`;
- если безопасного remediation-path нет или попытка исправления не удалась,
  доменный слой собирает единое user-facing сообщение с объяснением причины и
  ручным fallback.

Такой подход нужен, чтобы одна и та же логика сообщения не размазывалась между
`src/app.rs`, `src/domain.rs` и будущими CLI-path.

## Affected Areas

- `src/app.rs`
  `run_manual_run` перестает делать ранний `not linked to the project` и
  переходит к пошаговому определению причины отказа и remediation-path;
- `src/domain.rs`
  контракт `RunDecision` или соседний domain-тип должен хранить данные,
  необходимые для детального user-facing сообщения:
  `current_status`, `allowed_statuses`, причина отказа, тип отказной ветки и
  план автоисправления;
- `src/github.rs`
  нужен отдельный repo-level lookup для одной issue вне `ProjectSnapshot`,
  чтобы различать `not found`, `closed` и `not in project`, а также mutation
  path для добавления issue в project и смены status;
- unit-тесты доменной логики;
- integration tests для CLI `run`.

## Interfaces And Data

### 1. Repo-level issue lookup

Нужен новый adapter path, который по owner/repo и issue number возвращает
минимальные данные об issue:

- `id` или другой идентификатор issue, пригодный для project mutation;
- `number`;
- `state` (`OPEN` / `CLOSED`);
- `url`.

Этого достаточно, чтобы:

- отличить несуществующую issue от закрытой;
- иметь данные для автоматического добавления в project;
- не тащить в domain лишние поля.

### 2. Domain-модель отказа

Текущего `allowed: bool + reason: &'static str` недостаточно. Нужен
структурированный контракт, который отвечает на два вопроса:

1. можно ли продолжать `run` уже сейчас;
2. если нельзя, можно ли систему безопасно исправить автоматически.

Минимально полезные поля:

- `kind` или эквивалентный discriminant для веток:
  `IssueNotFound`, `IssueClosed`, `AttachToProject`, `NormalizeStatus`,
  `ExplainOnlyStatusDenied`;
- `issue_number`;
- `issue_url` для ветки `IssueNotInProject`;
- `project_id`;
- `current_status` для `StatusDenied`;
- `allowed_statuses` для `StatusDenied`;
- `target_status`, если remediation требует перевода issue в другой status;
- человеко-понятная функция форматирования результата, например
  `format_run_denied_message()`.

Практически это означает:

- `IssueNotFound` и `IssueClosed` являются explain-only ветками;
- `AttachToProject` означает: issue открыта, ее можно автоматически добавить в
  project и выставить стартовый status;
- `NormalizeStatus` означает: issue уже в project, и система знает
  детерминированный target status, к которому можно перевести issue перед
  повторным dispatch;
- `ExplainOnlyStatusDenied` означает: system не может однозначно выбрать target
  status и должна эскалировать на пользователя с объяснением причины.

### 3. Allowed statuses contract

Список допустимых статусов не должен хардкодиться в сообщении отдельно от
логики маршрутизации.

Предпочтительный путь:

- `decide_run_stage()` или соседняя domain-функция возвращает не только факт
  отказа, но и полный список entry status, которые считаются валидными для
  текущего project contract;
- этот список собирается из analysis и implementation status-конфигов;
- formatter отказа использует именно этот список;
- remediation-план для `NormalizeStatus` не дублирует отдельный hardcoded
  список, а опирается на ту же каноническую domain-логику.

Это нужно, чтобы сообщение автоматически оставалось согласованным с accepted
stage-aware dispatch и не возвращало analysis-only набор статусов.

## Configuration And Runtime Assumptions

- `github.project_id` берется только из `./.ai-teamlead/settings.yml`;
- owner/repo по-прежнему определяются из текущего git-репозитория, а не из
  пользовательского ввода;
- источником истины о project status остается snapshot configured GitHub
  Project;
- repo-level lookup issue не заменяет `ProjectSnapshot`, а дополняет его для
  более точной диагностики и последующего auto-remediation;
- форматирование сообщений выполняется локально в приложении, а не делегируется
  внешнему выводу `gh`.

## Risks

- Если список допустимых статусов продублировать отдельно от
  `decide_run_stage()`, сообщение быстро устареет относительно stage-aware
  dispatch.
- Если repo-level lookup будет выполнен только после project lookup и его
  ошибки будут замаскированы, пользователь снова получит ложный `not in
  project`.
- Если auto-remediation запустить без явного domain-решения, `run` может
  silently менять status в неверную сторону.
- Если ветки `IssueClosed` и `IssueNotFound` не развести явно, оператор не
  поймет, нужно ли переоткрывать issue или искать ошибку во входном номере.
- Если форматирование сообщений останется в `src/app.rs`, покрытие unit-тестами
  будет хрупким, а будущие CLI-path снова начнут выводить разные формулировки.

## External Interfaces

Внешний интерфейс меняется только на уровне UX сообщений CLI:

- `ai-teamlead run <issue>` начинает печатать не только более подробные ошибки,
  но и сообщения об auto-remediation;
- для `AttachToProject` пользователь видит, что система сама добавила issue в
  target project и какой status установила;
- для `NormalizeStatus` пользователь видит, что система сама изменила status и
  после этого продолжила `run`;
- для explain-only веток сообщение показывает текущее значение status field и
  разрешенные entry statuses;
- команда перестает быть strictly read-only на preflight-ветках, потому что при
  безопасном auto-remediation она может выполнять project mutation до launch
  path.

## Architecture Notes

- Не стоит решать задачу простым расширением текущей строки `reason`.
  Структурированный remediation-aware тип лучше соответствует contract-first
  подходу и quality bar проекта.
- Новый repo-level issue lookup лучше держать в том же `GhProjectClient` или в
  близком adapter layer, а не вызывать `gh issue view` напрямую из `app.rs`.
- User-facing формат сообщения должен жить рядом с domain-решением, чтобы
  integration и unit tests проверяли один канонический formatter.
- Project mutations для auto-remediation лучше делать через тот же GitHub
  adapter, а не через shell snippets в `app.rs`.
- Конфликт из issue с примером `Ready for Implementation` нужно трактовать как
  устаревший пример, а не как новое требование менять stage-aware behavior.

## ADR Impact

Новый ADR не требуется.

Причина: задача не меняет source of truth, не меняет allowed transitions и не
пересматривает execution model. Она уточняет operator-facing UX и adapter-path
внутри уже принятого stage-aware и GitHub-backed контракта.

## Alternatives Considered

### Оставить текущий lookup только по `ProjectSnapshot`

Отклонено.

Такой путь не позволяет корректно различить `issue not found`, `issue closed`
и `issue not in project`, а значит и не позволяет выбрать корректный
remediation-path.

### Формировать подробные сообщения прямо в `app.rs`

Отклонено.

Это оставит бизнес-логику, orchestration и user-facing string assembly в одном
слое и ухудшит тестируемость.

### Ограничиться только объяснением и не пытаться исправлять автоматически

Отклонено.

Это противоречит review пользователя и сохраняет лишний ручной шаг там, где у
системы уже есть доступ и данные для исправления проблемы.

### Возвращать коду только готовую строку из GitHub adapter

Отклонено.

Adapter должен возвращать факты, а не финальный операторский UX, иначе логика
останется завязана на нестабильный текст внешнего CLI и потеряет доменный
контроль.

## Migration Or Rollout Notes

- Изменение обратно совместимо по CLI API: сигнатура `run <issue>` не меняется;
- изменятся тексты отказов, появятся auto-remediation шаги и coverage для них;
- smoke и integration tests, которые проверяют отказ `run`, нужно выровнять с
  новым более подробным сообщением;
- если в проекте уже есть тесты на старые строковые ошибки, их нужно заменить
  на новый канонический formatter, а не поддерживать оба варианта параллельно.
