# Issue 56: Как проверяем

Статус: draft
Последнее обновление: 2026-03-15

## Acceptance Criteria

- runtime различает как минимум `public`, `private` и `unknown`
  `repo_visibility`;
- для `public` и `unknown` visibility включается `public-safe` baseline;
- hostile GitHub content, repo content и runtime output не трактуются как
  trusted control plane;
- auto-intake policy для public repos ограничивает старт issue по author policy
  и не делает comments trusted;
- high-risk filesystem, network, execution и publication actions не происходят
  без deterministic deny или explicit approval;
- diagnostics позволяют понять, какой security mode и какой gate сработал;
- документация, prompts и runtime не противоречат друг другу по security
  contract.

## Ready Criteria

- issue зафиксирована как `large feature` для `infra/platform`;
- implementation опирается на feature `0006`, SSOT по hostile input и ADR
  `0029/0030`, а не вводит параллельный security contract;
- определен минимальный набор enforcement points в `run`/`poll`, GitHub layer,
  shell layer и publication path;
- выбран headless verification path для сценариев, затрагивающих `zellij`;
- есть отдельный implementation plan с прослеживаемостью к документам и tests.

## Invariants

- hostile input не может сам объявить себя trusted;
- отсутствие metadata о visibility не ослабляет policy;
- issue author и comment author рассматриваются независимо;
- explicit approval относится к конкретному risky action, а не к произвольному
  будущему поведению сессии;
- публикация наружу не должна включать локальные чувствительные данные без
  отдельного осознанного operator approval.

## Test Plan

Unit tests:

- resolution `repo_visibility -> operating_mode` покрыт для `public`,
  `private` и `unknown`;
- intake policy покрыта кейсами `owner-only`, `allowlist`, `open-intake`;
- policy не повышает trust comments только из-за owner-authored issue;
- classification risky actions покрыта для filesystem, network, execution и
  publication paths;
- diagnostics формируют понятную причину block/approval без вывода секретов.

Integration tests:

- `run` для public issue включает `public-safe` до запуска agent workflow;
- `poll` пропускает public issue от автора вне allowlist, если активен
  ограничительный intake policy;
- попытка dangerous execution из hostile issue/comments не доходит до
  автоматического исполнения;
- publication path не публикует сырые sensitive local artifacts без approval;
- fallback при `unknown` visibility остается fail-closed.

Headless agent-flow / sandbox tests:

- сценарий hostile issue с инструкцией прочитать `~/.ssh` останавливается на
  deny или approval gate;
- scenario с prompt injection в comment не приводит к auto-execution;
- scenario с hostile repo-local docs не расширяет filesystem/network scope;
- scenario с instruction-looking shell output не превращается в новый control
  plane;
- все `zellij`-related проверки выполняются только в headless/Docker path.

Manual validation:

- review operator-visible diagnostics для deny/approval paths;
- review логики publication gate на отсутствие локальных секретов в
  issue/PR/comments output;
- review project-local prompts и launcher context на различение
  `operator intent` и `content suggestion`.

## Verification Checklist

- есть unit coverage для visibility resolution и intake policy;
- есть integration coverage для permission gates и fail-closed fallback;
- headless сценарии покрывают hostile issue, hostile comments, hostile
  repo-local docs и hostile runtime output;
- проверки не трогают host `zellij` пользователя;
- runtime diagnostics позволяют восстановить причину блокировки;
- docs, prompts и code review не выявляют противоречий между SSOT, ADR и
  runtime behavior.

## Happy Path

1. Оператор запускает `run` для issue в public repo.
2. Runtime определяет `repo_visibility = public`.
3. До любых risky actions включается `operating_mode = public-safe`.
4. Issue content используется как данные для анализа, но не как permission
   override.
5. Если выполнение требует risky action, runtime либо запрашивает явный
   approval, либо детерминированно запрещает его.

## Edge Cases

- visibility определить не удалось;
- issue создана владельцем, но hostile content приходит из comments;
- allowlist настроен частично или отсутствует;
- shell output после тестов содержит instruction-looking текст;
- project-local docs пытаются расширить scope доступа.

## Failure Scenarios

- `poll` автоматически берет hostile issue из public repo без проверки author
  policy;
- `run` разрешает risky action только потому, что команда была предложена в
  issue/comment;
- publication path отправляет локальные секреты в GitHub comment или PR body;
- diagnostics скрывают причину отказа, и оператор не понимает, почему сработал
  safe mode;
- runtime ослабляет policy при `unknown` visibility.

## Observability

- operator должен видеть, какой `operating_mode` применился к запуску;
- diagnostics должны указывать, какой input source вызвал block или approval;
- лог должен различать deny из policy, отсутствие metadata и sandbox
  ограничения;
- observability не должна сама становиться каналом утечки локальных секретов.
