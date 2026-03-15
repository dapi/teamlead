# Issue 56: Security review: безопасное использование `ai-teamlead` в публичных репозиториях

Статус: draft
Тип задачи: `feature`
Тип проекта: `infra/platform`
Размер: `large`
Последнее обновление: 2026-03-15

## Контекст

Issue: `Security review: безопасное использование ai-teamlead в публичных репозиториях`

- GitHub: https://github.com/dapi/ai-teamlead/issues/56
- Analysis branch: `analysis/issue-56`
- Session UUID: `33b29674-85e9-4a72-8228-902d346cff39`

Issue требует не локальной правки, а отдельного security contract layer для
сценария `public repo -> GitHub content/repo content/runtime outputs ->
ai-teamlead -> local machine`.

Ключевой результат анализа:

- threat model и contract pack по теме уже зафиксированы в feature
  `0006-public-repo-security`;
- SSOT для hostile input и safe mode уже вынесен в
  [../../../docs/untrusted-input-security.md](../../../docs/untrusted-input-security.md);
- архитектурные решения уже приняты в
  [../../../docs/adr/0029-untrusted-github-content-as-hostile-input.md](../../../docs/adr/0029-untrusted-github-content-as-hostile-input.md)
  и
  [../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md](../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md);
- для этой issue implementation-ready handoff состоит не в повторном описании
  threat model, а в привязке существующего contract pack к конкретному
  runtime rollout внутри `ai-teamlead`.

## Артефакты

## Что строим

- [01-what-we-build.md](./01-what-we-build.md)

## Как строим

- [02-how-we-build.md](./02-how-we-build.md)

## Как проверяем

- [03-how-we-verify.md](./03-how-we-verify.md)

## План имплементации

- [04-implementation-plan.md](./04-implementation-plan.md)

## Related Context

- [../../../docs/features/0006-public-repo-security/README.md](../../../docs/features/0006-public-repo-security/README.md)
- [../../../docs/untrusted-input-security.md](../../../docs/untrusted-input-security.md)
- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../docs/issue-implementation-flow.md](../../../docs/issue-implementation-flow.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [../../../docs/features/0005-agent-flow-integration-testing/README.md](../../../docs/features/0005-agent-flow-integration-testing/README.md)
- [../../../docs/adr/0029-untrusted-github-content-as-hostile-input.md](../../../docs/adr/0029-untrusted-github-content-as-hostile-input.md)
- [../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md](../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md)

## Вывод анализа

Информации в issue и уже созданных канонических документах достаточно, чтобы
готовить implementation plan без дополнительных вопросов пользователю.

Предлагаемый implementation baseline:

- трактовать GitHub content, repo-local docs, runtime output и external content
  как hostile-by-default input в public-repo сценарии;
- вычислять `repo_visibility` и `operating_mode` до запуска действий с риском;
- вводить `public-safe` режим как fail-closed baseline для `public` и
  `unknown` visibility;
- ограничивать auto-intake policy для public repos и не считать comments
  trusted даже внутри owner-authored issue;
- пропускать filesystem/network/execution/publication actions через явные
  permission gates;
- выровнять runtime enforcement, project-local prompts, launcher и диагностику
  под единый security contract.

## Open Questions

Блокирующих вопросов по текущему issue не выявлено.

Неблокирующие решения, которые должны быть зафиксированы по ходу реализации:

- где хранить каноническую config schema для security policy до отдельного
  config ADR;
- как именно runtime определяет `repo_visibility` и какой fallback используется
  при частичной недоступности GitHub metadata;
- насколько глубоко нужно ограничивать repo-local assets в `public-safe`
  режиме на первом implementation этапе.
