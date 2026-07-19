#!/usr/bin/env bash
# Đẩy CodeKey lên GitHub.
# Repo: https://github.com/toilaai2212hp22-lgtm/codekeyvn
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# git portable (nếu hệ thống chưa cài git)
if ! command -v git >/dev/null 2>&1; then
  if [[ -x /tmp/git-extract/usr/bin/git ]]; then
    export PATH="/tmp/git-extract/usr/bin:/tmp/git-extract/usr/lib/git-core:$PATH"
    export GIT_EXEC_PATH=/tmp/git-extract/usr/lib/git-core
  else
    echo "Chưa có git. Cài: sudo apt install git"
    exit 1
  fi
fi

REMOTE="${1:-https://github.com/toilaai2212hp22-lgtm/codekeyvn.git}"

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "Chưa phải git repo."
  exit 1
fi

git remote remove origin 2>/dev/null || true
git remote add origin "$REMOTE"
git branch -M main

echo "==> Push lên $REMOTE"
echo "    GitHub sẽ hỏi đăng nhập."
echo "    Username: toilaai2212hp22-lgtm"
echo "    Password: dán Personal Access Token (không phải mật khẩu GitHub)"
echo
echo "    Tạo token: https://github.com/settings/tokens"
echo "    Quyền cần: repo"
echo

git push -u origin main

echo
echo "✓ Xong: https://github.com/toilaai2212hp22-lgtm/codekeyvn"
