# Issue 76: вынести `complete-stage` git/gh side effects в host-side supervisor

Статус: draft
Тип задачи: `chore`
Тип проекта: `infra/platform`
Размер: `medium`
Последнее обновление: 2026-03-15

## Контекст

Issue: `Архитектура: вынести complete-stage git/gh side effects в host-side supervisor`

- GitHub: https://github.com/dapi/ai-teamlead/issues/76
- Analysis branch: `analysis/issue-76`
- Session UUID: `9e388d33-2c29-4ccd-addd-9b07c1607e2c`

Текущий контракт `internal complete-stage` был принят как единая точка
finalization для analysis и implementation stage, но фактически выполняет
trusted host-side операции из той же agent session, которая работает с
недоверенным issue-контентом и может быть ограничена sandbox-режимом
`workspace-write`.

Для linked worktree это создает конфликт с общей git metadata в
`.git/worktrees/...`: агент может подготовить артефакты в worktree, но не иметь
надежного права записать индекс, commit, push или выполнить `gh`-операции.

Цель анализа: зафиксировать новый архитектурный контракт, в котором agent
session подает только структурированный completion signal, а все privileged
git/gh/project side effects выполняются отдельным host-side supervisor path.

## Артефакты

## Что строим

- [01-what-we-build.md](./01-what-we-build.md)

## Как строим

- [02-how-we-build.md](./02-how-we-build.md)

## Как проверяем

- [03-how-we-verify.md](./03-how-we-verify.md)

## План имплементации

- [04-implementation-plan.md](./04-implementation-plan.md)

## Связанный контекст

- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../docs/issue-implementation-flow.md](../../../docs/issue-implementation-flow.md)
- [../../../docs/untrusted-input-security.md](../../../docs/untrusted-input-security.md)
- [../../../docs/implementation-plan.md](../../../docs/implementation-plan.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
- [../../../docs/features/0004-issue-implementation-flow/README.md](../../../docs/features/0004-issue-implementation-flow/README.md)
- [../../../docs/features/0006-public-repo-security/README.md](../../../docs/features/0006-public-repo-security/README.md)
- [../../../docs/adr/0020-agent-session-completion-signal.md](../../../docs/adr/0020-agent-session-completion-signal.md)
- [../../../docs/adr/0026-stage-aware-complete-stage.md](../../../docs/adr/0026-stage-aware-complete-stage.md)
- [../../../docs/adr/0028-github-first-reconcile-and-runtime-cache-only.md](../../../docs/adr/0028-github-first-reconcile-and-runtime-cache-only.md)
- [../../../docs/adr/0029-untrusted-github-content-as-hostile-input.md](../../../docs/adr/0029-untrusted-github-content-as-hostile-input.md)
- [../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md](../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md)
- [../51/README.md](../51/README.md)

## Вывод анализа

Информации в issue достаточно, чтобы готовить план реализации без
дополнительных вопросов пользователю.

Предлагаемый контракт:

- agent-side `complete-stage` перестает выполнять privileged side effects и
  становится механизмом записи структурированного completion request;
- host-side supervisor, запущенный вне sandbox агента, валидирует request
  against trusted runtime manifest и только после этого выполняет `git`, `gh`
  и GitHub Project transitions;
- trusted runtime state остается в `.git/.ai-teamlead/`, а worktree-local
  mailbox используется только как transport layer между sandbox и host;
- analysis и implementation сохраняют единый stage-aware completion contract,
  но получают replayable supervisor path, audit trail и явные failure semantics;
- изменение требует обновления SSOT и нового ADR, который supersede-ит
  privileged-execution часть ADR-0020 и уточняет границу ADR-0026.

Блокирующих вопросов по текущему issue не выявлено.

## Журнал изменений

### 2026-03-15

- создан начальный analysis package для issue `#76`
