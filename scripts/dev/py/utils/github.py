"""GitHub API 操作工具（使用 urllib.request，零依赖）"""
import os
import json
import sys
import urllib.request
import urllib.parse
import urllib.error
from pathlib import Path
from typing import Optional, Tuple, Dict, Any

# 添加父目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils.logger import log_error


def _get_github_token() -> Optional[str]:
    """获取 GitHub token"""
    # 优先级：GITHUB_TOKEN > GITHUB_PAT
    return os.environ.get('GITHUB_TOKEN') or os.environ.get('GITHUB_PAT')


def _get_owner_and_repo() -> Tuple[str, str]:
    """从环境变量或 Git remote 获取 owner 和 repo"""
    # 尝试从环境变量获取
    owner = os.environ.get('GITHUB_REPOSITORY_OWNER')
    repo = os.environ.get('GITHUB_REPOSITORY')

    if owner and repo:
        # GITHUB_REPOSITORY 格式：owner/repo
        if '/' in repo:
            parts = repo.split('/', 1)
            return parts[0], parts[1]
        return owner, repo

    # 从 Git remote 获取
    try:
        import subprocess
        result = subprocess.run(
            ['git', 'remote', 'get-url', 'origin'],
            capture_output=True,
            text=True,
            check=True
        )
        url = result.stdout.strip()

        # 解析 GitHub URL
        # 格式：https://github.com/owner/repo.git 或 git@github.com:owner/repo.git
        if 'github.com' in url:
            if url.startswith('https://'):
                parts = url.replace('https://github.com/', '').replace('.git', '').split('/')
            elif url.startswith('git@'):
                parts = url.replace('git@github.com:', '').replace('.git', '').split('/')
            else:
                parts = url.split('/')

            if len(parts) >= 2:
                return parts[-2], parts[-1].replace('.git', '')
    except Exception:
        pass

    log_error("Failed to get GitHub owner and repo")
    sys.exit(1)


def _make_request(method: str, url: str, data: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
    """发送 GitHub API 请求"""
    token = _get_github_token()
    if not token:
        log_error("GitHub token not found. Set GITHUB_TOKEN or GITHUB_PAT")
        sys.exit(1)

    headers = {
        'Authorization': f'token {token}',
        'Accept': 'application/vnd.github.v3+json',
        'Content-Type': 'application/json',
    }

    req_data = None
    if data:
        req_data = json.dumps(data).encode('utf-8')

    request = urllib.request.Request(url, data=req_data, headers=headers, method=method)

    try:
        with urllib.request.urlopen(request) as response:
            response_data = response.read().decode('utf-8')
            return json.loads(response_data)
    except urllib.error.HTTPError as e:
        error_body = e.read().decode('utf-8')
        try:
            error_data = json.loads(error_body)
            error_msg = error_data.get('message', error_body)
        except Exception:
            error_msg = error_body
        log_error(f"GitHub API error ({e.code}): {error_msg}")
        raise
    except Exception as e:
        log_error(f"GitHub API request failed: {e}")
        raise


def create_pull_request(
    title: str,
    body: str,
    head: str,
    base: str = 'master'
) -> Tuple[str, str]:
    """创建 Pull Request

    返回: (pr_url, pr_number)
    """
    owner, repo = _get_owner_and_repo()
    url = f"https://api.github.com/repos/{owner}/{repo}/pulls"

    data = {
        'title': title,
        'body': body,
        'head': head,
        'base': base,
    }

    try:
        response = _make_request('POST', url, data)
        pr_url = response['html_url']
        pr_number = str(response['number'])
        return pr_url, pr_number
    except urllib.error.HTTPError as e:
        if e.code == 422:
            # PR 可能已存在，尝试查找
            return _find_existing_pr(head, base)
        raise


def _find_existing_pr(head: str, base: str) -> Tuple[str, str]:
    """查找现有的 PR"""
    owner, repo = _get_owner_and_repo()
    url = f"https://api.github.com/repos/{owner}/{repo}/pulls?head={owner}:{head}&base={base}&state=open"

    try:
        response = _make_request('GET', url)
        if response and len(response) > 0:
            pr = response[0]
            return pr['html_url'], str(pr['number'])
    except Exception:
        pass

    log_error(f"PR not found for {head} -> {base}")
    sys.exit(1)


def get_pull_request(pr_number: str) -> Dict[str, Any]:
    """获取 PR 信息"""
    owner, repo = _get_owner_and_repo()
    url = f"https://api.github.com/repos/{owner}/{repo}/pulls/{pr_number}"

    return _make_request('GET', url)


def merge_pull_request(
    pr_number: str,
    commit_title: Optional[str] = None,
    commit_message: Optional[str] = None,
    merge_method: str = 'squash'
) -> bool:
    """合并 Pull Request

    返回: 是否成功合并
    """
    owner, repo = _get_owner_and_repo()
    url = f"https://api.github.com/repos/{owner}/{repo}/pulls/{pr_number}/merge"

    data = {
        'merge_method': merge_method,
    }

    if commit_title:
        data['commit_title'] = commit_title
    if commit_message:
        data['commit_message'] = commit_message

    try:
        response = _make_request('PUT', url, data)
        return response.get('merged', False)
    except urllib.error.HTTPError as e:
        if e.code == 405:
            # PR 可能已经合并或无法合并
            return False
        raise

