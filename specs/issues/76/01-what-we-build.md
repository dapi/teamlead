# Issue 76: Что строим

Статус: draft
Последнее обновление: 2026-03-15

## Problem

Сейчас `internal complete-stage` совмещает два разных execution/trust слоя:

- sandboxed agent session внутри issue worktree;
- privileged host-side операции, которые меняют git state, push-ят ветку,
  создают или обновляют PR и двигают GitHub Project status.

Это приводит к двум системным проблемам:

1. `workspace-write` и linked worktree плохо совместимы с прямым `git` path из
   agent session, потому что индекс и часть metadata лежат в общем gitdir вне
   writable scope sandbox-а.
2. Для public repo и hostile-input model privileged side effects выполняются из
   среды, которая обрабатывает недоверенный контент, что ухудшает security
   posture и противоречит направлению feature `0006`.

## Who Is It For

- для владельца репозитория и оператора `ai-teamlead`, которому нужен
  предсказуемый и безопасный lifecycle завершения stage;
- для analysis/implementation agent session, которой нужен узкий и
  воспроизводимый completion contract без прямого доступа к host-side side
  effects;
- для будущего `public-safe` operating mode, которому нужна четкая граница
  между sandboxed обработкой контента и trusted публикацией результатов.

## Outcome

После изменения stage completion должен работать так:

- агент внутри worktree создает и обновляет только task artifacts;
- агент подает структурированный сигнал завершения stage с `stage`, `outcome`
  и кратким сообщением;
- trusted host-side supervisor валидирует сигнал и выполняет только разрешенный
  набор privileged действий;
- analysis и implementation flow остаются end-to-end рабочими, replayable и
  совместимыми с `workspace-write`.

## Scope

В scope текущего изменения входят:

- новый completion contract между agent session и host-side supervisor;
- разделение agent-side signal writing и host-side finalization execution;
- trusted validation слоя для branch, artifacts path, stage, outcome и
  допустимых side effects;
- failure/retry semantics для частичных ошибок commit/push/PR/status update;
- audit trail и operator-visible diagnostics для supervisor path;
- синхронизация analysis/implementation SSOT, feature-docs и ADR;
- явная связь решения с security baseline для public repos и issue `#56`.

## Non-Goals

Вне scope текущего изменения:

- отдельный третий flow поверх analysis/implementation только ради supervisor-а;
- полный redesign `run`/`poll`/`loop` orchestration beyond finalization path;
- перенос всей runtime state из `.git/.ai-teamlead/` в worktree;
- общий daemon для произвольных background jobs вне stage finalization;
- полный security hardening всех остальных high-risk actions, не связанных с
  `complete-stage`.

## Constraints And Assumptions

- GitHub Project status остается единственным semantic source of truth по
  lifecycle issue;
- analysis и implementation продолжают использовать один stage-aware finalization
  vocabulary;
- prompt-слой не должен снова знать детали `git add`, `git commit`, `git push`,
  `gh pr create`, `gh pr ready` или project transitions;
- worktree-local transport допустим только как transient mailbox, а не как
  новый durable source of truth;
- zellij-related проверки и regression tests должны оставаться в headless path,
  а не в host session пользователя;
- если supervisor не смог применить request, оператор должен иметь безопасный
  replay path без ручного восстановления hidden state.

## Motivation

- вернуть `workspace-write` в роль жизнеспособного default sandbox path для
  `codex`;
- перестать оправдывать `danger-full-access` только необходимостью дописать
  git metadata и выполнить `gh`;
- вынести privileged side effects из окружения, которое читает hostile input;
- сделать completion behavior детерминированным, auditable и пригодным для
  `public-safe` режима.

## Operational Goal

Оператор должен получать такой runtime contract:

- агент завершает stage одной командой без прямого `git`/`gh`;
- host-side supervisor отрабатывает после завершения agent session и либо
  доводит lifecycle до целевого статуса, либо оставляет request и диагностику
  для повторного запуска;
- частичный сбой не теряет artifacts и не требует угадывать, какой шаг уже был
  выполнен.

## Dependencies

- [../../../docs/adr/0020-agent-session-completion-signal.md](../../../docs/adr/0020-agent-session-completion-signal.md)
- [../../../docs/adr/0026-stage-aware-complete-stage.md](../../../docs/adr/0026-stage-aware-complete-stage.md)
- [../../../docs/adr/0028-github-first-reconcile-and-runtime-cache-only.md](../../../docs/adr/0028-github-first-reconcile-and-runtime-cache-only.md)
- [../../../docs/adr/0029-untrusted-github-content-as-hostile-input.md](../../../docs/adr/0029-untrusted-github-content-as-hostile-input.md)
- [../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md](../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md)
- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
- [../../../docs/features/0004-issue-implementation-flow/README.md](../../../docs/features/0004-issue-implementation-flow/README.md)
- [../../../docs/features/0006-public-repo-security/README.md](../../../docs/features/0006-public-repo-security/README.md)
- issue `#56` как связанный security baseline для public repos
