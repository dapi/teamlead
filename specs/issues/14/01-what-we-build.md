# Issue 14: Что строим

## Problem

Сейчас `poll` выбирает любую подходящую issue из `Backlog` по репозиторию,
issue state и project status, но не учитывает assignee.

В результате несколько экземпляров `ai-teamlead`, работающие на одном GitHub
Project в командном режиме, могут конкурировать за общий backlog и забирать
задачи не для "своего" оператора.

## Who Is It For

- оператор, который запускает свой экземпляр `ai-teamlead` в общем GitHub
  Project;
- владелец репозитория, который настраивает repo-local `settings.yml`;
- разработчик, который поддерживает poll-selection, GitHub snapshot и
  bootstrap template.

## Outcome

Нужен опциональный repo-local контракт:

```yaml
poll:
  assignee_filter: "$me"
```

Семантика результата:

- если `poll.assignee_filter` не задан, `poll` и `loop` сохраняют текущее
  поведение без фильтрации по assignee;
- если значение равно `"$me"`, eligible считаются только issue, заасайненные
  на текущего GitHub-пользователя;
- если значение равно `"username"`, eligible считаются только issue,
  заасайненные на указанного пользователя;
- ручной `run` не меняет поведение и не зависит от `assignee_filter`;
- `"$me"` резолвится один раз на старте процесса и затем переиспользуется в
  течение жизни `poll` или `loop`.

## Scope

В текущую задачу входит:

- добавить в `settings.yml` опциональную секцию `poll.assignee_filter`;
- расширить snapshot GitHub Project списком assignee login-ов для issue;
- фильтровать backlog-элементы по assignee только в selection path команд
  `poll` и `loop`;
- поддержать special value `"$me"` через `gh api user`;
- добавить documented comment-only пример в `templates/init/settings.yml`;
- покрыть новую семантику unit- и integration-тестами.

## Non-Goals

В текущую задачу не входит:

- ограничение ручной команды `run` по assignee;
- изменение project statuses, claim semantics или общего flow анализа;
- изменение детерминированного порядка среди уже eligible backlog-issue;
- хранение resolved current user в persistent runtime-state;
- поддержка более сложных правил фильтрации вроде нескольких usernames,
  teams или labels.

## Constraints And Assumptions

- source of truth для настройки остается repo-local `./.ai-teamlead/settings.yml`
  по ADR-0001;
- `poll.assignee_filter` должен быть optional, чтобы старые конфиги оставались
  валидными;
- special value `"$me"` опирается на активную GitHub-аутентификацию `gh`;
- если фильтр задан, issue без assignee не должны попадать в выборку;
- если у issue несколько assignee, достаточно совпадения хотя бы одного login-а;
- отсутствие фильтра должно полностью сохранять текущее поведение `poll`;
- шаблон `settings.yml` должен оставаться comment-only documented template по
  ADR-0027.

## User Story

Как оператор командного GitHub Project, я хочу настроить `poll` так, чтобы мой
экземпляр `ai-teamlead` забирал только задачи, назначенные мне или конкретному
пользователю, чтобы несколько экземпляров инструмента не конкурировали за один
общий backlog.

## Use Cases

1. Владелец репозитория не задает `poll.assignee_filter`, и `poll` продолжает
   выбирать верхнюю подходящую backlog-issue без дополнительных ограничений.
2. Владелец репозитория задает `poll.assignee_filter: "$me"`, и `loop`
   последовательно подхватывает только задачи текущего GitHub-пользователя.
3. Владелец репозитория задает `poll.assignee_filter: "alice"`, и shared
   instance для `alice` забирает только ее backlog-issue.
4. В backlog есть issue без assignee и issue с несколькими assignees; при
   активном фильтре первые игнорируются, а вторые считаются eligible при
   совпадении любого assignee.

## Dependencies

- данные GitHub Project snapshot должны включать assignee login-ы для issue;
- доступный `gh` CLI с валидной авторизацией нужен для режима `"$me"`;
- текущий общий `run`-path после выбора issue должен оставаться неизменным;
- существующие integration test stubs для `poll` и шаблон `settings.yml`
  требуют синхронного обновления.
