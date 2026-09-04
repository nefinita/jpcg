import type { CalculateRequest, FormData } from "../types";

// 数字兜底：空串/NaN → 0（前端表单可存 ""，提交 core 前统一归一）
export function toNum(v: unknown): number {
  const n = Number(v);
  return Number.isFinite(n) ? n : 0;
}

// 宽松表单（数字字段可为 ""）→ 具名 DTO（全数字），供 calculate/combo/derivatives 提交
export function toCalculateRequest(f: FormData): CalculateRequest {
  return {
    player: {
      jcsx: f.xinfa_config.xinfa_nom,
      jichu_shuxing: toNum(f.player.jichu_shuxing),
      jichu_gongji: toNum(f.player.jichu_gongji),
      huixin_dengji: toNum(f.player.huixin_dengji),
      huixin_xiaoguo: toNum(f.player.huixin_xiaoguo),
      pofang_dengji: toNum(f.player.pofang_dengji),
      wuqi_shanghai: toNum(f.player.wuqi_shanghai),
    },
    hostile: {
      waigong_fangyu: toNum(f.hostile.waigong_fangyu),
      neigong_fangyu: toNum(f.hostile.neigong_fangyu),
      yujin_dengji: toNum(f.hostile.yujin_dengji),
      huajin_dengji: toNum(f.hostile.huajin_dengji),
      jianshang_bili: toNum(f.hostile.jianshang_bili),
      target_hp: toNum(f.hostile.target_hp),
      max_hp: toNum(f.hostile.max_hp),
      current_hp: toNum(f.hostile.current_hp),
    },
    xinfa_config: {
      profession: f.xinfa_config.profession,
      xinfa_name: f.xinfa_config.xinfa_name,
      xinfa_nom: f.xinfa_config.xinfa_nom,
      atk_up: toNum(f.xinfa_config.atk_up),
      pofang_up: toNum(f.xinfa_config.pofang_up),
      huixin_up: toNum(f.xinfa_config.huixin_up),
    },
    buff: {
      base_atk_pct: toNum(f.buff.base_atk_pct),
      huixin_pct: toNum(f.buff.huixin_pct),
      huixiao_pct: toNum(f.buff.huixiao_pct),
      pofang_pct: toNum(f.buff.pofang_pct),
      wushi_fangyu_pct: toNum(f.buff.wushi_fangyu_pct),
      shanghai_pct: toNum(f.buff.shanghai_pct),
      mode_is_point: !!f.buff.mode_is_point,
    },
    coefficient: {
      pofang_xishu: toNum(f.coefficient.pofang_xishu),
      huixin_xishu: toNum(f.coefficient.huixin_xishu),
      huixiao_xishu: toNum(f.coefficient.huixiao_xishu),
      yujin_xishu: toNum(f.coefficient.yujin_xishu),
      yuhui_xishu: toNum(f.coefficient.yuhui_xishu),
      huajin_xishu: toNum(f.coefficient.huajin_xishu),
      fangyu_xishu: toNum(f.coefficient.fangyu_xishu),
      pvp_global_jianshang: toNum(f.coefficient.pvp_global_jianshang),
    },
  };
}