# complete-stage Implementation Plan

Статус: выполнен

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Реализовать CLI-команду `ai-teamlead internal complete-stage`, которая позволяет agent session сигналить core о результате анализа (plan-ready / needs-clarification / blocked) и автоматически выполняет git commit, push, draft PR и смену статуса issue в GitHub Project.

**Architecture:** Новый вариант `CompleteStage` в enum `InternalCommand` (cli.rs). Логика выделена в отдельный модуль `src/complete_stage.rs`. Команда принимает `session_uuid`, `--outcome` и `--message`, читает session manifest из primary repo (через env var `AI_TEAMLEAD_REPO_ROOT`), выполняет git/gh операции через существующий `Shell` trait, обновляет session.json и issue index. Flow prompt получает секцию завершения.

**Tech Stack:** Rust (clap, anyhow, serde_json, chrono), gh CLI, git CLI, существующие модули `shell.rs`, `runtime.rs`, `github.rs`, `config.rs`.

---

### Task 1: Добавить `CompleteStage` в CLI enum

**Files:**
- Modify: `src/cli.rs:26-31`

**Step 1: Добавить вариант в `InternalCommand`**

В `src/cli.rs` добавить новый вариант в enum `InternalCommand`:

```rust
#[derive(Debug, Subcommand)]
pub enum InternalCommand {
    BindZellijPane { session_uuid: String },
    LaunchZellijFixture { issue: u64 },
    RenderLaunchAgentContext { issue: String },
    CompleteStage {
        session_uuid: String,
        #[arg(long)]
        outcome: String,
        #[arg(long)]
        message: String,
    },
}
```

**Step 2: Проверить компиляцию**

Run: `cargo check 2>&1 | head -30`
Expected: warning о неиспользованном `CompleteStage` в match (в `app.rs`), но без ошибок компиляции.

**Step 3: Commit**

```bash
git add src/cli.rs
git commit -m "feat(cli): add CompleteStage variant to InternalCommand enum"
```

---

### Task 2: Создать модуль `complete_stage.rs` со скелетом

**Files:**
- Create: `src/complete_stage.rs`
- Modify: `src/lib.rs:1-11`
- Modify: `src/app.rs:257-268` (run_internal match)

**Step 1: Создать скелет модуля**

Создать `src/complete_stage.rs`:

```rust
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::config::Config;
use crate::github::GhProjectClient;
use crate::runtime::RuntimeLayout;
use crate::shell::Shell;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StageOutcome {
    PlanReady,
    NeedsClarification,
    Blocked,
}

impl StageOutcome {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "plan-ready" => Ok(Self::PlanReady),
            "needs-clarification" => Ok(Self::NeedsClarification),
            "blocked" => Ok(Self::Blocked),
            other => bail!("invalid outcome: {other}. Expected: plan-ready, needs-clarification, blocked"),
        }
    }

    pub fn target_status<'a>(&self, statuses: &'a crate::config::FlowStatuses) -> &'a str {
        match self {
            Self::PlanReady => &statuses.waiting_for_plan_review,
            Self::NeedsClarification => &statuses.waiting_for_clarification,
            Self::Blocked => &statuses.analysis_blocked,
        }
    }
}

pub fn run_complete_stage(
    shell: &dyn Shell,
    session_uuid: &str,
    outcome: &str,
    message: &str,
) -> Result<()> {
    let outcome = StageOutcome::parse(outcome)?;
    let repo_root = resolve_repo_root(shell)?;
    let config = Config::load_from_repo_root(&repo_root)?;
    let runtime = RuntimeLayout::from_repo_root(&repo_root);

    let manifest = runtime
        .load_session_manifest(session_uuid)?
        .ok_or_else(|| anyhow::anyhow!("session not found: {session_uuid}"))?;

    if manifest.status == "completed" {
        eprintln!("warning: session {session_uuid} is already completed, skipping");
        return Ok(());
    }

    let issue_number = manifest.issue_number;
    let worktree_root = resolve_worktree_root()?;
    let artifacts_dir = std::env::var("AI_TEAMLEAD_ANALYSIS_ARTIFACTS_DIR")
        .unwrap_or_else(|_| format!("specs/issues/{issue_number}"));
    let branch = std::env::var("AI_TEAMLEAD_ANALYSIS_BRANCH")
        .unwrap_or_else(|_| format!("analysis/issue-{issue_number}"));

    let commit_message = format!("analysis(#{issue_number}): {message}");

    // Step 1: git add + commit (if there are changes)
    let committed = git_add_and_commit(shell, &worktree_root, &artifacts_dir, &commit_message)?;

    // Step 2: git push
    if committed {
        git_push(shell, &worktree_root, &branch)?;
    }

    // Step 3: create draft PR (only for plan-ready)
    if matches!(outcome, StageOutcome::PlanReady) && committed {
        let pr_title = format!("analysis(#{issue_number}): {message}");
        let pr_body = format!("Ref #{issue_number}\n\nOutcome: plan-ready\nArtifacts: `{artifacts_dir}/`");
        create_draft_pr_if_needed(shell, &worktree_root, &branch, &pr_title, &pr_body)?;
    }

    // Step 4: update GitHub Project status
    let target_status = outcome.target_status(&config.issue_analysis_flow.statuses);
    update_project_status(shell, &repo_root, &config, &manifest, target_status)?;

    // Step 5: update runtime state
    runtime.update_session_status(session_uuid, "completed")?;
    runtime.update_issue_flow_status(issue_number, target_status)?;

    println!(
        "complete-stage: issue=#{issue_number} outcome={outcome_str} status={target_status}",
        outcome_str = outcome_to_str(&outcome),
    );

    Ok(())
}

fn resolve_repo_root(shell: &dyn Shell) -> Result<PathBuf> {
    if let Ok(root) = std::env::var("AI_TEAMLEAD_REPO_ROOT") {
        return Ok(PathBuf::from(root));
    }
    // Fallback: primary worktree from git
    let cwd = std::env::current_dir().context("failed to get cwd")?;
    let output = shell.run(&cwd, "git", &["worktree", "list", "--porcelain"])?;
    let first_line = output
        .lines()
        .find(|l| l.starts_with("worktree "))
        .ok_or_else(|| anyhow::anyhow!("cannot determine primary worktree"))?;
    Ok(PathBuf::from(first_line.strip_prefix("worktree ").unwrap()))
}

fn resolve_worktree_root() -> Result<PathBuf> {
    if let Ok(root) = std::env::var("AI_TEAMLEAD_WORKTREE_ROOT") {
        return Ok(PathBuf::from(root));
    }
    std::env::current_dir().context("failed to get cwd")
}

fn git_add_and_commit(
    shell: &dyn Shell,
    worktree: &Path,
    artifacts_dir: &str,
    commit_message: &str,
) -> Result<bool> {
    // Check if artifacts dir exists and has changes
    let artifacts_path = worktree.join(artifacts_dir);
    if !artifacts_path.exists() {
        eprintln!("complete-stage: no artifacts directory at {artifacts_dir}, skipping commit");
        return Ok(false);
    }

    // git add
    shell.run(worktree, "git", &["add", artifacts_dir])?;

    // Check if there are staged changes
    let diff_result = shell.run(worktree, "git", &["diff", "--cached", "--quiet"]);
    if diff_result.is_ok() {
        eprintln!("complete-stage: no staged changes, skipping commit");
        return Ok(false);
    }

    // git commit
    shell.run(worktree, "git", &["commit", "-m", commit_message])?;
    Ok(true)
}

fn git_push(shell: &dyn Shell, worktree: &Path, branch: &str) -> Result<()> {
    shell
        .run(worktree, "git", &["push", "origin", branch])
        .context("failed to push analysis branch")?;
    Ok(())
}

fn create_draft_pr_if_needed(
    shell: &dyn Shell,
    worktree: &Path,
    branch: &str,
    title: &str,
    body: &str,
) -> Result<()> {
    // Check if PR already exists for this branch
    let existing = shell.run(
        worktree,
        "gh",
        &["pr", "list", "--head", branch, "--json", "number", "--jq", "length"],
    );
    if let Ok(count) = existing {
        if count.trim() != "0" {
            eprintln!("complete-stage: draft PR already exists for branch {branch}");
            return Ok(());
        }
    }

    let result = shell.run(
        worktree,
        "gh",
        &["pr", "create", "--draft", "--title", title, "--body", body],
    );
    match result {
        Ok(url) => println!("complete-stage: created draft PR: {url}"),
        Err(e) => eprintln!("complete-stage: warning: failed to create draft PR: {e}"),
    }
    Ok(())
}

fn update_project_status(
    shell: &dyn Shell,
    repo_root: &Path,
    config: &Config,
    manifest: &crate::runtime::SessionManifest,
    target_status: &str,
) -> Result<()> {
    let github = GhProjectClient::new(shell);
    let snapshot = github.load_project_snapshot(repo_root, &config.github.project_id)?;

    let issue_item = snapshot
        .items
        .iter()
        .find(|item| {
            item.issue_number == manifest.issue_number
                && item.matches_repo(&manifest.github_owner, &manifest.github_repo)
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "issue #{} not found in project",
                manifest.issue_number
            )
        })?;

    let option_id = snapshot.option_id_by_name(target_status)?;
    github.update_status(
        repo_root,
        &config.github.project_id,
        &issue_item.item_id,
        &snapshot.status_field_id,
        option_id,
    )?;
    Ok(())
}

fn outcome_to_str(outcome: &StageOutcome) -> &'static str {
    match outcome {
        StageOutcome::PlanReady => "plan-ready",
        StageOutcome::NeedsClarification => "needs-clarification",
        StageOutcome::Blocked => "blocked",
    }
}
```

**Step 2: Добавить модуль в `lib.rs`**

В `src/lib.rs` добавить строку:

```rust
pub mod complete_stage;
```

**Step 3: Подключить в `app.rs`**

В `src/app.rs` добавить import `use crate::complete_stage::run_complete_stage;` и ветку match:

```rust
fn run_internal(shell: &dyn Shell, internal: InternalCommand) -> Result<()> {
    match internal {
        InternalCommand::BindZellijPane { session_uuid } => {
            run_internal_bind_zellij_pane(shell, &session_uuid)
        }
        InternalCommand::LaunchZellijFixture { issue } => {
            run_internal_launch_zellij_fixture(shell, issue)
        }
        InternalCommand::RenderLaunchAgentContext { issue } => {
            run_internal_render_launch_agent_context(shell, &issue)
        }
        InternalCommand::CompleteStage {
            session_uuid,
            outcome,
            message,
        } => run_complete_stage(shell, &session_uuid, &outcome, &message),
    }
}
```

**Step 4: Проверить компиляцию**

Run: `cargo check`
Expected: компиляция успешна (возможно, warnings о unused).

**Step 5: Commit**

```bash
git add src/complete_stage.rs src/lib.rs src/app.rs
git commit -m "feat: add complete_stage module with full finalization logic"
```

---

### Task 3: Добавить `update_session_status` в `runtime.rs`

**Files:**
- Modify: `src/runtime.rs:114-124`
- Test: `src/runtime.rs` (inline tests)

**Step 1: Написать failing тест**

В `src/runtime.rs` в mod `tests` добавить:

```rust
#[test]
fn updates_session_status_to_completed() {
    let temp = tempdir().expect("temp dir");
    let repo_root = temp.path().join("repo");
    let git_dir = repo_root.join(".git");
    std::fs::create_dir_all(&git_dir).expect("git dir");

    let layout = RuntimeLayout::from_repo_root(&repo_root);
    layout.ensure_exists().expect("runtime layout");

    let repo = RepoContext {
        repo_root: repo_root.clone(),
        git_dir,
        github_owner: "dapi".into(),
        github_repo: "teamlead".into(),
    };
    let zellij = ZellijConfig {
        session_name: "ai-teamlead".into(),
        tab_name: "issue-analysis".into(),
    };

    let manifest = layout
        .create_claim_binding(&repo, "PVT_project", &zellij, 42)
        .expect("claim binding");

    assert_eq!(manifest.status, "active");

    let updated = layout
        .update_session_status(&manifest.session_uuid, "completed")
        .expect("status updated");

    assert_eq!(updated.status, "completed");

    // Verify persistence
    let reloaded = layout
        .load_session_manifest(&manifest.session_uuid)
        .expect("reload")
        .expect("manifest exists");
    assert_eq!(reloaded.status, "completed");
}
```

**Step 2: Проверить что тест не компилируется**

Run: `cargo test --lib runtime::tests::updates_session_status_to_completed 2>&1 | tail -5`
Expected: ошибка компиляции — метод `update_session_status` не существует.

**Step 3: Реализовать `update_session_status`**

В `src/runtime.rs` impl `RuntimeLayout` добавить после `update_issue_flow_status`:

```rust
pub fn update_session_status(
    &self,
    session_uuid: &str,
    status: &str,
) -> Result<SessionManifest> {
    let mut manifest = self
        .load_session_manifest(session_uuid)?
        .ok_or_else(|| anyhow!("missing session manifest for session_uuid={session_uuid}"))?;
    manifest.status = status.to_string();
    manifest.updated_at = Utc::now().to_rfc3339();

    let session_path = self.sessions_dir.join(session_uuid).join("session.json");
    write_json_pretty(session_path, &manifest)?;
    Ok(manifest)
}
```

**Step 4: Запустить тест**

Run: `cargo test --lib runtime::tests::updates_session_status_to_completed`
Expected: PASS

**Step 5: Commit**

```bash
git add src/runtime.rs
git commit -m "feat(runtime): add update_session_status method"
```

---

### Task 4: Тесты для `StageOutcome::parse` и `target_status`

**Files:**
- Modify: `src/complete_stage.rs` (добавить тесты в конец)

**Step 1: Добавить unit тесты**

В конец `src/complete_stage.rs` добавить:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FlowStatuses;

    fn sample_statuses() -> FlowStatuses {
        FlowStatuses {
            backlog: "Backlog".into(),
            analysis_in_progress: "Analysis In Progress".into(),
            waiting_for_clarification: "Waiting for Clarification".into(),
            waiting_for_plan_review: "Waiting for Plan Review".into(),
            ready_for_implementation: "Ready for Implementation".into(),
            analysis_blocked: "Analysis Blocked".into(),
        }
    }

    #[test]
    fn parses_valid_outcomes() {
        assert!(matches!(StageOutcome::parse("plan-ready").unwrap(), StageOutcome::PlanReady));
        assert!(matches!(StageOutcome::parse("needs-clarification").unwrap(), StageOutcome::NeedsClarification));
        assert!(matches!(StageOutcome::parse("blocked").unwrap(), StageOutcome::Blocked));
    }

    #[test]
    fn rejects_invalid_outcome() {
        let err = StageOutcome::parse("unknown").unwrap_err();
        assert!(err.to_string().contains("invalid outcome"));
    }

    #[test]
    fn maps_outcome_to_correct_status() {
        let statuses = sample_statuses();
        assert_eq!(
            StageOutcome::PlanReady.target_status(&statuses),
            "Waiting for Plan Review"
        );
        assert_eq!(
            StageOutcome::NeedsClarification.target_status(&statuses),
            "Waiting for Clarification"
        );
        assert_eq!(
            StageOutcome::Blocked.target_status(&statuses),
            "Analysis Blocked"
        );
    }
}
```

**Step 2: Запустить тесты**

Run: `cargo test --lib complete_stage::tests`
Expected: 3 теста PASS

**Step 3: Commit**

```bash
git add src/complete_stage.rs
git commit -m "test(complete_stage): add unit tests for StageOutcome parsing and mapping"
```

---

### Task 5: Обновить flow prompt

**Files:**
- Modify: `.ai-teamlead/flows/issue-analysis-flow.md`

**Step 1: Добавить секцию завершения анализа**

В конец `.ai-teamlead/flows/issue-analysis-flow.md` добавить:

```markdown

## Завершение анализа

После завершения работы вызови ОДНУ из команд:

Если SDD-комплект собран и полон:
```
$AI_TEAMLEAD_BIN internal complete-stage $AI_TEAMLEAD_SESSION_UUID \
  --outcome plan-ready \
  --message "краткое описание результата"
```

Если нужны ответы пользователя:
```
$AI_TEAMLEAD_BIN internal complete-stage $AI_TEAMLEAD_SESSION_UUID \
  --outcome needs-clarification \
  --message "краткое описание вопросов"
```

Если заблокирован:
```
$AI_TEAMLEAD_BIN internal complete-stage $AI_TEAMLEAD_SESSION_UUID \
  --outcome blocked \
  --message "причина блокировки"
```

Команда сама выполнит коммит, пуш и создание draft PR.
НЕ выполняй git commit, git push, gh pr create самостоятельно.

Нотация commit message: `analysis(#N): <описание>`
Нотация PR title: `analysis(#N): <описание>`
В PR body укажи `Ref #N` и список артефактов.
```

**Step 2: Commit**

```bash
git add .ai-teamlead/flows/issue-analysis-flow.md
git commit -m "docs(flow): add completion section to issue-analysis-flow prompt"
```

---

### Task 6: Обновить SSOT `docs/issue-analysis-flow.md`

**Files:**
- Modify: `docs/issue-analysis-flow.md`

**Step 1: Обновить секцию «Открытые вопросы»**

Удалить пункт «нужно ли позже возвращать machine-readable артефакты поверх истории агентской сессии» (решено в ADR-0020).

**Step 2: Добавить секцию «Контракт завершения стадии»**

Перед секцией «Связанные документы» добавить:

```markdown
## Контракт завершения стадии

Agent session сигналит core о результате анализа через CLI-команду:

```
ai-teamlead internal complete-stage <session_uuid> --outcome <outcome> --message <msg>
```

Допустимые значения `outcome`:

- `plan-ready` — SDD-комплект собран, issue → `Waiting for Plan Review`
- `needs-clarification` — нужны ответы, issue → `Waiting for Clarification`
- `blocked` — технический блокер, issue → `Analysis Blocked`

Команда инкапсулирует: git add/commit, git push, draft PR (для plan-ready),
смену статуса в GitHub Project, обновление session.json.

Агент НЕ выполняет git/gh операции самостоятельно.

Спецификация: ADR-0020, `docs/adr/0020-agent-session-completion-signal.md`.
```

**Step 3: Добавить запись в журнал изменений**

В секцию «Журнал изменений» добавить:

```markdown
### 2026-03-14

- добавлен контракт завершения стадии `complete-stage` (ADR-0020)
- закрыт открытый вопрос о machine-readable артефактах — решение через CLI-команду
```

**Step 4: Commit**

```bash
git add docs/issue-analysis-flow.md
git commit -m "docs(ssot): add complete-stage contract to issue-analysis-flow"
```

---

### Task 7: Полная проверка — все тесты и компиляция

**Files:** (нет изменений, только проверка)

**Step 1: Запустить все тесты**

Run: `cargo test`
Expected: все тесты PASS

**Step 2: Проверить cargo clippy**

Run: `cargo clippy -- -D warnings 2>&1 | tail -20`
Expected: без ошибок (warnings допустимы, но не clippy::error)

**Step 3: Исправить проблемы если есть**

Если clippy/tests обнаружили проблемы — исправить и закоммитить.

**Step 4: Commit (если были исправления)**

```bash
git commit -m "fix: address clippy/test issues in complete-stage"
```

---

## Порядок acceptance criteria

| AC из issue | Покрывается в Task |
|-|-|
| Agent session может перевести issue из Analysis In Progress в нужный статус | Task 2 (complete_stage.rs: update_project_status) |
| Переход автоматический, без ручного редактирования | Task 2 (единая CLI-команда) |
| Контракт описан в документации | Task 5 (flow prompt), Task 6 (SSOT) |
| Integration test в headless zellij/docker CI | Отдельный follow-up (требует CI-инфраструктуру) |
| Явная диагностика при невалидном сигнале | Task 2 (StageOutcome::parse, eprintln diagnostics) |

## Примечание по integration tests

AC требует integration test в headless zellij/docker CI. Это требует:
- stub-агент (bash-скрипт, который вызывает complete-stage)
- headless zellij setup (или docker)
- CI pipeline

Это отдельная задача, выходящая за scope этого плана. Рекомендуется создать отдельный issue для CI-инфраструктуры после того, как complete-stage работает end-to-end.
