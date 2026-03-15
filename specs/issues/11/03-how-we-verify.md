# Issue 11: Как проверяем

Статус: approved
Последнее обновление: 2026-03-15
Approved By: dapi
Approved At: 2026-03-15T11:52:35+03:00

## Acceptance Criteria

1. Если пользователь запускает `run` для несуществующей issue текущего
   репозитория, сообщение явно говорит, что issue не найдена и не может быть
   исправлена автоматически.
2. Если issue существует, но закрыта, сообщение показывает `CLOSED`-состояние и
   объясняет, что `run` работает только для открытых issue и не переоткрывает
   их самостоятельно.
3. Если issue открыта, но не находится в configured GitHub Project, система
   сама добавляет ее в target project, устанавливает стартовый status и
   информирует пользователя об этом.
4. Если issue находится в проекте, но ее текущий project status не допускает
   `run`, система либо сама переводит issue в корректный entry status и
   сообщает об этом, либо показывает фактический статус, полный список
   допустимых entry status и причину, почему автоисправление не выполнено.
5. Статус `Ready for Implementation` не попадает в отказной кейс, а по-прежнему
   считается допустимым entrypoint implementation stage.
6. Для remediation-aware formatter-а и отказных веток есть unit-тесты.
7. Для CLI `run` есть integration coverage как минимум на успешный
   auto-remediation path и на explain-only path.

## Ready Criteria

- issue-spec и implementation plan явно фиксируют конфликт со старым примером
  из issue про `Ready for Implementation`;
- доменная модель отказа отделена от orchestration и пригодна для unit-тестов;
- repo-level issue lookup покрывает минимум `not found` и `closed`;
- allowed statuses в сообщении формируются из канонического domain-решения, а
  не из отдельного hardcoded списка;
- для auto-remediation есть явный decision path, а не скрытый ad-hoc mutation;
- integration coverage проверяет наблюдаемое CLI-сообщение и факт mutation там,
  где команда обязана исправить проблему сама.

## Invariants

- `run <issue>` остается stage-aware публичным entrypoint;
- source of truth по project status остается в configured GitHub Project;
- отсутствие issue в `ProjectSnapshot` само по себе не означает ни
  `not found`, ни `closed`;
- formatter ошибок и remediation-решение используют структурированные данные, а
  не ветвятся по произвольным ad-hoc строкам;
- список допустимых статусов в сообщении совпадает со статусами, реально
  принимаемыми dispatch-логикой.

## Test Plan

Unit tests:

- formatter для `IssueNotFound` возвращает явную диагностическую строку;
- formatter для `IssueClosed` включает состояние issue;
- formatter для explain-only ветки `IssueNotInProject` включает `project_id` и
  причину, почему auto-remediation не выполнен;
- formatter для explain-only `StatusDenied` включает `current_status` и полный
  список `allowed_statuses`;
- decision для `AttachToProject` строит корректный remediation-план;
- decision для `NormalizeStatus` строит корректный target status, когда он
  определяется однозначно;
- domain-решение для `Ready for Implementation` по-прежнему считается allowed и
  маршрутизируется в implementation stage;
- список `allowed_statuses` строится детерминированно и не теряет ни analysis,
  ни implementation entry statuses.

Integration tests:

- `run` на issue вне project snapshot, но существующей в репозитории,
  автоматически добавляет issue в project и устанавливает стартовый status;
- `run` на issue с недопустимым статусом автоматически нормализует status, если
  у домена есть однозначный remediation-path;
- `run` на explain-only `status denied` выводит current status и допустимые
  entry statuses;
- при наличии `Ready for Implementation` в snapshot `run` не падает на новом
  formatter path, а идет в implementation branch как раньше.

Manual / smoke:

- выполнить `cargo test`;
- вручную проверить одну issue вне project и одну issue с заведомо
  недопустимым статусом;
- убедиться, что при auto-remediation пользователь видит факт исправления, а
  при explain-only fallback получает причину и явный следующий шаг.

## Verification Checklist

- доменная модель отказа введена и покрыта unit-тестами;
- `run_manual_run` больше не делает ранний blanket-error
  `is not linked to the project`;
- новый repo-level issue lookup используется до финального вывода
  `not in project`;
- auto-remediation выполняется через явный GitHub adapter path;
- `project_id`, `issue_url`, `current_status` и `allowed_statuses` попадают в
  соответствующие explain-only ветки ошибок;
- `Ready for Implementation` не сломан регрессией;
- integration test на CLI-path обновлен под новый ожидаемый вывод.

## Regression Checks

- существующий успешный `run` для `Backlog` не меняет поведение;
- существующий успешный `run` для `Waiting for Clarification`,
  `Waiting for Plan Review` и `Analysis Blocked` не ломается;
- existing stage-aware path для `Ready for Implementation`,
  `Implementation In Progress`, `Waiting for CI`,
  `Waiting for Code Review` и `Implementation Blocked` остается валидным;
- auto-remediation не ломает уже работающий happy path, когда issue с самого
  начала находится в корректном project status;
- отказной путь по missing approved analysis artifacts в implementation stage не
  меняет своей семантики из-за нового formatter-а ручного `run`.

## Failure Scenarios

- formatter выводит analysis-only список статусов и вводит оператора в
  заблуждение относительно implementation entrypoints;
- `IssueClosed` по ошибке трактуется как `IssueNotInProject`;
- repo-level lookup не находит issue из-за неверного owner/repo и дает
  misleading сообщение;
- auto-remediation выбирает неверный target status и silently меняет issue не в
  ту сторону;
- integration tests проверяют слишком точный whitespace/format и становятся
  хрупкими без пользы.

## Observability

- отказ `run` должен явно различать тип ветки хотя бы в тексте финальной
  ошибки;
- сообщения должны включать либо факт выполненного auto-remediation, либо
  диагностические факты, которые оператор реально может использовать:
  `project_id`, current status, issue state, issue URL;
- при отладке тестов должно быть видно, какая ветка доменного formatter-а
  сработала;
- integration assertions лучше проверять по устойчивым смысловым фрагментам,
  а не по полному byte-to-byte совпадению всего текста.
