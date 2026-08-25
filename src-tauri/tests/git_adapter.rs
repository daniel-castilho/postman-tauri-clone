// Git-native interface integration tests (P3 epic).
//
// Drives the real `GitProcessAdapter` against temporary repositories
// created with the system git executable. Skips silently when git is not
// available on the host so the suite stays green anywhere.

use std::path::PathBuf;
use std::process::Command;

use tyny_pulse_lib::application::ports::git_repository::GitRepositoryPort;
use tyny_pulse_lib::domain::models::GitFileStatusType;
use tyny_pulse_lib::infrastructure::git::git_process_adapter::GitProcessAdapter;

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn temp_workspace(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tyny-git-it-{}-{}-{}",
        std::process::id(),
        name,
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&dir).expect("create temp workspace");
    dir
}

fn init_repo(dir: &PathBuf) {
    let run = |args: &[&str]| {
        let output = Command::new("git")
            .current_dir(dir)
            .args(args)
            .output()
            .expect("spawn git");
        assert!(output.status.success(), "git {:?} failed: {}", args, String::from_utf8_lossy(&output.stderr));
    };
    // Default branch name varies across git versions; pin it.
    run(&["init", "-b", "main"]);
    run(&["config", "user.email", "ci@tyny.ca"]);
    run(&["config", "user.name", "Tyny CI"]);
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Runtime::new().expect("tokio runtime")
}

#[test]
fn status_stage_commit_diff_and_branch_lifecycle() {
    if !git_available() {
        eprintln!("skipping: git executable not available");
        return;
    }
    let workspace = temp_workspace("lifecycle");
    init_repo(&workspace);
    let path = workspace.to_string_lossy().to_string();
    let adapter = GitProcessAdapter::new();
    let rt = runtime();

    // 1. Untracked file appears as an unstaged change. The auto-generated
    // .gitignore legitimately shows up as untracked too.
    std::fs::write(workspace.join("sample.json"), "{\n  \"key\": \"value\"\n}\n").unwrap();
    let status = rt.block_on(adapter.get_status(&path)).unwrap();
    assert!(status.is_repository);
    assert_eq!(status.current_branch, "main");
    let sample_entry = status
        .files
        .iter()
        .find(|file| file.path == "sample.json")
        .expect("sample.json in status");
    assert_eq!(sample_entry.status, GitFileStatusType::Untracked);
    assert!(!sample_entry.is_staged);

    // 2. Staging moves the entry into the staged list.
    rt.block_on(adapter.stage_path(&path, "sample.json")).unwrap();
    let status = rt.block_on(adapter.get_status(&path)).unwrap();
    let sample_entry = status
        .files
        .iter()
        .find(|file| file.path == "sample.json")
        .expect("staged sample.json in status");
    assert!(sample_entry.is_staged);
    assert_eq!(sample_entry.status, GitFileStatusType::Added);

    // 3. Commit returns a hash and sample.json leaves the change list
    // (only the untracked .gitignore may remain).
    let hash = rt.block_on(adapter.commit(&path, "feat: initial sample")).unwrap();
    assert!(!hash.is_empty());
    let status = rt.block_on(adapter.get_status(&path)).unwrap();
    assert!(
        !status.files.iter().any(|file| file.path == "sample.json"),
        "sample.json must be committed: {:?}",
        status.files
    );

    // 4. Modifying the file yields an unstaged modification with diff chunks.
    std::fs::write(workspace.join("sample.json"), "{\n  \"key\": \"changed\"\n}\n").unwrap();
    let status = rt.block_on(adapter.get_status(&path)).unwrap();
    assert_eq!(status.files.iter().filter(|file| file.path == "sample.json").count(), 1);
    let sample_entry = status.files.iter().find(|file| file.path == "sample.json").unwrap();
    assert!(!sample_entry.is_staged);
    assert_eq!(sample_entry.status, GitFileStatusType::Modified);

    let diff = rt.block_on(adapter.get_diff(&path, "sample.json", false)).unwrap();
    assert_eq!(diff.path, "sample.json");
    // One replaced line = one delete chunk plus one add chunk.
    assert_eq!(diff.chunks.len(), 2);
    assert_eq!(diff.chunks[0].change_type, "delete");
    assert!(diff.chunks[0].content.contains("\"value\""));
    assert_eq!(diff.chunks[1].change_type, "add");
    assert!(diff.chunks[1].content.contains("\"changed\""));

    let staged_diff = rt.block_on(adapter.get_diff(&path, "sample.json", true)).unwrap();
    assert!(staged_diff.chunks.is_empty(), "index still matches HEAD before staging");

    // 5. Branch creation and checkout reflect in the summary.
    rt.block_on(adapter.checkout_branch(&path, "feature/x", true)).unwrap();
    let status = rt.block_on(adapter.get_status(&path)).unwrap();
    assert_eq!(status.current_branch, "feature/x");
    assert!(status.branches.iter().any(|name| name == "feature/x"));
    assert!(status.branches.iter().any(|name| name == "main"));

    // 6. main is checkoutable again.
    rt.block_on(adapter.checkout_branch(&path, "main", false)).unwrap();
    let status = rt.block_on(adapter.get_status(&path)).unwrap();
    assert_eq!(status.current_branch, "main");

    std::fs::remove_dir_all(workspace).ok();
}

#[test]
fn security_exclusions_keep_vault_files_out_of_status() {
    if !git_available() {
        eprintln!("skipping: git executable not available");
        return;
    }
    let workspace = temp_workspace("vault");
    init_repo(&workspace);
    let path = workspace.to_string_lossy().to_string();

    // Simulate a tracked baseline so the repo is valid, then drop secret
    // files next to it.
    std::fs::write(workspace.join("collection.json"), "{}\n").unwrap();
    let adapter = GitProcessAdapter::new();
    let rt = runtime();
    rt.block_on(adapter.stage_path(&path, "collection.json")).unwrap();
    rt.block_on(adapter.commit(&path, "chore: baseline")).unwrap();

    // get_status must create the security exclusions block on first read.
    let status = rt.block_on(adapter.get_status(&path)).unwrap();
    let gitignore = std::fs::read_to_string(workspace.join(".gitignore"))
        .expect("get_status must generate .gitignore");
    for required in ["*.vault.enc", ".vault.key", ".env.local"] {
        assert!(gitignore.contains(required), ".gitignore missing {}", required);
    }

    std::fs::write(workspace.join("secrets.vault.enc"), "cipher").unwrap();
    std::fs::write(workspace.join(".vault.key"), "master-key").unwrap();
    let status = rt.block_on(adapter.get_status(&path)).unwrap();
    assert!(
        !status.files.iter().any(|file| file.path.contains("vault")),
        "vault artifacts leaked into status: {:?}",
        status.files
    );

    std::fs::remove_dir_all(workspace).ok();
}

#[test]
fn non_repository_workspace_reports_is_repository_false() {
    if !git_available() {
        eprintln!("skipping: git executable not available");
        return;
    }
    let workspace = temp_workspace("plain");
    let path = workspace.to_string_lossy().to_string();

    let adapter = GitProcessAdapter::new();
    let rt = runtime();
    let status = rt.block_on(adapter.get_status(&path)).unwrap();
    assert!(!status.is_repository);
    assert!(status.branches.is_empty());
    assert!(status.files.is_empty());
    // Security initializer must NOT have written a .gitignore outside a repo.
    assert!(!workspace.join(".gitignore").exists());

    std::fs::remove_dir_all(workspace).ok();
}
