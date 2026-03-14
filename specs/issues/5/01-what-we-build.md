# Issue 5: Что строим

Статус: approved
Последнее обновление: 2026-03-14
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-14T19:14:28+03:00

## Problem

Текущий MVP осознанно заканчивается на analysis stage:

- `issue-analysis-flow` формирует SDD-комплект;
- human gate переводит issue в `Ready for Implementation`;
- дальше у проекта нет отдельного flow-контракта для coding stage.

Из-за этого следующий этап остается неявным:

- не определено, как единая команда `run <issue>` должна распознавать переход
  от analysis stage к implementation stage;
- не зафиксировано, какие analysis artifacts считаются обязательным входом;
- branch/worktree lifecycle для реализации не отделен от analysis lifecycle;
- нет явного контракта для commit, push, PR и final quality gate;
- невозможно безопасно проектировать implementation orchestration, не смешивая
  product decisions, launcher behavior и runtime state.

## Who Is It For

- владелец репозитория, который хочет получить предсказуемый путь от принятого
  плана к implementation PR;
- оператор `ai-teamlead`, который запускает следующий stage после human review
  плана;
- агент реализации, которому нужен явный, versioned и проверяемый контракт;
- ревьюер, которому важно понимать, на что опирается кодовая реализация и какие
  quality gates уже пройдены.

## Outcome

Нужен отдельный `issue-implementation-flow`, который:

- стартует только после принятого анализа;
- вызывается через тот же канонический `run <issue>`, который сам маршрутизирует
  issue по текущей стадии;
- использует approved analysis artifacts как входной контракт;
- подготавливает отдельный implementation workspace для кода;
- ведет issue через явный implementation lifecycle;
- завершает stage не просто локальными изменениями, а оформленным PR и
  проверяемым набором quality gates.

Результат должен быть достаточным для следующего шага: реализации
implementation orchestration в коде и project-local assets без дополнительных
архитектурных догадок.

## Scope

В scope входит:

- отдельный системный SSOT для `issue-implementation-flow`;
- lifecycle issue после `Ready for Implementation`;
- stage-aware dispatch внутри единого issue-level entrypoint `run`;
- контракт использования analysis artifacts;
- branch/worktree/session lifecycle для implementation stage;
- contract для commit, push, PR и завершения стадии;
- quality gates и expected outcomes implementation stage;
- список обязательных feature-docs, project-local assets и ADR, нужных для
  реализации.
- contract approval metadata для SDD-комплекта и плана реализации.

## Non-Goals

Вне scope:

- auto-merge implementation PR;
- release, deploy и post-merge operation flow;
- переписывание уже принятого `issue-analysis-flow` в универсальный
  multi-stage superflow;
- поддержка произвольных git-стратегий без явного versioned config contract;
- автоматическое принятие code review или CI-решений без human gate.

## Constraints And Assumptions

- проект должен оставаться reusable для внешних репозиториев, поэтому
  repo-specific naming и launcher behavior должны жить в versioned config и
  project-local assets;
- analysis и implementation являются разными stage и не должны делить один
  неявный prompt contract, даже если верхнеуровневый CLI entrypoint у них
  общий;
- approved analysis artifacts должны быть доступны как стабильный вход для
  implementation flow;
- implementation stage не должен нарушать уже принятый принцип:
  GitHub Project status остается source of truth по состоянию issue;
- host `zellij` пользователя остается off-limits для небезопасных тестов,
  поэтому проверка flow должна быть headless-friendly;
- finalization implementation stage должна инкапсулировать VCS и GitHub
  операции через CLI-контракт, а не через ручные последовательности команд в
  prompt.
- момент approval плана должен быть наблюдаемым: документы обязаны хранить
  статус согласования, а после утверждения фиксировать кто и когда принял план.

## User Story

Как владелец репозитория, я хочу после принятия analysis-плана запускать
тот же `run <issue>`, который сам понимает, что issue уже на implementation
stage, использует утвержденный SDD-комплект, создает правильный coding
workspace, проводит агент через тесты и публикует implementation PR, чтобы
переход от plan-ready к реальной реализации был детерминированным и
проверяемым.

## Use Cases

### Use Case 1. Старт реализации из принятого плана

- issue уже прошла analysis stage;
- план принят человеком;
- оператор вызывает `run <issue>`;
- dispatcher определяет, что issue находится в `Ready for Implementation`, и
  запускает implementation flow;
- создается implementation branch/worktree;
- агент реализует код, проходит проверки и публикует PR.

### Use Case 2. Повторный вход после implementation blocker

- issue находится в `Implementation Blocked`;
- оператор снимает блокер и повторно запускает implementation flow;
- повторный вход снова идет через `run <issue>`;
- flow переиспользует stage-specific binding и продолжает реализацию без
  смешения с analysis session.

### Use Case 3. Возврат из review или CI на доработку

- implementation PR уже создан;
- ревью или CI требуют изменений;
- issue возвращается в implementation stage;
- следующий retry снова начинается через единый `run <issue>`;
- агент дорабатывает код в существующем implementation branch lifecycle, не
  меняя analysis artifacts.

## Dependencies

- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
  задает точку входа `Ready for Implementation` и запрещает повторный `run`
  analysis из этого статуса;
- [../../../docs/features/0001-ai-teamlead-cli/README.md](../../../docs/features/0001-ai-teamlead-cli/README.md)
  задает текущую CLI-модель `poll` / `run` / `loop`;
- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
  фиксирует analysis launcher contract и branch/worktree orchestration для
  текущего stage;
- [../../../docs/adr/0008-bind-issue-to-agent-session-uuid.md](../../../docs/adr/0008-bind-issue-to-agent-session-uuid.md)
  задает текущую модель durable session-binding;
- [../../../docs/adr/0015-versioned-launch-agent-contract.md](../../../docs/adr/0015-versioned-launch-agent-contract.md)
  и [../../../docs/adr/0016-configurable-analysis-workspace-templates.md](../../../docs/adr/0016-configurable-analysis-workspace-templates.md)
  задают pattern для versioned launcher/config contract;
- [../../../docs/adr/0020-agent-session-completion-signal.md](../../../docs/adr/0020-agent-session-completion-signal.md)
  задает pattern stage finalization через internal CLI-команду.
