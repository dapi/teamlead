# Issue 14: Как строим

## Approach

Решение делается как локальное расширение существующего `poll`-контракта без
изменения общего issue-level flow:

1. Добавить в config optional блок `poll` с полем `assignee_filter`.
2. Расширить GitHub snapshot так, чтобы каждый `ProjectIssueItem` содержал
   список assignee login-ов.
3. В `app`-слое вычислять effective assignee filter для текущего процесса:
   - `None`, если настройка не задана;
   - resolved login текущего пользователя, если задано `"$me"`;
   - literal username, если задан конкретный login.
4. Передавать в `domain::select_next_backlog_project_item` уже зарезолвленный
   `Option<&str>`.
5. Оставить `run` вне зоны изменения: фильтрация действует только для selection
   path `poll`, а `loop` наследует ее через переиспользование `poll`.

Это сохраняет текущее разделение ответственности:

- `config` отвечает за parsing optional-настройки;
- `github` отвечает за получение assignees из snapshot и resolve current user;
- `domain` отвечает за pure filtering logic;
- `app` отвечает за orchestration `poll`/`loop` и lifetime cache для `"$me"`.

## Affected Areas

- `src/config.rs`
  новая optional-конфигурация `poll.assignee_filter`;
- `src/github.rs`
  загрузка assignees из GraphQL snapshot и helper `resolve_current_user`;
- `src/domain.rs`
  дополнительная фильтрация backlog-элементов по `Option<&str>`;
- `src/app.rs`
  resolve `"$me"` и передача effective filter в `run_poll_cycle`;
- `templates/init/settings.yml`
  закомментированный documented example для `poll.assignee_filter`;
- unit tests и integration tests для config, snapshot parsing и selection logic.

## Interfaces And Data

Целевой config contract:

```yaml
poll:
  assignee_filter: "$me"
```

Минимальная схема в `config`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PollConfig {
    pub assignee_filter: Option<String>,
}
```

Требования к поведению:

- `poll` блок optional;
- `poll.assignee_filter` optional;
- отсутствие active override означает `None` и не меняет текущее поведение;
- `"$me"` и literal username являются единственными поддерживаемыми режимами;
- special value не должен просачиваться в `domain`: `domain` получает либо
  `None`, либо уже зарезолвленный login.

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

- `load_project_snapshot` должен запрашивать assignees в том же GraphQL
  snapshot, где уже читаются `number`, `state` и `repository`;
- helper `resolve_current_user` должен вызывать
  `gh api user --jq '.login'`;
- для one-shot `poll` resolved login достаточно вычислить один раз на вызов
  команды;
- для `loop` resolved login нужно вычислить один раз до входа в цикл и затем
  переиспользовать.

Domain contract:

- фильтрация по repo, `issue_state` и status остается прежней;
- при `assignee_filter = None` поведение идентично текущему;
- при `Some(login)` eligible считается только issue, у которой `assignees`
  содержит совпадающий login;
- порядок среди eligible элементов остается прежним: первый подходящий элемент
  snapshot.

## Configuration And Runtime Assumptions

- старые `settings.yml` без блока `poll` должны загружаться без изменений;
- `templates/init/settings.yml` должен показывать только comment-only пример,
  а не включать новый active override по умолчанию;
- режим `"$me"` требует валидной авторизации `gh`;
- ошибка resolve current user должна останавливать `poll`/`loop` до попытки
  claim issue;
- `run` не читает и не применяет `assignee_filter`.

## Risks

- если assignees не будут добавлены в GraphQL snapshot, настройка появится в
  config, но selection logic не сможет работать корректно;
- если `"$me"` будет резолвиться на каждом цикле `loop`, процесс начнет делать
  лишние вызовы `gh` и станет менее предсказуемым;
- если фильтрация случайно попадет в `run`, задача расширит scope относительно
  исходного issue;
- если старые конфиги без блока `poll` перестанут загружаться, будет нарушена
  обратная совместимость;
- если не обновить test fixtures, integration tests продолжат проверять старый
  snapshot без `assignees`.

## External Interfaces

Внешние интерфейсы:

- GitHub GraphQL API через `gh api graphql` для чтения assignees в project
  snapshot;
- GitHub REST API через `gh api user --jq '.login'` для режима `"$me"`;
- repo-local `./.ai-teamlead/settings.yml` и bootstrap template
  `templates/init/settings.yml`.

Практические требования:

- вызовы `gh` должны идти через существующий `Shell` abstraction;
- новый GraphQL payload должен быть отражен в тестовых fake/stub ответах;
- selection logic должна оставаться чистой и тестируемой отдельно от shell.

## ADR Impact

Новый ADR не требуется.

Причина:

- задача не меняет repo-local природу `settings.yml` по ADR-0001;
- задача не меняет публичный CLI-контракт `poll` / `run` / `loop` из ADR-0021,
  а только уточняет selection semantics `poll`;
- детерминированный порядок backlog-выбора из ADR-0009 сохраняется после
  фильтрации;
- comment-only nature bootstrap-template остается в рамках ADR-0027.

Нужно синхронизировать:

- feature 0001 про CLI `poll` / `run` / `loop`;
- feature 0002 про bootstrap `settings.yml`;
- task-specific analysis docs issue `#14`.

## Alternatives Considered

### 1. Сделать `"$me"` runtime default при отсутствии настройки

Отклонено.

Это противоречит исходному issue, где отсутствие `assignee_filter` означает
сохранение текущего поведения без фильтрации.

### 2. Фильтровать и ручной `run`

Отклонено.

Исходный issue явно фиксирует, что `assignee_filter` влияет на `poll`, а ручной
запуск должен работать независимо от assignee.

### 3. Резолвить `"$me"` прямо в `domain`

Отклонено.

`domain` должен оставаться pure-слоем без shell/GitHub зависимостей.

## Migration Or Rollout Notes

- изменение обратно совместимо: старые конфиги без блока `poll` продолжают
  работать;
- rollout требует обновления GraphQL test fixtures и stub-ответов для `gh`;
- documented template должен получить только закомментированный пример
  `poll.assignee_filter: "$me"`;
- после реализации нужно вручную проверить, что `poll` без фильтра по-прежнему
  берет первую подходящую issue, а с фильтром пропускает чужие backlog-issue.
