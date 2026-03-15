# Feature 0007: План реализации

Статус: draft
Последнее обновление: 2026-03-15

## Назначение

Этот план задает порядок изменения launcher contract, при котором
issue-aware имя вкладки становится default для `launch_target = tab`.

## Scope

В план входят:

- новый ADR для смены default contract;
- обновление канонической и summary-документации;
- изменение runtime/config resolution в коде;
- bootstrap synchronization;
- verification и regression coverage.

## Вне scope

В план не входят:

- изменение default `launch_target`;
- re-entry/reuse live tab для одной и той же issue;
- расширение placeholder policy;
- поддержка issue title в tab name.

## Связанные документы

- Feature: [README.md](./README.md)
- Что строим: [01-what-we-build.md](./01-what-we-build.md)
- Как строим: [02-how-we-build.md](./02-how-we-build.md)
- Как проверяем: [03-how-we-verify.md](./03-how-we-verify.md)
- SSOT: [docs/issue-analysis-flow.md](../../issue-analysis-flow.md)
- Config: [docs/config.md](../../config.md)
- Verification: [docs/code-quality.md](../../code-quality.md)
- Базовая launcher feature:
  [Feature 0003](../0003-agent-launch-orchestration/README.md)
- Repo-init templates:
  [Feature 0002](../0002-repo-init/README.md)
- Текущие ADR:
  [ADR-0031](../../adr/0031-zellij-issue-aware-tab-name-template.md),
  [ADR-0032](../../adr/0032-zellij-launch-target-pane-tab.md)

## План изменений документации

Канонические документы, которые нужно обновить:

- новый ADR, который supersede-ит opt-in semantics для `tab` naming;
- [docs/config.md](../../config.md);
- [docs/features/0003-agent-launch-orchestration/02-how-we-build.md](../0003-agent-launch-orchestration/02-how-we-build.md);
- [docs/features/0003-agent-launch-orchestration/03-how-we-verify.md](../0003-agent-launch-orchestration/03-how-we-verify.md);
- [docs/features/0002-repo-init/02-how-we-build.md](../0002-repo-init/02-how-we-build.md).

Summary-документы и шаблоны, которые нужно синхронизировать:

- [README.md](../../../README.md), если launcher defaults описываются на
  repo-level;
- [templates/init/settings.yml](../../../templates/init/settings.yml);
- repo-local generated `./.ai-teamlead/settings.yml` contract description.

Документы, которые сознательно не меняются:

- закрытые issue-level artifacts `specs/issues/47` и `specs/issues/49` остаются
  историческими артефактами;
- runtime artifact docs не требуют отдельной migration note, если resolved
  `tab_name` сохраняется в прежней форме.

## Зависимости и предпосылки

- distinction `pane` vs `tab` уже зафиксирован и не пересматривается;
- изменение должно идти через docs-first contract update;
- headless zellij test path уже существует и остается обязательным для
  integration coverage.

## Порядок работ

### Этап 1. Новый ADR для default tab naming

Цель:

- зафиксировать новый contract layer, где issue-aware naming становится
  default для `tab`-ветки.

Основание:

- [ADR-0031](../../adr/0031-zellij-issue-aware-tab-name-template.md)
- [ADR-0032](../../adr/0032-zellij-launch-target-pane-tab.md)
- [02-how-we-build.md](./02-how-we-build.md)

Результат этапа:

- создан новый ADR;
- в ADR явно описан supersede старой opt-in semantics;
- отдельно зафиксирован explicit opt-out через literal `tab_name_template`.

Проверка:

- manual review ADR на отсутствие противоречий с Feature 0003 и config docs.

### Этап 2. Синхронизация канонической документации

Цель:

- обновить docs так, чтобы новый default был описан одинаково на всех слоях.

Основание:

- [docs/config.md](../../config.md)
- [Feature 0003](../0003-agent-launch-orchestration/README.md)
- [Feature 0002](../0002-repo-init/README.md)

Результат этапа:

- docs больше не обещают `issue-analysis` как default имя отдельной tab issue;
- distinction между `tab_name` и `tab_name_template` объяснен без двусмысленности;
- template semantics синхронизированы с новым runtime default.

Проверка:

- review linked docs;
- поиск по репозиторию не находит старых contradictory формулировок.

### Этап 3. Config и runtime resolution

Цель:

- реализовать новый default в коде без поломки `pane`-path.

Основание:

- новый ADR из этапа 1;
- [02-how-we-build.md](./02-how-we-build.md)

Результат этапа:

- `src/config.rs` / `src/app.rs` используют `#${ISSUE_NUMBER}` как default
  naming source для `tab`;
- `pane` по-прежнему использует `zellij.tab_name`;
- explicit literal override остается рабочим.

Проверка:

- unit tests на resolution logic;
- review runtime manifest behavior.

### Этап 4. Bootstrap templates и generated config guidance

Цель:

- сделать так, чтобы шаблон и runtime вели себя одинаково с точки зрения
  ожиданий пользователя.

Основание:

- [Feature 0002](../0002-repo-init/README.md)
- [docs/config.md](../../config.md)

Результат этапа:

- `templates/init/settings.yml` синхронизирован с новым default;
- generated config guidance не вводит пользователя в заблуждение.

Проверка:

- review template;
- при необходимости smoke-проверка `init`.

### Этап 5. Verification и regression coverage

Цель:

- подтвердить новый default и защитить `pane`-ветку от регрессии.

Основание:

- [03-how-we-verify.md](./03-how-we-verify.md)
- [docs/code-quality.md](../../code-quality.md)

Результат этапа:

- есть unit tests на default и opt-out;
- есть headless integration scenario для `tab` и `pane`;
- regression suite не требует host `zellij`.

Проверка:

- `cargo test`;
- headless integration path.

## Критерий завершения

- issue-aware tab title является default для `launch_target = tab`;
- `pane`-режим сохраняет shared tab `issue-analysis`;
- docs, templates и runtime не противоречат друг другу;
- есть тестовое покрытие на default path и explicit opt-out.

## Открытые вопросы и риски

- не потребует ли изменение отдельного migration note для уже созданных repo
  templates;
- не всплывут ли тесты, завязанные на старую literal строку `issue-analysis`
  в `tab`-ветке.

## Журнал изменений

### 2026-03-15

- создан отдельный план реализации для Feature 0007
- выбран путь через canonical defaulted-by-application поле
  `zellij.tab_name_template`
