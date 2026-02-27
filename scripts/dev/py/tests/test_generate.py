import subprocess
import textwrap
from pathlib import Path

import pytest

from version.generate import update_cargo_files


def write_cargo_toml(path: Path, version: str):
    content = textwrap.dedent(f"""
    [workspace.package]
    name = "workflow"
    version = "0.0.1"

    [workspace.dependencies]
    client = {{ path = "crates/client", version = "0.0.1" }}
    domain = {{ path = "crates/domain", version = "0.0.1" }}
    """
    )
    path.write_text(content)


def test_update_cargo_files_updates_version(tmp_path, monkeypatch, capsys):
    # prepare a temporary Cargo.toml
    cargo = tmp_path / "Cargo.toml"
    write_cargo_toml(cargo, "0.0.1")

    # monkeypatch subprocess.run to record call
    calls = []

    def fake_run(args, capture_output, text, check):
        calls.append(args)
        # simulate success
        class Result:
            returncode = 0
            stdout = ""
            stderr = ""
        return Result()

    monkeypatch.setattr(subprocess, "run", fake_run)

    # change cwd to tmp_path for the duration
    import os
    old_cwd = os.getcwd()
    try:
        os.chdir(tmp_path)
        update_cargo_files("1.2.3")
    finally:
        os.chdir(old_cwd)

    # ensure Cargo.toml updated
    updated = cargo.read_text()
    assert "version = \"1.2.3\"" in updated
    assert calls == [["cargo", "update", "--workspace"]]


def test_update_cargo_files_no_cargo_toml(tmp_path):
    # nothing in directory
    import os
    old = os.getcwd()
    try:
        os.chdir(tmp_path)
        with pytest.raises(SystemExit) as exc:
            update_cargo_files("1.0.0")
        assert exc.value.code == 1
    finally:
        os.chdir(old)


def test_generate_master_uses_cargo_version_when_higher(tmp_path, monkeypatch, capsys):
    # create a Cargo.toml with version higher than computed
    cargo = tmp_path / "Cargo.toml"
    cargo.write_text(textwrap.dedent("""
    [workspace.package]
    version = "2.0.0"
    """))

    # monkeypatch git helpers to simulate no tags and simple commits
    monkeypatch.setattr('version.generate.get_last_commit_sha', lambda: None)
    monkeypatch.setattr('version.generate.get_commits_between', lambda a, b: [])
    monkeypatch.setattr('version.generate.get_branch_commits', lambda n=10: [])
    monkeypatch.setattr('version.generate.run_git_command', lambda args, check=True: subprocess.CompletedProcess(args, 1))

    # run in tmp directory
    import os
    old = os.getcwd()
    try:
        os.chdir(tmp_path)
        ver, tag, inc = __import__('version.generate', fromlist=['generate_master_version']).generate_master_version('1.0.0', '')
    finally:
        os.chdir(old)

    assert ver == '2.0.0'
    assert tag == 'v2.0.0'
    assert inc is False


def test_output_github_actions_writes_file(tmp_path):
    from version.generate import output_github_actions
    out = tmp_path / "gh.txt"
    env = {
        **__import__('os').environ,
        'GITHUB_OUTPUT': str(out)
    }
    # use isolated environment
    import os, subprocess
    old_env = os.environ.copy()
    os.environ.update(env)
    try:
        output_github_actions("1.2.3", "v1.2.3", True)
    finally:
        os.environ.clear(); os.environ.update(old_env)

    contents = out.read_text().splitlines()
    assert "version=1.2.3" in contents
    assert "tag=v1.2.3" in contents
    assert "needs_increment=true" in contents


def test_generate_with_ci_flag_sets_output(tmp_path, monkeypatch):
    # stub underlying helpers to return deterministic values
    monkeypatch.setattr('version.generate.get_latest_version', lambda: ("0.0.0", ""))
    monkeypatch.setattr('version.generate.generate_master_version', lambda a, b: ("0.0.1", "v0.0.1", True))

    out = tmp_path / "gh.txt"
    import os
    old_env = os.environ.copy()
    os.environ['GITHUB_OUTPUT'] = str(out)

    try:
        # invoke generate with master=True and ci=True
        import argparse
        from version.generate import generate
        args = argparse.Namespace(master=True, update=False, ci=True)
        version, tag, inc = generate(args)
    finally:
        os.environ.clear(); os.environ.update(old_env)

    assert version == "0.0.1"
    assert tag == "v0.0.1"
    assert inc is True
    assert "version=0.0.1" in out.read_text()

