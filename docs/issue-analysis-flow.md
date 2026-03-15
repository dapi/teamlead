# issue-analysis-flow

Статус: draft, evolving
Владелец: владелец репозитория
Роль: SSOT для flow анализа issue
Последнее обновление: 2026-03-14

## Назначение

Этот документ определяет единый источник истины для flow, который анализирует
GitHub issue и приводит к одному из двух результатов:

- сформирован список блокирующих вопросов для пользователя в агентской сессии
- сформирован план реализации, готовый к ревью человеком

Этот flow не запускает реализацию. Реализация будет вынесена в отдельный flow и
сейчас явно вне scope.

После перевода issue в `Ready for Implementation` дальнейший lifecycle должен
обслуживаться отдельным SSOT:

- [issue-implementation-flow.md](./issue-implementation-flow.md)

Этот документ будет заметно развиваться со временем. По мере зрелости workflow
в него могут добавляться новые фазы, статусы, артефакты, правила валидации и
операторские контроли.

## Scope

Вход:

- GitHub issue находится в состоянии `open`
- issue находится в настроенном default GitHub Project
- project status = `Backlog`

Выход:

- issue переведена в статус ожидания уточнений
- или issue переведена в статус ожидания ревью плана

Контекст исполнения:

- flow должен быть применим к произвольному подключенному GitHub-репозиторию
- текущее репо используется как dogfooding-репозиторий для разработки и
  обкатки этого же flow
- у каждого репозитория должен быть свой собственный repo-local конфиг
  `./.ai-teamlead/settings.yml`
- разные репозитории должны иметь возможность запускать независимые экземпляры
  `ai-teamlead` параллельно

## Вне scope

- автоматический старт кодинга
- хранение долгоживущего локального workflow state

## Политика развития

`issue-analysis-flow` это живая спецификация и отдельный SSOT для своего
семейства workflow.

Правила развития:

- каждое существенное изменение flow сначала фиксируется в этом файле
- каждое значимое решение по flow должно быть оформлено отдельным ADR с номером
- при изменении статусов нужно обновлять и список статусов, и правила переходов
- новые обязательные выходные артефакты должны добавляться в контракт результата
- несовместимые изменения должны явно отмечаться в журнале изменений
- реализация должна следовать спецификации, а не формировать ее задним числом
- если документ становится слишком большим, связанные части должны выноситься в
  отдельные документы рядом с основным SSOT

Цель этого правила в том, чтобы flow можно было расширять без потери
аудируемости и без тихого дрейфа поведения.

## Источник истины

Источник истины по состоянию issue это поле статуса в настроенном default
GitHub Project.

Приложение не должно опираться на постоянную локальную базу или локальный state
file, чтобы понимать, на каком этапе находится issue.

Локальные файлы допустимы только как временные рабочие артефакты на время
анализа и могут безопасно удаляться после завершения запуска.

## Требование к переносимости

`issue-analysis-flow` не должен проектироваться как flow, привязанный только к
этому репозиторию.

Обязательные требования:

- repo-specific параметры должны приходить из конфигурации
- конфигурация должна жить в самом целевом репозитории
- flow должен работать на собственном репозитории проекта и на внешнем
  репозитории по одному и тому же контракту
- код flow не должен предполагать фиксированные owner, repo, project id,
  структуру labels или локальные пути, кроме явно сконфигурированных

Для MVP GitHub owner/repo всегда выводятся из текущего git-репозитория и не
переопределяются через `./.ai-teamlead/settings.yml`.

Это требование нужно, чтобы проект можно было использовать как инструмент
общего назначения, а текущее репо служило рабочей dogfooding-средой.

## Статусы GitHub Project

Для `issue-analysis-flow` определяются следующие статусы:

1. `Backlog`
   Значение: issue готова к запуску анализа.
2. `Analysis In Progress`
   Значение: инструмент забрал issue и сейчас выполняет анализ.
3. `Waiting for Clarification`
   Значение: для продолжения анализа нужны ответы пользователя в агентской
   сессии.
4. `Waiting for Plan Review`
   Значение: анализ завершен, план подготовлен и ожидает ревью или подтверждения
   человеком.
5. `Ready for Implementation`
   Значение: план принят человеком, issue готова к отдельному flow реализации.
6. `Analysis Blocked`
   Значение: flow не может продолжаться из-за технической проблемы, нехватки
   доступа или другого нештатного блокера.

## Правила переходов

Разрешенные переходы:

- `Backlog` -> `Analysis In Progress`
- `Analysis In Progress` -> `Waiting for Clarification`
- `Analysis In Progress` -> `Waiting for Plan Review`
- `Analysis In Progress` -> `Analysis Blocked`
- `Waiting for Clarification` -> `Analysis In Progress`
- `Waiting for Plan Review` -> `Ready for Implementation`
- `Waiting for Plan Review` -> `Analysis In Progress`
- `Analysis Blocked` -> `Analysis In Progress`

Запрещенные переходы:

- прямой переход из `Backlog` в `Ready for Implementation`
- прямой переход из `Waiting for Clarification` в `Ready for Implementation`
- любой автоматический переход к реализации

## Условия входа

Issue может войти в этот flow только если одновременно выполняются все условия:

- GitHub issue state = `open`
- issue прикреплена к настроенному default GitHub Project
- project status = `Backlog`

Определение типа задачи:

- в первую очередь используется GitHub labels
- если labels недостаточно, тип может быть выведен из текста issue

Поддерживаемые типы:

- `bug`
- `feature`
- `chore`

Это правило относится к первичному входу issue в анализ через backlog.

Повторный ручной вход в уже начатый analysis flow через `run` допускается по
отдельным правилам запуска ниже.

## Шаги flow

### 1. Claim

Poller или ручной запуск выбирает одну подходящую issue и переводит ее из
`Backlog` в `Analysis In Progress`.

Именно это изменение статуса считается механизмом claim.

При первом успешном claim для issue создается durable-связка с агентской
сессией:

- генерируется `session_uuid`
- issue связывается с этим `session_uuid` в отношении `1 <-> 1`
- target `zellij` session определяется в порядке:
  `--zellij-session` -> `ZELLIJ_SESSION_NAME` -> `zellij.session_name`
- `zellij.tab_name` задается конфигурацией проекта
- orchestration-слой создает или находит нужные session/tab по effective target
  session и `tab_name`
- existing session не должна содержать panes из другого GitHub repo
- после запуска pane в runtime state сохраняются `zellij.session_id`,
  `zellij.tab_id` и `zellij.pane_id`
- session-артефакты сохраняются в `.git/.ai-teamlead/`

### 2. Анализ

Агент читает issue, существующий контекст, labels и контекст репозитория,
который нужен для построения анализа и плана.

Агент должен определить:

- достаточно ли задача специфицирована для подготовки плана
- какой тип issue применим
- какие допущения нужны
- какие есть риски и неизвестные

### 3. Уточнение или план

Если для реализации не хватает критически важной информации:

- агент формулирует конкретные вопросы пользователю
- вопросы задаются в агентской сессии, а не в комментариях GitHub issue
- issue переводится в `Waiting for Clarification`

Если информации достаточно:

- агент формирует пакет анализа
- агент создает versioned analysis-артефакты в каталоге issue
- агент внутренне ревьюит план на противоречия, пробелы и размытые шаги
- результат публикуется пользователю в агентской сессии
- issue переводится в `Waiting for Plan Review`

### 4. Human gate

Human gate обязателен в двух местах:

- при ответах на блокирующие вопросы
- при принятии или отклонении предложенного плана

Если пользователь отвечает на вопросы, issue может быть возвращена в
`Analysis In Progress` и запущена повторно.

Если пользователь принимает план, issue переводится в
`Ready for Implementation`.

После этого дальнейший `run <issue>` должен маршрутизироваться уже в
implementation flow, а не обратно в analysis flow.

Если пользователь отклоняет план или просит доработки, issue может быть
возвращена в `Analysis In Progress`.

## Протокол оператора

В MVP пользователь взаимодействует с flow через агентскую сессию.

На первом этапе не требуется парсер специальных команд. Достаточно явных
намерений пользователя, распознаваемых по смыслу ответа в агентской сессии.

Поддерживаемые действия оператора:

1. Ответить на вопросы.
   Результат: issue переводится из `Waiting for Clarification` в
   `Analysis In Progress`, после чего flow запускается повторно с учетом новых
   данных.
2. Подтвердить план.
   Результат: issue переводится из `Waiting for Plan Review` в
   `Ready for Implementation`.
3. Вернуть план на доработку.
   Результат: issue переводится из `Waiting for Plan Review` в
   `Analysis In Progress`, после чего агент повторно анализирует задачу с учетом
   замечаний.
4. Отложить решение.
   Результат: issue остается в текущем waiting-статусе без перехода.

Для MVP достаточно следующих нормализованных намерений:

- ответы на уточняющие вопросы
- подтверждение плана
- запрос доработки плана
- отсутствие решения

В будущем этот протокол может быть расширен до явных slash-команд, отдельного
TUI/CLI-контрола или формализованных action-кнопок.

## Контракт результата анализа

Каждый успешный анализ должен производить структурированный результат. Минимум
он должен содержать:

- краткое резюме issue
- scope и non-goals
- допущения
- риски и открытые вопросы
- план реализации

Для `feature` issue дополнительно обязательны:

- `User Story`
- `Use Cases`

План должен пройти внутреннее ревью до публикации и не должен содержать
известных пробелов или противоречий.

Если результат анализа включает `План имплементации`, он должен соответствовать
требованиям `docs/implementation-plan.md`.

## Контракт завершения стадии

После завершения работы внутри agent session анализ должен завершаться явным
вызовом:

`ai-teamlead internal complete-stage <session_uuid> --outcome <outcome> --message <message>`

Допустимые значения `outcome`:

- `plan-ready`
- `needs-clarification`
- `blocked`

Семантика:

- `plan-ready`:
  - по возможности коммитит и пушит analysis-артефакты
  - создает draft PR или переиспользует уже существующий
  - переводит issue в `Waiting for Plan Review`
- `needs-clarification`:
  - по возможности коммитит и пушит analysis-артефакты
  - переводит issue в `Waiting for Clarification`
- `blocked`:
  - по возможности коммитит и пушит analysis-артефакты
  - переводит issue в `Analysis Blocked`

Общие правила:

- именно CLI-команда, а не prompt агента, инкапсулирует `git add`,
  `git commit`, `git push`, `gh pr create` и смену project status
- агент не должен выполнять `git commit`, `git push` или `gh pr create`
  самостоятельно
- при успешной смене статуса команда переводит `session.json.status` в
  `completed`
- если push analysis branch не удался, статус issue не должен меняться
- если создание draft PR не удалось, это считается warning, а не blocker для
  смены статуса
- если смена статуса не удалась, session должна оставаться `active`

## Versioned analysis-артефакты

Результат анализа должен быть сохранен как versioned SDD-комплект в каталоге:

- `specs/issues/${ISSUE_NUMBER}/`

Минимальный обязательный набор:

- `README.md`
- `01-what-we-build.md`
- `02-how-we-build.md`
- `03-how-we-verify.md`

Это означает:

- минимум один документ на ось `Что строим`
- минимум один документ на ось `Как строим`
- минимум один документ на ось `Как проверяем`

По умолчанию первая версия должна оставаться компактной:

- если issue небольшая, достаточно именно трех базовых документов по осям
- дополнительные документы создаются только если они реально нужны

`README.md` в каталоге issue должен содержать:

- краткое резюме issue
- ссылку на GitHub issue
- список артефактов по трем осям
- текущий статус анализа
- ссылки на документы, которые нужны для понимания итогового плана реализации

`01-what-we-build.md` должен содержать минимум:

- проблему
- пользователя или роль
- ожидаемый результат
- scope и non-goals
- ограничения, предпосылки и допущения
- минимально достаточный продуктовый контракт

`02-how-we-build.md` должен содержать минимум:

- архитектурный подход
- данные, интерфейсы и ограничения
- конфигурацию и runtime-допущения, если они влияют на поведение
- ключевые технические решения

`03-how-we-verify.md` должен содержать минимум:

- acceptance criteria
- критерии готовности
- инварианты
- test plan
- verification checklist

Для `feature` issue в комплекте артефактов дополнительно обязательны:

- `User Story`
- `Use Cases`

Они могут быть как отдельными разделами внутри `01-what-we-build.md`, так и
вынесенными в отдельные связанные документы, если issue этого требует.

Если для issue создается отдельный `План имплементации`, он должен:

- содержать ссылки на все документы, без которых нельзя корректно выполнить
  задачу
- содержать явный план изменений документации: какие канонические документы,
  summary-слои или шаблоны нужно обновить и что допустимо оставить без
  изменений
- связывать этапы реализации с SSOT, ADR, verification-документами и quality
  bar
- позволять агенту быстро восстанавливать причинно-следственные связи решений
  без ручного поиска по репозиторию

## Правила выбора секций внутри артефактов

Секции внутри analysis artifacts не должны быть одинаковыми для всех задач.

Нужно использовать rule-based модель:

- `core` — обязательны всегда
- `conditional` — обязательны только для релевантных типов задач или проектов
- `scaling` — добавляются для `medium` и `large` задач

Факторы выбора:

- тип задачи: `feature`, `bug`, `chore`
- тип проекта: `product/UI`, `library/API`, `infra/platform`
- размер задачи: `small`, `medium`, `large`

Порядок выбора:

1. Сначала определи тип задачи.
2. Затем оцени размер задачи.
3. Затем определи преобладающий тип проекта.
4. Сначала включи все `core`-секции.
5. Затем добавь релевантные `conditional`-секции.
6. Для `medium` и `large` задач добавь `scaling`-секции.
7. Не добавляй секции, которые не дают новой информации для конкретной issue.

### `README.md`

Обязателен всегда как компактный индекс issue-спеки.

Минимум:

- `Issue`
- `Summary`
- `Status`
- `Artifacts`
- `Open Questions`

### `01-what-we-build.md`

`core`:

- `Problem`
- `Who Is It For`
- `Outcome`
- `Scope`
- `Non-Goals`
- `Constraints And Assumptions`

`conditional`:

- для `feature`:
  - `User Story`
  - `Use Cases`
- для `bug`:
  - `Observed Behavior`
  - `Expected Behavior`
  - `Impact`
- для `chore`:
  - `Motivation`
  - `Operational Goal`

`scaling`:

- для `medium` и `large`:
  - `Dependencies`

### `02-how-we-build.md`

`core`:

- `Approach`
- `Affected Areas`
- `Interfaces And Data`
- `Configuration And Runtime Assumptions`
- `Risks`

`conditional`:

- если есть внешние интеграции:
  - `External Interfaces`
- если есть заметные архитектурные изменения:
  - `Architecture Notes`
- если по issue требуется решение уровня проекта:
  - `ADR Impact`

`scaling`:

- для `medium` и `large`:
  - `Alternatives Considered`
  - `Migration Or Rollout Notes`

### `03-how-we-verify.md`

`core`:

- `Acceptance Criteria`
- `Ready Criteria`
- `Invariants`
- `Test Plan`
- `Verification Checklist`

`conditional`:

- для `bug`:
  - `Regression Checks`
- для `feature`:
  - `Happy Path`
  - `Edge Cases`
- для `chore`:
  - `Operational Validation`
- если есть заметные риски отказа:
  - `Failure Scenarios`
- если для проверки важны runtime-сигналы:
  - `Observability`

Правило компактности:

- для `small` issue включай только `core` и минимально нужные `conditional`
- для `medium` issue включай `core` и все релевантные `conditional`
- для `large` issue включай `core` и все релевантные `conditional`, а при
  необходимости выноси части в отдельные связанные документы

### Акценты по типу проекта

Для `product/UI`:

- в оси `Что строим` усиливай `User Story`, `Use Cases`, `Scope`
- в оси `Как проверяем` усиливай `Acceptance Criteria`, `Happy Path`,
  `Edge Cases`

Для `library/API`:

- в оси `Как строим` усиливай `Interfaces And Data`
- при необходимости добавляй совместимость, контракты и ограничения API
- в оси `Как проверяем` усиливай сценарии совместимости и регрессии

Для `infra/platform`:

- в оси `Что строим` усиливай `Operational Goal`
- в оси `Как строим` усиливай `Risks`, `Migration Or Rollout Notes`
- в оси `Как проверяем` усиливай `Operational Validation`,
  `Failure Scenarios`, `Observability`

### Definition of Done для SDD-комплекта

Анализ считается завершенным только если одновременно выполнены все условия:

- создан `README.md` как индекс issue-спеки
- создан минимум один документ на каждую из трех осей
- внутри каждого документа присутствуют все обязательные `core`-секции
- добавлены все релевантные `conditional`-секции
- комплект остается компактным и не содержит формальных пустых разделов

## Модель выполнения

- `ai-teamlead` реализуется как foreground CLI-утилита с командами `init`,
  `poll`, `run`, `loop`
- issue-level orchestration должна иметь один общий `run`-path
- `poll` и `loop` не должны дублировать claim/re-entry/launch логику, а должны
  переиспользовать этот общий `run`-path
- ручные команды: `poll`, `run`, `loop`
- `max_parallel` хранится в `./.ai-teamlead/settings.yml`
- интервал polling хранится в `./.ai-teamlead/settings.yml`
- целевой режим MVP: `max_parallel: 1`

Экземпляр `ai-teamlead` привязан к одному конкретному репозиторию и его
`./.ai-teamlead/settings.yml`.

Несколько репозиториев могут одновременно иметь свои собственные запущенные
экземпляры `ai-teamlead`, если у каждого есть свой repo-local конфиг и свой
отдельный runtime-контекст.

Команда `poll` выполняет один цикл просмотра project snapshot и не требует
внешнего scheduler для one-shot запуска.

Команда `loop` выполняет бесконечный foreground loop, в котором каждая итерация
переиспользует поведение `poll`, а пауза между итерациями задается через
`runtime.poll_interval_seconds`.

`loop` не является отдельным daemon/supervisor model: остановка foreground
процесса останавливает весь loop.

Если настроена одна общая `zellij` tab, значения выше `1` считаются
некорректными до тех пор, пока не будет спроектирован dispatcher для нескольких
tab или session.

## Правила запуска

### Команда `poll`

`poll` предназначена для одного цикла просмотра проекта и выбора следующей
подходящей issue.

Правила:

- `poll` выбирает только issues со статусом `Backlog`
- `poll` не должен забирать issue из других статусов
- `poll` должен claim-ить не более чем `max_parallel` новых issues за один цикл
- в MVP при `max_parallel: 1` команда `poll` забирает не более одной issue
- при наличии нескольких подходящих issues `poll` выбирает верхнюю issue в
  порядке GitHub Project
- если подходящая issue найдена, `poll` не должен реализовывать отдельный
  issue-level flow; он должен передать выбранную issue в тот же общий `run`-path,
  который используется явной командой `run`
- если подходящая issue не найдена, `poll` завершает цикл без ошибки и пишет
  диагностируемый результат пустого цикла

Результат:

- выбранная issue передается в общий issue-level `run`-path
- внутри этого общего `run`-path issue переводится в `Analysis In Progress`,
  если входной статус и правила переходов это допускают
- после этого для нее запускается тот же launch-flow, что и у явной команды
  `run`
- команда выполняет ровно один polling cycle

### Команда `run`

`run` предназначена для явного запуска flow по конкретной issue по инициативе
пользователя.

Правила:

- `run` принимает явный идентификатор issue или URL issue
- `run` не ищет issues автоматически
- `run` может быть использована для повторного запуска анализа после human gate
- `run` работает только в контексте текущего репозитория и его
  `./.ai-teamlead/settings.yml`
- `run` является каноническим issue-level entrypoint и stage-aware dispatcher
  для claim, re-entry и launcher orchestration
- `run` должен использоваться и при явном ручном запуске, и как внутренний
  orchestration-path после выбора issue командой `poll`
- `run` решает issue-level сценарий целиком: проверка допустимости входа,
  перевод статуса, работа с `session_uuid`, запуск или восстановление launcher
  path в стабильном launch context
- если по текущему status выбран analysis stage, `run` передает агенту
  project-local `issue-analysis-flow`
- `run` передает агенту ссылку на GitHub issue как аргумент запуска
- versioned launcher-файл должен лежать в `./.ai-teamlead/launch-agent.sh`
- `launch-agent.sh` должен запускаться из корня репозитория
- `run` и `poll` должны использовать один и тот же launcher contract
- после старта pane launcher должен вызвать внутреннюю команду
  `ai-teamlead internal bind-zellij-pane <session_uuid>`, которая читает
  `ZELLIJ_PANE_ID` и дописывает `pane_id` в session binding

В рамках analysis branch допустимые входные статусы для `run`:

- `Backlog`
- `Waiting for Clarification`
- `Waiting for Plan Review`
- `Analysis Blocked`

Правила переходов для `run`:

- если issue находится в `Backlog`, она переводится в `Analysis In Progress`
- если issue находится в `Waiting for Clarification`, она переводится в
  `Analysis In Progress` по явной команде оператора `run`, которая считается
  намерением продолжить анализ с reuse существующего `session_uuid` и запуском
  новой pane в том же stable launch context
- если issue находится в `Waiting for Plan Review`, она переводится в
  `Analysis In Progress` по явной команде оператора `run`, которая считается
  намерением вернуть план в доработку с reuse существующего `session_uuid` и
  запуском новой pane
- если issue находится в `Analysis Blocked`, она переводится в
  `Analysis In Progress` только после явного ручного ретрая через `run`
- для waiting-статусов и `Analysis Blocked` `run` требует существующий
  runtime-binding `issue <-> session_uuid`; при его отсутствии запуск
  завершается ошибкой
- при повторном `run` система должна принимать явное решение между
  восстановлением существующего launcher path и созданием нового launcher path с
  тем же `session_uuid`; это решение не должно расходиться между явным `run` и
  внутренним запуском из `poll`

### Команда `loop`

`loop` предназначена для непрерывной foreground-обработки backlog без внешнего
scheduler.

Правила:

- `loop` работает только в контексте текущего репозитория и его
  `./.ai-teamlead/settings.yml`
- `loop` не принимает issue как аргумент
- `loop` выполняет бесконечную последовательность циклов `poll`
- между циклами `loop` делает паузу по `runtime.poll_interval_seconds`
- bootstrap/config/runtime ошибки до входа в loop считаются фатальными и
  завершают команду
- пустой цикл `poll` не завершает `loop`
- ошибка одного цикла не должна делать `loop` непригодным для следующих циклов
- `loop` должен оставлять оператору понятную диагностику старта, исхода и
  завершения каждого цикла
- `loop` не вводит отдельную модель статусов и не меняет правила выбора issue
  относительно `poll`

Результат:

- `loop` переиспользует selection semantics команды `poll`
- `loop` переиспользует тот же issue-level `run`-path, который уже используется
  `poll`
- foreground процесс продолжает работу до внешней остановки оператором или до
  фатальной ошибки bootstrap-уровня

Degraded mode для launcher:

- если в analysis worktree доступен `codex`, launcher запускает его
- если `codex` отсутствует, launcher оставляет shell внутри уже подготовленного
  analysis worktree

Недопустимые сценарии для analysis branch `run`:

- повторный запуск issue из `Ready for Implementation`, если оператор ожидает
  implementation stage, а не явный возврат к analysis
- запуск issue, которая не находится в настроенном default GitHub Project
- запуск issue со статусом, не входящим в контракт `issue-analysis-flow`

### Общие ограничения запуска

- любой запуск должен сначала проверять текущий статус issue в GitHub Project
- изменение статуса в GitHub Project должно происходить до старта анализа
- если изменение статуса не удалось, анализ не должен стартовать
- локальный runtime state не должен использоваться для обхода этих правил
- `poll` после выбора issue должен использовать тот же issue-level `run`-path и
  тот же launch-контракт, что и явная команда `run`
- `loop` должен использовать ровно те же selection semantics, что и `poll`, и
  ровно тот же issue-level `run`-path, что и `poll` и `run`

## Runtime state

Постоянный runtime state намеренно минимален.

Допустимый локальный state:

- временные prompt-файлы
- временные артефакты анализа
- lock-файлы уровня процесса, если они нужны для защиты от параллельного запуска
  poller
- repo-local runtime-артефакты в `.git/.ai-teamlead/`
- durable session-артефакты, связывающие issue и `session_uuid`
- launcher-артефакты и диагностические файлы, нужные для входа в агентскую
  сессию

Точная схема runtime/session-артефактов должна задаваться отдельным документом
feature-уровня.

При чтении runtime-документов нужно различать три разных status-словаря:

- `GitHub Project status`:
  `Backlog`, `Analysis In Progress`, `Waiting for Clarification`,
  `Waiting for Plan Review`, `Ready for Implementation`, `Analysis Blocked`
- `session.json.status`: локальный lifecycle session-binding, например
  `active` или `completed`
- `issues/<issue_number>.json.last_known_flow_status`: последнее локально
  известное значение flow-статуса из GitHub Project

Недопустимый локальный state:

- постоянная локальная база состояния issue
- постоянный локальный state, используемый как источник истины по прогрессу

## Обработка сбоев

Issue должна переводиться в `Analysis Blocked`, если:

- после повторных попыток не удалось изменить GitHub Project
- не удалось получить доступ к нужному контексту репозитория
- недоступен обязательный tooling
- содержимое issue противоречиво и по нему невозможно сформулировать
  содержательные вопросы

По возможности агент должен сообщать пользователю краткую диагностику в своей
сессии.

## Открытые вопросы

- точный шаблон вопросов на уточнение
- точный шаблон финального пакета анализа
- нужно ли вводить явные текстовые команды для оператора уже в первой версии
## Контракт завершения стадии

Agent session сигналит core о результате анализа через CLI-команду:

```
ai-teamlead internal complete-stage <session_uuid> --outcome <outcome> --message <msg>
```

Допустимые значения `outcome`:

- `plan-ready` — SDD-комплект собран, issue → `Waiting for Plan Review`
- `needs-clarification` — нужны ответы, issue → `Waiting for Clarification`
- `blocked` — технический блокер, issue → `Analysis Blocked`

Команда инкапсулирует: git add/commit, git push, draft PR (для plan-ready),
смену статуса в GitHub Project, обновление session.json.

Агент НЕ выполняет git/gh операции самостоятельно.

Спецификация: ADR-0020, `docs/adr/0020-agent-session-completion-signal.md`.

## Связанные документы

- [docs/adr/0017-minimal-sdd-artifact-set-for-issue-analysis.md](adr/0017-minimal-sdd-artifact-set-for-issue-analysis.md)
- [docs/adr/0019-conditional-sections-by-task-type-project-type-and-size.md](adr/0019-conditional-sections-by-task-type-project-type-and-size.md)
- [docs/adr/0013-agent-session-history-as-dialog-source.md](adr/0013-agent-session-history-as-dialog-source.md)
- [docs/adr/0020-agent-session-completion-signal.md](adr/0020-agent-session-completion-signal.md)
- [docs/implementation-plan.md](./implementation-plan.md)
- [docs/features/0001-ai-teamlead-cli/README.md](features/0001-ai-teamlead-cli/README.md)

## Журнал изменений

### 2026-03-14

- добавлен контракт завершения стадии `complete-stage` (ADR-0020)
- закрыт открытый вопрос о machine-readable артефактах — решение через
  CLI-команду `ai-teamlead internal complete-stage`

### 2026-03-13

- создан начальный SSOT для `issue-analysis-flow`
- зафиксирован scope только на анализе и планировании
- определена модель статусов GitHub Project как источник истины
- добавлены обязательные human gate для уточнений и принятия плана
- для `feature` анализа сделаны обязательными `User Story` и `Use Cases`
- документ объявлен evolving SSOT с обязательным журналом изменений
- зафиксировано, что вопросы пользователю задаются в агентской сессии, а не в
  комментариях GitHub issue
- добавлен протокол оператора для human gate в агентской сессии
- добавлены явные правила запуска для команд `poll` и `run`
- добавлено требование к переносимости flow между разными репозиториями
- зафиксирована repo-local модель `./.ai-teamlead/settings.yml` и независимые
  экземпляры на разные репозитории
- execution model MVP изменен на foreground CLI-утилиту с командой `poll`
- добавлено требование оформлять значимые решения по flow отдельными ADR с
  номером
- добавлено правило выносить разросшуюся документацию фич в отдельные
  директории связанных документов
- зафиксировано хранение repo-local runtime-артефактов в `.git/.ai-teamlead/`
- зафиксирован минимальный CLI-контракт с командами `poll` и `run`
- GitHub owner/repo для MVP жестко привязаны к текущему git-репозиторию
- добавлена durable-связка `issue <-> session_uuid` и сохранение session-
  артефактов
- зафиксирован детерминированный порядок выбора issue из `Backlog`
- зафиксировано, что источником диалога для MVP является история агентской
  сессии, а не отдельные JSON-артефакты

### 2026-03-14

- выровнены минимальные требования к issue-analysis артефактам с
  repo-level documentation structure
- стандартизированы названия секций `User Story` и `Use Cases`
- уточнен контракт повторного `run` и degraded mode launcher-а без `codex`
- добавлено требование к `Плану имплементации` и его обязательным ссылкам на
  связанные документы
- зафиксирован проект нового CLI-контракта: `poll` как one-shot цикл выбора,
  `run` как общий issue-level orchestration-path и `loop` как foreground-обертка
  над `poll`
- уточнено, что первичный вход в flow идет через `Backlog`, а повторный ручной
  вход регулируется отдельными правилами `run`
- добавлен контракт завершения стадии через
  `ai-teamlead internal complete-stage`
- orchestration commit/push/draft PR выведен из prompt-обязанностей агента в
  CLI-команду завершения стадии

### 2026-03-15

- добавлено требование включать в `План имплементации` явный план изменений
  документации
