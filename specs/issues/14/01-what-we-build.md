# Issue 14: Что строим

## Problem

Сейчас `poll` выбирает первую подходящую issue из `Backlog`, если совпадают:

- репозиторий;
- `issue_state == "OPEN"`;
- project status = `Backlog`.

При этом assignee не учитывается. В общем GitHub Project это мешает закрепить
ownership policy через назначение задач: один экземпляр `ai-teamlead` не может
отбирать backlog только для своего пользователя.

## Who Is It For

- оператор `ai-teamlead`, который запускает `poll` или `loop` в общем GitHub
  Project;
- владелец репозитория, который настраивает repo-local `settings.yml`;
- разработчик, который поддерживает selection logic, GitHub snapshot и шаблон
  bootstrap-конфига.

## Outcome

Нужен optional repo-local контракт:

```yaml
poll:
  assignee_filter: "$me"
```

Семантика результата:

- если `poll.assignee_filter` не задан, `poll` сохраняет текущее поведение и не
  фильтрует backlog по assignee;
- если значение равно `"$me"`, `poll` выбирает только issue, где среди
  assignees есть текущий GitHub-пользователь;
- если значение равно `"username"`, `poll` выбирает только issue, где среди
  assignees есть указанный пользователь;
- `"$me"` резолвится через `gh api user --jq '.login'` один раз на старте
  процесса и затем кэшируется на время жизни `poll` или `loop`;
- `run` не зависит от `assignee_filter` и сохраняет текущее поведение.

## Scope

В текущую задачу входит:

- добавить в `Config` optional секцию `poll` с полем `assignee_filter`;
- расширить `ProjectIssueItem` списком assignee login-ов;
- запросить assignees в GraphQL snapshot `load_project_snapshot`;
- добавить helper для resolve текущего GitHub-пользователя через `gh api user`;
- фильтровать backlog-issue по assignee в selection path команд `poll` и
  `loop`;
- добавить в `templates/init/settings.yml` закомментированный пример
  `poll.assignee_filter: "$me"`;
- покрыть изменение unit- и integration-тестами.

## Non-Goals

В текущую задачу не входит:

- изменение поведения ручного `run`;
- введение новых специальных режимов вроде `"$all"` или `"$unassigned"`;
- смена default policy на `"$me"` при отсутствии настройки;
- поддержка фильтрации по нескольким usernames, teams или labels;
- изменение детерминированного порядка среди уже eligible backlog-issue;
- хранение resolved current user в persistent runtime-state.

## Constraints And Assumptions

- source of truth для настройки остается repo-local
  `./.ai-teamlead/settings.yml`;
- отсутствие `poll.assignee_filter` должно быть обратно совместимо с текущим
  поведением;
- если задано `"$me"`, нужен доступный `gh` CLI с валидной авторизацией;
- если `gh api user` недоступен, ошибка должна проявляться на старте процесса,
  а не после частичного `poll`-цикла;
- issue без assignee не должны матчиться, если фильтр задан;
- если у issue несколько assignees, достаточно совпадения хотя бы одного login;
- `loop` должен переиспользовать тот же контракт фильтрации, потому что
  является foreground loop поверх `poll`.

## User Story

Как оператор общего GitHub Project, я хочу ограничить `poll` задачами,
назначенными текущему пользователю или конкретному login, чтобы автоматический
отбор backlog не забирал чужие issue.

## Use Cases

1. Владелец репозитория не задает `poll.assignee_filter`, и `poll` работает
   ровно как сейчас, без фильтра по assignee.
2. Владелец репозитория задает `poll.assignee_filter: "$me"`, и `poll`
   подхватывает только backlog-issue текущего GitHub-пользователя.
3. Владелец репозитория задает `poll.assignee_filter: "alice"`, и `poll`
   подхватывает только backlog-issue, у которых среди assignees есть `alice`.

## Dependencies

- GitHub Project snapshot должен возвращать assignee login-ы для issue;
- `gh api user` нужен только для режима `"$me"`;
- тестовые stubs и fixtures для `gh`/GraphQL нужно синхронно обновить под новое
  поле `assignees`.
