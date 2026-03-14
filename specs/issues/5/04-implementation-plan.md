# Issue 5: План имплементации

Статус: approved
Последнее обновление: 2026-03-14
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-14T19:14:28+03:00

## Назначение

Этот план задает порядок реализации `issue-implementation-flow` как отдельного
stage после analysis, с собственным lifecycle, launcher/runtime contract и
quality gates до human code review.

## Scope

В scope входит:

- отдельный SSOT `issue-implementation-flow`;
- feature-спека implementation stage;
- новые ADR по input contract, runtime/session-binding и finalization;
- stage-aware dispatch внутри единого `run <issue>`;
- stage-specific launcher/config/runtime contract;
- commit/push/PR/CI transitions implementation stage;
- unit, integration и headless-friendly smoke coverage.
- approval metadata для SDD-комплекта и implementation plan.

Вне scope:

- merge automation;
- deploy/release flow;
- универсальный multi-stage orchestrator для всех будущих стадий;
- поддержка нескольких параллельных implementation PR на одну issue;
- постфактум переписывание analysis flow без необходимости.

## Связанные документы

- [README.md](./README.md)
- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../docs/implementation-plan.md](../../../docs/implementation-plan.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [../../../docs/features/0001-ai-teamlead-cli/README.md](../../../docs/features/0001-ai-teamlead-cli/README.md)
- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
- [../../../docs/adr/0008-bind-issue-to-agent-session-uuid.md](../../../docs/adr/0008-bind-issue-to-agent-session-uuid.md)
- [../../../docs/adr/0015-versioned-launch-agent-contract.md](../../../docs/adr/0015-versioned-launch-agent-contract.md)
- [../../../docs/adr/0016-configurable-analysis-workspace-templates.md](../../../docs/adr/0016-configurable-analysis-workspace-templates.md)
- [../../../docs/adr/0020-agent-session-completion-signal.md](../../../docs/adr/0020-agent-session-completion-signal.md)

## Зависимости и предпосылки

- текущий `issue-analysis-flow` остается analysis-only и передает issue в
  `Ready for Implementation`, после чего единый `run` должен уметь
  маршрутизировать issue уже в implementation flow;
- approved analysis artifacts доступны как versioned вход для coding stage;
- проект готов добавить новые GitHub Project statuses для implementation
  lifecycle;
- launcher и тесты смогут работать в headless-friendly режиме там, где нельзя
  безопасно трогать host `zellij`;
- naming branch/worktree и launcher files должен оставаться configurable на
  уровне repo-local contract.

## Порядок работ

### Этап 1. Зафиксировать документационный контракт implementation stage

Цель:

- создать отдельный SSOT и feature-спеку implementation stage;
- явно отделить lifecycle реализации от analysis lifecycle.

Основание:

- issue требует отдельный flow после `Ready for Implementation`;
- текущий SSOT analysis не покрывает coding stage.

Результат этапа:

- существует `docs/issue-implementation-flow.md`;
- создана feature-спека implementation stage по трем осям;
- зафиксирован approval contract: кто утверждает план, когда меняется статус
  документов и где хранятся `Approved By` / `Approved At`;
- `README.md` и связанные overview-документы знают о новом stage только как о
  summary с ссылками на профильные документы.

Проверка:

- документация покрывает статусы, переходы, human gates и артефакты;
- ссылки между SSOT, feature и issue-spec прослеживаемы.

### Этап 2. Принять ADR по входному контракту и runtime/finalization model

Цель:

- убрать скрытые архитектурные решения до начала кода.

Основание:

- issue прямо требует ADR-решения для implementation stage;
- текущие ADR покрывают только analysis launcher/runtime/finalization.

Результат этапа:

- принят ADR про approved analysis artifacts как canonical input;
- принят ADR про stage-aware dispatch внутри единого `run`;
- принят ADR про stage-scoped runtime/session-binding;
- принят ADR про implementation finalization contract для commit/push/PR/CI.

Проверка:

- новые ADR не противоречат ADR-0008, ADR-0015, ADR-0016 и ADR-0020;
- по ним можно восстановить точные границы launcher, runtime и CLI.

### Этап 3. Добавить implementation statuses и stage-aware dispatch в `run`

Цель:

- сделать lifecycle implementation stage исполнимым через текущий core CLI
  без отдельной пользовательской команды.

Основание:

- без stage-aware dispatch внутри `run` статус `Ready for Implementation`
  остается тупиковым
  статусом;
- пользователь ожидает всегда вызывать `run <issue>` независимо от стадии.

Результат этапа:

- config contract поддерживает implementation statuses;
- `run` получает stage dispatcher для implementation statuses;
- GitHub Project transitions валидируются так же строго, как в analysis flow.

Проверка:

- unit-тесты на status guards и mapping статусов;
- integration-тесты на dispatch, claim/re-entry/reject paths implementation
  stage.

### Этап 4. Реализовать stage-specific launcher и workspace contract

Цель:

- подготовить корректный coding workspace без смешения с analysis launcher.

Основание:

- implementation stage требует отдельный branch/worktree lifecycle и prompt.

Результат этапа:

- есть versioned implementation launcher или явный stage-aware launcher wrapper;
- `settings.yml` содержит implementation naming/templates;
- создаются implementation branch/worktree и нужный prompt context;
- runtime binding различает analysis и implementation sessions.

Проверка:

- unit-тесты рендера templates и runtime schema;
- integration-тесты подготовки workspace и повторного запуска.

### Этап 5. Реализовать finalization path для commit, push, PR и CI gates

Цель:

- сделать completion implementation stage детерминированным и проверяемым.

Основание:

- issue требует явный контракт для VCS и PR lifecycle;
- ручные последовательности git/gh в prompt ненадежны.

Результат этапа:

- агент завершает stage одной internal CLI-командой;
- локальные проверки выполняются до push;
- draft PR создается автоматически;
- status `Waiting for CI` и `Waiting for Code Review` переключаются через
  явный contract.

Проверка:

- integration-тесты на success, push failure, existing PR и red CI path;
- диагностические сообщения различают каждый шаг finalization.

### Этап 6. Закрыть quality bar тестами и headless smoke

Цель:

- доказать, что новый stage не ломает существующий analysis MVP.

Основание:

- `docs/code-quality.md` требует тест для каждой значимой feature;
- `zellij`-related проверки нельзя бездумно гонять в host-среде.

Результат этапа:

- добавлены unit и integration tests по новым contract points;
- есть headless-friendly smoke path для implementation flow;
- подтверждено, что analysis и implementation bindings не конфликтуют.

Проверка:

- `cargo test` или эквивалентный test suite зеленый;
- отдельный smoke-прогон использует изолированную среду;
- regression-сценарии analysis flow остаются зелеными.

## Критерий завершения

Issue можно считать реализованной, если:

- `issue-implementation-flow` задокументирован отдельным SSOT и feature-спекой;
- approved analysis artifacts, runtime/session-binding и finalization path
  зафиксированы отдельными ADR;
- `run <issue>` умеет маршрутизировать `Ready for Implementation` в
  implementation flow;
- implementation branch/worktree/PR lifecycle configurable и проверяем;
- implementation stage проходит хотя бы один целевой end-to-end сценарий до
  `Waiting for Code Review`;
- quality gates подтверждены unit, integration и headless smoke coverage.

## Риски и открытые вопросы

- потребуется уточнить, в какой момент approved analysis artifacts попадают в
  стабильный источник для implementation stage;
- CI checks могут быть слишком медленными или flaky для одной непрерывной
  агентской сессии;
- выбор между отдельными runtime директориями и общей stage-aware schema влияет
  на объем миграции текущего runtime-контракта;
- если implementation stage потребует отдельного review-comment ingestion path,
  это, вероятно, отдельная follow-up задача, а не часть первого MVP.

## Журнал изменений

### 2026-03-14

- создан issue-level implementation plan для issue 5
