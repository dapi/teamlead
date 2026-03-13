# Issue 12: `zellij` layout при создании сессии

Статус: draft, ready for plan review
Тип задачи: feature
Размер: medium
Последнее обновление: 2026-03-13

## Контекст

Issue: `zellij: опциональный layout при создании сессии`

- GitHub: https://github.com/dapi/ai-teamlead/issues/12
- Analysis branch: `analysis/issue-12`
- Session UUID: `e4c49c59-1bb8-4550-8e89-eb00515ea098`

Проблема состоит из двух связанных частей:

1. Сейчас новая `zellij` session создается через сгенерированный минимальный
   `launch-layout.kdl`, поэтому пользователь не может подключить свой именованный
   layout из `zellij`.
2. Когда `zellij.layout` не задан, launcher все равно стартует session в
   "bare" режиме и теряет привычный default UX `zellij`.

Цель анализа: зафиксировать минимальный дизайн, в котором новая session может
стартовать либо с пользовательским layout, либо с нормальным built-in default
UX, а analysis tab продолжает добавляться автоматически.

## Артефакты

## Что строим

- [01-what-we-build.md](/home/danil/worktrees/ai-teamlead/analysis/issue-12/specs/issues/12/01-what-we-build.md)

## Как строим

- [02-how-we-build.md](/home/danil/worktrees/ai-teamlead/analysis/issue-12/specs/issues/12/02-how-we-build.md)

## Как проверяем

- [03-how-we-verify.md](/home/danil/worktrees/ai-teamlead/analysis/issue-12/specs/issues/12/03-how-we-verify.md)

## Вывод анализа

Информации в issue достаточно, чтобы готовить план реализации без дополнительных
вопросов пользователю.

Новый ADR на текущем этапе не требуется, если в реализации останется контракт:
`zellij.layout` принимает только строковое имя layout, а fallback без поля
сохраняет обычный UX `zellij` без расширения формата конфига.
