# ADR-0027: zero-config `settings.yml` как documented template и canonical runtime defaults

Статус: accepted
Дата: 2026-03-14

## Контекст

`ai-teamlead init` bootstrap-ил `./.ai-teamlead/settings.yml` как полностью
активный YAML с рабочими значениями. Это смешивало две роли одного артефакта:

- versioned overview доступных настроек и bootstrap contract;
- фактический runtime source of truth для default-значений.

Из-за этого схема конфига дрейфовала между Rust-кодом, шаблоном
`templates/init/settings.yml` и документацией. Новый config key можно было
добавить без явного решения, является ли он required-полем, runtime default или
только примером расширения.

Дополнительно `init` не давал настоящий zero-config для полей, у которых
приложение уже знает canonical default.

## Решение

`./.ai-teamlead/settings.yml` остается repo-local versioned конфигом, но его
bootstrap-template перестает быть fully materialized runtime YAML.

Приняты следующие правила:

1. `settings.yml` из `templates/init/` создается как comment-only documented
   template.
2. Каждый config key обязан быть явно классифицирован как:
   - `required-without-default`
   - `defaulted-by-application`
   - `example-only extension`
3. Для `defaulted-by-application` полей canonical source of truth живет в одном
   Rust default-layer приложения.
4. Runtime loading строится как `defaults + active YAML overrides`.
5. `required-without-default` поля не получают скрытый fallback. Для текущего
   контракта таким полем остается `github.project_id`.
6. Закомментированный пример в template может отличаться от runtime default
   только для явно помеченных `example-only extension`.
7. В текущем MVP-контракте таким исключением является `zellij.layout`: template
   показывает opt-in пример `compact`, но отсутствие active override не меняет
   launcher behavior.
8. Эволюция схемы защищается guardrail-тестом, который проверяет:
   - что все config keys классифицированы;
   - что runtime defaults совпадают с documented defaults в template;
   - что required fields не теряют явную диагностику.

## Последствия

Плюсы:

- `init` создает настоящий zero-config template для defaulted-полей;
- source of truth для runtime defaults становится один;
- новые config keys нельзя тихо добавить без обновления contract layer;
- template остается человеко-ориентированным bootstrap документом, а не
  автогенерированным dump.

Минусы:

- loader конфига становится сложнее из-за merge `defaults + overrides`;
- требуется поддерживать явный metadata-layer для guardrail-тестов;
- старые документы, описывающие fully materialized bootstrap YAML, нужно
  синхронизировать.

## Связанные документы

- [README.md](../../README.md)
- [docs/adr/0001-repo-local-ai-config.md](./0001-repo-local-ai-config.md)
- [docs/adr/0012-repo-init-command-and-project-contract-layer.md](./0012-repo-init-command-and-project-contract-layer.md)
- [docs/features/0002-repo-init/README.md](../features/0002-repo-init/README.md)
- [specs/issues/33/README.md](../../specs/issues/33/README.md)
