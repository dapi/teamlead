# Issue 23: унифицировать шаблоны config для `zellij.session_name`

## Issue

- GitHub issue: https://github.com/dapi/ai-teamlead/issues/23
- Тип: `feature`
- Размер: `small`
- Тип проекта: `infra/platform`

## Summary

Issue устраняет разрыв между двумя моделями конфигурации:

- `zellij.session_name` сейчас bootstrap-ится через специальный токен
  `__SESSION_NAME__`
- `launch_agent.*` уже использует versioned template contract с `${...}`

Целевое состояние: `zellij.session_name` становится template-capable полем с
поддержкой `${REPO}`, bootstrap placeholder исчезает, canonical repo identifier
для `zellij` и `launch_agent` выравнивается, а literal значения остаются
обратно совместимыми.

## Status

Черновик анализа готов к human review и переводу issue в `Waiting for Plan Review`.

## Artifacts

- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

## Open Questions

Блокирующих вопросов по текущему issue не выявлено.
