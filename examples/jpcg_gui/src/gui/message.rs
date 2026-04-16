use crate::gui::xinfa_table::Xinfa;

#[derive(Debug, Clone)]
pub enum Message {
    XinfaSelected(Xinfa),
    JcsxChanged(String),
    JcgjChanged(String),
    HxdjChanged(String),
    HxxgChanged(String),
    PfdjChanged(String),
    WqshChanged(String),
    WgfyChanged(String),
    NgfyChanged(String),
    YjdjChanged(String),
    HjdjChanged(String),
    JsblChanged(String),
    Calculator,
    SaveConfig,
    LoadConfig,
    ClearConfig,
}
