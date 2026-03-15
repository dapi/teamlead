# Issue 56: Как строим

Статус: draft
Последнее обновление: 2026-03-15

## Approach

Реализацию нужно делать как поэтапное внедрение уже принятого security
contract, а не как новую перепридуманную модель.

Технический подход:

- использовать feature `0006`, SSOT `untrusted-input-security` и ADR
  `0029/0030` как канонический источник требований;
- добавить в runtime детерминированное вычисление `repo_visibility` и
  `operating_mode` до входа в execution path;
- внедрить intake layer для public repos, который ограничивает auto-start по
  issue author policy, но не повышает trust comments;
- маркировать действия по типу риска и направлять high-risk actions в явные
  permission gates;
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
  будущая config surface для security policy, allowlist и safe-mode override;
- `src/shell.rs`:
  execution gate для опасных команд и diagnostics по отказам;
- `src/complete_stage.rs` и publication path:
  защита от публикации локальных чувствительных данных;
- `.ai-teamlead/launch-agent.sh` и staged prompts:
  выравнивание operator messaging и различение `operator intent` против
  hostile content;
- `docs/untrusted-input-security.md`, feature `0006`, ADR `0029/0030`:
  возможная синхронизация, если implementation уточнит config/runtime детали;
- `tests/integration/` и `.ai-teamlead/tests/agent-flow/`:
  headless verification сценарии для public-safe режима и hostile inputs.

## Interfaces And Data

Минимальная domain-модель для реализации:

- `repo_visibility`: `public`, `private`, `unknown`;
- `operating_mode`: `standard`, `public-safe`;
- `intake_policy`: `owner-only`, `allowlist`, `open-intake`;
- `input_trust_class`: `trusted-control`, `semi-trusted-repo`, `untrusted`,
  `sensitive-local`;
- `approval_state`: `not-required`, `required`, `granted`, `denied`.

Минимальные правила обработки:

- `repo_visibility = public` всегда включает `operating_mode = public-safe`;
- `repo_visibility = unknown` не может ослаблять ограничения;
- `intake_policy` влияет только на auto-start, но не повышает trust-класс
  issue thread;
- `input_trust_class = untrusted` не может инициировать собственное повышение
  привилегий;
- `approval_state = granted` относится к конкретному действию, а не к
  неограниченному сеансу целиком.

Ключевые интерфейсы:

- `run` и `poll` как entrypoints, где выбираются `repo_visibility`,
  `operating_mode` и intake policy;
- GitHub integration layer, который должен вернуть достаточно metadata для
  visibility и issue author policy;
- shell execution layer, который должен различать обычное исполнение и
  dangerous execution;
- publication path, который должен учитывать риск data exfiltration;
- project-local prompts и launcher context, которые не должны трактовать
  hostile content как trusted instruction.

## Configuration And Runtime Assumptions

- текущий `settings.yml` пока не содержит финальную security schema, поэтому
  первая реализация должна быть fail-closed и не опираться на отсутствие полей
  как на разрешение;
- default policy для public repos должна быть безопасной даже без явной
  конфигурации;
- если visibility не удается определить через GitHub metadata, runtime должен
  оставаться в `public-safe`;
- enforcement нельзя сводить только к системному prompt или дисциплине модели;
- проверки, затрагивающие `zellij`, допустимы только в headless path и не
  должны использовать host `zellij` пользователя.

## Risks

- слишком ранний ввод config surface без отдельного ADR может привести к
  нестабильному контракту;
- частичный enforcement только в prompt-layer создаст ложное ощущение
  защищенности;
- visibility resolution может оказаться неоднозначным для fork-ов,
  временно недоступного GitHub API или деградировавших metadata;
- publication path легко упустить, хотя именно там возможна утечка локальных
  данных наружу;
- если diagnostics не будут объяснять причину отказа, оператор начнет обходить
  policy вручную.

## Architecture Notes

- visibility и mode resolution должны происходить до запуска workflow-ветки,
  а не после чтения hostile content как будто оно уже trusted;
- permission gates лучше концентрировать в небольшом числе runtime boundaries,
  а не размазывать по call sites;
- repo-local docs, `AGENTS.md` и `.ai-teamlead/` assets не должны быть
  неявным способом расширить filesystem или network scope;
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
