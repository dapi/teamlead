# Issue 51: Что строим

Статус: draft
Последнее обновление: 2026-03-14
Статус согласования: pending human review

## Problem

Текущий implementation flow заканчивается на `Waiting for Code Review` и
явно не покрывает merge implementation PR и последующий lifecycle.

Из-за этого после merge возникают пробелы в контракте:

- issue может оставаться `OPEN`, хотя код уже в default branch;
- GitHub Project status не получает канонического terminal state;
- не определено, какой PR считается завершением issue;
- не описан post-merge cleanup для implementation runtime, worktree и branch;
- future release/deploy concerns смешиваются с минимальным post-merge
  завершением задачи.

В результате merge PR не завершает lifecycle детерминированно и оставляет
несколько конкурирующих источников истины: merge commit, состояние issue,
статус project item и локальные runtime artifacts.

## Who Is It For

- владелец репозитория, который хочет видеть детерминированное завершение issue
  после merge implementation PR;
- оператор `ai-teamlead`, который повторно запускает `run` и ожидает, что flow
  понимает post-merge состояние без ручных догадок;
- разработчик, который мержит implementation PR и не хочет вручную разруливать
  issue state, project status и cleanup локальных артефактов;
- сопровождающий `ai-teamlead`, которому нужен явный контракт для runtime,
  finalization и дальнейших follow-up flow.

## Outcome

Нужен минимальный post-merge contract, в котором:

- merge tracked implementation PR считается явным событием завершения coding
  stage;
- `issue-implementation-flow` получает terminal path после merge без создания
  отдельного третьего flow для MVP;
- GitHub Project status получает явный terminal state `Done`;
- issue закрывается как часть post-merge finalization;
- implementation runtime, worktree и local branch очищаются по явному,
  idempotent и best-effort правилу;
- release/deploy/post-merge operations остаются отдельным будущим scope и не
  тормозят базовое завершение issue.

## Scope

В текущую задачу входит:

- расширить существующий post-review lifecycle в `issue-implementation-flow`;
- определить canonical relation между merge tracked PR, issue state и
  GitHub Project status;
- ввести terminal status `Done` для завершенной implementation issue;
- определить, как post-merge path находит tracked implementation PR и
  соответствующий runtime context;
- зафиксировать cleanup contract для implementation session-binding, worktree и
  local branch;
- определить verification strategy для merged path, cleanup и повторных
  запусков;
- явно отделить базовый post-merge contract от будущих release/deploy flow.

## Non-Goals

В текущую задачу не входит:

- автоматический merge PR;
- автоматический deploy или release после merge;
- полноценный новый `issue-post-merge-flow` для MVP;
- поддержка нескольких implementation PR для одной issue;
- очистка всех исторических worktree/branch в репозитории вне tracked
  implementation context;
- переопределение approval или code review contract, уже принятого для
  implementation flow.

## Constraints And Assumptions

- GitHub Project status остается source of truth по lifecycle issue даже после
  merge;
- tracked implementation PR должен определяться явным contract-способом, а не
  эвристикой по любому merge commit в default branch;
- post-merge finalization должна быть idempotent, потому что повторный `run`
  или повторная reconciliation попытка возможны;
- cleanup локальных implementation artifacts не должен удалять versioned
  analysis/implementation docs в `specs/issues/${ISSUE_NUMBER}`;
- best-effort cleanup не должен возвращать issue в активный статус только из-за
  локальной проблемы вроде занятого worktree;
- если проекту позже понадобится release/deploy gate после merge, это должно
  оформляться отдельным flow или отдельным расширением SSOT, а не скрытым
  усложнением базового post-merge contract.

## User Story

Как владелец и оператор `ai-teamlead`, я хочу, чтобы merge implementation PR
переводил issue в детерминированное terminal state с понятным cleanup
поведением, чтобы merged код, issue state, project status и runtime artifacts
не расходились между собой.

## Use Cases

1. PR для issue уже находится в `Waiting for Code Review`, reviewer мержит его,
   и следующий post-merge path закрывает issue, переводит project item в
   `Done` и очищает implementation runtime artifacts.
2. Оператор повторно запускает `run` для issue с уже merged tracked PR и
   получает idempotent terminalization вместо повторного coding path.
3. Remote branch уже удалена GitHub после merge, а local worktree еще есть;
   cleanup удаляет только локальные implementation artifacts и не считает
   отсутствие remote branch ошибкой.
4. Cleanup local worktree не удался из-за блокировки файловой системы; issue и
   project status все равно приходят в terminal business state, а оператор
   получает явную диагностику по cleanup.

## Dependencies

- [../../../docs/issue-implementation-flow.md](../../../docs/issue-implementation-flow.md)
- [../../../docs/features/0004-issue-implementation-flow/README.md](../../../docs/features/0004-issue-implementation-flow/README.md)
- [../../../docs/adr/0024-stage-aware-run-dispatch.md](../../../docs/adr/0024-stage-aware-run-dispatch.md)
- [../../../docs/adr/0025-stage-aware-runtime-bindings.md](../../../docs/adr/0025-stage-aware-runtime-bindings.md)
- [../../../docs/adr/0026-stage-aware-complete-stage.md](../../../docs/adr/0026-stage-aware-complete-stage.md)
- [../5/README.md](../5/README.md)
