use crate::db::Store;
use crate::model::{
    HeroBasic, HeroProfile, HeroSkill, HeroSkillSearchHit, Item, LineupContext, SummonerSkill,
};
use anyhow::Result;
use rmcp::{
    Json, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
};
use schemars::JsonSchema;

use serde::{Deserialize, Serialize};
use tokio::runtime::Builder;

#[derive(Debug, Clone)]
struct WzryMcpServer {
    db_path: String,
    tool_router: ToolRouter<Self>,
}

impl WzryMcpServer {
    fn new(db_path: impl Into<String>) -> Self {
        Self {
            db_path: db_path.into(),
            tool_router: Self::tool_router(),
        }
    }

    fn store(&self) -> std::result::Result<Store, String> {
        Store::open_existing(&self.db_path).map_err(|err| format!("{err:#}"))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for WzryMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("wzry-search-mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions(
                "王者荣耀本地事实检索 MCP；返回英雄、技能、装备、召唤师技能与阵容证据上下文。阵容推荐由调用方模型完成。",
            )
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct Limit500Params {
    /// Maximum number of rows to return. Values above 500 are capped.
    #[schemars(range(min = 1, max = 500))]
    limit: Option<usize>,
}

impl Limit500Params {
    fn limit(&self, default: usize) -> usize {
        self.limit.unwrap_or(default).min(500)
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchParams {
    /// Search query.
    query: String,
    /// Maximum number of rows to return. Values above 50 are capped.
    limit: Option<usize>,
}

impl SearchParams {
    fn limit(&self, default: usize) -> usize {
        self.limit.unwrap_or(default).min(50)
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct HeroParam {
    /// Hero name, id, id_name, or unambiguous fuzzy name.
    hero: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct HeroesParam {
    /// Hero names, ids, id_names, or unambiguous fuzzy names.
    heroes: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct HeroSkillParam {
    /// Hero name, id, id_name, or unambiguous fuzzy name.
    hero: String,
    /// Skill selector: passive/被动/1/2/3/大招 or exact skill name.
    skill: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ItemParam {
    /// Item name or unambiguous fuzzy name.
    item: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SummonerSkillParam {
    /// Summoner skill name or unambiguous fuzzy name.
    skill: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct LineupContextParams {
    /// Allied heroes.
    #[serde(default)]
    allies: Vec<String>,
    /// Enemy heroes.
    #[serde(default)]
    enemies: Vec<String>,
    /// Candidate heroes to compare.
    #[serde(default)]
    candidate_pool: Vec<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
struct HeroesOutput {
    heroes: Vec<HeroBasic>,
}

#[derive(Debug, Serialize, JsonSchema)]
struct HeroProfilesOutput {
    heroes: Vec<HeroProfile>,
}

#[derive(Debug, Serialize, JsonSchema)]
struct HeroSkillSearchOutput {
    hits: Vec<HeroSkillSearchHit>,
}

#[derive(Debug, Serialize, JsonSchema)]
struct ItemsOutput {
    items: Vec<Item>,
}

#[derive(Debug, Serialize, JsonSchema)]
struct SummonerSkillsOutput {
    summoner_skills: Vec<SummonerSkill>,
}

#[tool_router(router = tool_router)]
impl WzryMcpServer {
    /// List local heroes so agents can discover valid hero names before detailed queries.
    #[tool(name = "wzry_list_heroes")]
    async fn list_heroes(
        &self,
        Parameters(params): Parameters<Limit500Params>,
    ) -> std::result::Result<Json<HeroesOutput>, String> {
        let heroes = self
            .store()?
            .list_heroes(params.limit(200))
            .map_err(|err| format!("{err:#}"))?;
        Ok(Json(HeroesOutput { heroes }))
    }

    /// Search local hero candidates by name/id_name/title.
    #[tool(name = "wzry_search_heroes")]
    async fn search_heroes(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> std::result::Result<Json<HeroesOutput>, String> {
        let heroes = self
            .store()?
            .search_heroes(&params.query, params.limit(10))
            .map_err(|err| format!("{err:#}"))?;
        Ok(Json(HeroesOutput { heroes }))
    }

    /// Get bound hero basic info plus passive and active skills.
    #[tool(name = "wzry_get_hero_profile")]
    async fn get_hero_profile(
        &self,
        Parameters(params): Parameters<HeroParam>,
    ) -> std::result::Result<Json<HeroProfile>, String> {
        let profile = self
            .store()?
            .get_hero_profile(&params.hero)
            .map_err(|err| format!("{err:#}"))?;
        Ok(Json(profile))
    }

    /// Batch get complete hero profiles for lineup reasoning.
    #[tool(name = "wzry_get_hero_profiles")]
    async fn get_hero_profiles(
        &self,
        Parameters(params): Parameters<HeroesParam>,
    ) -> std::result::Result<Json<HeroProfilesOutput>, String> {
        if params.heroes.is_empty() {
            return Err("missing non-empty string array `heroes`".to_string());
        }
        let store = self.store()?;
        let profiles = params
            .heroes
            .iter()
            .map(|hero| {
                store
                    .get_hero_profile(hero)
                    .map_err(|err| format!("{err:#}"))
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(Json(HeroProfilesOutput { heroes: profiles }))
    }

    /// Get one hero skill; skill accepts passive/被动/1/2/3/大招 or exact skill name.
    #[tool(name = "wzry_get_hero_skill")]
    async fn get_hero_skill(
        &self,
        Parameters(params): Parameters<HeroSkillParam>,
    ) -> std::result::Result<Json<HeroSkill>, String> {
        let skill = self
            .store()?
            .get_hero_skill(&params.hero, &params.skill)
            .map_err(|err| format!("{err:#}"))?;
        Ok(Json(skill))
    }

    /// Search across hero skill names/descriptions and return hero + skill hits.
    #[tool(name = "wzry_search_hero_skills")]
    async fn search_hero_skills(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> std::result::Result<Json<HeroSkillSearchOutput>, String> {
        let hits = self
            .store()?
            .search_hero_skills(&params.query, params.limit(10))
            .map_err(|err| format!("{err:#}"))?;
        Ok(Json(HeroSkillSearchOutput { hits }))
    }

    /// List local items so agents can discover valid equipment names.
    #[tool(name = "wzry_list_items")]
    async fn list_items(
        &self,
        Parameters(params): Parameters<Limit500Params>,
    ) -> std::result::Result<Json<ItemsOutput>, String> {
        let mut items = self
            .store()?
            .all_items()
            .map_err(|err| format!("{err:#}"))?;
        items.truncate(params.limit(200));
        Ok(Json(ItemsOutput { items }))
    }

    /// Search local item data.
    #[tool(name = "wzry_search_items")]
    async fn search_items(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> std::result::Result<Json<ItemsOutput>, String> {
        let items = self
            .store()?
            .search_items(&params.query, params.limit(10))
            .map_err(|err| format!("{err:#}"))?;
        Ok(Json(ItemsOutput { items }))
    }

    /// Get one item by name.
    #[tool(name = "wzry_get_item")]
    async fn get_item(
        &self,
        Parameters(params): Parameters<ItemParam>,
    ) -> std::result::Result<Json<Item>, String> {
        let item = self
            .store()?
            .get_item(&params.item)
            .map_err(|err| format!("{err:#}"))?;
        Ok(Json(item))
    }

    /// List all summoner skills.
    #[tool(name = "wzry_get_summoner_skills")]
    async fn get_summoner_skills(&self) -> std::result::Result<Json<SummonerSkillsOutput>, String> {
        let skills = self
            .store()?
            .get_summoner_skills()
            .map_err(|err| format!("{err:#}"))?;
        Ok(Json(SummonerSkillsOutput {
            summoner_skills: skills,
        }))
    }

    /// Get one summoner skill by name.
    #[tool(name = "wzry_get_summoner_skill")]
    async fn get_summoner_skill(
        &self,
        Parameters(params): Parameters<SummonerSkillParam>,
    ) -> std::result::Result<Json<SummonerSkill>, String> {
        let skill = self
            .store()?
            .get_summoner_skill(&params.skill)
            .map_err(|err| format!("{err:#}"))?;
        Ok(Json(skill))
    }

    /// Return allies/enemies/candidate hero profiles for model-side lineup recommendation. MCP does not score or choose lineups.
    #[tool(name = "wzry_get_lineup_context")]
    async fn get_lineup_context(
        &self,
        Parameters(params): Parameters<LineupContextParams>,
    ) -> std::result::Result<Json<LineupContext>, String> {
        let store = self.store()?;
        let allies = params
            .allies
            .iter()
            .map(|hero| {
                store
                    .get_hero_profile(hero)
                    .map_err(|err| format!("{err:#}"))
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let enemies = params
            .enemies
            .iter()
            .map(|hero| {
                store
                    .get_hero_profile(hero)
                    .map_err(|err| format!("{err:#}"))
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let candidate_pool = params
            .candidate_pool
            .iter()
            .map(|hero| {
                store
                    .get_hero_profile(hero)
                    .map_err(|err| format!("{err:#}"))
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(Json(LineupContext {
            allies,
            enemies,
            candidate_pool,
            recommendation_should_be_done_by_model: true,
        }))
    }
}

pub fn serve_stdio(db_path: &str) -> Result<()> {
    let server = WzryMcpServer::new(db_path);
    Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async move {
            let service = server.serve(stdio()).await?;
            service.waiting().await?;
            Result::<()>::Ok(())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{HeroBasic, HeroSkill, Item, SourceInfo, SummonerSkill};
    use rmcp::handler::server::tool::IntoCallToolResult;
    use serde_json::Value;
    use tempfile::NamedTempFile;

    fn source(url: &str) -> SourceInfo {
        SourceInfo {
            url: url.to_string(),
            fetched_at: "2026-01-01T00:00:00Z".to_string(),
            content_hash: "hash".to_string(),
        }
    }

    fn fixture_db() -> (NamedTempFile, String) {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_string_lossy().to_string();
        let mut store = Store::open(&path).unwrap();
        let hero = HeroBasic {
            hero_id: 105,
            ename: 105,
            cname: "廉颇".to_string(),
            id_name: Some("lianpo".to_string()),
            title: Some("正义爆轰".to_string()),
            hero_type: Some(3),
            roles: vec!["坦克".to_string()],
            moss_id: Some(3627),
            source: source("https://pvp.qq.com/web201605/js/herolist.json"),
        };
        store.upsert_hero(&hero).unwrap();
        let skills = vec![
            HeroSkill {
                hero_id: 105,
                slot: "passive".to_string(),
                name: "勇士之魂".to_string(),
                cooldown: Some("0".to_string()),
                cost: Some("0".to_string()),
                description: "被动描述".to_string(),
                source: source("https://pvp.qq.com/web201605/herodetail/105.shtml"),
            },
            HeroSkill {
                hero_id: 105,
                slot: "skill_1".to_string(),
                name: "爆裂冲撞".to_string(),
                cooldown: Some("9".to_string()),
                cost: Some("0".to_string()),
                description: "一技能描述".to_string(),
                source: source("https://pvp.qq.com/web201605/herodetail/105.shtml"),
            },
        ];
        store.replace_hero_skills(105, &skills, &[]).unwrap();
        store
            .upsert_item(&Item {
                item_id: 1136,
                item_name: "破军".to_string(),
                item_type: Some(1),
                price: Some(1770),
                total_price: Some(2950),
                description_html: Some("<p>+180物理攻击</p>".to_string()),
                description_text: Some("+180物理攻击".to_string()),
                source: source("https://pvp.qq.com/web201605/js/item.json"),
            })
            .unwrap();
        store
            .upsert_summoner_skill(&SummonerSkill {
                skill_id: 80115,
                name: "闪现".to_string(),
                rank: Some(13),
                description: Some("向指定方向位移".to_string()),
                source: source("https://pvp.qq.com/web201605/js/summoner.json"),
            })
            .unwrap();
        drop(store);
        (file, path)
    }

    #[tokio::test]
    async fn tool_router_exposes_expected_tools_with_output_schema() {
        let server = WzryMcpServer::new("/tmp/not-needed.sqlite");
        let tools = server.tool_router.list_all();
        let names = tools
            .iter()
            .map(|tool| tool.name.to_string())
            .collect::<Vec<_>>();
        assert!(names.contains(&"wzry_list_heroes".to_string()));
        assert!(names.contains(&"wzry_search_hero_skills".to_string()));
        assert!(names.contains(&"wzry_list_items".to_string()));
        assert!(names.contains(&"wzry_get_lineup_context".to_string()));
        assert!(names.contains(&"wzry_get_hero_profile".to_string()));
        assert_eq!(names.len(), 12);
        assert!(tools.iter().all(|tool| tool.output_schema.is_some()));
    }

    #[tokio::test]
    async fn tool_methods_return_structured_content() {
        let (_file, path) = fixture_db();
        let server = WzryMcpServer::new(path);
        let result = server
            .get_hero_profile(Parameters(HeroParam {
                hero: "廉颇".to_string(),
            }))
            .await
            .unwrap()
            .into_call_tool_result()
            .unwrap();
        assert!(result.structured_content.is_some());
        let structured = result.structured_content.unwrap();
        assert_eq!(structured["hero"]["cname"], "廉颇");
        assert!(!result.content.is_empty());
        let text = result.content[0].as_text().unwrap();
        let text_value: Value = serde_json::from_str(&text.text).unwrap();
        assert_eq!(text_value["hero"]["cname"], "廉颇");
    }

    #[tokio::test]
    async fn discovery_tools_return_lists_and_skill_search_hits() {
        let (_file, path) = fixture_db();
        let server = WzryMcpServer::new(path);
        let heroes = server
            .list_heroes(Parameters(Limit500Params { limit: Some(5) }))
            .await
            .unwrap()
            .0
            .heroes;
        assert_eq!(heroes.len(), 1);
        assert_eq!(heroes[0].cname, "廉颇");

        let skills = server
            .search_hero_skills(Parameters(SearchParams {
                query: "冲撞".to_string(),
                limit: Some(5),
            }))
            .await
            .unwrap()
            .0
            .hits;
        assert_eq!(skills[0].hero.cname, "廉颇");
        assert_eq!(skills[0].skill.name, "爆裂冲撞");

        let items = server
            .list_items(Parameters(Limit500Params { limit: Some(5) }))
            .await
            .unwrap()
            .0
            .items;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item_name, "破军");
    }
}
