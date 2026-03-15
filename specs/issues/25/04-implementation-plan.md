# Issue 25: План имплементации

Статус: draft
Последнее обновление: 2026-03-15

## Назначение

Этот план связывает analysis-решения по issue `#25` с порядком реализации
re-entry contract для `run`, чтобы изменения в runtime schema, launcher path,
agent-specific resume logic и verification были прослеживаемыми.

## Scope

В план входит:

- расширение runtime schema для agent-specific resume metadata;
- pre-launch reconcile с decision matrix `new` / `reuse` / `restore`;
- restore/reuse path в `zellij` launcher;
- agent-specific resume integration для `codex` и `claude`;
- обновление документации, diagnostics и headless tests.

## Вне scope

- поддержка multiplexer кроме `zellij`;
- redesign GitHub Project lifecycle;
- общая унификация всех agent backend за пределами resume contract;
- host-side `zellij` e2e.

## Связанные документы

- Issue: https://github.com/dapi/ai-teamlead/issues/25
- Feature / issue spec:
  - [README.md](./README.md)
  - [01-what-we-build.md](./01-what-we-build.md)
  - [02-how-we-build.md](./02-how-we-build.md)
  - [03-how-we-verify.md](./03-how-we-verify.md)
- SSOT:
  - [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
  - [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
  - [../../../docs/features/0001-ai-teamlead-cli/05-runtime-artifacts.md](../../../docs/features/0001-ai-teamlead-cli/05-runtime-artifacts.md)
- ADR:
  - [../../../docs/adr/0008-bind-issue-to-agent-session-uuid.md](../../../docs/adr/0008-bind-issue-to-agent-session-uuid.md)
  - [../../../docs/adr/0013-agent-session-history-as-dialog-source.md](../../../docs/adr/0013-agent-session-history-as-dialog-source.md)
  - [../../../docs/adr/0023-zellij-session-target-resolution.md](../../../docs/adr/0023-zellij-session-target-resolution.md)
  - [../../../docs/adr/0031-zellij-issue-aware-tab-name-template.md](../../../docs/adr/0031-zellij-issue-aware-tab-name-template.md)
  - [../../../docs/adr/0032-zellij-launch-target-pane-tab.md](../../../docs/adr/0032-zellij-launch-target-pane-tab.md)
- Verification:
  - [03-how-we-verify.md](./03-how-we-verify.md)
- Code quality:
  - [../../../docs/code-quality.md](../../../docs/code-quality.md)
- Зависимые планы или фичи:
  - [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)

## План изменений документации

- Канонические документы, которые нужно обновить:
  - новый ADR про restore existing agent session;
  - [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md);
  - [../../../docs/features/0001-ai-teamlead-cli/05-runtime-artifacts.md](../../../docs/features/0001-ai-teamlead-cli/05-runtime-artifacts.md).
- Summary-документы и шаблоны, которые нужно синхронизировать:
  - при необходимости `README.md`, если в repo-level summary должен появиться
    новый launcher behavior;
  - project-local flow docs, если там описывается re-entry behavior.
- Документы, которые сознательно не меняются, и почему:
  - `docs/issue-analysis-flow.md` не требует изменения статусной модели, потому
    что issue меняет launcher/runtime semantics, а не flow transitions;
  - `docs/issue-implementation-flow.md` не меняется отдельно, если runtime
    решение формулируется как shared launcher contract для обоих stage.

## Зависимости и предпосылки

- schema change в `session.json` должна быть backward compatible;
- для `claude` deterministic resume path уже есть;
- для `codex` потребуется явный adapter-layer для получения и повторного
  использования native session id;
- verification по `zellij` должна выполняться только в headless-среде;
- restore path обязан уважать уже принятые `pane` / `tab` semantics.

## Порядок работ

### Этап 1. Зафиксировать канонический контракт restore path

Цель:

- синхронизировать documentation layer до правок runtime-кода.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [../../../docs/documentation-process.md](../../../docs/documentation-process.md)

Результат этапа:

- создан новый ADR про restore existing agent session;
- обновлены feature/runtime docs вокруг launcher contract и `session.json`;
- зафиксирован decision matrix `new/reuse/restore`.

Проверка:

- docs review на непротиворечивость;
- ссылки между issue spec, feature docs и ADR полны.

### Этап 2. Расширить runtime schema и decision layer

Цель:

- ввести machine-readable основу для safe restore/reuse.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- `session.json` поддерживает optional block `agent`;
- app-layer умеет вычислять `create_new_session`, `reuse_live_pane` или
  `restore_in_new_pane`;
- старые manifests без нового блока продолжают работать.

Проверка:

- unit-тесты на schema backward compatibility;
- unit-тесты на decision matrix.

### Этап 3. Добавить zellij live-check и no-duplicate launcher behavior

Цель:

- исключить запуск второго agent process при живой pane и поддержать restore в
  новую pane.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- `src/zellij.rs` умеет проверять живость stored session/tab/pane;
- `reuse_live_pane` path не запускает новый launcher process;
- `restore_in_new_pane` path создает новую pane в корректном launch context.

Проверка:

- headless integration tests на live pane reuse и pane recreation;
- регрессия существующих launcher tests не проявляется.

### Этап 4. Реализовать agent-specific resume adapters

Цель:

- запустить агента в native resume mode вместо старта новой независимой session.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- `launch-agent.sh` или соседний adapter-layer различает initial launch и
  resume launch;
- `claude` использует deterministic resume contract;
- для `codex` появился способ сохранить и повторно использовать native
  resume handle;
- stdout и `launch.log` показывают `created/reused/restored`.

Проверка:

- unit/integration checks для formatting и adapter behavior;
- headless scenario `pane deleted -> session restored`.

### Этап 5. Синхронизировать verification и diagnostics surface

Цель:

- сделать новое поведение проверяемым и диагностируемым в dogfooding path.

Основание:

- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)

Результат этапа:

- integration suite покрывает ключевые restore/reuse сценарии;
- `session.json`, stdout и `launch.log` отражают outcome;
- документация и runtime diagnostics не расходятся.

Проверка:

- headless suite проходит;
- ручная проверка артефактов подтверждает различие `created/reused/restored`.

## Критерий завершения

- `run` больше не создает вторую независимую agent session при живой уже
  связанной session;
- потерянная pane может быть восстановлена в новом `zellij` context, если
  backend поддерживает resume contract;
- runtime schema и diagnostics различают `new`, `reuse`, `restore`;
- docs, ADR и tests синхронизированы с итоговым поведением;
- все проверки проходят в headless path без обращения к host `zellij`.

## Открытые вопросы и риски

- для `codex` нужно выбрать надежный способ получения native resume handle после
  первичного запуска;
- reuse существующей pane может быть ограничен IPC-возможностями `zellij`,
  поэтому exact pane focus может остаться best-effort поведением;
- важно не допустить ситуации, в которой failed resume silently уходит в новый
  session start без явного diagnostics outcome.

## Журнал изменений

### 2026-03-15

- создан начальный план имплементации для issue `#25`
