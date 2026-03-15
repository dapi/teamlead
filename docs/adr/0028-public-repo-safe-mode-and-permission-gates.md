# ADR-0028: public repo safe mode and permission gates

Статус: proposed
Дата: 2026-03-14

## Контекст

После принятия hostile-input model недостаточно просто считать GitHub-контент
недоверенным. Нужен отдельный execution contract, который определяет:

- когда включается более строгий режим работы;
- какие действия относятся к high-risk;
- какие действия запрещаются автоматически;
- какие действия требуют явного human approval.

Без такого решения public-repo support останется неявным набором рекомендаций,
а не enforce-able policy.

## Решение

Вводится `public-safe mode` как обязательный operating mode для public
репозиториев и для случаев, когда visibility репозитория не удалось надежно
определить.

В `public-safe mode`:

- auto-intake по умолчанию ограничивается issue, созданными оператором или
  автором из явного allowlist;
- high-risk filesystem actions требуют explicit approval;
- network access к неразрешенным направлениям требует explicit approval;
- dangerous execution и sandbox escalation требуют explicit approval;
- publication actions, которые могут вывести sensitive local data наружу,
  требуют explicit approval;
- отсутствие информации о visibility трактуется fail-closed, а не fail-open.

Минимальный набор high-risk actions:

- чтение файлов вне целевого repo/worktree;
- запись вне целевого repo/worktree и разрешенного runtime-каталога;
- открытие внешних ссылок и загрузка удаленного контента;
- выполнение команд с изменением системного состояния;
- публикация в GitHub или во внешние сервисы данных, которые могут включать
  локальные секреты или сырые runtime artifacts.

Отдельное правило:

- `owner-authored issue` рассматривается только как более безопасный intake
  trigger;
- comments даже внутри такой issue остаются hostile-by-default;
- если системе нужен доверенный управляющий сигнал от оператора, он должен
  приходить через explicit mechanism, а не через произвольный GitHub comment.

## Последствия

Плюсы:

- public repo support получает явный security baseline;
- owner-only intake снижает риск автоматического запуска hostile issue;
- становится возможным детерминированно объяснять operator-у, почему действие
  заблокировано или требует approval;
- enforcement может быть реализован поэтапно, не ломая общий hostile-input
  контракт.

Минусы:

- interactive flow станет строже и местами медленнее;
- comments требуют отдельной trust policy и не решаются одним только owner-only
  intake;
- потребуется отдельная диагностика причин блокировки;
- часть текущих integration paths придется пересмотреть через allowlist model.

## Альтернативы

### 1. Не вводить отдельный режим, а просто усиливать осторожность в prompt

Отклонено.

Prompt discipline важна, но она не заменяет permission model и runtime gates.

### 2. Включать safe mode только по явному флагу пользователя

Отклонено.

Это делает опасный режим значением по умолчанию для public repos.

### 3. Всегда блокировать любые опасные действия без возможности approval

Отклонено.

Для части операторских сценариев нужен управляемый path с явным human
подтверждением, а не абсолютный запрет.

### 4. Считать весь thread trusted, если issue создана владельцем

Отклонено.

Комментарии в public repo могут писать разные участники, поэтому owner-authored
issue не решает риск prompt injection внутри discussion thread.

## Связанные документы

- [../untrusted-input-security.md](../untrusted-input-security.md)
- [../features/0006-public-repo-security/README.md](../features/0006-public-repo-security/README.md)
- [./0027-untrusted-github-content-as-hostile-input.md](./0027-untrusted-github-content-as-hostile-input.md)

## Журнал изменений

### 2026-03-14

- создан ADR о `public-safe mode` и permission gates для public repos
- добавлен owner-only intake как дополнительный gate без trust upgrade для
  comments
