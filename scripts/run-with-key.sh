#!/usr/bin/env bash
# Key exists only in this process environment and the application it launches.
set +x
set -euo pipefail
task_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
binary="$task_root/artifacts/zwen"
if [[ ! -x "$binary" ]]; then
  echo '尚无普通 Zwen 程序，请先构建。' >&2
  exit 1
fi
if pgrep -u "$UID" -x zwen >/dev/null; then
  "$binary" quit
  for attempt in {1..80}; do
    if ! pgrep -u "$UID" -x zwen >/dev/null; then break; fi
    sleep 0.1
  done
  if pgrep -u "$UID" -x zwen >/dev/null; then
    echo 'Zwen 尚未保存退出。请保留当前内容，保存后再运行本脚本。' >&2
    exit 1
  fi
fi
read -r -s -p '请输入智谱 API Key（不回显、不保存到文件）: ' ZWEN_API_KEY
printf '\n'
if [[ -z "$ZWEN_API_KEY" ]]; then echo '密钥为空，未启动。' >&2;exit 1;fi
export ZWEN_API_KEY
exec "$binary" show
