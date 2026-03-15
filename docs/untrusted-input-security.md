# untrusted-input-security

Статус: draft, evolving
Владелец: владелец репозитория
Роль: SSOT для правил обращения с недоверенным вводом и safe operating mode
Последнее обновление: 2026-03-14

## Назначение

Этот документ определяет единый источник истины для того, как `ai-teamlead`
должен обращаться с недоверенным вводом при работе с GitHub, содержимым
репозитория, runtime-артефактами и внешним контентом.

Цель документа:

- зафиксировать hostile-input model;
- определить trust boundaries;
- описать safe operating mode для public repos;
- определить, какие действия требуют human approval или запрещены.

Этот документ применим и к analysis flow, и к implementation flow.

## Scope

В scope входят:

- GitHub issue, comments, labels, titles, linked issues и linked PR;
- файлы репозитория, включая docs, templates, generated artifacts и project-local
  agent assets;
- shell output, test output, CI output и прочие runtime артефакты;
- внешние ссылки и загружаемый по ним контент;
- policy для filesystem, network, execution и publication actions.

## Вне scope

- защита от скомпрометированной ОС или скомпрометированного user account;
- формальная security-модель сторонних LLM providers;
- защита от ручных действий оператора, который сознательно обходит политику;
- hardening произвольных внешних tools вне control plane `ai-teamlead`.

## Термины

- `trusted control plane`:
  CLI-контракт, локальные системные инструкции, явно принятые ADR, явное
  подтверждение оператора.
- `untrusted input`:
  любой контент, пришедший из issue, comments, linked sources, repo files,
  shell output или generated artifacts, если он не является частью trusted
  control plane.
- `public-safe mode`:
  режим исполнения, в котором hostile-by-default inputs не могут инициировать
  high-risk действия без явного human gate.
- `high-risk action`:
  действие, которое может читать чувствительные локальные данные, менять
  систему, публиковать данные наружу или расширять permission scope.

## Trust boundaries

Система должна различать как минимум следующие границы:

1. GitHub content boundary
   Сюда входят issue, comments, labels, linked issues, PR metadata и все
   текстовые поля, доступные злоумышленнику через GitHub.
2. Repo content boundary
   Сюда входят markdown-документы, `AGENTS.md`, `.ai-teamlead/` assets,
   templates и любые versioned файлы целевого репозитория.
3. Runtime output boundary
   Сюда входят shell output, test logs, generated files и другие артефакты,
   возникшие после обработки недоверенной задачи.
4. Sensitive local boundary
   Сюда входят файлы вне целевого репозитория, токены, SSH-ключи, глобальные
   конфиги и другая информация машины пользователя.
5. External network boundary
   Сюда входят любые веб-сайты, API, file downloads и remote hosts вне явно
   разрешенного набора интеграций.

## Базовая классификация входов

По умолчанию:

- public GitHub content = `untrusted`;
- linked external content = `untrusted`;
- shell output и generated artifacts = `untrusted`, если они возникли как
  следствие обработки недоверенного ввода;
- repo-local docs и prompts = не являются автоматическим trust anchor;
- операторский ответ в агентской сессии = `trusted control plane`, если он
  выражает явное намерение и не подменяется GitHub-контентом.

Следствие:

- issue или comment не могут сами назначить себе привилегии;
- текст "игнорируй предыдущие инструкции" или аналогичные конструкции
  рассматриваются как hostile content, а не как команды;
- repo-local assets из public repo не могут расширять filesystem или network
  scope без отдельного trusted mechanism.

## Safe operating mode для public repos

Если репозиторий `public` или его visibility не удалось надежно определить,
должен включаться `public-safe mode`.

В этом режиме:

- недоверенный контент разрешено читать и анализировать;
- auto-intake по умолчанию должен ограничиваться issue, созданными самим
  оператором или заранее разрешенным автором;
- запрещено автоматически читать чувствительные локальные файлы вне целевого
  repo/worktree;
- запрещено автоматически публиковать наружу данные из sensitive local boundary;
- запрещено автоматически открывать внешние ссылки и скачивать контент;
- запрещено автоматически выполнять команды вне sandbox или с расширенным
  permission scope;
- любые high-risk actions требуют явного human approval.

Важно:

- `owner-authored issue` это только intake mitigation;
- он не превращает все обсуждение issue в trusted control plane;
- comments в public repo остаются `untrusted input`, если нет отдельного
  механизма доверенного операторского сигнала.

## Policy для issue author и comments

Для public repos допускается более строгая intake policy:

- автоматически брать в работу только issue, созданные оператором;
- или использовать явный allowlist авторов issue.

Но при этом:

- `issue author` и `comment author` рассматриваются независимо;
- owner-authored issue не делает comments trusted;
- comment, даже от доверенного автора, не может сам по себе повышать
  filesystem, network или execution privileges;
- если нужен управляющий сигнал от оператора, он должен передаваться через
  agent session или другой explicit mechanism, а не через обычный GitHub
  comment.

## Human gates

Явное подтверждение оператора обязательно минимум для следующих действий:

- чтение файлов вне целевого репозитория;
- запись вне целевого репозитория или его разрешенного runtime-каталога;
- сетевой доступ к хостам, не входящим в явный allowlist;
- публикация текста в GitHub, если в него может попасть локальная секретная
  информация;
- выполнение команд, которые меняют системное состояние или требуют escalation.

Human approval должен быть:

- привязан к конкретному действию;
- понятен оператору по последствиям;
- fail-closed при отсутствии ответа или при ошибке определения контекста.

## Запрещенные автоматические действия

В `public-safe mode` запрещены без специального trusted override:

- чтение `~/.ssh`, `~/.aws`, `~/.config`, `.env` вне целевого репозитория и
  аналогичных директорий;
- отправка локальных секретов или сырых конфигов в issue, PR, comments или
  сторонние сервисы;
- исполнение команд, предложенных напрямую недоверенным контентом, только на
  основании того, что они "нужны для диагностики";
- интерпретация shell output или test logs как нового control plane.

## Связь с flow

Для [issue-analysis-flow.md](./issue-analysis-flow.md) это означает:

- вопросы пользователю должны задаваться в агентской сессии, а не в GitHub
  comments;
- intake policy может ограничивать auto-start только owner-authored issue, но
  не меняет трактовку comments;
- analysis artifacts не должны включать локальные секретные данные;
- human gate на уточнениях и плане не заменяет security gates.

Для [issue-implementation-flow.md](./issue-implementation-flow.md) это означает:

- approved analysis artifacts не делают вход trusted автоматически;
- кодинг и тесты не отменяют запрет на unsafe filesystem/network actions;
- GitHub comments не должны использоваться как implicit operator override;
- PR/issue publication path должен учитывать риск data exfiltration.

## Направление runtime enforcement

Реализация должна двигаться в таком порядке:

1. определить `repo_visibility` и `operating_mode`;
2. применить intake policy к автору issue;
3. маркировать действия по типу риска;
4. перед high-risk action требовать явный approval или останавливать выполнение;
5. логировать причину блокировки или запроса на approval без утечки секретов.

## Связанные документы

- [README.md](../README.md)
- [issue-analysis-flow.md](./issue-analysis-flow.md)
- [issue-implementation-flow.md](./issue-implementation-flow.md)
- [features/0006-public-repo-security/README.md](./features/0006-public-repo-security/README.md)
- [adr/0027-untrusted-github-content-as-hostile-input.md](./adr/0027-untrusted-github-content-as-hostile-input.md)
- [adr/0028-public-repo-safe-mode-and-permission-gates.md](./adr/0028-public-repo-safe-mode-and-permission-gates.md)

## Журнал изменений

### 2026-03-14

- создан SSOT для hostile-input model и `public-safe mode`
- добавлена policy для `owner-authored issue` intake и hostile-by-default
  comments
