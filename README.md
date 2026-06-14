# wzry-search-mcp

`wzry-search-mcp` 是一个面向 AI Agent 的王者荣耀本地事实检索 MCP。它从公开官方资料构建本地 SQLite 数据集，提供英雄基础信息及被动/技能信息、装备信息、召唤师技能信息、更新检测、本地查询，以及供模型进行阵容推荐的证据上下文。

## 功能范围

### 已包含

- 英雄基础信息及技能信息：英雄基础信息与被动/主动技能在查询层强绑定。
- 装备信息。
- 召唤师技能信息。
- 更新检测。
- 本地查询。
- 阵容推荐支持：MCP 提供事实材料，推荐判断由调用方模型完成。

### 不包含

- 皮肤信息。
- 图片素材。
- 皮肤图 URL。
- 大量静态媒体资源。
- 铭文。
- 写死的阵容推荐规则、英雄梯度或协同评分。

## 数据来源

- 英雄列表：<https://pvp.qq.com/web201605/js/herolist.json>
- 英雄详情页：`https://pvp.qq.com/web201605/herodetail/{hero_id}.shtml`
  - 若数字详情页返回 404，则回退到 `https://pvp.qq.com/web201605/herodetail/{id_name}.shtml`。
- 装备列表：<https://pvp.qq.com/web201605/js/item.json>
- 召唤师技能 JSON：<https://pvp.qq.com/web201605/js/summoner.json>
- 新闻/公告列表：可作为人工排查来源；当前不纳入确定性哈希快照，因为页面动态片段可能导致连续检查抖动。

这些 URL 是公开页面/静态资料入口；项目不镜像图片和大量媒体资源。

## 从发布版二进制文件安装

当前稳定版本：[`v0.3.0`](https://github.com/Develata/wzry-search-mcp/releases/tag/v0.3.0)。

Linux x86_64 发布包包含一个独立应用程序二进制文件及配套文档。干净安装时，最终只会在你指定的位置留下可执行文件；下载与解压产生的临时文件可以自动清理。

```bash
set -euo pipefail

tmp="$(mktemp -d /tmp/wzry-search-mcp-install.XXXXXX)"
trap 'rm -rf "$tmp"' EXIT

cd "$tmp"

curl -L -O https://github.com/Develata/wzry-search-mcp/releases/download/v0.3.0/wzry-search-mcp-linux-x86_64.tar.gz
curl -L -O https://github.com/Develata/wzry-search-mcp/releases/download/v0.3.0/wzry-search-mcp-linux-x86_64.tar.gz.sha256

sha256sum -c wzry-search-mcp-linux-x86_64.tar.gz.sha256

tar -xzf wzry-search-mcp-linux-x86_64.tar.gz

install -Dm755 \
  wzry-search-mcp-linux-x86_64/wzry-search-mcp \
  /opt/data/bin/wzry-search-mcp

/opt/data/bin/wzry-search-mcp --version
```

预期版本输出：

```text
wzry-search-mcp 0.3.0
```

### 二进制安装会创建哪些文件

上面的安装命令最终只会创建或替换：

```text
/opt/data/bin/wzry-search-mcp
```

临时文件会出现在 `mktemp` 创建的目录下，例如：

```text
/tmp/wzry-search-mcp-install.xxxxxx/
  wzry-search-mcp-linux-x86_64.tar.gz
  wzry-search-mcp-linux-x86_64.tar.gz.sha256
  wzry-search-mcp-linux-x86_64/
    wzry-search-mcp
    README.md
    SPEC.md
    LICENSE
    config.example.toml
    docs/
    schemas/
```

`trap 'rm -rf "$tmp"' EXIT` 会在脚本结束时删除这个临时目录，因此二进制安装不会散落包管理器、npm、cargo 或 Docker 缓存文件。

### 持久数据文件

二进制文件本身不会写入全局配置。只有在你用数据库路径运行命令时，才会创建项目持久数据，例如：

```bash
mkdir -p /opt/data/wzry-search-mcp

/opt/data/bin/wzry-search-mcp \
  --db /opt/data/wzry-search-mcp/wzry.sqlite \
  sync
```

这会创建本地规范 SQLite 数据库：

```text
/opt/data/wzry-search-mcp/wzry.sqlite
```

写入过程中，SQLite 可能会在旁边短暂创建正常的日志文件：

```text
/opt/data/wzry-search-mcp/wzry.sqlite-journal
/opt/data/wzry-search-mcp/wzry.sqlite-wal
/opt/data/wzry-search-mcp/wzry.sqlite-shm
```

这些是 SQLite 的记账文件，不是安装器残留碎片。

### 卸载

删除已安装的二进制文件：

```bash
rm -f /opt/data/bin/wzry-search-mcp
```

同时删除本地数据集：

```bash
rm -rf /opt/data/wzry-search-mcp
```

如果你曾把该 MCP 加入某个 Agent 的配置，还需要从配置中删除 `wzry-search` 条目，然后重启 Agent 或重新加载 MCP 配置。

## 从源码快速开始

```bash
# 构建
cargo build

# 首次同步；默认启用礼貌抓取
cargo run -- --db ./wzry.sqlite sync

# 查询本地数据
cargo run -- --db ./wzry.sqlite list-heroes --limit 20
cargo run -- --db ./wzry.sqlite hero 廉颇
cargo run -- --db ./wzry.sqlite search-skills 护盾 --limit 10
cargo run -- --db ./wzry.sqlite list-items --limit 20
cargo run -- --db ./wzry.sqlite item 破军
cargo run -- --db ./wzry.sqlite summoner 闪现

# 仅检查更新；这里只检查确定性的列表 JSON 源，不逐个检查所有英雄详情页
cargo run -- --db ./wzry.sqlite check-updates

# 基于官方更新类新闻做受影响英雄详情页增量同步；dry-run 只分析不刷新
cargo run -- --db ./wzry.sqlite sync-changed --news-limit 10 --dry-run
cargo run -- --db ./wzry.sqlite sync-changed --news-limit 10

# 启动 MCP 标准输入/输出服务器
cargo run -- --db ./wzry.sqlite serve

# 导出本地数据集
cargo run -- --db ./wzry.sqlite export --format json --out ./wzry.json
cargo run -- --db ./wzry.sqlite export --format csv --out ./csv
```

安装本地构建出的二进制文件：

```bash
cargo build --release
install -Dm755 target/release/wzry-search-mcp /opt/data/bin/wzry-search-mcp
/opt/data/bin/wzry-search-mcp --version
```

## 本地命令行用法

安装发布版二进制文件并同步数据后，可以这样查询：

```bash
/opt/data/bin/wzry-search-mcp --db /opt/data/wzry-search-mcp/wzry.sqlite list-heroes --limit 20
/opt/data/bin/wzry-search-mcp --db /opt/data/wzry-search-mcp/wzry.sqlite hero 廉颇
/opt/data/bin/wzry-search-mcp --db /opt/data/wzry-search-mcp/wzry.sqlite search-heroes 廉
/opt/data/bin/wzry-search-mcp --db /opt/data/wzry-search-mcp/wzry.sqlite search-skills 护盾 --limit 10
/opt/data/bin/wzry-search-mcp --db /opt/data/wzry-search-mcp/wzry.sqlite summoner 闪现
```

导出本地数据集：

```bash
/opt/data/bin/wzry-search-mcp \
  --db /opt/data/wzry-search-mcp/wzry.sqlite \
  export --format json --out /opt/data/wzry-search-mcp/wzry.json

/opt/data/bin/wzry-search-mcp \
  --db /opt/data/wzry-search-mcp/wzry.sqlite \
  export --format csv --out /opt/data/wzry-search-mcp/csv
```

如果只想做一次短冒烟测试，而不是完整同步：

```bash
/opt/data/bin/wzry-search-mcp \
  --db /opt/data/wzry-search-mcp/wzry.sqlite \
  sync \
  --limit-heroes 2 \
  --min-delay-ms 300 \
  --max-delay-ms 800
```

## 数据更新与定时维护

`wzry-search-mcp` 自身不内置常驻 scheduler；它只提供一次性 CLI 命令。定时更新应由宿主机 cron、systemd timer、Hermes cron 或其它运维平台调用这些命令。

### 首次建库或手动全量刷新

```bash
mkdir -p /opt/data/wzry-search-mcp

/opt/data/bin/wzry-search-mcp \
  --db /opt/data/wzry-search-mcp/wzry.sqlite \
  sync
```

### 手动检查确定性源是否变化

`check-updates` 只检查英雄列表、装备列表、召唤师技能列表这些确定性 JSON 源；它不会逐个抓取英雄详情页。

```bash
/opt/data/bin/wzry-search-mcp \
  --db /opt/data/wzry-search-mcp/wzry.sqlite \
  check-updates --write-snapshots
```

### 手动按官方更新新闻增量刷新受影响英雄

先 dry-run 查看会命中哪些公告和英雄：

```bash
/opt/data/bin/wzry-search-mcp \
  --db /opt/data/wzry-search-mcp/wzry.sqlite \
  sync-changed --news-limit 10 --dry-run
```

确认后执行真实刷新：

```bash
/opt/data/bin/wzry-search-mcp \
  --db /opt/data/wzry-search-mcp/wzry.sqlite \
  sync-changed --news-limit 10
```

### cron 示例

下面是一个保守的宿主机 cron 示例：每天跑一次新闻增量刷新，每周跑一次确定性源快照检查，每月跑一次礼貌全量刷新。路径按你的实际安装位置调整。

```bash
mkdir -p /opt/data/wzry-search-mcp/logs
crontab -e
```

加入：

```cron
# 每天 05:20：按官方更新类新闻刷新受影响英雄详情页
20 5 * * * /opt/data/bin/wzry-search-mcp --db /opt/data/wzry-search-mcp/wzry.sqlite sync-changed --news-limit 10 >> /opt/data/wzry-search-mcp/logs/sync-changed.log 2>&1

# 每周一 05:40：记录确定性列表源 hash 快照和变化事件
40 5 * * 1 /opt/data/bin/wzry-search-mcp --db /opt/data/wzry-search-mcp/wzry.sqlite check-updates --write-snapshots >> /opt/data/wzry-search-mcp/logs/check-updates.log 2>&1

# 每月 1 日 04:30：礼貌全量刷新，作为保守兜底
30 4 1 * * /opt/data/bin/wzry-search-mcp --db /opt/data/wzry-search-mcp/wzry.sqlite sync >> /opt/data/wzry-search-mcp/logs/full-sync.log 2>&1
```

查看最近日志：

```bash
tail -n 80 /opt/data/wzry-search-mcp/logs/sync-changed.log
tail -n 80 /opt/data/wzry-search-mcp/logs/check-updates.log
tail -n 80 /opt/data/wzry-search-mcp/logs/full-sync.log
```

如果部署在 AstrBot 容器挂载目录中，数据库路径和二进制路径应替换为你实际映射的宿主机路径；MCP 配置里的容器内 `command` / `args` 不需要因为定时更新而改变。

## Hermes MCP 配置

Hermes Agent 支持通过 `mcp_servers` 配置本地标准输入/输出 MCP 服务器。官方说明见：<https://hermes-agent.nousresearch.com/docs/user-guide/features/mcp>。

```yaml
mcp_servers:
  wzry-search:
    enabled: true
    command: /opt/data/bin/wzry-search-mcp
    args:
      - --db
      - /opt/data/wzry-search-mcp/wzry.sqlite
      - serve
```

如果配置中已经存在 `mcp_servers:`，只合并 `wzry-search:` 这一项，不要覆盖整个配置段。

验证服务器：

```bash
hermes mcp test wzry-search
```

修改 MCP 配置后，需要重启 Agent 或重新加载 MCP。Hermes 命令行会话中可以开启新会话或使用 `/reload-mcp`；网关会话中通常使用 `/restart` 更稳。

## MCP 协议兼容性

本项目是基于官方 Rust MCP SDK（RMCP）的标准输入/输出 MCP 服务器：

- 从标准输入读取 UTF-8 JSON-RPC 消息；
- 向标准输出写出换行分隔的 JSON-RPC 消息；
- 日志只写入标准错误，不污染标准输出；
- 支持 `initialize`、`notifications/initialized`、`ping`、`tools/list`、`tools/call`；
- 工具能力声明为 tools；
- 所有工具均提供 `outputSchema`；
- 工具调用返回 `structuredContent`，并同时返回文本形式的 JSON，兼容只能读取 `content` 的旧客户端。

`v0.2.0` 起，MCP 协议层改用官方 RMCP SDK，并启用结构化输出。标准输出使用 MCP stdio 规范要求的换行分隔 JSON-RPC。`v0.1.1` 曾额外兼容旧式 `Content-Length` 输入帧；迁移到 RMCP 后不再维护这条非主路径兼容性。

如果在 AstrBot 中使用，AstrBot 可能会拦截不在默认白名单中的标准输入/输出命令。Docker 部署时可以在 `astrbot` 服务中加入：

```yaml
environment:
  - TZ=Asia/Shanghai
  - ASTRBOT_MCP_STDIO_ALLOWED_COMMANDS=bun,bunx,deno,node,npm,npx,pnpm,py,python,python3,uv,uvx,yarn,wzry-search-mcp
```

随后重启 AstrBot，并在 WebUI 中配置容器内路径，例如：

```json
{
  "command": "/AstrBot/data/wzry-search-mcp/bin/wzry-search-mcp",
  "args": [
    "--db",
    "/AstrBot/data/wzry-search-mcp/wzry.sqlite",
    "serve"
  ]
}
```

## MCP 工具

当前共 12 个工具，全部是本地 SQLite 查询；`tools/call` 不会联网抓取页面。

- `wzry_list_heroes`：列出本地英雄，方便 Agent 先发现可用英雄名。
- `wzry_search_heroes`：模糊搜索英雄候选。
- `wzry_get_hero_profile`：返回英雄基础信息 + 被动/主动技能。
- `wzry_get_hero_profiles`：批量返回完整英雄资料；阵容分析优先使用。
- `wzry_get_hero_skill`：查询指定英雄的被动/一/二/三技能或精确技能名。
- `wzry_search_hero_skills`：跨英雄搜索技能名/技能文本，返回英雄 + 技能命中。
- `wzry_list_items`：列出本地装备，方便 Agent 先发现可用装备名。
- `wzry_search_items` / `wzry_get_item`：装备查询。
- `wzry_get_summoner_skills` / `wzry_get_summoner_skill`：召唤师技能查询；前者即召唤师技能名单。
- `wzry_get_lineup_context`：返回己方/敌方/候选英雄完整资料，供模型自行做阵容推荐。

更新检查只通过命令行子命令 `check-updates` 提供；MCP 工具刻意保持为本地事实查询工具。

## 阵容推荐边界

阵容推荐由调用方模型完成。

`wzry-search-mcp` 不写死英雄梯度、协同评分或固定阵容模板。它只提供带来源的英雄完整资料、被动/技能文本、装备信息、召唤师技能信息与批量阵容上下文。调用方模型基于这些事实自行推理并给出推荐。

## 附加文档

- [SPEC.md](SPEC.md)：功能范围、数据结构、更新语义与 MCP 契约。
- [CHANGELOG.md](CHANGELOG.md)：版本变更摘要、验证证据与兼容性说明。
- [docs/AGENTS.md](docs/AGENTS.md)：Agent 阅读顺序与文档权威层级。
- [docs/coverage-matrix.md](docs/coverage-matrix.md)：工程契约、功能、验收用例、代码与测试的映射矩阵。
- [docs/architecture.md](docs/architecture.md)：分层骨架、模块边界与工程宪法对齐。
- [docs/tools.md](docs/tools.md)：MCP 工具列表与输出形态。
- [docs/plan/](docs/plan/)：scope、data model、source sync、MCP contract、error semantics、release/deployment 等工程契约。
- [docs/features/](docs/features/)：Agent 使用流程与 source sync/update 可观察行为。
- [docs/acceptance-cases/](docs/acceptance-cases/)：本地事实查询、source sync、MCP stdio、release artifact 的验收证据。
- [docs/registry/tool-contracts.md](docs/registry/tool-contracts.md)：稳定 MCP 工具名、参数与输出根对象登记表。
- [docs/update-detection.md](docs/update-detection.md)：源哈希检查、本地定时维护策略与 GitHub 源冒烟策略。
- [docs/development.md](docs/development.md)：本地验证、冒烟测试与审查材料清单。
- [docs/hermes-mcp.md](docs/hermes-mcp.md)：Hermes MCP 配置示例。
- [docs/release-artifacts.md](docs/release-artifacts.md)：GitHub 发布产物与数据集策略。

## 礼貌抓取

默认同步策略是低频、串行、带随机延迟的礼貌抓取：

- 不并发请求英雄详情页。
- 默认请求间隔随机化。
- 失败有限重试。
- 本地记录哈希，只有变化时更新。
- 每周更新检测应优先只拉核心源和公告列表。

## 许可证

MIT
