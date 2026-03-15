# Issue 14: Как строим

## Approach

Решение делается как локальное расширение существующего `poll`-контракта без
смены общего issue-level flow:

1. Добавить в config optional блок `poll` с полем `assignee_filter`.
2. Расширить GitHub snapshot так, чтобы каждый `ProjectIssueItem` содержал
   список assignee login-ов.
3. Перед выбором backlog-issue вычислять effective assignee filter:
   - `"$me"`, если настройка отсутствует;
   - resolved login текущего пользователя, если задано `"$me"`;
   - отсутствие фильтрации, если задано `"$all"`;
   - специальный режим unassigned, если задано `"$unassigned"`;
   - literal username, если задано конкретное имя.
4. Передавать уже resolved filter в domain-функцию выбора backlog-issue.
5. Добавить в `run` проверку соответствия issue effective filter с warning /
   approve semantics и `--force` bypass.
6. Считать issue `#11` обязательным prerequisite для финальной реализации и
   rollout этой части `run`.

Это сохраняет текущее разделение ответственности:

- `config` отвечает за parsing/default semantics;
- `github` отвечает за получение snapshot и resolve текущего пользователя через
  `gh`;
- `domain` отвечает за pure selection logic;
- `app` отвечает за orchestration команд `poll`, `loop` и operator-facing
  поведение `run` при mismatch.

## Affected Areas

- `src/config.rs`
  новая optional-конфигурация `poll.assignee_filter`, defaults и validation;
- `src/github.rs`
  загрузка assignees из GraphQL snapshot и helper для resolve current user;
- `src/domain.rs`
  фильтрация backlog-элементов по effective assignee policy;
- `src/app.rs`
  вычисление effective filter для `poll`, `loop` и `run`, warning/approve path,
  а также `--force` override;
- `templates/init/settings.yml`
  закомментированный documented пример `poll.assignee_filter` с default
  `"$me"` и override-режимами;
- unit tests и integration tests для config, selection logic и `poll`.

## Interfaces And Data

Целевой config contract:

```yaml
poll:
  assignee_filter: "$me"
```

Семантика поля:

- `poll` блок optional;
- `poll.assignee_filter` optional как active override, но отсутствие active
  значения не означает отсутствие политики;
- если значение не задано, effective value считается `"$me"`;
- `"$me"` резолвится в login текущего GitHub-пользователя;
- `"$all"` отключает фильтрацию по assignee;
- `"$unassigned"` выбирает только issue без assignee;
- любое другое непустое значение трактуется как GitHub login.

Целевое расширение snapshot-модели:

```rust
pub struct ProjectIssueItem {
    pub item_id: String,
    pub issue_number: u64,
    pub issue_state: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub assignees: Vec<String>,
    pub status_name: Option<String>,
    pub status_option_id: Option<String>,
}
```

Требования к GitHub integration:

- `load_project_snapshot` должен запрашивать assignee login-ы у issue в том же
  GraphQL snapshot, где уже читаются number/state/repository;
- helper для `"$me"` должен вызывать `gh api user --jq '.login'`;
- resolved login должен возвращаться как обычная строка и передаваться дальше
  без знания о special value.

Domain contract для выбора backlog-issue:

- фильтрация по repo, issue state и status остается прежней;
- при effective mode `"$all"` поведение идентично текущему;
- при effective mode `"$unassigned"` eligible считается только issue с пустым
  `assignees`;
- при login-based mode eligible считается только issue, у которой `assignees`
  содержит совпадающий login;
- порядок среди eligible элементов остается прежним: верхняя issue из snapshot
  GitHub Project.

Contract для ручного `run`:

- `run` вычисляет тот же effective assignee policy, что и `poll`;
- если issue policy не нарушает, запуск идет обычным путем;
- если issue не соответствует policy, `run` выводит warning;
- без `--force` после warning нужен явный approve пользователя;
- с `--force` warning остается, но approve path пропускается.

## Configuration And Runtime Assumptions

- старые `settings.yml` без блока `poll` должны загружаться без изменений;
- comment-only template должен показывать закомментированный пример с
  `assignee_filter: "$me"` и рядом допустимые override-режимы;
- для `poll` команда `"$me"` может резолвиться непосредственно перед
  `run_poll_cycle`;
- для `loop` resolved login должен вычисляться один раз до входа в цикл и
  затем переиспользоваться во всех итерациях процесса;
- ошибка `gh api user` должна останавливать запуск `poll` или `loop` на старте,
  а не проявляться поздно после частичного claim;
- `run` должен читать effective assignee policy, но его детальная UX-ветка
  зависит от реализации issue `#11`.

## Risks

- если assignees не будут добавлены в GraphQL snapshot, фильтр может выглядеть
  работающим в config, но всегда давать пустую выборку;
- если `"$me"` будет резолвиться на каждом poll-cycle, `loop` начнет делать
  лишние вызовы `gh` и хуже диагностироваться при flaky auth;
- если `run` и `poll` будут вычислять разные effective modes, контракт станет
  непредсказуемым для оператора;
- если `run` mismatch path реализовать до закрытия issue `#11`, можно получить
  второй несовместимый слой user-facing диагностики;
- если не обновить test snapshots и gh stubs, integration tests начнут
  проверять старый payload и потеряют ценность;
- если сравнение login-ов реализовать неявно или непоследовательно, возможны
  труднодиагностируемые пропуски eligible issue.

## External Interfaces

Внешние интерфейсы:

- GitHub GraphQL API через `gh api graphql` для чтения assignees в project
  snapshot;
- GitHub REST API через `gh api user --jq '.login'` для режима `"$me"`;
- repo-local `./.ai-teamlead/settings.yml` и bootstrap template
  `templates/init/settings.yml`.

Практические требования:

- новый GraphQL payload должен оставаться совместимым с существующими test
  stubs;
- shell-вызов `gh api user` должен идти через существующий `Shell`
  abstraction, а не через ad-hoc `Command::new`.
- interactive approve path для `run` должен проектироваться вместе с issue
  `#11`, а не как отдельный локальный prompt-layer внутри `#14`.

## ADR Impact

Новый ADR не требуется.

Причина:

- issue не меняет repo-local природу `settings.yml`, а лишь расширяет ее по
  ADR-0001;
- issue не меняет модель `poll` / `loop` как CLI-команд, а лишь уточняет
  selection semantics и policy-check в рамках уже существующего CLI-контракта
  из ADR-0021;
- детерминированный порядок backlog-выбора из ADR-0009 сохраняется после
  фильтрации;
- работа с documented template и default-layer остается в рамках ADR-0027.

Нужно синхронизировать:

- feature 0001 про CLI `poll` / `run` / `loop`;
- feature 0002 про `settings.yml` bootstrap contract;
- SDD-комплект issue `#14` как task-specific контракт изменения.

## Alternatives Considered

### 1. Резолвить `"$me"` на каждом poll-cycle

Отклонено.

Это противоречит требованию issue про кэширование на время жизни процесса,
добавляет лишние вызовы `gh` и делает `loop` менее предсказуемым.

### 2. Применять assignee filter и к ручному `run`

Принято после review.

Review уточнил, что `run` тоже должен учитывать policy, но через более мягкий
contract: warning и approve, а не через немой отказ.

### 3. Считать отсутствие active значения эквивалентом отсутствия фильтра

Отклонено.

Review явно уточнил, что default policy должна быть `"$me"`. Полное отключение
должно быть отдельным явным режимом вроде `"$all"`.

## Migration Or Rollout Notes

- схема обратно совместима: существующие конфиги без `poll` продолжают
  работать, но начинают наследовать default policy `"$me"`;
- rollout требует обновления test fixtures с assignees в GitHub snapshot;
- documented template должен получить только закомментированный пример и
  пояснение про `"$all"` / `"$unassigned"`;
- до реализации issue `#11` задача должна считаться blocked и не должна
  переходить к coding stage;
- после закрытия `#11` нужно проверить, что `poll` с default `"$me"` берет
  верхнюю подходящую issue текущего пользователя, а `run` корректно проходит
  через warning/approve semantics.
