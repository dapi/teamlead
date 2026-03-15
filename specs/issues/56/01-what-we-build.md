# Issue 56: Что строим

Статус: draft
Последнее обновление: 2026-03-15

## Problem

`ai-teamlead` запускается локально на машине пользователя, но значительную часть
контекста получает из GitHub issue, comments, файлов репозитория, linked
artifacts и runtime output.

Для public repos это создает базовый security-риск:

- внешний автор может встроить prompt injection в issue или comments;
- repo-local docs и project-local assets могут маскировать hostile input под
  "нормальные инструкции";
- shell output, test logs и generated artifacts могут стать вторичным каналом
  injection;
- агент может быть склонен к выполнению нежелательных команд, чтению
  чувствительных файлов или публикации локальных данных наружу.

Проблема уже достаточно исследована на уровне документации, но текущему
runtime все еще нужен явный implementation baseline, который переведет принятый
security contract в детерминированное поведение `run`, `poll`, launcher-а,
shell layer и publication path.

## Who Is It For

- владелец `ai-teamlead`, который хочет безопасно использовать workflow в
  public repos;
- оператор, который запускает `run` или `poll` и должен понимать, когда
  действие запрещено или требует approval;
- разработчик `ai-teamlead`, который реализует security policy в runtime,
  launcher и prompts;
- reviewer, которому нужен проверяемый baseline вместо набора неявных
  предосторожностей.

## Outcome

Нужен implementation-ready security baseline, в котором:

- все основные hostile input paths уже признаны частью `untrusted input`;
- существует обязательный `public-safe` режим для `public` и `unknown`
  visibility;
- auto-intake для public repos ограничивается отдельной policy и не делает
  весь thread trusted;
- high-risk filesystem, network, execution и publication actions проходят через
  явные permission gates;
- runtime, documentation, prompts и operator diagnostics не противоречат друг
  другу в трактовке trust boundaries.

## Scope

В текущую issue входит:

- привязать существующий contract pack по security к issue-specific plan
  внедрения;
- зафиксировать implementation baseline для `repo_visibility`,
  `operating_mode`, `intake_policy` и `approval_state`;
- определить первые runtime enforcement points в `run`, `poll`, GitHub layer,
  shell execution и publication paths;
- определить минимальный safe mode для public repos и для случаев, когда
  visibility не удалось определить;
- определить verification strategy для hostile inputs, prompt injection,
  execution abuse и data exfiltration paths;
- выровнять implementation path с уже существующими feature docs, SSOT и ADR.

## Non-Goals

В текущую issue не входит:

- полная sandbox-платформа для произвольных agent providers и сторонних tools;
- гарантия безопасности при скомпрометированной локальной машине пользователя;
- автоматическая санация всего hostile content до передачи в агент;
- формальная security-модель всех внешних LLM providers;
- замена explicit operator intent любыми GitHub comments или repo-local
  инструкциями.

## Constraints And Assumptions

- `public` repo и `unknown` visibility должны трактоваться fail-closed;
- issue body, comments, linked PR/issues, repo files, shell output и generated
  artifacts не являются trusted control plane;
- owner-authored issue допустима как intake gate, но не как trust upgrade для
  comments;
- documentation layer уже является частью решения, поэтому реализация должна
  следовать существующим SSOT и ADR, а не переписывать их молча;
- verification для сценариев, связанных с `zellij`, должна идти только в
  headless-окружении;
- поддержка public repos не считается production-ready, пока high-risk actions
  не проходят через enforce-able policy.

## User Story

Как оператор и владелец `ai-teamlead`, я хочу запускать workflow на public
repos только в режиме, где hostile input не может сам расширить filesystem,
network или execution privileges, чтобы issue/comments/repo content оставались
источником данных для анализа, а не скрытым каналом управления моей локальной
машиной.

## Use Cases

1. Оператор запускает `ai-teamlead run <issue>` для issue из public repo, и
   runtime автоматически включает `public-safe` режим до любых опасных
   действий.
2. `poll` находит public issue, созданную не владельцем и не allowlist-автором,
   и не берет ее в auto-intake path.
3. Comment внутри owner-authored issue предлагает "безопасную" shell-команду,
   но runtime не исполняет ее автоматически только из-за текста comment.
4. Репозиторий содержит `AGENTS.md` или docs с инструкциями открыть внешнюю
   ссылку, но repo-local content не повышает permission scope без отдельного
   trusted mechanism.
5. Visibility репозитория определить не удалось, и runtime остается в
   `public-safe`, а не ослабляет ограничения.

## Dependencies

- [../../../docs/features/0006-public-repo-security/README.md](../../../docs/features/0006-public-repo-security/README.md)
- [../../../docs/untrusted-input-security.md](../../../docs/untrusted-input-security.md)
- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../docs/issue-implementation-flow.md](../../../docs/issue-implementation-flow.md)
- [../../../docs/features/0005-agent-flow-integration-testing/README.md](../../../docs/features/0005-agent-flow-integration-testing/README.md)
- [../../../docs/adr/0029-untrusted-github-content-as-hostile-input.md](../../../docs/adr/0029-untrusted-github-content-as-hostile-input.md)
- [../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md](../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md)
