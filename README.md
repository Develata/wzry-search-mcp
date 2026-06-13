# wzry-search-mcp

`wzry-search-mcp` 是一个面向 AI Agent 的王者荣耀本地事实检索 MCP。它从公开官方资料构建本地 SQLite 数据集，提供英雄基础信息及被动/技能信息、装备信息、召唤师技能信息、更新检测、本地查询，以及供模型进行阵容推荐的 evidence context。

## Scope

### Included

- 英雄基础信息及技能信息：英雄基础信息与被动/主动技能在查询层强绑定。
- 装备信息。
- 召唤师技能信息。
- 更新检测。
- 本地查询。
- 阵容推荐支持：MCP 提供事实材料，推荐判断由调用方模型完成。

### Excluded

- 皮肤信息。
- 图片素材。
- 皮肤图 URL。
- 大量静态媒体资源。
- 铭文。
- 写死的阵容推荐规则、英雄梯度或协同评分。

## Data Sources

- 英雄列表：<https://pvp.qq.com/web201605/js/herolist.json>
- 英雄详情页：`https://pvp.qq.com/web201605/herodetail/{hero_id}.shtml`
  - 若数字详情页 404，则 fallback 到 `https://pvp.qq.com/web201605/herodetail/{id_name}.shtml`。
- 装备列表：<https://pvp.qq.com/web201605/js/item.json>
- 召唤师技能 JSON：<https://pvp.qq.com/web201605/js/summoner.json>
- 新闻/公告列表：可作为人工排查来源；当前不纳入 deterministic hash snapshot，因为页面动态片段可能导致连续检查抖动。

这些 URL 是公开页面/静态资料入口；项目不镜像图片和大量媒体资源。

## Quick Start

```bash
# build
cargo build

# first sync, polite crawling enabled by default
cargo run -- --db ./wzry.sqlite sync

# query local data
cargo run -- --db ./wzry.sqlite list-heroes --limit 20
cargo run -- --db ./wzry.sqlite hero 廉颇
cargo run -- --db ./wzry.sqlite search-skills 护盾 --limit 10
cargo run -- --db ./wzry.sqlite list-items --limit 20
cargo run -- --db ./wzry.sqlite item 破军
cargo run -- --db ./wzry.sqlite summoner 闪现

# update check only; this checks deterministic list JSON sources, not every hero detail page
cargo run -- --db ./wzry.sqlite check-updates

# start MCP stdio server
cargo run -- --db ./wzry.sqlite serve

# export local dataset
cargo run -- --db ./wzry.sqlite export --format json --out ./wzry.json
cargo run -- --db ./wzry.sqlite export --format csv --out ./csv
```

## MCP Tools

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

Update checks are CLI-only via `check-updates`; MCP tools intentionally remain local factual query tools.

## Lineup Recommendation Boundary

阵容推荐由调用方模型完成。

`wzry-search-mcp` 不写死英雄梯度、协同评分或固定阵容模板。它只提供带来源的英雄完整资料、被动/技能文本、装备信息、召唤师技能信息与批量阵容上下文。调用方模型基于这些事实自行推理并给出推荐。

## Additional Docs

- [SPEC.md](SPEC.md) — scope, schema, update semantics, and MCP contract.
- [docs/architecture.md](docs/architecture.md) — layered skeleton, module boundaries, and engineering-constitution alignment.
- [docs/tools.md](docs/tools.md) — MCP tool list and output shapes.
- [docs/update-detection.md](docs/update-detection.md) — source hash checking, local cron data-maintenance policy, and GitHub source-smoke policy.
- [docs/development.md](docs/development.md) — local validation, smoke tests, and review packet checklist.
- [docs/hermes-mcp.md](docs/hermes-mcp.md) — Hermes MCP configuration example.
- [docs/release-artifacts.md](docs/release-artifacts.md) — GitHub release artifact and dataset policy.

## Polite Crawling

默认同步策略是低频、串行、带随机 delay 的 polite crawling：

- 不并发请求英雄详情页。
- 默认请求间隔随机化。
- 失败有限重试。
- 本地记录 hash，只有变化时更新。
- 每周更新检测应优先只拉核心源和公告列表。

## License

MIT
