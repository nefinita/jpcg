export interface XinfaDraft {
  xinfa_name: string;
  xinfa_nom: string;
  atk_up: number;
  pofang_up: number;
  huixin_up: number;
}

export interface VersionDraft {
  level: number;
  season: number;
  modified: number;
}

export interface SkillDraft {
  row_id: string;
  skill_name: string;
  skill_id: number;
  sub_id: number;
  group: number;
  weapon_request: number;
  design_effect: number;
  kind_type: number;
  cast_mode: number;
  guaranteed_hit: boolean;
  has_critical_strike: boolean;
  effect_type: number;
  jihuoqixue: string;
  base_damage1: number;
  base_damage2: number;
  atk_xishu: number;
  watk_xishu: number;
  hit_up: number;
  huixin_up: number;
  huixiao_up: number;
  wushifangyu: number;
  wushihuajin: number;
  wushijianshang: number;
  zhenshishanghai: number;
  dot_flag: number;
  dot_up: number;
}

export interface AttributeDraft {
  xinfa: XinfaDraft;
  version?: VersionDraft;
  skills: SkillDraft[];
}

let nextSkillId = 5;

function nextRowId() {
  const id = `skill-${Date.now()}-${nextSkillId}`;
  nextSkillId += 1;
  return id;
}

const TEMPLATE_SKILL: Omit<SkillDraft, "row_id"> = {
  skill_name: "宫",
  skill_id: 10447,
  sub_id: 14474,
  group: 1,
  weapon_request: 0,
  design_effect: 0,
  kind_type: 0,
  cast_mode: 0,
  guaranteed_hit: false,
  has_critical_strike: true,
  effect_type: 0,
  jihuoqixue: "",
  base_damage1: 160,
  base_damage2: 200,
  atk_xishu: 501,
  watk_xishu: 0,
  hit_up: 0,
  huixin_up: 0,
  huixiao_up: 0,
  wushifangyu: 0,
  wushihuajin: 0,
  wushijianshang: 0,
  zhenshishanghai: 0,
  dot_flag: 0,
  dot_up: 0,
};

export function createSkill(
  overrides: Partial<Omit<SkillDraft, "row_id">> = {},
  rowId = nextRowId(),
): SkillDraft {
  return { ...TEMPLATE_SKILL, ...overrides, row_id: rowId };
}

export function createDefaultDraft(): AttributeDraft {
  return {
    xinfa: {
      xinfa_name: "莫问",
      xinfa_nom: "根骨",
      atk_up: 1.96,
      pofang_up: 2.0,
      huixin_up: 0,
    },
    skills: [
      createSkill({}, "skill-1"),
      createSkill(
        {
          skill_name: "商",
          skill_id: 10448,
          sub_id: 14475,
          base_damage1: 14,
          base_damage2: 19,
          atk_xishu: 2.4479166667,
        },
        "skill-2",
      ),
      createSkill(
        {
          skill_name: "商（dot）",
          skill_id: 10448,
          sub_id: 14476,
          design_effect: 1,
          base_damage1: 58,
          base_damage2: 58,
          atk_xishu: 0.2083333333,
          dot_flag: 1,
          dot_up: 0.2,
        },
        "skill-3",
      ),
      createSkill(
        {
          skill_name: "徵",
          skill_id: 10450,
          sub_id: 14480,
          jihuoqixue: "豪情",
          base_damage1: 190,
          base_damage2: 210,
          atk_xishu: 1.7760416667,
          hit_up: 20,
        },
        "skill-4",
      ),
    ],
  };
}

export function duplicateSkill(skill: SkillDraft): SkillDraft {
  const { row_id: _rowId, ...values } = skill;
  return createSkill({ ...values, skill_name: `${skill.skill_name} 副本` });
}
