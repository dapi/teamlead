# Feature 0007: Что строим

## Проблема

Сейчас в `launch_target = tab` система по умолчанию создает вкладку с именем
`issue-analysis`, хотя issue-specific вкладка ожидается как отдельный рабочий
контекст конкретной задачи.

В результате:

- пользователь не видит номер issue в tab bar без дополнительной настройки;
- текущий default выглядит как внутреннее техническое имя, а не как полезный
  операторский UX;
- stable shared tab semantics для `pane` режима смешиваются с tab-launch path.

## Пользователь

- оператор, который запускает `ai-teamlead run <issue>`;
- владелец репозитория, который ожидает понятное default behavior без ручной
  доводки `settings.yml`;
- разработчик, который поддерживает launcher contracts и bootstrap templates.

## Результат

При `launch_target = tab` effective имя вкладки по умолчанию строится как
`#${ISSUE_NUMBER}`.

При этом:

- `zellij.tab_name = issue-analysis` остается shared tab contract для
  `pane`-режима;
- repo owner может явно переопределить tab title через
  `zellij.tab_name_template`;
- старое literal имя `issue-analysis` в `tab`-режиме остается доступным как
  explicit opt-out, а не как runtime default.

## Scope

В scope текущей feature входит:

- смена default naming semantics для `launch_target = tab`;
- фиксация нового contract layer в ADR, feature-docs и config docs;
- обновление bootstrap templates и summary-слоев документации;
- обновление runtime resolution в коде;
- unit и headless integration verification.

## Вне scope

В scope не входит:

- изменение default `launch_target`;
- изменение shared tab semantics для `pane`;
- live reuse/re-entry существующего tab/pane для одной и той же issue;
- расширение placeholder set за пределы `${ISSUE_NUMBER}`;
- поддержка issue title в имени tab.

## Ограничения и предпосылки

- issue-aware naming должно оставаться вычислимым до генерации
  `launch-layout.kdl`;
- runtime manifests и diagnostics должны хранить уже resolved `tab_name`;
- `pane` и `tab` не должны снова смешивать одну и ту же семантику имени;
- изменение нужно проводить через отдельный contract update, а не тихим
  patch-only изменением кода.
