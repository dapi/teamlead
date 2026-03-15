# Issue 5: Как проверяем

Статус: draft
Последнее обновление: 2026-03-14
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-14T19:14:28+03:00

## Acceptance Criteria

- существует отдельный SSOT `issue-implementation-flow`, который описывает
  lifecycle после `Ready for Implementation` и не смешивает его с analysis
  flow;
- `run <issue>` остается единым issue-level entrypoint и корректно
  маршрутизирует issue между analysis и implementation stage;
- approved analysis artifacts явно зафиксированы как обязательный входной
  контракт implementation stage;
- branch/worktree/PR lifecycle implementation stage описан отдельно от
  analysis branch lifecycle;
- contract commit/push/PR/finalization описан через CLI-инкапсуляцию, а не
  через неформальный список shell-команд в prompt;
- quality gates явно различают локальную валидацию, CI и human review;
- approval metadata позволяет понять, кто и когда утвердил план.

## Ready Criteria

Issue можно считать готовой к реализации, если подготовлены и согласованы:

- issue-level SDD-комплект в `specs/issues/5/`;
- отдельный implementation SSOT;
- отдельная feature-спека implementation stage;
- новые ADR по dispatch/runtime/finalization/input contract;
- config и project-local asset changes для implementation launcher;
- test strategy для unit, integration и smoke coverage implementation stage.

## Invariants

- `issue-analysis-flow` и `issue-implementation-flow` остаются отдельными
  flow-контрактами;
- `run` является единым issue-level entrypoint и не требует от пользователя
  выбирать flow вручную;
- implementation flow принимает issue только из `Ready for Implementation` или
  его stage-specific follow-up статусов;
- implementation session-binding не перезаписывает analysis session-binding;
- approved analysis artifacts являются входом в implementation stage и не
  подменяются произвольным пересказом issue;
- implementation branch всегда содержит номер issue и не совпадает с analysis
  branch;
- без обязательных локальных тестов implementation stage не может считаться
  готовой к push/review;
- проверки, которые могут задеть host `zellij`, запускаются только в
  изолированном headless environment.
- approved SDD-комплект должен содержать metadata про `Approved By` и
  `Approved At`.

## Test Plan

### Unit tests

- парсинг новых implementation statuses и stage-specific config templates;
- проверка допустимости входных статусов implementation entrypoint;
- проверка stage-aware dispatch внутри `run` по project status;
- рендер implementation branch/worktree naming из config;
- stage-aware runtime/session-binding без конфликта с analysis binding;
- mapping outcomes implementation finalization в project statuses;
- генерация commit/PR metadata по issue number и stage contract.

### Integration tests

- запуск `run <issue>` на fake runtime с issue в `Ready for Implementation`;
- создание или reuse implementation worktree и launcher context;
- чтение approved analysis artifacts как input contract;
- stage finalization: commit, push, draft PR, переход в `Waiting for CI`;
- переход из `Waiting for CI` в `Waiting for Code Review` по green checks;
- возврат из `Implementation Blocked` и `Waiting for Code Review` в
  `Implementation In Progress`.

### Smoke tests

- headless-friendly end-to-end прогон implementation flow на тестовом issue;
- подтверждение, что analysis и implementation могут жить как отдельные stages
  одной issue без конфликта runtime state;
- ручная проверка PR/CI/review lifecycle в реальном репозитории после того, как
  unit и integration path стабилизированы.

## Happy Path

1. Issue находится в `Ready for Implementation`.
2. Approved analysis artifacts доступны по versioned path.
3. `run` распознает implementation stage и переводит issue в
   `Implementation In Progress`.
4. Создается implementation branch/worktree и stage-specific launcher context.
5. Агент вносит кодовые изменения и запускает обязательные локальные тесты.
6. Finalization path делает commit, push и создает draft PR.
7. Issue переходит в `Waiting for CI`.
8. После green CI issue переходит в `Waiting for Code Review`.

## Edge Cases

- analysis artifacts отсутствуют или не соответствуют ожидаемому path;
- implementation branch уже существует и должна быть безопасно переиспользована;
- issue возвращена из review на доработку и требует повторного запуска stage;
- локальные тесты проходят, но обязательные CI checks падают;
- draft PR уже существует для implementation branch.

## Failure Scenarios

- GitHub Project status не удалось обновить, поэтому stage не должен тихо
  продолжаться;
- implementation finalization не смог сделать push, поэтому issue не должна
  переходить к CI/review;
- PR creation упала после push и требует явной диагностики и retry path;
- runtime-binding отсутствует или поврежден для повторного запуска;
- implementation prompt или launcher пытается использовать analysis-only paths.

## Observability

Нужны диагностические сигналы минимум по следующим точкам:

- issue number, stage, `session_uuid`, branch и worktree path;
- выбранный launcher path и entry status;
- источник analysis artifacts, по которому стартовала реализация;
- результат локальных тестов;
- URL PR и состояние draft/ready-for-review;
- summary обязательных CI checks;
- причина перехода в blocked или rework status.

## Verification Checklist

- новый SSOT описывает все implementation statuses и переходы;
- issue-level spec не противоречит `docs/issue-analysis-flow.md`;
- feature-docs, ADR и config contract ссылаются друг на друга;
- contract единого `run` как stage dispatcher описан явно;
- stage-aware runtime contract не ломает существующий analysis binding;
- commit/push/PR contract проверяем и не спрятан в prompt;
- unit coverage закрывает status/config/runtime branching;
- integration coverage закрывает entrypoint, finalization и retry paths;
- smoke strategy не требует опасного запуска `zellij` в host-сессии;
- human review gate после CI описан явно, а не подразумевается.
