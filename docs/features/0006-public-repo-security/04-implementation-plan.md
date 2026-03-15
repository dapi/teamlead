# Feature 0006: План имплементации

Статус: draft
Последнее обновление: 2026-03-14

## Назначение

Этот план задает порядок работ для ввода security baseline, необходимого для
безопасного использования `ai-teamlead` в публичных репозиториях.

## Scope

В scope плана входят:

- threat model и canonical contract layer;
- safe mode и permission gates;
- runtime enforcement baseline;
- verification для hostile inputs.

## Связанные документы

- GitHub issue: <https://github.com/dapi/ai-teamlead/issues/56>
- [README.md](../../../README.md)
- [docs/code-quality.md](../../code-quality.md)
- [docs/untrusted-input-security.md](../../untrusted-input-security.md)
- [docs/issue-analysis-flow.md](../../issue-analysis-flow.md)
- [docs/issue-implementation-flow.md](../../issue-implementation-flow.md)
- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [docs/adr/0027-untrusted-github-content-as-hostile-input.md](../../adr/0027-untrusted-github-content-as-hostile-input.md)
- [docs/adr/0028-public-repo-safe-mode-and-permission-gates.md](../../adr/0028-public-repo-safe-mode-and-permission-gates.md)

## Зависимости и предпосылки

- поддержка public repos не считается production-ready без минимального safe
  mode;
- policy должна быть закреплена в документации раньше или одновременно с code
  enforcement;
- enforcement нельзя полностью делегировать prompt-тексту или operator memory.

## Порядок работ

### Этап 1. Threat model и contract pack

Цель:
создать канонический набор документов, который фиксирует hostile-input model,
trust boundaries и минимальные safe-mode правила.

Основание:
[docs/untrusted-input-security.md](../../untrusted-input-security.md),
[docs/adr/0027-untrusted-github-content-as-hostile-input.md](../../adr/0027-untrusted-github-content-as-hostile-input.md),
[docs/adr/0028-public-repo-safe-mode-and-permission-gates.md](../../adr/0028-public-repo-safe-mode-and-permission-gates.md).

Результат этапа:
документация позволяет без догадок восстановить, какие входы считаются
недоверенными и какие действия требуют human gate.

Проверка:
manual review на отсутствие противоречий между feature docs, SSOT и ADR.

### Этап 2. Runtime visibility и mode resolution

Цель:
добавить детерминированный механизм определения `repo_visibility` и выбора
`operating_mode`.

Основание:
[02-how-we-build.md](./02-how-we-build.md),
[docs/adr/0028-public-repo-safe-mode-and-permission-gates.md](../../adr/0028-public-repo-safe-mode-and-permission-gates.md).

Результат этапа:
каждый запуск имеет вычисленный security mode, причем `unknown` visibility не
ослабляет политику.

Проверка:
unit и integration tests для `public`, `private` и `unknown` visibility.

### Этап 3. Permission gates

Цель:
внедрить enforcement для чтения/записи вне repo, network access, dangerous
execution и публикации потенциально чувствительных данных.

Основание:
[docs/untrusted-input-security.md](../../untrusted-input-security.md),
[03-how-we-verify.md](./03-how-we-verify.md),
[docs/code-quality.md](../../code-quality.md).

Результат этапа:
high-risk actions проходят через явный approval path или детерминированный
запрет.

Проверка:
abuse-case сценарии с hostile issue/comments и smoke tests для escalation path.

### Этап 4. Prompt и launcher alignment

Цель:
выровнять project-local prompts, launcher assets и runtime messaging с новым
security contract.

Основание:
[docs/issue-analysis-flow.md](../../issue-analysis-flow.md),
[docs/issue-implementation-flow.md](../../issue-implementation-flow.md),
[docs/features/0003-agent-launch-orchestration/README.md](../0003-agent-launch-orchestration/README.md).

Результат этапа:
prompt-layer не противоречит runtime policy и явно различает operator intent и
hostile content.

Проверка:
review сценариев, в которых GitHub-текст пытается замаскироваться под команды
оператора.

## Критерий завершения

План можно считать завершенным, когда:

- `public-safe` режим документирован и enforce-ится хотя бы для high-risk
  действий;
- visibility resolution и permission gates покрыты тестами;
- operator видит, почему действие было запрещено или требует approval;
- README и связанные документы ссылаются на канонический security layer.

## Открытые вопросы и риски

- где должен жить source of truth для security config schema до появления
  отдельного config ADR;
- насколько глубоко нужно ограничивать repo-local assets в public-safe режиме;
- потребуется ли отдельный secure runner для полной изоляции zellij- и
  launcher-based сценариев.

## Журнал изменений

### 2026-03-14

- создан начальный план имплементации для security baseline публичных
  репозиториев
