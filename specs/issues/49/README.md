# Issue 49: `zellij` issue-aware tab naming for `tab` launch target

Статус: approved
Тип задачи: `feature`
Тип проекта: `infra/platform`
Размер: `medium`
Последнее обновление: 2026-03-14
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-14T23:11:41+03:00

## Issue

- GitHub issue: https://github.com/dapi/ai-teamlead/issues/49
- Тип: `feature`
- Размер: `medium`
- Тип проекта: `infra/platform`

## Summary

Issue добавляет отдельный naming contract для ветки `zellij.launch_target = tab`.

Целевое состояние:

- `zellij.tab_name` остается stable semantic именем shared tab context;
- новый optional `zellij.tab_name_template` задает issue-aware имя вкладки
  только для режима `tab`;
- в режиме `tab` runtime рендерит effective имя вкладки из issue context;
- при отсутствии template режим `tab` fallback-ится на `zellij.tab_name`;
- невалидные placeholders дают явную ошибку конфигурации до запуска `zellij`.

## Status

Пакет анализа утвержден и готов быть каноническим входом для implementation
stage после перевода issue в `Ready for Implementation`.

## Artifacts

- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

## Open Questions

Блокирующих вопросов по текущему issue не выявлено.

Для реализации нужно синхронизироваться с issue `#47`: `#49` расширяет уже
введенный или одновременно вводимый контракт `zellij.launch_target`, а не
подменяет его.
