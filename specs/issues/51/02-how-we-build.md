# Issue 51: Как строим

Статус: draft
Последнее обновление: 2026-03-14
Статус согласования: pending human review

## Approach

Post-merge lifecycle расширяет существующий `issue-implementation-flow`, а не
создает отдельный третий flow для MVP.

Технический подход:

- расширить `docs/issue-implementation-flow.md` terminal section после
  `Waiting for Code Review`;
- добавить в implementation lifecycle terminal project status `Done`;
- хранить identity tracked implementation PR и workspace metadata в
  stage-aware runtime binding или в явно связанном session metadata;
- при повторном `run` или другом explicit reconciliation path проверять,
  merged ли tracked PR;
- выполнять terminal finalization через stage-aware internal command,
  добавив implementation outcome `merged`;
- в рамках `merged` finalization закрывать issue, переводить project item в
  `Done` и запускать idempotent best-effort cleanup implementation artifacts;
- оставить release/deploy semantics за пределами текущего SSOT и оформлять их
  отдельно, если появится реальная потребность.

Этот вариант удерживает один канонический implementation lifecycle и не
размазывает merge semantics между несколькими слабо связанными flow.

## Affected Areas

- `docs/issue-implementation-flow.md`
  нужно расширить scope, статусы, переходы и finalization path после merge;
- `docs/features/0004-issue-implementation-flow/*`
  нужно синхронизировать product, technical и verification оси;
- `README.md`
  должен отражать новый terminal state только как repo-level summary;
- `./.ai-teamlead/settings.yml`
  потребуется расширить новым status mapping для `Done`;
- `src/domain.rs`
  должен валидировать post-merge allowed transitions и re-entry guards;
- `src/runtime.rs`
  должен сохранить metadata tracked implementation PR и cleanup-relevant
  workspace fields;
- `src/app.rs`
  должен различать обычный implementation re-entry и post-merge reconciliation;
- `src/complete_stage.rs`
  должен получить implementation outcome `merged` и post-merge cleanup path;
- GitHub adapter layer
  должен уметь читать merged state tracked PR и закрывать issue после terminal
  finalization;
- integration tests и docs
  должны покрывать merged path, idempotency и cleanup diagnostics.

## Interfaces And Data

### Tracked implementation PR

Post-merge path не должен опираться на эвристику вроде "найти любой merged PR,
где упоминается issue".

Минимальный contract identity:

- implementation runtime хранит `pr_number` или другой устойчивый PR identifier;
- рядом сохраняются `branch`, `worktree_root` и stage, к которым привязан этот
  PR;
- finalization читает именно tracked PR, созданный или переиспользованный в
  `ready-for-ci` path;
- если tracked PR отсутствует, issue не закрывается автоматически, а попадает в
  blocker/manual reconciliation path.

### Status model

Минимальная status model после изменения:

- `Ready for Implementation`
- `Implementation In Progress`
- `Waiting for CI`
- `Waiting for Code Review`
- `Done`
- `Implementation Blocked`

Ключевые переходы post-merge части:

- `Waiting for Code Review` -> `Done` при подтвержденном merge tracked PR;
- `Waiting for Code Review` -> `Implementation In Progress` если review вернул
  issue в работу без merge;
- `Implementation Blocked` остается fallback для технических проблем до merge;
- post-merge cleanup не вводит отдельный долгоживущий статус, потому что это
  operational finalization, а не самостоятельный пользовательский stage.

### Finalization surface

Для единообразия с ADR-0026 предлагается расширить vocabulary
`internal complete-stage`:

```text
ai-teamlead internal complete-stage <session_uuid> \
  --stage implementation \
  --outcome merged \
  --message "implementation PR merged"
```

Семантика `merged`:

- подтверждает, что tracked implementation PR merged в default branch;
- не делает новый commit, push или PR create, потому что код уже зафиксирован
  и смержен ранее;
- переводит project item в `Done`;
- закрывает GitHub issue;
- помечает implementation session как completed;
- запускает best-effort cleanup runtime/worktree/local branch;
- выводит диагностику по cleanup без отката terminal business result.

### Cleanup contract

Cleanup должен быть привязан только к implementation artifacts этой issue.

Минимальный cleanup path:

- удалить или пометить как завершенный implementation session-binding;
- удалить implementation worktree, если он еще существует и не содержит
  незакоммиченных локальных изменений;
- удалить local implementation branch, если она уже merged и не используется
  другим worktree;
- не удалять `specs/issues/${ISSUE_NUMBER}` и другие versioned artifacts;
- не требовать удаления remote branch как обязательного условия успеха:
  GitHub auto-delete branch policy или ручная политика репозитория допустимы
  отдельно.

Если cleanup какого-то шага не удался:

- issue и project status не откатываются назад;
- команда пишет явный warning/diagnostic;
- повторный запуск cleanup остается безопасным.

## Configuration And Runtime Assumptions

Минимальное расширение project-local config:

```yaml
issue_implementation_flow:
  statuses:
    ready_for_implementation: "Ready for Implementation"
    implementation_in_progress: "Implementation In Progress"
    waiting_for_ci: "Waiting for CI"
    waiting_for_code_review: "Waiting for Code Review"
    done: "Done"
    implementation_blocked: "Implementation Blocked"
```

Допущения runtime:

- stage-aware runtime schema можно расширять без ломки analysis binding;
- implementation metadata должна хранить минимум PR identity и workspace
  coordinates для cleanup;
- repeated post-merge reconciliation может происходить уже после удаления
  worktree или branch и должна считаться валидной;
- session status `completed` недостаточен без project status `Done`, поэтому
  оба слоя должны синхронизироваться явно.

## Risks

- существующие issues в `Waiting for Code Review` могут не иметь сохраненного
  `pr_number`, и им потребуется ручная one-off reconciliation;
- если tracked PR перепутан с другим PR той же issue, можно закрыть issue по
  неверному merge событию;
- cleanup local worktree/branch зависит от состояния файловой системы и других
  worktree, поэтому требует аккуратной идемпотентной деградации;
- репозитории, которым нужен deploy/release gate перед закрытием issue, не
  смогут использовать `Done` как конечный terminal state без follow-up
  расширения.

## External Interfaces

- GitHub Pull Request API/CLI
  нужен для чтения `state`, `mergedAt`, base/head branch и identity tracked PR;
- GitHub Issue API/CLI
  нужен для закрытия issue после terminal finalization;
- GitHub Project status update
  остается каноническим lifecycle transition;
- Git
  используется для проверки merged local branch и cleanup worktree/branch;
- runtime artifacts в `.git/.ai-teamlead/`
  остаются техническим storage для session/binding metadata.

## Architecture Notes

### Почему не отдельный `issue-post-merge-flow`

Для MVP post-merge path не является новым долгоживущим операторским stage.
Это terminal reconciliation существующего implementation lifecycle.

Отдельный third flow сейчас дал бы:

- новую статусную модель;
- новый prompt entrypoint;
- лишнюю связанность вокруг merge event, который пока не требует отдельного
  human gate.

Поэтому минимальный осознанный компромисс: расширить implementation SSOT и
оставить отдельный post-merge flow только как будущую опцию для release/deploy.

### Cleanup как best-effort, а не как blocker

Merged код в default branch уже завершает основную бизнес-цель issue.

Следовательно:

- cleanup локальных runtime/worktree artifacts важен;
- но он не должен оставлять issue навсегда в `Waiting for Code Review` только
  потому, что какой-то локальный worktree занят;
- диагностика обязательна, silent failure недопустим.

## ADR Impact

По правилам [../../../docs/documentation-process.md](../../../docs/documentation-process.md)
это новое значимое решение уровня flow и runtime.

Нужен как минимум один новый ADR, который зафиксирует:

- что post-merge lifecycle становится частью `issue-implementation-flow`;
- что `Done` является terminal project status после merge tracked PR;
- что post-merge finalization закрывает issue и выполняет best-effort cleanup;
- что tracked PR identity должна храниться в runtime contract.

Также потребуется синхронизация существующих ADR:

- `ADR-0025`, если меняется runtime schema;
- `ADR-0026`, если `complete-stage` получает outcome `merged`.

## Alternatives Considered

### 1. Отдельный `issue-post-merge-flow`

Отклонено для MVP.

Это усложняет lifecycle и создает новый operator-facing stage без достаточной
пользы для минимального post-merge contract.

### 2. Закрывать issue только по merge PR без обновления project status

Отклонено.

Это нарушает правило GitHub Project status как source of truth по lifecycle.

### 3. Делать cleanup обязательным условием перехода в `Done`

Отклонено.

Operational cleanup может не получиться по локальным причинам и не должен
отменять факт успешного merge.

## Migration Or Rollout Notes

- сначала нужно обновить SSOT, feature docs и ADR, затем код;
- GitHub Project и `settings.yml` должны быть синхронизированы новым статусом
  `Done`;
- уже merged issues, зависшие в `Waiting for Code Review`, потребуют отдельного
  ручного reconcile script или операторской инструкции;
- rollout должен включать regression coverage для analysis и pre-merge
  implementation paths, чтобы post-merge logic не сломала текущий MVP.
