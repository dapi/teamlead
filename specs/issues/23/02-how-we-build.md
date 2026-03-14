# Issue 23: Как строим

## Approach

Изменение делаем как локальное ужесточение существующего template contract, а
не как новую общую template-подсистему.

Технический подход:

- убрать bootstrap placeholder `__SESSION_NAME__` из `templates/init/settings.yml`
- перестать материализовывать `zellij.session_name` в `src/init.rs`
- рендерить `zellij.session_name` в рантайме из raw config тем же базовым
  механизмом string interpolation, который уже используется для
  `launch_agent.*`
- ввести для `zellij.session_name` отдельное правило допустимых переменных:
  только `${REPO}`
- если после рендера в `zellij.session_name` остаются `${...}`, завершать
  запуск диагностической ошибкой

## Affected Areas

- `templates/init/settings.yml`
- `src/init.rs`
- `src/app.rs` или соседний shared render-layer для config templates
- `src/config.rs` и тесты конфигурации
- путь запуска `zellij`, который использует `zellij.session_name`
- документация в `README.md`, feature docs, SSOT и ADR

## Interfaces And Data

Входные данные:

- raw `zellij.session_name` из `./.ai-teamlead/settings.yml`
- canonical `REPO` из `RepoContext.github_repo`

Выходные данные:

- resolved `session_name`, которое используется для поиска или создания
  `zellij` session

Контракт поля:

- literal строка без `${...}` допустима и используется как есть
- template строка с `${REPO}` допустима и рендерится в runtime
- любая другая переменная в `zellij.session_name` недопустима
- остаток `${...}` после рендера считается config error

Желательная форма реализации:

- не размазывать special-case по `init`, `app` и `zellij`
- вынести рендеринг `zellij.session_name` в одно место с явной проверкой
  допустимых placeholders

## Configuration And Runtime Assumptions

- `zellij.tab_name` остается literal stable name и не требует template support
- `${REPO}` для `zellij.session_name` должен совпадать с `${REPO}` в
  `launch_agent.*`
- значение `REPO` определяется по GitHub remote slug, а не по имени локальной
  директории
- новый bootstrap default меняет только freshly initialized репозитории;
  существующие literal-конфиги остаются валидными

## Risks

- более строгая валидация может сломать репозитории, которые уже записали в
  `zellij.session_name` неподдерживаемый placeholder
- для новых репозиториев bootstrap default изменит имя session относительно
  старого паттерна `{repo_name}-ai-teamlead`
- если документация будет обновлена не полностью, репозиторий получит дрейф
  между README, feature docs, ADR и кодом

## Architecture Notes

- текущий hardcoded suffix `-ai-teamlead` должен исчезнуть из runtime-кода и из
  bootstrap path
- существующий `render_template` можно переиспользовать, но рядом нужна явная
  policy-check логика для `zellij.session_name`
- ошибка должна возникать до попытки открыть `zellij` session, чтобы оператор
  видел причину в конфигурации, а не в побочных ошибках launcher path

## ADR Impact

По правилам [docs/documentation-process.md](../../../docs/documentation-process.md)
это новое значимое решение уровня проекта, поэтому нужен новый ADR, а не просто
расширение старых.

Новый ADR должен зафиксировать:

- что `zellij.session_name` становится template-capable полем
- что для него поддерживается только `${REPO}`
- что bootstrap default переносится в versioned config template
- что canonical repo identifier для launcher templates и `zellij` общий

Существующие документы `ADR-0014` и `ADR-0016`, README, SSOT и feature-спеки
нужно синхронизировать ссылками и формулировками с новым решением.
