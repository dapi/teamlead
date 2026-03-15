# issue-implementation-flow

Статус: project-local flow entrypoint

## Назначение

Этот файл является entrypoint prompt для implementation stage.

Он не должен содержать весь flow целиком. Вместо этого он маршрутизирует
реализацию по staged prompts в каталоге:

- `./.ai-teamlead/flows/issue-implementation/`

## Порядок работы

1. Сначала восстанови approved analysis context из `specs/issues/${ISSUE_NUMBER}/`.
2. Восстанови план изменений документации и зафиксируй, какие канонические и
   summary-документы нужно обновить вместе с кодом.
3. Затем выполни реализацию по шагам из staged prompts.
4. После code changes обязательно выполни релевантные проверки.
5. Заверши stage через `complete-stage` с `--stage implementation`.

## Где искать context

- `./.ai-teamlead/settings.yml`
- `./AURA.md`
- `./docs/issue-implementation-flow.md`
- approved analysis artifacts в `specs/issues/${ISSUE_NUMBER}/`
- staged prompts в `./.ai-teamlead/flows/issue-implementation/`

## Завершение стадии

Для implementation stage используй:

```bash
$AI_TEAMLEAD_BIN internal complete-stage "$AI_TEAMLEAD_SESSION_UUID" \
  --stage implementation \
  --outcome ready-for-ci \
  --message "краткое описание результата"
```

Если stage заблокирован:

```bash
$AI_TEAMLEAD_BIN internal complete-stage "$AI_TEAMLEAD_SESSION_UUID" \
  --stage implementation \
  --outcome blocked \
  --message "краткая причина блокировки"
```
