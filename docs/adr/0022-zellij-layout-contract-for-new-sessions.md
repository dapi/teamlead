# ADR-0022: Контракт `zellij.layout` для создания новой session

Статус: accepted
Дата: 2026-03-14
Связанный issue: #12

## Контекст

Launcher `ai-teamlead` уже умеет:

- переиспользовать existing `zellij` session;
- добавлять analysis tab через generated `launch-layout.kdl`;
- хранить stable naming contract в `zellij.session_name` и `zellij.tab_name`.

Dogfooding показал два пробела в path создания новой session:

1. пользователь не может задать именованный `zellij` layout для новой session;
2. fallback без `zellij.layout` создает session через bare generated layout, а
   не как обычную базовую session `zellij`.

Изменение затрагивает:

- versioned config contract;
- `zellij` integration contract;
- verification contract для branch `session missing`.

## Решение

В project-local config вводится опциональное поле:

```yaml
zellij:
  session_name: "${REPO}"
  tab_name: "issue-analysis"
  layout: "my-custom-layout" # optional
```

Семантика:

- `layout = Some(name)`:
  launcher создает новую session через пользовательский layout;
- `layout = None`:
  launcher создает новую session без использования generated analysis layout как
  базовой session;
- analysis tab в обоих случаях добавляется отдельным действием через generated
  `launch-layout.kdl`.

## Ограничения

- в первую версию поддерживается только строковое имя layout;
- путь к `.kdl` файлу не поддерживается;
- contract касается только branch `session missing`;
- path `existing session` остается без изменения.

## Последствия

Плюсы:

- пользователь получает явный repo-level hook для собственного `zellij` layout;
- fallback-path перестает зависеть от технического analysis layout;
- generated `launch-layout.kdl` сохраняет одну ответственность:
  доставка analysis tab.

Минусы:

- launcher path становится ветвистее;
- между созданием session и добавлением analysis tab появляется дополнительный
  шаг, который нужно диагностировать отдельно;
- “обычный UX zellij” нельзя проверять только визуально, поэтому verification
  должна опираться на path команд и runtime artifacts.

## Почему не поддерживаем путь к `.kdl`

Это бы добавило:

- новый формат значения;
- отдельную валидацию путей;
- неоднозначность между layout name и filesystem path.

Для первой версии этого не требуется.

## Связанные документы

- [../features/0003-agent-launch-orchestration/README.md](../features/0003-agent-launch-orchestration/README.md)
- [../features/0003-agent-launch-orchestration/02-how-we-build.md](../features/0003-agent-launch-orchestration/02-how-we-build.md)
- [../features/0003-agent-launch-orchestration/03-how-we-verify.md](../features/0003-agent-launch-orchestration/03-how-we-verify.md)
- [../adr/0011-use-zellij-main-release-in-ci.md](../adr/0011-use-zellij-main-release-in-ci.md)
- [../../specs/issues/12/README.md](../../specs/issues/12/README.md)

## Журнал изменений

### 2026-03-14

- зафиксирован config contract для `zellij.layout`
- зафиксирован fallback-path для новой session без bare generated layout
- зафиксировано, что analysis tab добавляется отдельным generated layout
