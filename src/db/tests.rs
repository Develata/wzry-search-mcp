use super::*;
use crate::model::*;
use tempfile::NamedTempFile;

fn source() -> SourceInfo {
    SourceInfo {
        url: "u".to_string(),
        fetched_at: "t".to_string(),
        content_hash: "h".to_string(),
    }
}

fn hero(id: i64, name: &str) -> HeroBasic {
    HeroBasic {
        hero_id: id,
        ename: id,
        cname: name.to_string(),
        id_name: None,
        title: None,
        hero_type: None,
        roles: vec![],
        moss_id: None,
        source: source(),
    }
}

#[test]
fn retain_heroes_cascades_skills_and_warnings() {
    let file = NamedTempFile::new().unwrap();
    let mut store = Store::open(file.path()).unwrap();
    store.upsert_hero(&hero(1, "保留")).unwrap();
    store.upsert_hero(&hero(2, "删除")).unwrap();
    store
        .replace_hero_skills(
            2,
            &[HeroSkill {
                hero_id: 2,
                slot: "passive".to_string(),
                name: "旧技能".to_string(),
                cooldown: None,
                cost: None,
                description: "旧描述".to_string(),
                source: source(),
            }],
            &["旧 warning".to_string()],
        )
        .unwrap();
    let deleted = store.retain_heroes_by_ids(&[1]).unwrap();
    assert_eq!(deleted, 1);
    assert!(store.resolve_hero("删除").is_err());
    assert_eq!(store.get_hero_profile("保留").unwrap().skills.len(), 0);
}

#[test]
fn replace_hero_skills_removes_stale_slots() {
    let file = NamedTempFile::new().unwrap();
    let mut store = Store::open(file.path()).unwrap();
    store.upsert_hero(&hero(1, "英雄")).unwrap();
    store
        .replace_hero_skills(
            1,
            &[
                HeroSkill {
                    hero_id: 1,
                    slot: "passive".to_string(),
                    name: "被动".to_string(),
                    cooldown: None,
                    cost: None,
                    description: "被动描述".to_string(),
                    source: source(),
                },
                HeroSkill {
                    hero_id: 1,
                    slot: "extra_4".to_string(),
                    name: "旧额外".to_string(),
                    cooldown: None,
                    cost: None,
                    description: "旧额外描述".to_string(),
                    source: source(),
                },
            ],
            &["旧 warning".to_string()],
        )
        .unwrap();
    store
        .replace_hero_skills(
            1,
            &[HeroSkill {
                hero_id: 1,
                slot: "passive".to_string(),
                name: "新被动".to_string(),
                cooldown: None,
                cost: None,
                description: "新被动描述".to_string(),
                source: source(),
            }],
            &[],
        )
        .unwrap();
    let profile = store.get_hero_profile("英雄").unwrap();
    assert_eq!(profile.skills.len(), 1);
    assert_eq!(profile.skills[0].name, "新被动");
    assert!(profile.parse_warnings.is_empty());
}

#[test]
fn replace_hero_skills_rejects_mismatched_skill_hero_id() {
    let file = NamedTempFile::new().unwrap();
    let mut store = Store::open(file.path()).unwrap();
    store.upsert_hero(&hero(1, "英雄")).unwrap();
    let err = store
        .replace_hero_skills(
            1,
            &[HeroSkill {
                hero_id: 2,
                slot: "passive".to_string(),
                name: "错配技能".to_string(),
                cooldown: None,
                cost: None,
                description: "错配描述".to_string(),
                source: source(),
            }],
            &[],
        )
        .unwrap_err();
    assert!(err.to_string().contains("expected 1"));
    assert!(store.get_hero_profile("英雄").unwrap().skills.is_empty());
}

#[test]
fn invalid_roles_json_surfaces_query_error() {
    let file = NamedTempFile::new().unwrap();
    let store = Store::open(file.path()).unwrap();
    store
        .conn
        .execute(
            r#"INSERT INTO heroes
            (hero_id, ename, cname, roles_json, source_url, fetched_at, content_hash)
            VALUES (1, 1, '坏英雄', 'not-json', 'u', 't', 'h')"#,
            [],
        )
        .unwrap();
    assert!(store.resolve_hero("坏英雄").is_err());
}
