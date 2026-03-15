# Issue 33: Как строим

Статус: approved
Последнее обновление: 2026-03-14
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-14T23:06:53+03:00

## Approach

Решение строится как явное разделение трех слоев, которые сейчас смешаны в
одном активном YAML:

1. `schema classification`
   Каждый config key получает явную категорию:
   `required-without-default`, `defaulted-by-application` или
   `example-only extension`.
2. `canonical default-layer`
   Все runtime defaults для defaulted-полей живут в одном Rust-layer и
   используются при загрузке конфига.
3. `documented template`
   `templates/init/settings.yml` остается versioned bootstrap-артефактом, но
   показывает defaulted-поля и примеры в закомментированном виде, а не как
   обязательный активный YAML.

Практически это означает:

- перестать считать отсутствие defaulted-поля ошибкой само по себе;
- грузить YAML как слой override-значений поверх canonical defaults;
- валидировать required-поля отдельно и явно;
- ввести тесты, которые сравнивают default-layer со структурой template и не
  дают схеме эволюционировать молча.

## Affected Areas

- `src/config.rs`
  модель загрузки конфига, merge default-layer и validation required-полей;
- `templates/init/settings.yml`
  переход от активного runtime YAML к documented zero-config шаблону;
- `src/init.rs`
  bootstrap `settings.yml` без материализации runtime defaults в активном виде;
- `README.md` и `docs/features/0002-repo-init/*`
  синхронизация repo-level и feature-level описания нового контракта;
- unit и integration tests для config loader, `init` и guardrail;
- новый ADR по default-layer и классификации schema keys.

## Interfaces And Data

### 1. Классификация полей

Для текущего контракта issue предлагает следующую базовую классификацию.

`required-without-default`:

- `github.project_id`

`defaulted-by-application`:

- `issue_analysis_flow.statuses.*`
- `issue_implementation_flow.statuses.*`
- `runtime.max_parallel`
- `runtime.poll_interval_seconds`
- `zellij.session_name`
- `zellij.tab_name`
- `zellij.layout`
- `launch_agent.analysis_branch_template`
- `launch_agent.worktree_root_template`
- `launch_agent.analysis_artifacts_dir_template`
- `launch_agent.implementation_branch_template`
- `launch_agent.implementation_worktree_root_template`
- `launch_agent.implementation_artifacts_dir_template`

`example-only extension`:

- отдельные закомментированные примеры допустимы только при явной пометке, что
  они не являются runtime default и лишь показывают возможный opt-in режим.

### 2. Модель загрузки

Предпочтительное направление реализации:

- ввести raw-структуру override-конфига, где секции и поля могут отсутствовать;
- ввести canonical default-layer для всех defaulted-полей;
- после парсинга строить итоговый `Config` как `defaults + overrides`;
- затем отдельно валидировать `required-without-default` и запреты на пустые
  строки.

Такая модель нужна, потому что comment-only `settings.yml` не должен ломать
парсинг defaulted-полей, но required-поля все равно должны диагностироваться
явно и детерминированно.

### 3. Guardrail contract

Guardrail должен отвечать на три вопроса:

1. Все ли schema keys попали в одну из допустимых категорий.
2. Все ли defaulted-поля отражены в bootstrap template.
3. Не разошлись ли template-примеры с фактическими runtime defaults.

Практическая форма guardrail:

- тест, который проверяет, что каждый schema key учтен в metadata-слое
  классификации;
- тест, который извлекает из `templates/init/settings.yml` комментарии,
  относящиеся к runtime defaults, и сравнивает их с canonical default-layer;
- отдельный тест, который проверяет presence и диагностику для
  `required-without-default` полей;
- явный escape hatch только для `example-only extension`, чтобы допустимое
  отличие шаблона от runtime default не было неявной дырой в контракте.

## Configuration And Runtime Assumptions

- `settings.yml` по-прежнему загружается только из `./.ai-teamlead/settings.yml`
  текущего репозитория.
- Отсутствие defaulted-полей в активном YAML должно быть эквивалентно явной
  записи canonical default-значений.
- Пустой или comment-only `settings.yml` допустим как bootstrap-состояние для
  defaulted-полей, но не отменяет обязательность `github.project_id` для
  runtime-path, который работает с GitHub Project.
- Bootstrap template должен оставаться удобным для чтения человеком и не
  превращаться в автоматически сгенерированный безликий dump.
- Старый fully materialized YAML остается валидным override-слоем и не требует
  миграции.

## Risks

- Если merge-path будет смешан с validation без явного разделения, loader
  начнет давать неочевидные ошибки на пустой YAML или частично отсутствующие
  секции.
- Если guardrail проверяет только template или только defaults, drift между
  слоями останется возможным.
- Если required-поля случайно попадут в default-layer, репозиторий получит
  скрытое поведение вместо явной operator-facing ошибки.
- Если `example-only extension` не будет явно размечен, он станет loophole для
  тихого расхождения template и runtime.
- Неаккуратное обновление README и feature docs оставит в проекте старое
  описание активного bootstrap YAML, хотя код уже будет жить по другой модели.

## Architecture Notes

- Не стоит решать задачу только добавлением `#[serde(default)]` к текущему
  `Config`: это не покрывает missing required-поля верхнего уровня и не дает
  явной классификации schema keys.
- Лучше отделить `raw config file` от итогового validated `Config`, чтобы merge
  defaults и runtime-валидация были прозрачны и тестируемы по отдельности.
- Canonical default-layer должен быть пригоден и для runtime merge, и для
  guardrail-тестов, иначе источник истины снова раздвоится.
- Template должен документировать и required placeholders, и defaulted values,
  но эти две группы нельзя проверять одной и той же логикой.

## ADR Impact

Нужен отдельный ADR.

Причина: issue меняет устойчивый repo-level контракт `settings.yml` сразу по
трем измерениям:

- разделяет documented template и runtime override-layer;
- вводит canonical default-layer как единственный источник истины для
  defaulted-полей;
- фиксирует policy-level guardrail на эволюцию схемы.

Новый ADR должен зафиксировать:

- категории schema keys;
- rule, что required-поля не получают скрытые defaults;
- rule, что runtime defaults живут в одном кодовом слое;
- policy для `example-only extension` и допустимых отличий шаблона;
- роль `templates/init/settings.yml` как documented bootstrap, а не активного
  источника runtime defaults.

## Alternatives Considered

### Оставить активный YAML, но смягчить validation

Отклонено.

Это временно уменьшит количество ошибок у пользователя, но не уберет главную
проблему: template по-прежнему будет дублировать runtime defaults и оставаться
вторым источником истины.

### Генерировать весь `settings.yml` на лету только из Rust defaults

Не брать в первую версию.

Такой путь уменьшает drift, но делает bootstrap шаблон слишком техническим и
плохо приспособленным для человеко-ориентированных комментариев, секций и
example-only подсказок. Для первой версии достаточно versioned template плюс
жесткий guardrail.

### Дать defaults всем полям, включая `github.project_id`

Отклонено.

Это противоречит repo-local контракту и скрывает обязательную привязку к
конкретному GitHub Project за фиктивным или магическим fallback.

## Migration Or Rollout Notes

- Изменение должно быть backward-compatible для уже существующих репозиториев с
  полным `settings.yml`.
- Новый zero-config шаблон затрагивает прежде всего freshly initialized
  репозитории.
- README, feature `0002-repo-init` и новый ADR нужно обновлять вместе с кодом,
  иначе проект получит документированный drift в верхнем слое.
- Integration-тест `init` должен проверять не только наличие `settings.yml`, но
  и форму шаблона: defaulted-поля закомментированы, required placeholders
  остаются явно видимыми.
