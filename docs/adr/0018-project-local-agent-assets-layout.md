# ADR-0018: структура project-local агентских ресурсов

Статус: accepted
Дата: 2026-03-13

## Контекст

`ai-teamlead` должен уметь опираться не только на flow-документы, но и на
project-local agent assets: роли, подагенты, routing, skills и другие
дополнительные материалы.

Для Claude в экосистеме естественно использовать `./.claude/`. Для Codex
проекту тоже нужен repo-level convention, но нельзя смешивать его с главным
instruction contract в `AGENTS.md`.

## Решение

В репозитории принимается следующий layout:

- `./.claude/` — project-local assets для Claude-specific workflow
- `./.codex/` — project-local convention для Codex-specific assets
- `AGENTS.md` — основной repo-level instruction contract для Codex

Назначение:

- `.claude/` может содержать agents, commands, skills и другие материалы,
  которые использует Claude workflow
- `.codex/` может содержать prompts, routing, agents, skills и другие
  project-local материалы для Codex workflow
- `AGENTS.md` не заменяется `.codex/`, а остается основным instruction entrypoint

## Последствия

Плюсы:

- project-local agent assets становятся versioned частью репозитория
- Claude и Codex получают явные каталоги для своих project-specific материалов
- основной `issue-analysis-flow` может ссылаться на эти каталоги по мере
  необходимости, а не тащить все в один prompt

Минусы:

- появляется еще один слой repo conventions
- нужно следить, чтобы `AGENTS.md` и `.codex/` не дублировали друг друга хаотично

## Альтернативы

### 1. Хранить все только в `.ai-teamlead/`

Отклонено.

`.ai-teamlead/` должен оставаться местом для flow, launcher и repo-local
контрактов самого `ai-teamlead`, а не общим складом всех agent assets.

### 2. Не вводить `.codex/` вообще

Отклонено.

Для проекта полезно иметь repo-level convention для Codex-specific материалов,
даже если это не внешний стандарт продукта.

## Связанные документы

- [AGENTS.md](../../AGENTS.md)
- [README.md](../../README.md)
- [docs/adr/0017-minimal-sdd-artifact-set-for-issue-analysis.md](./0017-minimal-sdd-artifact-set-for-issue-analysis.md)

## Журнал изменений

### 2026-03-13

- зафиксирован layout `.claude/`, `.codex/` и роль `AGENTS.md`
