# Feature 0004: Как строим

## Архитектура

Решение строится из следующих частей:

- stage-aware dispatcher внутри `run`;
- отдельный SSOT `issue-implementation-flow`;
- stage-scoped runtime/session-binding для implementation stage;
- implementation launcher path с отдельным branch/worktree lifecycle;
- stage-aware finalization command;
- GitHub adapter для project status, PR и CI-related checks.

Архитектурный принцип:

- top-level entrypoint общий;
- flow-контракты отдельные;
- launcher/runtime/finalization выбираются по stage.

## Данные и состояния

Ключевые входные данные:

- текущий project status issue;
- approved SDD-комплект в `specs/issues/${ISSUE_NUMBER}/`;
- repo-local config implementation stage;
- runtime binding текущего stage.

Ключевые состояния implementation stage:

- `Ready for Implementation`
- `Implementation In Progress`
- `Waiting for CI`
- `Waiting for Code Review`
- `Done`
- `Implementation Blocked`

В runtime должны различаться:

- `analysis` binding;
- `implementation` binding;
- branch/worktree и launcher context для каждого stage;
- optional cache/diagnostic metadata без semantic роли source of truth.

## Интерфейсы

Внешние интерфейсы:

- `gh` CLI для project status, PR и checks;
- Git для branch/worktree lifecycle;
- `zellij` для stage-specific pane/session context;
- test runner проекта для локальных проверок.

Внутренние интерфейсы:

- stage dispatcher в `run`;
- runtime binding store;
- implementation launcher;
- finalization handler;
- approval metadata reader для analysis artifacts;
- GitHub PR reader для merge detection и issue close.

## Follow-up acceptance 2026-03-15

Принятый
[ADR-0028](../../adr/0028-github-first-reconcile-and-runtime-cache-only.md)
зафиксировал:

- source of truth по lifecycle остается в GitHub Project;
- implementation PR определяется по canonical branch contract;
- runtime хранит execution/cache metadata, но не semantic state issue.

## Технические решения

Ключевые решения feature:

- `run` не делится на несколько top-level команд, а становится stage-aware
  dispatcher;
- approved analysis artifacts являются обязательным входом реализации;
- implementation stage получает собственный naming contract для branch/worktree;
- runtime binding обобщается до stage-aware модели;
- `internal complete-stage` расширяется до stage-aware finalization с
  implementation outcomes;
- post-merge path требует явного branch contract и GitHub-first reconcile;
- issue может одновременно иметь history analysis stage и активный
  implementation binding без конфликта.

## Конфигурация

Минимальный versioned config contract:

```yaml
issue_implementation_flow:
  statuses:
    ready_for_implementation: "Ready for Implementation"
    implementation_in_progress: "Implementation In Progress"
    waiting_for_ci: "Waiting for CI"
    waiting_for_code_review: "Waiting for Code Review"
    done: "Done"
    implementation_blocked: "Implementation Blocked"

launch_agent:
  implementation_branch_template: "implementation/issue-${ISSUE_NUMBER}"
  implementation_worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
  implementation_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
```

## Ограничения реализации

- cleanup после merge остается best-effort и не должен откатывать `Done`;
- CI gating может опираться на `gh pr checks`, а не на собственный GitHub API
  client;
- для MVP допускается переиспользование части analysis launcher logic, но не
  через скрытый stage-agnostic god-script;
- migration существующих runtime-файлов должна быть совместима с уже созданными
  analysis session artifacts;
- локальный runtime может отсутствовать или быть неполным без потери
  восстановимости implementation state.
