# Feature 0003: Что строим

## Проблема

Нужно отделить два разных слоя:

- `issue-analysis-flow` как markdown prompt для агента
- orchestration flow, который подготавливает среду и запускает этого агента

Без такого разделения prompt и launcher начинают смешиваться, а branch/worktree,
`zellij`, session binding и запуск агента оказываются размазаны по разным
документам без явного контракта.

## Пользователь

Основной пользователь:

- владелец репозитория
- оператор, который запускает `poll` или `run`
- разработчик, который работает через `zellij` и agent session

## Результат

Полезным результатом считается orchestration flow, в котором:

- `poll` или `run` выбирают issue
- issue связывается с `session_uuid`
- открывается новая pane в корректном `zellij` context, причем repo может
  выбрать default mode между shared `pane` и отдельным `tab`
- запускается versioned `./.ai-teamlead/launch-agent.sh`
- именно `launch-agent.sh` делает branch/worktree/init/agent start
- реальный агент стартует только после подготовки analysis worktree
- analysis tab выглядит как родной tab выбранной `zellij` session, а не как
  минимальная техническая вкладка launcher'а

## Scope

В первую версию входит:

- единый launch path для `poll` и `run`
- versioned `./.ai-teamlead/launch-agent.sh`
- naming contract для `zellij.session_name` и `zellij.tab_name`
- launch-target contract для `zellij.launch_target` и `run --launch-target`
- session binding с сохранением `pane_id`
- правила поведения при corner cases для session/tab
- минимальный lifecycle вокруг analysis branch/worktree
- versioned contract для внешнего вида analysis tab

## Вне scope

- resurrect/restore `zellij` session
- автоматическое восстановление агентской сессии после падения
- отдельный implementation launcher
- автоматическое лечение конфликтных `zellij` состояний

## Ограничения и предпосылки

- `poll` фактически вызывает тот же launch path, что и `run`
- `issue-analysis-flow` остается отдельным prompt-файлом
- `launch-agent.sh` исполняется из корня репозитория
- `launch-agent.sh` получает первым аргументом `session_uuid`, вторым `issue_url`
- внешний вид analysis tab должен задаваться явным contract-level способом, а
  не неявной попыткой восстановить live-state session
