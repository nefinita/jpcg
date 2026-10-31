use crate::model::*;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplatesFile {
    version: Version,
    templates: Vec<Template>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SchoolsFile {
    version: Version,
    schools: Vec<School>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetsFile {
    version: Version,
    sets: Vec<GearSet>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OverridesFile {
    version: Version,
    overrides: Vec<Override>,
}
fn read<T: serde::de::DeserializeOwned>(dir: &Path, file: &str) -> Result<T, String> {
    let path = dir.join(file);
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&raw).map_err(|e| format!("{}: {e}", path.display()))
}
impl Catalog {
    pub fn load(dir: &Path) -> Result<Self, String> {
        let templates: TemplatesFile = read(dir, "templates.toml")?;
        let schools: SchoolsFile = read(dir, "schools.toml")?;
        let sets: SetsFile = read(dir, "sets.toml")?;
        let overrides: OverridesFile = read(dir, "overrides.toml")?;
        for version in [&schools.version, &sets.version, &overrides.version] {
            if version != &templates.version {
                return Err("四个文件的 version 必须完全一致".into());
            }
        }
        let mut catalog = Self {
            version: templates.version,
            templates: templates.templates,
            schools: schools.schools,
            sets: sets.sets,
        };
        // 先校验原始唯一性，不能用 override 隐藏重复模板。
        catalog.validate()?;
        let mut seen = BTreeSet::new();
        for item in overrides.overrides {
            if !seen.insert((item.school.clone(), item.choice.clone())) {
                return Err("重复 override".into());
            }
            if item.green.is_none() && item.stats.is_none() {
                return Err("空 override".into());
            }
            let template = catalog
                .templates
                .iter_mut()
                .find(|t| t.school == item.school && t.choice == item.choice)
                .ok_or("override 没有对应模板")?;
            if let Some(green) = item.green {
                template.green = green;
            }
            if let Some(stats) = item.stats {
                template.stats = stats;
            }
        }
        catalog.validate()?;
        Ok(catalog)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.version.schema != 1
            || self.version.season == 0
            || self.version.revision.trim().is_empty()
            || self.version.value_set.trim().is_empty()
            || self.version.provenance.trim().is_empty()
        {
            return Err(
                "无效 version（schema=1，season/revision/value_set/provenance 必填）".into(),
            );
        }
        if self.schools.is_empty() || self.templates.is_empty() {
            return Err("门派与模板不能为空".into());
        }
        let mut schools = BTreeSet::new();
        for school in &self.schools {
            if school.id.trim().is_empty()
                || !school
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
                || (matches!(school.zhushuxing, MainStat::Gengu | MainStat::Yuanqi)
                    != (school.gongji_type == DamageType::Neigong))
                || !schools.insert(&school.id)
                || school.gongji_type != school.pofang_type
            {
                return Err("门派重复、空 id 或攻击/破防类型不一致".into());
            }
        }
        let mut sets = BTreeSet::new();
        let mut source_kinds = BTreeSet::new();
        for set in &self.sets {
            if set.id.trim().is_empty()
                || !sets.insert(&set.id)
                || set.source == Source::Baowu
                || (set.kind == SetKind::Accessory && set.source != Source::Weiwang)
                || !source_kinds.insert((set.source, set.kind))
            {
                return Err("套装 id/来源/类型重复或无效（同源只允许一组同类套装）".into());
            }
            let mut tiers = BTreeSet::new();
            for tier in &set.tiers {
                if !tiers.insert(tier.tier) {
                    return Err("重复套装档位".into());
                }
                match set.kind {
                    SetKind::Armor
                        if tier.bonus3.is_some()
                            || tier.bonus2 == Stats::default()
                            || tier.cooldown4.is_empty() =>
                    {
                        return Err("防具须有非零2件属性和4件CD，不能有3件套".into());
                    }
                    SetKind::Accessory => {
                        let only_huajin = Stats {
                            huajin: tier.bonus2.huajin,
                            ..Default::default()
                        };
                        let bonus3 = tier.bonus3.as_ref().ok_or("饰品缺少3件套")?;
                        let only_yujin = Stats {
                            yujin: bonus3.yujin,
                            ..Default::default()
                        };
                        if tier.bonus2.huajin == 0
                            || bonus3.yujin == 0
                            || tier.bonus2 != only_huajin
                            || bonus3 != &only_yujin
                            || !tier.cooldown4.is_empty()
                        {
                            return Err("饰品2件只能加化劲、3件只能加御劲，无4件CD".into());
                        }
                    }
                    _ => (),
                }
                let mut skills = BTreeSet::new();
                for cd in &tier.cooldown4 {
                    if cd.skill_id == 0
                        || !cd.seconds.is_finite()
                        || cd.seconds <= 0.0
                        || !skills.insert(cd.skill_id)
                    {
                        return Err("CD效果必须是唯一技能id与有限正秒数".into());
                    }
                }
            }
            if !tiers.contains(&Tier::Base) {
                return Err("套装缺少基础档".into());
            }
        }
        let mut keys = BTreeSet::new();
        let mut greens = BTreeMap::new();
        let mut variants: BTreeMap<_, BTreeSet<u8>> = BTreeMap::new();
        for t in &self.templates {
            let c = &t.choice;
            if !schools.contains(&t.school) || !keys.insert((&t.school, c)) {
                return Err("模板门派不存在或唯一键重复".into());
            }
            if c.source == Source::Baowu && (c.tier == Tier::Upgraded || t.set.is_some()) {
                return Err("宝物堆无升级/套装".into());
            }
            if c.slot.accessory() {
                if c.source != Source::Weiwang || !(1..=2).contains(&c.variant) {
                    return Err("饰品仅威望且属性套为1/2".into());
                }
                variants
                    .entry((&t.school, c.slot, c.tier))
                    .or_default()
                    .insert(c.variant);
            } else if c.variant != 0 {
                return Err("非饰品不能选属性套".into());
            }
            if c.source == Source::Mingjian && t.green == Green::Yujin {
                return Err("名剑来源不允许御劲绿字".into());
            }
            let green_points = match t.green {
                Green::Pofang => t.stats.pofang,
                Green::Huixin => t.stats.huixin,
                Green::Jiasu => t.stats.jiasu,
                Green::Yujin => t.stats.yujin,
            };
            if green_points == 0 {
                return Err("绿字方向必须有对应的显式正属性点数".into());
            }
            if (c.slot.armor() || c.slot.accessory())
                && c.source != Source::Baowu
                && t.set.is_none()
            {
                return Err("参与套装的来源/部位必须声明套装归属".into());
            }
            // 同键的基础/升级不能改变固定绿字方向。
            if let Some(prev) = greens.insert((&t.school, c.slot, c.source, c.variant), t.green)
                && prev != t.green
            {
                return Err("升级不能改变固定绿字方向".into());
            }
            if let Some(id) = &t.set {
                let set = self
                    .sets
                    .iter()
                    .find(|s| &s.id == id)
                    .ok_or("模板套装不存在")?;
                if set.source != c.source
                    || !(match set.kind {
                        SetKind::Armor => c.slot.armor(),
                        SetKind::Accessory => c.slot.accessory(),
                    })
                    || !set.tiers.iter().any(|x| x.tier == c.tier)
                {
                    return Err("套装来源/参与部位/档位不匹配（裤子不计套装）".into());
                }
            }
        }
        if variants.values().any(|v| v.len() != 2) {
            return Err("每个已提供的饰品部位/档位必须恰好两套属性".into());
        }
        for school in &self.schools {
            for slot in Slot::ALL {
                if !self
                    .templates
                    .iter()
                    .any(|t| t.school == school.id && t.choice.slot == slot)
                {
                    return Err(format!("门派 {} 缺少槽位 {slot:?}", school.id));
                }
            }
        }
        Ok(())
    }

    /// 每次公开合成入口均验证目录，防止 JSON API 绕过文件加载校验。
    pub fn compose(&self, loadout: &Loadout) -> Result<Composed, String> {
        self.validate()?;
        self.compose_validated(loadout)
    }
    pub(crate) fn compose_validated(&self, loadout: &Loadout) -> Result<Composed, String> {
        if loadout.revision != self.version.revision || loadout.value_set != self.version.value_set
        {
            return Err("配装的 revision/value_set 与目录不一致".into());
        }
        let profile = self
            .schools
            .iter()
            .find(|x| x.id == loadout.school)
            .ok_or("未知门派")?
            .clone();
        if loadout.choices.len() != 12 {
            return Err("必须选择完整12槽位".into());
        }
        let mut slots = BTreeSet::new();
        let mut stats = loadout.base.clone();
        let mut counts: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
        for choice in &loadout.choices {
            if !slots.insert(choice.slot) {
                return Err("槽位重复".into());
            }
            let template = self
                .templates
                .iter()
                .find(|t| t.school == loadout.school && &t.choice == choice)
                .ok_or_else(|| format!("模板不存在: {choice:?}"))?;
            stats.add(&template.stats)?;
            if let Some(id) = &template.set {
                let count = counts.entry(id).or_default();
                count.0 += 1;
                count.1 += usize::from(choice.tier == Tier::Upgraded);
            }
        }
        let mut active_bonuses = Vec::new();
        let mut cooldowns = Vec::new();
        for (id, (pieces, upgraded)) in counts {
            let set = self.sets.iter().find(|s| s.id == id).ok_or("套装不存在")?;
            // 混档约定：每个阈值独立判断升级件数。升级未达该阈值时使用基础奖励，不重复叠基础档。
            for threshold in [2, 3, 4] {
                if pieces < threshold
                    || (set.kind == SetKind::Armor && threshold == 3)
                    || (set.kind == SetKind::Accessory && threshold == 4)
                {
                    continue;
                }
                let tier = if upgraded >= threshold {
                    Tier::Upgraded
                } else {
                    Tier::Base
                };
                let effect = set
                    .tiers
                    .iter()
                    .find(|t| t.tier == tier)
                    .ok_or("套装缺少所需档位")?;
                match threshold {
                    2 => stats.add(&effect.bonus2)?,
                    3 => stats.add(effect.bonus3.as_ref().ok_or("饰品缺3件奖励")?)?,
                    4 => cooldowns.extend(effect.cooldown4.clone()),
                    _ => (),
                }
                active_bonuses.push(ActiveBonus {
                    set: id.into(),
                    pieces,
                    threshold,
                    tier,
                });
            }
        }
        let mut warnings = Vec::new();
        if self.version.sample {
            warnings.push("仅合成样例数据，不代表游戏装备或推荐配装".into());
        }
        if !cooldowns.is_empty() {
            warnings.push("4件套CD仅返回元数据，当前计算引擎未应用".into());
        }
        let panel = match loadout.attribute_rules.as_deref() {
            Some(rule_id) => {
                let panel =
                    crate::attributes::convert(&stats, &profile, &loadout.value_set, rule_id)?;
                warnings.push("属性换算使用 2026-08-14 的130级莫问来源快照，尚未经真实游戏面板验证；不含战斗体质/气血增益、强化、镶嵌、附魔及装备特效规则".into());
                Some(panel)
            }
            None => {
                crate::attributes::validate_legacy(&stats)?;
                warnings
                    .push("旧属性模式未换算体质及基础属性会心；可显式选择有来源的历史规则".into());
                None
            }
        };
        Ok(Composed {
            loadout: loadout.clone(),
            profile,
            stats,
            panel,
            active_bonuses,
            cooldowns,
            warnings,
        })
    }
}
