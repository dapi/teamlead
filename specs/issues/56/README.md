# Issue 56: Security review: безопасное использование `ai-teamlead` в публичных репозиториях

Статус: draft
Тип задачи: `feature`
Тип проекта: `infra/platform`
Размер: `large`
Последнее обновление: 2026-03-15

## Issue

Issue: `Security review: безопасное использование ai-teamlead в публичных репозиториях`

- GitHub: https://github.com/dapi/ai-teamlead/issues/56
- Analysis branch: `analysis/issue-56`
- Session UUID: `33b29674-85e9-4a72-8228-902d346cff39`

Issue требует не локальной правки, а отдельного security contract layer для
сценария `public repo -> GitHub content/repo content/runtime outputs ->
ai-teamlead -> local machine`.

Этот пакет является issue-level implementation handoff поверх уже существующих
канонических документов:

- feature `0006-public-repo-security`;
- SSOT [../../../docs/untrusted-input-security.md](../../../docs/untrusted-input-security.md);
- ADR `0029` и `0030`.

Формулировка `Security review` относится к типу исследовательской задачи, но
результатом этого issue должен быть именно task-specific implementation contract
для runtime. Канонический feature-level слой остается в `docs/features/0006-*`,
а `specs/issues/56/*` конкретизирует runtime rollout и verification для issue
`#56`.

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

## Summary

Информации в issue и уже созданных канонических документах достаточно, чтобы
готовить implementation plan без дополнительных вопросов пользователю.

Предлагаемый implementation baseline:

- трактовать GitHub content, repo-local docs, runtime output и external content
  как hostile-by-default input в public-repo сценарии;
- вычислять `repo_visibility` и `operating_mode` до запуска действий с риском;
- вводить `public-safe` режим как fail-closed baseline для `public` и
  `unknown` visibility;
- использовать в MVP единственный trusted approval channel: явный ответ
  оператора в agent session, привязанный к конкретному risky action;
- ограничивать auto-intake policy для public repos и не считать comments
  trusted даже внутри owner-authored issue; явный `run` допускает
  `manual-override`, но не ослабляет security policy;
- фиксировать policy-матрицу для `filesystem`, `network`, `execution` и
  `publication`, включая `allow`, `approval` и `deny`;
- пропускать filesystem/network/execution/publication actions через явные
  permission gates;
- выровнять runtime enforcement, project-local prompts, launcher и диагностику
  под единый security contract.

Связанный канонический контекст:

- [../../../docs/features/0006-public-repo-security/README.md](../../../docs/features/0006-public-repo-security/README.md)
- [../../../docs/untrusted-input-security.md](../../../docs/untrusted-input-security.md)
- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../docs/issue-implementation-flow.md](../../../docs/issue-implementation-flow.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [../../../docs/features/0005-agent-flow-integration-testing/README.md](../../../docs/features/0005-agent-flow-integration-testing/README.md)
- [../../../docs/adr/0029-untrusted-github-content-as-hostile-input.md](../../../docs/adr/0029-untrusted-github-content-as-hostile-input.md)
- [../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md](../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md)

## Status

Issue-level пакет считается implementation-ready, потому что для текущего scope
уже зафиксированы временные operational contracts:

- source of truth для runtime security contract:
  SSOT `untrusted-input-security` + ADR `0029/0030` + этот issue-level handoff;
- `repo_visibility` fallback:
  GitHub visibility metadata -> локальная repo metadata -> `unknown`;
- self-hosted trust boundary:
  локальный contract layer установленного `ai-teamlead` trusted только как
  launcher/control plane до чтения task inputs; repo-local content target
  задачи после входа в task input остается hostile-by-default;
- private baseline:
  `repo_visibility = private` по умолчанию остается в `standard`, если
  repo-level config явно не включает `force-public-safe`;
- MVP approval contract:
  approval только через agent session, one-shot, action-bound, session-bound,
  с binding к `action_kind` и `target_fingerprint`;
- publication boundary:
  внешние publish sinks в MVP остаются `deny-by-default`, а approval допустим
  только для канонического GitHub workflow внутри заранее определенного sink-а.

Flow status для issue:

- analysis stage завершен с outcome `plan-ready`;
- issue находится в `Waiting for Plan Review`;
- этот пакет является handoff для human review и последующего implementation
  flow.

## Artifacts

- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [04-implementation-plan.md](./04-implementation-plan.md)

## Open Questions

Неблокирующие follow-up вопросы, которые не блокируют implementation этой issue:

- вынос полной security config schema в отдельный config ADR;
- возможный additional trusted approval mechanism сверх agent session;
- дальнейшее ужесточение repo-local assets в self-hosted/dogfooding path.
