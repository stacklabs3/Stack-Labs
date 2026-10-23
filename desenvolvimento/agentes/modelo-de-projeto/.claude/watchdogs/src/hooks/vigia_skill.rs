// Watchdog PostToolUse (Skill): quando uma skill de grilling carrega, marca o grilling na sessão e
// injeta a regra de linguagem simples logo depois das instruções da skill.

use crate::comum::*;

pub fn rodar() -> i32 {
    let entrada_hook = ler_entrada_do_hook();
    if watchdogs_desligados() || nome_da_skill_de_grilling(texto(&entrada_hook["tool_input"]["skill"])).is_none() {
        return 0;
    }
    let diretorio_projeto = diretorio_do_projeto(&entrada_hook);
    let id_sessao = texto(&entrada_hook["session_id"]);
    let mut estado_sessao = ler_estado_sessao(&diretorio_projeto, id_sessao);
    estado_sessao["grillingAtivo"] = true.into();
    gravar_estado_sessao(&diretorio_projeto, id_sessao, &estado_sessao);
    emitir_contexto_adicional("PostToolUse", REGRA_LINGUAGEM_GRILLING);
    0
}
