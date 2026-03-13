# issue-analysis-flow

Статус: project-local flow entrypoint

## Назначение

Этот файл является entrypoint prompt для анализа issue.

Он не должен содержать весь flow целиком. Вместо этого он маршрутизирует
анализ по staged prompts в каталоге:

- `./.ai-teamlead/flows/issue-analysis/`

## Порядок работы

Ты должен выполнять анализ последовательно по трем осям:

1. `./.ai-teamlead/flows/issue-analysis/01-what-we-build.md`
2. `./.ai-teamlead/flows/issue-analysis/02-how-we-build.md`
3. `./.ai-teamlead/flows/issue-analysis/03-how-we-verify.md`

Не перепрыгивай к следующей оси, пока предыдущая не собрана достаточно хорошо.

## Общие инварианты

- результат должен быть versioned SDD-комплектом в каталоге issue
- минимальный комплект документов:
  - `README.md`
  - `01-what-we-build.md`
  - `02-how-we-build.md`
  - `03-how-we-verify.md`
- минимум один документ на каждую из трех осей обязателен
- если issue маленькая, не создавай лишние документы сверх этого минимума
- выбор секций внутри документов должен быть rule-based:
  - сначала `core`
  - затем релевантные `conditional`
  - затем `scaling` для `medium` и `large`
- при выборе секций учитывай:
  - тип задачи: `feature`, `bug`, `chore`
  - тип проекта: `product/UI`, `library/API`, `infra/platform`
  - размер задачи: `small`, `medium`, `large`
- вопросы пользователю задавай в агентской сессии
- если критичной информации не хватает, остановись и запроси уточнение

## Где искать project-local context

- `./.ai-teamlead/settings.yml`
- `./.ai-teamlead/README.md`
- staged prompts в `./.ai-teamlead/flows/issue-analysis/`
- project-local agent assets, если они есть:
  - `./.claude/`
  - `./.codex/`

## Связанные системные документы

- системный SSOT `docs/issue-analysis-flow.md` из репозитория `ai-teamlead`
