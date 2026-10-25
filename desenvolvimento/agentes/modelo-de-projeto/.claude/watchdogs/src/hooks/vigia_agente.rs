// Watchdog PreToolUse (Agent): subagente de busca não precisa do modelo da sessão. Lançado sem
// `model`, lembra o agente de passar um modelo mais barato. Não bloqueia.

use crate::comum::*;

// Tipos de subagente que só leem e resumem.
pub const SUBAGENTES_DE_BUSCA: [&str; 2] = ["Explore", "claude-code-guide"];

pub fn rodar() -> i32 {
    let entrada_hook = ler_entrada_do_hook();
    if watchdogs_desligados() {
        return 0;
    }
    let tipo_subagente = texto(&entrada_hook["tool_input"]["subagent_type"]);
    let modelo_pedido = texto(&entrada_hook["tool_input"]["model"]);
    if !SUBAGENTES_DE_BUSCA.contains(&tipo_subagente) || !modelo_pedido.is_empty() {
        return 0;
    }
    emitir_contexto_adicional(
        "PreToolUse",
        &format!("🪙 [watchdog vigia-agente] O subagente {tipo_subagente} saiu sem `model` e roda no modelo da sessão. Busca e leitura cabem no haiku (ou no sonnet, se a busca exige julgamento): passe `model` nos próximos."),
    );
    0
}
