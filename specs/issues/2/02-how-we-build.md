# Issue 2: Как строим

Статус: draft
Последнее обновление: 2026-03-14

## Approach

Эта задача не должна превращаться в новый orchestration-layer. Базовая идея
проще: закрепить content-contract анализа на уже существующем launch path.

Предпочтительный подход:

- использовать текущий `run`/`poll` и project-local `launch-agent.sh` как
  единственный реальный путь запуска анализа
- оставить ответственность Rust core за claim, session binding, worktree
  preparation и финализацию стадии
- оставить ответственность markdown flow за содержание analysis output
- сделать минимальный SDD-комплект обязательным результатом каждого analysis
  run
- усилить staged prompts так, чтобы они последовательно проводили агента по
  осям `Что строим` -> `Как строим` -> `Как проверяем`
- покрыть решение integration/smoke проверками, которые подтверждают не только
  запуск агента, но и появление ожидаемого набора файлов и секций

Иначе говоря, задача доводит уже существующий flow до состояния
`launcher + prompts + verification`, а не создает новый runtime-контур.

## Affected Areas

- `docs/issue-analysis-flow.md`
  системный SSOT минимального SDD-комплекта, rule-based секций и критериев
  результата
- `./.ai-teamlead/flows/issue-analysis-flow.md`
  project-local entrypoint с прямой инструкцией создавать артефакты в
  `specs/issues/${ISSUE_NUMBER}`
- `./.ai-teamlead/flows/issue-analysis/`
  staged prompts по трем осям, которые задают секции и порядок сбора контекста
- `./.ai-teamlead/launch-agent.sh`
  должен гарантировать корректный worktree context, `AI_TEAMLEAD_*` env vars и
  готовый каталог артефактов до старта агента
- `templates/init/**`
  должны зеркалить актуальный project-local contract, чтобы новые репозитории
  не bootstrap-ились со старым flow
- integration fixtures и headless smoke tests
  должны проверять полный минимальный комплект, а не только факт запуска

## Interfaces And Data

Ключевые входы:

- `AI_TEAMLEAD_ISSUE_URL`
- `AI_TEAMLEAD_SESSION_UUID`
- `AI_TEAMLEAD_ANALYSIS_BRANCH`
- `AI_TEAMLEAD_WORKTREE_ROOT`
- `AI_TEAMLEAD_ANALYSIS_ARTIFACTS_DIR`
- GitHub issue body, labels и project status
- project-local flow и staged prompts из репозитория

Ключевые выводы:

- каталог `specs/issues/${ISSUE_NUMBER}/`
- `README.md`
- `01-what-we-build.md`
- `02-how-we-build.md`
- `03-how-we-verify.md`

Логика выбора секций опирается на три внутренних классификации:

- тип задачи: `feature`, `bug`, `chore`
- тип проекта: для текущего репозитория `infra/platform`
- размер задачи: `small`, `medium`, `large`

Именно эта классификация должна определять, какие секции появляются в
артефактах, а не случайная манера формулировок конкретного прогона.

## Configuration And Runtime Assumptions

- `launch_agent.analysis_artifacts_dir_template` остается каноническим способом
  вычислить путь к каталогу issue-артефактов
- агент стартует из analysis worktree, а не из primary repo root
- каталог артефактов создается launcher-ом заранее, чтобы агент не тратил
  контекст на shell-подготовку
- `complete-stage` завершает analysis session после того, как артефакты уже
  созданы; сама задача не должна подменять content-contract логикой finalization
- `zellij`-связанные проверки выполняются только в headless/sandbox-safe среде
- уже существующие analysis-артефакты прошлых issue не требуют миграции, если
  соответствуют действующему минимальному контракту

## Risks

- staged prompts могут оставаться слишком общими, и тогда агент будет
  варьировать названия секций или пропускать обязательные части
- integration tests могут проверить только existence файлов, но не подтвердить
  rule-based содержание
- repo-local flow и `templates/init` могут разойтись, из-за чего новые
  репозитории получат устаревший contract layer
- small issues могут перегружаться лишними разделами, если не зафиксировать
  правило минимальности достаточно явно
- real smoke run может пройти на одной issue, но не выявить gaps для других
  task types, если не добавить targeted fixture coverage

## External Interfaces

- `codex` или `claude`
  реальный агент, который получает markdown flow и должен сформировать
  артефакты
- `gh`
  источник issue context и проектных статусов
- `git worktree`
  обеспечивает отдельное analysis workspace
- `zellij`
  предоставляет runtime-контекст agent session

## Architecture Notes

Нужное разделение ответственности:

- CLI/core orchestration отвечает за запуск, binding и завершение стадии
- project-local flow отвечает за поведение агента в рамках анализа
- staged prompts отвечают за структуру и полноту контента
- SDD-комплект в `specs/issues/<issue>/` является task-specific результатом, а
  не новым SSOT для уровня репозитория

Это важно, потому что попытка зашить смысл секций в Rust-код сломает
contract-first модель и сделает эволюцию flow дороже, чем обновление
versioned markdown layer.

## ADR Impact

Задача реализует уже принятые решения:

- [../../../docs/adr/0017-minimal-sdd-artifact-set-for-issue-analysis.md](../../../docs/adr/0017-minimal-sdd-artifact-set-for-issue-analysis.md)
- [../../../docs/adr/0019-conditional-sections-by-task-type-project-type-and-size.md](../../../docs/adr/0019-conditional-sections-by-task-type-project-type-and-size.md)

Новый ADR не требуется, если в ходе реализации не меняются:

- минимальный набор обязательных файлов
- rule-based модель выбора секций
- границы ответственности между core, launcher и prompt layer

## Alternatives Considered

### Оставить результат анализа только в чате

Отклонено, потому что это не создает versioned и переиспользуемый артефакт для
human review и следующего implementation stage.

### Свести весь output к одному `analysis.md`

Отклонено, потому что это разрушает трехосевую структуру документации и
противоречит принятому SDD-контракту.

### Проверять только launcher, не проверяя содержимое prompt layer

Отклонено, потому что issue прямо про реальное создание SDD-артефактов, а не
про сам факт запуска агента.

### Жестко валидировать секции только кодом core-приложения

Отклонено как преждевременное усложнение. Канонический контракт секций должен
оставаться в versioned docs/prompt layer, а не дублироваться hardcoded логикой
без явной необходимости.

## Migration Or Rollout Notes

- сначала должен быть выровнен канонический слой:
  SSOT, project-local flow, staged prompts и init templates
- затем нужен как минимум один живой dogfooding run на реальной issue
- после этого integration fixtures стоит усилить проверками полного минимального
  комплекта и ключевых conditional sections
- отдельной миграции уже созданных `specs/issues/*` не требуется, если их
  структура согласуется с принятым контрактом
