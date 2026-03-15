# Issue 14: Что строим

## Problem

Сейчас `poll` выбирает любую подходящую issue из `Backlog` по репозиторию,
issue state и project status, но не учитывает assignee.

В результате инструмент не может опереться на ownership policy по assignee:
оператор не может гарантировать, что автоматический выбор backlog-issue
соответствует назначенному пользователю или явно выбранному режиму отбора.

Отдельно важно:

- отсутствие параллельного выполнения `ai-teamlead` должно обеспечиваться
  lock-механизмом и не является предметом этой задачи;
- issue `#14` не должна переопределять этот инвариант и не должна
  мотивироваться допущением о допустимых параллельных экземплярах.

## Who Is It For

- оператор, который запускает свой экземпляр `ai-teamlead` в общем GitHub
  Project;
- владелец репозитория, который настраивает repo-local `settings.yml`;
- разработчик, который поддерживает poll-selection, GitHub snapshot и
  bootstrap template.

## Outcome

Нужен repo-local контракт:

```yaml
poll:
  assignee_filter: "$me"
```

Семантика результата:

- если `poll.assignee_filter` не задан, effective value считается `"$me"`;
- если значение равно `"$me"`, eligible считаются только issue, заасайненные
  на текущего GitHub-пользователя;
- если значение равно `"$all"`, фильтр по assignee явно отключается;
- если значение равно `"$unassigned"`, eligible считаются только issue без
  assignee;
- если значение равно `"username"`, eligible считаются только issue,
  заасайненные на указанного пользователя;
- ручной `run` проверяет соответствие issue effective `assignee_filter`;
- если `run` находит mismatch, он выводит warning и просит approve у
  пользователя;
- если `run` запущен с `--force`, warning остается, но approve не требуется;
- `"$me"` резолвится один раз на старте процесса и затем переиспользуется в
  течение жизни `poll` или `loop`.

## Scope

В текущую задачу входит:

- добавить в `settings.yml` documented comment-only секцию
  `poll.assignee_filter`;
- расширить snapshot GitHub Project списком assignee login-ов для issue;
- фильтровать backlog-элементы по assignee только в selection path команд
  `poll` и `loop`;
- поддержать special values `"$me"`, `"$all"` и `"$unassigned"`;
- добавить warning/approve semantics для `run` при несоответствии effective
  filter и `--force` override;
- пояснить в документации, что comment-only template показывает рекомендуемый
  default и допустимые override-режимы, а не fully materialized active config;
- покрыть новую семантику unit- и integration-тестами.

## Non-Goals

В текущую задачу не входит:

- изменение project statuses, claim semantics или общего flow анализа;
- изменение детерминированного порядка среди уже eligible backlog-issue;
- хранение resolved current user в persistent runtime-state;
- поддержка более сложных правил фильтрации вроде нескольких usernames,
  teams или labels;
- ослабление отдельного инварианта single-instance execution через lock.

## Constraints And Assumptions

- source of truth для настройки остается repo-local `./.ai-teamlead/settings.yml`
  по ADR-0001;
- отсутствие active override должно трактоваться как effective default `"$me"`,
  а не как отключенный фильтр;
- special value `"$me"` опирается на активную GitHub-аутентификацию `gh`;
- `"$all"` должен быть явным способом отключить assignee filtering;
- `"$unassigned"` должен быть явным способом выбрать только issue без assignee;
- если у issue несколько assignee, достаточно совпадения хотя бы одного login-а;
- шаблон `settings.yml` должен оставаться comment-only documented template по
  ADR-0027;
- новая интерактивная семантика `run` зависит от более общего UX-слоя для
  диагностики `run`, поэтому дальнейшая реализация задачи заблокирована issue
  `#11`.

## User Story

Как оператор GitHub Project, я хочу, чтобы `poll` по умолчанию забирал задачи,
назначенные мне, но при этом владелец репозитория мог явно переключить режим на
конкретного пользователя, все задачи или только unassigned, а `run` предупреждал
о нарушении этой политики, чтобы автоматический и ручной запуск следовали одному
и тому же ownership contract.

## Use Cases

1. Владелец репозитория ничего не задает в active YAML, и `poll` использует
   default `"$me"`, подхватывая только задачи текущего GitHub-пользователя.
2. Владелец репозитория явно задает `poll.assignee_filter: "$all"`, и `poll`
   снова рассматривает все backlog-issue без ограничения по assignee.
3. Владелец репозитория задает `poll.assignee_filter: "$unassigned"`, и
   `poll` подхватывает только backlog-issue без assignee.
4. Владелец репозитория задает `poll.assignee_filter: "alice"`, и `poll`
   подхватывает только issue, где среди assignees есть `alice`.
5. Оператор запускает `run <issue>` для issue, которая не соответствует
   effective filter: система показывает warning и требует approve, если не
   передан `--force`.

## Dependencies

- данные GitHub Project snapshot должны включать assignee login-ы для issue;
- доступный `gh` CLI с валидной авторизацией нужен для режима `"$me"`;
- issue `#11` является hard blocker для дальнейшей разработки, потому что
  review расширил scope `#14` интерактивным поведением `run`;
- текущий общий `run`-path после выбора issue должен переиспользоваться и для
  warning/approve semantics;
- существующие integration test stubs для `poll` и шаблон `settings.yml`
  требуют синхронного обновления.
