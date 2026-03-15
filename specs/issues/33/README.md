# Issue 33: zero-config шаблон `settings.yml` с canonical default-layer и guardrail

Статус: approved
Тип задачи: feature
Размер: medium
Последнее обновление: 2026-03-14
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-14T23:06:53+03:00

## Контекст

Issue: `settings.yml: zero-config шаблон с закомментированными default-значениями и guardrail`

- GitHub: https://github.com/dapi/ai-teamlead/issues/33
- Analysis branch: `analysis/issue-33`
- Session UUID: `0679df4d-d45e-4f52-adfa-0015808c606a`

Сейчас `ai-teamlead init` копирует в репозиторий полностью активный
`./.ai-teamlead/settings.yml`. Из-за этого один и тот же файл одновременно
играет две роли:

- runtime-конфига, который приложение обязано прочитать как активный YAML;
- обзорного шаблона, который должен документировать доступные настройки и
  default-значения.

Такое совмещение ролей создает скрытый drift между кодом, шаблоном и
документацией. Issue переводит `settings.yml` в zero-config модель: defaulted
поля могут оставаться только в комментариях, runtime берет значения из
canonical default-layer приложения, а эволюция схемы защищается явным
guardrail.

## Approval

Пакет анализа считается approved после явного подтверждения плана в агентской
сессии и перевода issue в `Ready for Implementation`.

Для issue `#33` approval фиксируется следующими metadata:

- `Статус согласования: approved`
- `Approved By: dapi`
- `Approved At: 2026-03-14T23:06:53+03:00`

## Артефакты

## Что строим

- [01-what-we-build.md](./01-what-we-build.md)

## Как строим

- [02-how-we-build.md](./02-how-we-build.md)

## Как проверяем

- [03-how-we-verify.md](./03-how-we-verify.md)

## План имплементации

- [04-implementation-plan.md](./04-implementation-plan.md)

## Связанный контекст

- [../../../README.md](../../../README.md)
- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../docs/implementation-plan.md](../../../docs/implementation-plan.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [../../../docs/features/0002-repo-init/README.md](../../../docs/features/0002-repo-init/README.md)
- [../../../docs/adr/0001-repo-local-ai-config.md](../../../docs/adr/0001-repo-local-ai-config.md)
- [../../../docs/adr/0012-repo-init-command-and-project-contract-layer.md](../../../docs/adr/0012-repo-init-command-and-project-contract-layer.md)

## Вывод анализа

Информации в issue достаточно, чтобы готовить план реализации без
дополнительных вопросов пользователю.

План может идти в `Waiting for Plan Review`, если реализация зафиксирует
следующий контракт:

- у каждого config key есть явная категория:
  `required-without-default`, `defaulted-by-application` или
  `example-only extension`;
- canonical runtime defaults живут в одном Rust-layer и используются при
  загрузке конфига, а не дублируются в активном YAML;
- `templates/init/settings.yml` остается versioned обзором доступных настроек,
  но defaulted-поля показывает в закомментированном виде;
- для required-полей, прежде всего `github.project_id`, отдельно фиксируется
  диагностика и bootstrap-path;
- схема, default-layer и шаблон связываются guardrail-тестами, чтобы новый key
  нельзя было добавить молча.

## Открытые вопросы

Блокирующих вопросов по текущему issue не выявлено.
