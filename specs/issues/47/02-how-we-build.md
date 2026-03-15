# Issue 47: Как строим

Статус: approved
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-15T11:48:33+03:00

## Approach

Изменение оформляется как локальное расширение существующего launcher contract,
а не как redesign `zellij`-orchestration.

Технический подход:

- добавить в config/domain явный enum `launch_target` со значениями
  `pane | tab`;
- использовать `pane` как canonical default-layer при отсутствии поля в YAML;
- добавить optional CLI override `--launch-target` только для команды `run`;
- вычислять effective launch target один раз в общем app-layer до вызова
  runtime/zellij adapter;
- передавать в launcher уже resolved `session_name`, `tab_name` и
  `launch_target`, чтобы `run` и `poll` использовали один и тот же dispatch;
- разделить zellij launch path на две ветки:
  - `tab`: существующий `new-tab --layout` path;
  - `pane`: reuse existing shared tab или create shared tab first, затем
    открыть новую pane в нем;
- сохранить `launch-agent.sh` и `bind-zellij-pane` как неизменный contract:
  в обоих режимах конечной точкой остается zellij pane с тем же stage entrypoint.

## Affected Areas

- `templates/init/settings.yml` и repo-local `./.ai-teamlead/settings.yml`;
- `src/config.rs` и unit-тесты загрузки/валидации конфига;
- `src/cli.rs` для public `run --launch-target`;
- `src/app.rs` или соседний orchestration-layer для resolution precedence;
- `src/zellij.rs` для dispatch между existing `tab` path и новым `pane` path;
- `src/runtime.rs`, `launch.log` и operator-facing output, если нужно отразить
  effective launch mode и effective tab context;
- integration/headless tests для launcher behavior;
- SSOT/feature docs и новый ADR по launcher contract.

## Interfaces And Data

Входные данные:

- `zellij.launch_target` из `settings.yml`;
- optional `run --launch-target <pane|tab>`;
- уже существующие `zellij.session_name`, `zellij.tab_name`,
  optional `zellij.tab_name_template`, `zellij.layout`;
- `issue_number`, `session_uuid` и repo context.

Выходные данные:

- `effective_launch_target` в одном из двух значений: `pane` или `tab`;
- `effective_tab_name`:
  - для `pane` всегда stable `zellij.tab_name`;
  - для `tab` определяется существующим tab-naming contract;
- operator-visible diagnostics, где явно видно:
  - выбранный `launch_target`;
  - effective `session_name`;
  - effective `tab_name`;
  - `tab_id` и `pane_id` после привязки.

Предлагаемый контракт resolution:

1. `run --launch-target <...>`, если задан и непустой;
2. `zellij.launch_target` из repo-local config;
3. встроенный runtime default `pane`.

Public CLI contract:

- `run` получает `--launch-target`;
- `poll` и `loop` не получают этот флаг;
- внутренний dispatch должен принимать уже resolved target, чтобы `poll` и
  `run` работали через одну и ту же zellij-реализацию.

## Configuration And Runtime Assumptions

- `zellij.launch_target` сериализуется как строковый enum и принимает только
  `pane` или `tab`;
- отсутствие поля не является ошибкой: runtime применяет `pane`;
- bootstrap template и `init` должны явно показывать effective default через
  commented line `launch_target: "pane"`;
- `pane`-режим не использует `tab_name_template` даже если поле задано;
- `tab`-режим продолжает использовать уже принятый tab naming contract;
- выбор `launch_target` не влияет на target session resolution из
  [../../../docs/adr/0023-zellij-session-target-resolution.md](../../../docs/adr/0023-zellij-session-target-resolution.md);
- current repo-scope guard для existing session с panes другого repo остается
  обязательным для обеих веток.

## External Interfaces

Изменение затрагивает только уже существующие внешние интеграции:

- `clap`-CLI contract для `run`;
- `serde`/YAML contract для `settings.yml`;
- `zellij` IPC/CLI path:
  - `new-tab --layout` для режима `tab`;
  - tab lookup/create + `new-pane` в target tab для режима `pane`;
- runtime artifacts в `.git/.ai-teamlead/sessions/<session_uuid>/`.

## Risks

- если `pane` path не будет явно проверять уникальность target tab, launcher
  может silently уйти в произвольную вкладку;
- если precedence resolution будет размазан по нескольким местам, `run` и
  `poll` начнут расходиться по effective behavior;
- если `pane` path начнет использовать `tab_name_template`, shared tab contract
  сломается;
- если diagnostics не будут показывать выбранный mode, регрессии станет трудно
  разбирать по `launch.log` и stdout;
- если public override добавить и в `poll`, автоматический path потеряет
  детерминированность и станет хуже документироваться.

## Architecture Notes

- полезно ввести отдельный resolved launcher struct, а не передавать raw
  `ZellijConfig` плюс override кусками;
- текущая функция `resolve_launch_zellij_config(...)` должна эволюционировать из
  simple tab-name resolver в общий resolution layer для `tab_name` и
  `launch_target`;
- `runtime.create_claim_binding(...)` должен получать уже resolved
  `effective_tab_name`, чтобы `session.json` отражал реальный launch context;
- в `src/zellij.rs` нужны две явные ветки:
  - `launch_in_new_tab(...)`;
  - `launch_in_existing_or_new_shared_tab(...)`;
- versioned `.ai-teamlead/zellij/analysis-tab.kdl` остается source of truth для
  создания analysis tab:
  - всегда в режиме `tab`;
  - в режиме `pane`, когда shared tab еще не существует;
- direct `new-pane` path используется только когда target shared tab уже найден
  и однозначен.

## ADR Impact

По правилам
[../../../docs/documentation-process.md](../../../docs/documentation-process.md)
изменение затрагивает public CLI contract, versioned config contract и runtime
semantics launcher path, поэтому нужен отдельный ADR.

Предлагаемое решение:

- создать новый ADR про `zellij.launch_target`, runtime default `pane`,
  precedence `CLI -> settings -> default` и решение не добавлять public
  override в `poll`/`loop`.

При этом:

- [../../../docs/adr/0023-zellij-session-target-resolution.md](../../../docs/adr/0023-zellij-session-target-resolution.md)
  не переписывается, а получает ссылку как соседний precedence contract;
- `#49` остается отдельным ADR/analysis слоем для naming semantics ветки `tab`.

## Alternatives Considered

1. Добавить `--launch-target` и в `poll`, и в `loop`.

   Отклонено: это размывает deterministic repo-level behavior автоматического
   path и увеличивает CLI surface без подтвержденной пользы.

2. Сделать только CLI override без repo-level config поля.

   Отклонено: у владельца репозитория не останется versioned default contract.

3. Сохранить всегда `new-tab` и считать `pane` не отдельным launcher mode, а
   просто другой naming policy.

   Отклонено: это не решает shared tab сценарий и не дает reuse existing tab.

## Migration Or Rollout Notes

- существующие конфиги без `zellij.launch_target` миграции не требуют;
- repo-local `settings.yml` и `templates/init/settings.yml` должны быть
  синхронно обновлены, чтобы effective default был discoverable;
- headless integration coverage должна появиться раньше ручного dogfooding,
  потому что host `zellij` пользователя off-limits;
- если в основной ветке уже принят контракт issue `#49`, `tab`-ветка должна
  использовать его как готовый dependency, а не дублировать реализацию.
