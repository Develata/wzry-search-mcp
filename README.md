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
- 装备列表：<https://pvp.qq.com/web201605/js/item.json>
- 召唤师技能 JSON：<https://pvp.qq.com/web201605/js/summoner.json>；若不可用则 fallback 到 <https://pvp.qq.com/web201605/summoner.shtml>
- 新闻/公告列表：<https://pvp.qq.com/web201706/newsindex.shtml>

这些 URL 是公开页面/静态资料入口；项目不镜像图片和大量媒体资源。

## Quick Start

```bash
# build
cargo build

# first sync, polite crawling enabled by default
cargo run -- sync --db ./wzry.sqlite

# query local data
cargo run -- hero 廉颇 --db ./wzry.sqlite
cargo run -- item 破军 --db ./wzry.sqlite
cargo run -- summoner 闪现 --db ./wzry.sqlite

# update check only
cargo run -- check-updates --db ./wzry.sqlite

# start MCP stdio server
cargo run -- serve --db ./wzry.sqlite

# export local dataset
cargo run -- export --format json --out ./wzry.json --db ./wzry.sqlite
cargo run -- export --format csv --out ./csv --db ./wzry.sqlite
```

## MCP Tools

- `wzry_search_heroes`：模糊搜索英雄候选。
- `wzry_get_hero_profile`：返回英雄基础信息 + 被动/主动技能。
- `wzry_get_hero_profiles`：批量返回完整英雄资料；阵容分析优先使用。
- `wzry_get_hero_skill`：查询指定英雄的被动/一/二/三技能。
- `wzry_search_items` / `wzry_get_item`：装备查询。
- `wzry_get_summoner_skills` / `wzry_get_summoner_skill`：召唤师技能查询。
- `wzry_check_updates`：检查官方源是否更新。
- `wzry_get_lineup_context`：返回己方/敌方/候选英雄完整资料，供模型自行做阵容推荐。

## Lineup Recommendation Boundary

阵容推荐由调用方模型完成。

`wzry-search-mcp` 不写死英雄梯度、协同评分或固定阵容模板。它只提供带来源的英雄完整资料、被动/技能文本、装备信息、召唤师技能信息与批量阵容上下文。调用方模型基于这些事实自行推理并给出推荐。

## Additional Docs

- [SPEC.md](SPEC.md) — scope, schema, update semantics, and MCP contract.
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
