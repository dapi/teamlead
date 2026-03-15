# Issue 14: Как строим

## Approach

Решение делается как локальное расширение существующего `poll`-контракта без
смены общего issue-level flow:

1. Добавить в config optional блок `poll` с полем `assignee_filter`.
2. Расширить GitHub snapshot так, чтобы каждый `ProjectIssueItem` содержал
   список assignee login-ов.
3. Перед выбором backlog-issue вычислять effective assignee filter:
   - `None`, если настройка отсутствует;
   - resolved login текущего пользователя, если задано `"$me"`;
   - literal username, если задано конкретное имя.
4. Передавать уже resolved filter в domain-функцию выбора backlog-issue.
5. Сохранить ручной `run` вне влияния нового фильтра.

Это сохраняет текущее разделение ответственности:

- `config` отвечает за parsing/default semantics;
- `github` отвечает за получение snapshot и resolve текущего пользователя через
  `gh`;
- `domain` отвечает за pure selection logic;
- `app` отвечает за orchestration команд `poll` и `loop`.

## Affected Areas

- `src/config.rs`
  новая optional-конфигурация `poll.assignee_filter`, defaults и validation;
- `src/github.rs`
  загрузка assignees из GraphQL snapshot и helper для resolve current user;
- `src/domain.rs`
  фильтрация backlog-элементов по resolved assignee;
- `src/app.rs`
  вычисление effective filter для `poll` и `loop`, при этом `run` остается без
  нового ограничения;
- `templates/init/settings.yml`
  закомментированный documented пример `poll.assignee_filter`;
- unit tests и integration tests для config, selection logic и `poll`.

## Interfaces And Data

Целевой config contract:

```yaml
poll:
  assignee_filter: "$me"
```

Семантика поля:

- `poll` блок optional;
- `poll.assignee_filter` optional;
- если значение не задано, domain selection не получает дополнительный фильтр;
- `"$me"` является специальным значением и не должно попадать в domain как
  literal filter;
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
- если resolved filter отсутствует, поведение идентично текущему;
- если resolved filter задан, eligible считается только issue, у которой
  `assignees` содержит совпадающий login;
- порядок среди eligible элементов остается прежним: верхняя issue из snapshot
  GitHub Project.

## Configuration And Runtime Assumptions

- старые `settings.yml` без блока `poll` должны загружаться без изменений;
- comment-only template не обязан включать active `poll` block по default,
  потому что `assignee_filter` не является canonical runtime default;
- для `poll` команда `"$me"` может резолвиться непосредственно перед
  `run_poll_cycle`;
- для `loop` resolved login должен вычисляться один раз до входа в цикл и
  затем переиспользоваться во всех итерациях процесса;
- ошибка `gh api user` должна останавливать запуск `poll` или `loop` на старте,
  а не проявляться поздно после частичного claim;
- `run` продолжает находить issue по номеру и не читает `poll.assignee_filter`.

## Risks

- если assignees не будут добавлены в GraphQL snapshot, фильтр может выглядеть
  работающим в config, но всегда давать пустую выборку;
- если `"$me"` будет резолвиться на каждом poll-cycle, `loop` начнет делать
  лишние вызовы `gh` и хуже диагностироваться при flaky auth;
- если фильтр случайно применить к `run`, это сломает явно зафиксированное
  требование issue;
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

## ADR Impact

Новый ADR не требуется.

Причина:

- issue не меняет repo-local природу `settings.yml`, а лишь расширяет ее по
  ADR-0001;
- issue не меняет модель `poll` / `loop` как CLI-команд, а лишь уточняет
  selection semantics внутри уже существующего контракта из ADR-0021;
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

Отклонено.

Issue явно фиксирует, что `run` не зависит от `assignee_filter`, поэтому новый
контракт должен оставаться локальным именно для автоматического выбора backlog.

## Migration Or Rollout Notes

- схема обратно совместима: существующие конфиги без `poll` продолжают
  работать;
- rollout требует обновления test fixtures с assignees в GitHub snapshot;
- documented template должен получить только закомментированный пример, чтобы
  не навязывать новый opt-in фильтр как активное значение;
- после внедрения нужно проверить, что `poll` без фильтра по-прежнему берет
  верхнюю подходящую issue и что `loop` не повторяет resolve current user в
  каждой итерации.
