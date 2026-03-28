# Issue 21: Что строим

Статус: draft
Последнее обновление: 2026-03-15

## Problem

Operator-driven `run <issue>` должен помогать взять конкретную задачу в работу,
а не требовать ручной подготовки issue в GitHub перед запуском.

Проблема в том, что contract вокруг этой команды остается неполным и
неоднозначным:

- у issue может не быть project item в default GitHub Project;
- у project item может отсутствовать status;
- у самой issue может отсутствовать assignee;
- в коде уже есть частичный auto-heal path для project/status, но он еще не
  оформлен как явный flow contract и не покрывает ownership policy.

В результате ручной `run` либо оказывается слишком хрупким, либо начинает
иметь скрытые side effects без канонического описания.

## Who Is It For

- оператор, который вручную запускает `ai-teamlead run <issue>` и ожидает, что
  команда сама доведет issue до корректного входа в analysis flow;
- владелец репозитория, которому нужен предсказуемый contract между `run`,
  GitHub Project и ownership issue;
- разработчик `ai-teamlead`, который поддерживает stage-aware `run` и GitHub
  integration layer без дрейфа документации и реализации.

## Outcome

Нужен явный preflight-contract для `run`, в котором до обычной stage-логики
выполняется нормализация GitHub-side состояния issue:

- проверяется, что issue существует в текущем repo и находится в состоянии
  `open`;
- если issue не прикреплена к configured default project, `run` добавляет ее в
  project;
- если project item не имеет status, `run` выставляет стартовый status
  `Backlog`;
- если у issue нет assignee, `run` назначает ее на текущего пользователя из
  `gh` context;
- если assignee уже есть, `run` его не меняет;
- только после этого `run` продолжает обычную stage-aware flow-логику;
- ошибки GitHub API видны оператору и прерывают запуск, а не маскируются.

## Scope

В текущую задачу входит:

- фиксация explicit preflight normalization для ручного `run`;
- выравнивание policy, что assignment является частью operator-driven claim;
- использование текущего пользователя `gh` как единственного источника
  identity;
- assignment только при пустом `assignee`;
- сохранение существующего stage-aware `run` и продолжение обычного flow после
  нормализации;
- обновление SSOT, feature docs, ADR и verification-плана;
- тесты на normalize/add/status/assign behavior и на отсутствие регрессии у
  `poll`.

## Non-Goals

В текущую задачу не входит:

- автоматическая нормализация для `poll`;
- automatic reassign issue, у которой assignee уже задан;
- изменение уже установленного status, если он присутствует;
- новые public CLI-команды или отдельный operator subcommand для normalize;
- выбор identity из OS-user, git config или произвольных env vars;
- тихое продолжение flow после ошибки preflight mutation.

## Constraints And Assumptions

- источником истины по ownership является текущий пользователь `gh`, а не host
  user;
- для определения login используется `gh api user --jq ".login"`;
- `run` остается каноническим issue-level entrypoint и не делегирует этот
  preflight в prompt layer;
- `poll` должен оставаться строгим и выбирать только уже нормализованные issue
  из `Backlog`;
- auto-normalization выполняется только для open issue текущего репозитория;
- add-to-project и set-status уже частично реализованы в коде, поэтому новая
  работа должна не дублировать их, а оформить в единый explicit contract;
- документация должна быть обновлена до или одновременно с кодом.

## User Story

Как оператор, который явно запускает `ai-teamlead run <issue>`, я хочу, чтобы
команда сама доводила open issue до корректного входа в analysis flow
`project + status + assignee`, чтобы не выполнять вручную подготовительные
действия в GitHub и не терять ownership задачи при claim.

## Use Cases

1. Оператор запускает `run` для open issue, которой нет в default project, и
   команда сначала добавляет issue в project, выставляет `Backlog`, назначает
   issue на текущего `gh` user и только потом продолжает analysis flow.
2. Оператор запускает `run` для issue, которая уже находится в project, но у
   нее отсутствует status, и команда только выставляет `Backlog`, не меняя
   существующее назначение.
3. Оператор запускает `run` для issue, у которой уже есть assignee, и команда
   не делает reassign, а продолжает обычный flow.
4. Во время add-to-project, set-status или assign GitHub API возвращает ошибку,
   и `run` завершается явной ошибкой без тихого запуска analysis stage.

## Dependencies

- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
  должен быть обновлен так, чтобы preflight normalization была явно описана как
  часть `run`/claim path;
- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
  и его техническая ось должны отразить GitHub-side side effects до launcher
  orchestration;
- [../../../docs/adr/0021-cli-contract-poll-run-loop.md](../../../docs/adr/0021-cli-contract-poll-run-loop.md)
  и [../../../docs/adr/0024-stage-aware-run-dispatch.md](../../../docs/adr/0024-stage-aware-run-dispatch.md)
  задают существующий contract `run`, который нельзя ломать;
- issue [#14](../14/README.md), если будет реализована отдельно, должна
  учитывать, что manual `run` теперь может выставлять ownership автоматически.
