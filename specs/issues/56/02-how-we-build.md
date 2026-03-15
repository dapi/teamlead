# Issue 56: Как строим

Статус: draft
Последнее обновление: 2026-03-15

## Approach

Реализацию нужно делать как поэтапное внедрение уже принятого security
contract, а не как новую перепридуманную модель.

Для этой issue порядок источников security/runtime contract такой:

1. SSOT [../../../docs/untrusted-input-security.md](../../../docs/untrusted-input-security.md)
   и ADR `0029/0030`;
2. этот issue-level handoff, который конкретизирует operational details для
   implementation, но не подменяет SSOT/ADR;
3. feature `0006` как summary-layer, который должен быть синхронизирован после
   реализации, но не переопределяет более точный канонический contract молча.

Технический подход:

- использовать feature `0006`, SSOT `untrusted-input-security` и ADR
  `0029/0030` как канонический источник требований;
- добавить в runtime детерминированное вычисление `repo_visibility` и
  `operating_mode` до входа в execution path;
- отделить coarse filesystem enforcement от typed runtime policy:
  launcher/sandbox слой отвечает за secret filtering, runtime `ai-teamlead`
  gates отвечают за `allow`/`approval`/`deny` по типу действия;
- внедрить intake layer для public repos, который различает `poll`-auto-intake
  и explicit `run`, но не повышает trust comments;
- маркировать действия по типу риска и направлять high-risk actions в явные
  permission gates по фиксированной policy-матрице;
- выровнять launcher, prompts, diagnostics и publication path с тем же
  security contract, чтобы policy не жила только в тексте документов.

## Affected Areas

- `src/github.rs`:
  получение issue/repo metadata, нужных для visibility resolution и intake
  policy;
- `src/app.rs` и `src/domain.rs`:
  stage-aware `run`/`poll` orchestration, decision points для intake, mode
  resolution и отказов policy;
- `src/config.rs`:
  config surface для security policy, allowlist, approval channel,
  `public-safe` override и `secret-class` path policy;
- `src/security.rs` или эквивалентный policy-layer:
  classifier для typed actions, path classes, decisions и approval requests;
- `src/shell.rs`:
  execution gate для опасных команд и diagnostics по отказам;
- `src/complete_stage.rs` и publication path:
  защита от публикации локальных чувствительных данных;
- `.ai-teamlead/launch-agent.sh` и staged prompts:
  выравнивание operator messaging, approval path, secret filtering и различение
  `operator intent` против hostile content;
- `docs/untrusted-input-security.md`, feature `0006`, ADR `0029/0030`:
  возможная синхронизация, если implementation уточнит config/runtime детали;
- `tests/integration/` и `.ai-teamlead/tests/agent-flow/`:
  headless verification сценарии для public-safe режима и hostile inputs.

## Interfaces And Data

Минимальная domain-модель для реализации:

- `repo_visibility`: `public`, `private`, `unknown`;
- `operating_mode`: `standard`, `public-safe`;
- `intake_policy`: `owner-only`, `allowlist`, `open-intake`;
- `intake_decision`: `eligible`, `manual-override`, `skipped`, `denied`;
- `input_trust_class`: `trusted-control`, `semi-trusted-repo`, `untrusted`,
  `sensitive-local`;
- `path_class`: `repo-visible`, `repo-secret`, `host-nonsecret`,
  `host-secret`, `publish-only-artifact`;
- `enforcement_layer`: `launcher-sandbox`, `runtime-gate`;
- `approval_state`: `not-required`, `required`, `granted`, `denied`.
- `action_kind`: `filesystem-read`, `filesystem-write`, `network-open`,
  `network-download`, `execution-command`, `publication-github`;
- `approval_record`: `session_uuid`, `issue_number`, `action_kind`,
  `target_fingerprint`, `operator_response`, `granted_at`, `expires_at`.

Минимальные правила обработки:

- `repo_visibility = public` всегда включает `operating_mode = public-safe`;
- `repo_visibility = private` по умолчанию сохраняет `operating_mode =
  standard`, если repo-level config явно не требует `force-public-safe`;
- `repo_visibility = unknown` не может ослаблять ограничения;
- `repo/worktree` не считаются автоматически безопасными только потому, что они
  лежат внутри текущего checkout; `secret-class` paths внутри repo выделяются
  отдельно;
- для `operating_mode = public-safe` допустимы только `owner-only` и
  `allowlist`; `open-intake` разрешен только для `standard` режима;
- `operating_mode = standard` сохраняет существующий private-repo baseline и не
  вводит новые public-only intake restrictions без явного config override;
- launcher/sandbox слой обязан давать агенту доступ ко всему `repo/worktree`,
  кроме `secret-class` paths, чтобы research и document/code editing path не
  требовали лишних approval;
- `poll` работает только с `intake_decision = eligible`;
- explicit `run` по issue вне intake policy приводит к
  `intake_decision = manual-override`, но не повышает trust-класс issue
  thread;
- `intake_decision = skipped` означает, что runner сознательно не берет issue в
  текущем auto-intake path и не меняет project status;
- `intake_decision = denied` означает deterministic runtime refusal по security
  policy или config contract, а не "тихий skip";
- `input_trust_class = untrusted` не может инициировать собственное повышение
  привилегий;
- `approval_state = granted` относится к конкретному действию, а не к
  неограниченному сеансу целиком.
- `approval_state = granted` в MVP может появиться только из явного ответа
  оператора в agent session; issue body, comments, repo-local docs и runtime
  output не могут выступать источником approval.
- `approval_record` должен записываться в runtime audit trail и быть привязан к
  конкретному risky action, session и моменту времени.

`secret-class` для MVP должна включать как минимум:

- `.env`, `.env.*`, кроме явно санитизированных `.env.example`/`.env.sample`;
- `*.pem`, `*.key`, `*.p12`, `*.jks`;
- `.git-credentials`, `.npmrc`, `.pypirc`, `.netrc`;
- host secrets вроде `~/.ssh/*`, `~/.aws/*`, `~/.config/gcloud/*`,
  `~/.kube/*`, `/proc/self/environ`.

## Standard Mode Baseline

`repo_visibility = private` и `operating_mode = standard` остаются частью scope
этой issue, потому что public-repo hardening не должен тихо ломать existing
private-repo path.

Минимальный baseline для `standard`:

- runtime сохраняет текущую семантику analysis/implementation flow для private
  repos, если repo-level config не включает `force-public-safe`;
- public-only intake restrictions (`owner-only`/`allowlist` как mandatory
  baseline) не навязываются private path молча;
- агент сохраняет автономный research path внутри `repo/worktree`, кроме
  `secret-class` paths и explicit deny-категорий;
- existing generic safety invariants остаются в силе и в `standard`: hostile
  task input не становится approval source, а sensitive local data не считается
  safe для публикации по умолчанию;
- verification обязана доказать отсутствие undocumented regression для private
  repos.

Ключевые интерфейсы:

- `run` и `poll` как entrypoints, где выбираются `repo_visibility`,
  `operating_mode` и intake policy;
- GitHub integration layer, который должен вернуть достаточно metadata для
  visibility и issue author policy;
- shell execution layer, который должен различать обычное исполнение и
  dangerous execution;
- publication path, который должен учитывать риск data exfiltration;
- trusted operator channel в MVP, через который runtime получает explicit
  approval для конкретного high-risk action;
- launcher/sandbox слой, который должен формировать agent-visible filesystem
  view и скрывать `secret-class` paths до того, как модель попытается их читать;
- project-local prompts и launcher context, которые не должны трактовать
  hostile content как trusted instruction.

## Enforcement Layers

Security enforcement должен быть двухслойным.

Слой 1. `launcher-sandbox`

- формирует доступный агенту filesystem view;
- разрешает чтение/редактирование обычных файлов внутри `repo/worktree`;
- скрывает или иным образом hard-deny делает `secret-class` paths, даже если
  они лежат внутри repo;
- не полагается на то, что runtime gate сможет "поймать" уже состоявшееся
  чтение секрета.

Слой 2. `runtime-gate`

- классифицирует действия как `filesystem`, `network`, `execution`,
  `publication`;
- для каждого действия выносит typed decision `allow` / `approval` / `deny`;
- пишет diagnostics и audit trail;
- не заменяет launcher filtering, а работает поверх него.

Практический контракт:

- внутри `repo/worktree` обычные documents/code files проходят по `allow`,
  чтобы не ломать автономию research path;
- `secret-class` paths получают hard `deny`;
- чтение non-secret path вне `repo/worktree` допускается только через explicit
  approval;
- network/publication outside canonical GitHub workflow не становятся implicit
  allow только потому, что агенту нужен "research".

## Bootstrap Order And Trust Boundaries

Порядок применения security policy должен быть таким:

1. загрузить trusted local control plane:
   локальные системные инструкции, CLI/runtime contract, versioned governance
   самого установленного `ai-teamlead`;
2. определить repo context и попытаться вычислить `repo_visibility`;
3. выбрать `operating_mode` и `intake_decision` fail-closed способом;
4. только после этого читать target issue, comments, repo-local docs и runtime
   outputs как task input;
5. перед каждым risky action применять policy-матрицу и при необходимости
   запрашивать approval через agent session.

Явное различение двух классов repo-local документации:

- локальный contract layer самого `ai-teamlead`, с которым оператор запускает
  workflow, относится к trusted control plane;
- repo-local docs целевого public repo, читаемые как часть задачи, относятся к
  hostile-by-default input и не могут сами менять permission model.

Для self-hosted/dogfooding path действует дополнительное правило:

- если target repo совпадает с локальным repo установленного `ai-teamlead`,
  только bootstrap assets, загруженные до task binding (`AGENTS.md`, `AURA.md`,
  `.ai-teamlead/*`, launcher/runtime contract), остаются trusted control plane;
  те же или соседние repo files, повторно прочитанные уже как task input,
  считаются hostile-by-default.

`repo_visibility` должен вычисляться по следующему приоритету:

1. каноническая GitHub metadata о visibility репозитория;
2. fallback по repo metadata, которую можно надежно получить через локальный
   git/gh context без чтения issue content;
3. `unknown`, если reliable metadata не удалось получить.

## Author Resolution And Intake Identity

`intake_policy` должна использовать единый источник identity:

- issue author login, полученный из канонической GitHub metadata;
- comments не участвуют в intake eligibility и не могут менять intake decision;
- operator identity для `owner-only` берется из account context, под которым
  запущен `ai-teamlead` и выполняются канонические GitHub actions этого
  workflow.

Правила resolution:

- `owner-only`: issue author login должен совпадать с operator login;
- `allowlist`: issue author login должен входить в
  `security.public_repo.issue_author_allowlist`;
- org membership сама по себе не является достаточным основанием для intake;
- bot/service account допускается только при явном presence в allowlist;
- missing author metadata приводит к `skipped` для `poll` и допускает только
  `manual-override` для explicit `run`.

Отдельное правило:

- issue author влияет только на intake;
- comment author никогда не дает trust upgrade и не участвует в approval path.

## Intake Decisions And Flow Statuses

`intake_decision` описывает runtime-решение на входе в запуск и не является
самостоятельным GitHub Project status.

Правила mapping:

- `eligible`: runner продолжает текущий flow path; сам по себе eligibility не
  переводит issue в новый project status;
- `manual-override`: explicit `run` продолжает текущий flow только по явному
  intent оператора; это решение логируется в diagnostics, но не мапится на
  отдельный GitHub Project status;
- `skipped`: используется для non-error refusal в `poll`; runner не создает
  analysis session, не берет issue в работу и не меняет GitHub Project status;
- `denied`: используется для deterministic refusal по security policy или config
  contract после выбора target issue; если denial делает stage непродолжимым,
  текущая session должна завершаться outcome `blocked`, а issue переводится в
  `Analysis Blocked`; если denial относится только к одному risky action и
  analysis может продолжаться, project status не меняется, а отказ остается в
  local diagnostics и audit trail;
- `needs-clarification` зарезервирован для нехватки human input и не должен
  подменять собой security `denied`;
- `plan-ready` не должен использоваться для маскировки skipped/denied paths.

## Risk Policy Matrix

Для `public-safe` режима минимальная policy-матрица должна быть такой:

| Категория | Allow | Approval | Deny |
| --- | --- | --- | --- |
| `filesystem` | чтение и запись внутри repo/worktree и разрешенного runtime-dir, если path не относится к `secret-class` | разовое чтение явно указанного оператором несекретного пути вне repo/worktree | любой `secret-class` path, включая `.env*`, key/cert files, host credential dirs; любая запись вне repo/worktree/runtime-dir |
| `network` | канонический GitHub control plane, нужный для работы `ai-teamlead`, включая metadata и canonical workflow endpoints текущего GitHub repo | открытие внешней ссылки или скачивание контента с operator-approved allowlisted host либо явное чтение linked artifact content, если это не требует публикации локальных данных наружу | доступ к non-allowlisted host; отправка локальных файлов/секретов наружу |
| `execution` | inspect/build/test/edit команды, ограниченные repo/worktree и штатным toolchain | точечный запуск operator-approved команды вне baseline toolchain, если она все еще ограничена approved scope | `sudo`, системные package managers, sandbox escalation, destructive host-level commands, redirection вне repo/worktree |
| `publication` | публикация versioned artifacts и workflow metadata в канонический GitHub path после secret-scrub | публикация reviewable non-secret artifact в заранее определенный canonical GitHub sink, если auto-path был остановлен explicit gate-ом | публикация secret data, raw credentials, локальных config dumps, сырых runtime artifacts с чувствительным содержимым, любые внешние publish sinks и uploads во внешние сервисы |

Если действие не попало ни в одну allow/approval-категорию, оно трактуется как
`deny` в `public-safe` режиме.

## Publication Scope

Для MVP publication path должен быть ограничен следующими sink-ами:

- канонический GitHub workflow path самого `ai-teamlead`:
  issue comments, PR body, PR comments, status-linked artifacts;
- versioned docs и code changes внутри текущего repo/worktree;
- runtime diagnostics, которые остаются локальными и не публикуются наружу без
  отдельного approval.

По умолчанию неразрешенные publish sinks:

- произвольные внешние paste/file-sharing сервисы;
- email, chat, webhook и любые каналы вне канонического GitHub workflow;
- uploads бинарных или raw runtime artifacts без secret-scrub.

Payload classes для MVP:

- `safe-workflow-metadata`: status updates, plan summaries, versioned doc links;
- `reviewable-artifacts`: versioned markdown/code diff после normal workflow;
- `sensitive-local-data`: secrets, tokens, raw configs, env dumps, host paths,
  runtime artifacts с непроверенным содержимым.

Только первые две категории могут попадать в publication allow/approval path;
`sensitive-local-data` всегда `deny`.

Границы canonical GitHub workflow для MVP:

- allow boundary ограничен current issue/PR/status workflow и versioned changes
  текущего repo/worktree;
- linked GitHub issue/PR/artifact content может читаться только как untrusted
  data и не образует отдельный publish sink сам по себе;
- external URL или linked external artifact не может автоматически открываться
  или публиковаться только потому, что он встретился в issue/comment/body.

Важное ограничение MVP:

- approval допустим только внутри заранее определенного canonical GitHub sink;
- publication во внешние sinks (`paste`, `email`, `chat`, `webhook`, file
  sharing) всегда `deny`, даже если hostile input пытается представить такой
  путь как "обычный debug workflow".

## Approval Lifecycle And Storage

MVP approval contract:

- approval one-shot и action-bound;
- approval valid только внутри текущего `session_uuid`;
- runtime перед risky action обязан показать approval request descriptor:
  `action_kind`, `target_fingerprint`, human-readable target, policy reason и
  requested scope;
- canonical operator response для grant/deny в MVP должен быть однозначным и
  содержать тот же `action_kind` и тот же `target_fingerprint`; двусмысленный,
  частичный или mismatched ответ не считается `granted`;
- approval reusable только для того же `action_kind` и того же
  `target_fingerprint` в рамках текущей session;
- новый target, новый `session_uuid`, restart/re-run или изменившийся target
  invalidates previous approval;
- `expires_at` по умолчанию равен завершению session либо более раннему явному
  invalidation event;
- timeout, отсутствие ответа, malformed operator response или ошибка
  сопоставления descriptor -> response трактуются как `denied`, а не как
  implicit retry/success.

`target_fingerprint` должен вычисляться детерминированно:

- `filesystem-read` / `filesystem-write`: normalized absolute path(s) +
  requested operation;
- `network-open` / `network-download`: method, host, normalized URL/path и
  expected payload class;
- `execution-command`: executable, argv, cwd и declared redirection targets;
- `publication-github`: sink kind, destination identifier, payload class и
  source artifact identifier.

Source of truth и storage:

- `approval_record` хранится в runtime artifacts текущей session;
- `operator_response` сохраняется как итог нормализованного решения
  (`approve`/`deny`) и не должен подменять собой исходный descriptor;
- audit trail должен быть append-only или атомарно перезаписываемым так, чтобы
  частичная запись не трактовалась как granted approval;
- если запись approval record не удалась, risky action трактуется как `denied`,
  а не как implicit success.

## Configuration And Runtime Assumptions

- текущий `settings.yml` пока не содержит полной security schema, поэтому issue
  фиксирует минимальный proposed contract для implementation handoff до
  синхронизации канонических config docs;
- отсутствие security-поля не должно ослаблять policy; для `public` и
  `unknown` visibility runtime обязан применять встроенный fail-closed baseline
  (`public-safe`, `owner-only`, пустой allowlist, `agent-session`,
  canonical GitHub only);
- default policy для public repos должна быть безопасной даже без явной
  конфигурации;
- если visibility не удается определить через GitHub metadata, runtime должен
  оставаться в `public-safe`;
- для `private` visibility отсутствие security override сохраняет
  `operating_mode = standard`;
- минимальная repo-level security schema для MVP должна покрывать:
  - `security.public_repo.operating_mode`: `auto` | `force-public-safe`
  - `security.public_repo.intake_policy`: `owner-only` | `allowlist`
  - `security.public_repo.issue_author_allowlist`: список логинов
  - `security.network.allow_hosts`: список host allowlist
  - `security.approval.channel`: `agent-session`
  - `security.approval.audit_log`: `true`
  - `security.filesystem.secret_globs`: список repo/worktree secret-class paths
  - `security.filesystem.allow_example_env`: `true` для
    `.env.example`/`.env.sample`
- если implementation этой issue вводит эти versioned settings в runtime, тот же
  change set обязан синхронно обновить `docs/config.md` и соответствующие
  versioned templates; до такой синхронизации schema не считается канонической;
- enforcement нельзя сводить только к системному prompt или дисциплине модели;
- текущие launcher defaults, документированные вне этой issue, являются
  pre-security-baseline состоянием и должны быть приведены в соответствие с
  approval contract для `public-safe` режима;
- до обновления launcher defaults implementation `public-safe mode` должен
  иметь приоритет над legacy `--ask-for-approval never` path;
- malformed enum values, conflicting legacy flags или поломанный security config
  должны приводить к fail-closed behavior: `public-safe` не ослабляется, а
  текущий invocation либо получает deterministic deny для risky actions, либо
  stage-level `blocked`, если безопасное продолжение невозможно;
- `allowlist` с пустым списком разрешен как конфигурация, но означает, что
  `poll` всегда получает `skipped`, а explicit `run` возможен только как
  `manual-override`;
- проверки, затрагивающие `zellij`, допустимы только в headless path и не
  должны использовать host `zellij` пользователя.

## Risks

- слишком ранний ввод config surface без отдельного ADR может привести к
  нестабильному контракту;
- частичный enforcement только в prompt-layer создаст ложное ощущение
  защищенности;
- runtime-only gate без launcher filtering не сможет hard-deny уже видимый
  модели `secret-class` file;
- visibility resolution может оказаться неоднозначным для fork-ов,
  временно недоступного GitHub API или деградировавших metadata;
- publication path легко упустить, хотя именно там возможна утечка локальных
  данных наружу;
- если не зафиксировать lifecycle approval и audit trail, реализация быстро
  разойдется между prompt-layer, launcher-ом и runtime;
- если diagnostics не будут объяснять причину отказа, оператор начнет обходить
  policy вручную.

## Architecture Notes

- visibility и mode resolution должны происходить до запуска workflow-ветки,
  а не после чтения hostile content как будто оно уже trusted;
- permission gates лучше концентрировать в небольшом числе runtime boundaries,
  а не размазывать по call sites;
- secret filtering нужно делать до запуска агента; это не должна быть
  responsibility только prompt-layer или post-factum diagnostics;
- repo-local docs, `AGENTS.md` и `.ai-teamlead/` assets не должны быть
  неявным способом расширить filesystem или network scope;
- explicit approval в MVP должен приходить только через agent session и
  логироваться как отдельный runtime artifact, а не растворяться в общей
  истории диалога без action binding;
- shell output, test logs и generated artifacts нужно рассматривать как
  `untrusted` продолжение hostile scenario, если они возникли из обработки
  недоверенного issue;
- diagnostics должны показывать, какой именно gate сработал и какой
  `operating_mode` применился, не раскрывая локальные секреты.

## ADR Impact

Базовые решения по hostile-input model и `public-safe mode` уже приняты в
ADR `0029` и `0030`.

Отдельный новый ADR на этом этапе не обязателен, если реализация укладывается в
уже принятый контракт.

Новый ADR потребуется, если по ходу implementation будет принято хотя бы одно
из следующих решений:

- отдельная стабильная schema для security config в `settings.yml`;
- новый trusted mechanism для repo-local assets;
- отдельная execution/sandbox model поверх текущего shell/launcher layer.

## Alternatives Considered

1. Оставить security policy только в feature docs и prompt-тексте.

   Отклонено: это не дает enforce-able behavior и противоречит уже принятому
   contract layer.

2. Блокировать любые risky actions абсолютно, без explicit approval path.

   Отклонено: часть операторских сценариев требует управляемого human gate, а
   не только hard deny.

3. Считать owner-authored issue достаточным основанием доверять comments.

   Отклонено: comments остаются отдельным hostile input channel даже внутри
   owner-authored issue.

4. Разрешить `open-intake` и для `public-safe` режима.

   Отклонено: это разрушает baseline из owner/allowlist intake и противоречит
   fail-closed модели public repos.

5. Положиться только на runtime gates и не фильтровать filesystem view launcher-ом.

   Отклонено: если агент уже видит секретный файл, runtime gate не дает
   enforce-able hard deny на сам факт чтения.

## Migration Or Rollout Notes

- rollout должен идти по слоям: visibility/mode resolution, затем permission
  gates, затем prompt/launcher alignment и diagnostics;
- contract pack уже существует, поэтому документация не блокирует старт
  implementation, но должна обновляться раньше или одновременно с runtime
  изменениями;
- проверки для hostile-input paths нужно строить на unit/integration/headless
  уровнях, не полагаясь на host-run;
- до появления хотя бы минимального runtime enforcement public repo support
  нельзя считать production-ready.
