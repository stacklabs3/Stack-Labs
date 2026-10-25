// Watchdog PostToolUse: saída grande de ferramenta entra no contexto e é relida em todo turno
// seguinte. Passou do limite, lembra o agente de filtrar da próxima vez. Não bloqueia.

use crate::comum::*;

// Cerca de 5k tokens.
pub const LIMITE_SAIDA_EM_CARACTERES: usize = 20_000;

pub fn rodar() -> i32 {
    let entrada_hook = ler_entrada_do_hook();
    if watchdogs_desligados() {
        return 0;
    }
    let tamanho_saida = match &entrada_hook["tool_response"] {
        serde_json::Value::String(texto_saida) => texto_saida.len(),
        resposta_ferramenta => resposta_ferramenta.to_string().len(),
    };
    if tamanho_saida < LIMITE_SAIDA_EM_CARACTERES {
        return 0;
    }
    let nome_ferramenta = texto(&entrada_hook["tool_name"]);
    let como_filtrar = match nome_ferramenta {
        "Read" => "leia só o trecho que precisa, com offset e limit, depois de achar a linha com Grep",
        "Bash" => "filtre a saída com grep, tail ou head, e nos testes peça só as falhas",
        "Grep" | "Glob" => "estreite o padrão ou o caminho, ou use o modo que só lista os arquivos",
        _ => "peça só o que precisa, ou delegue a leitura ampla a um subagente que devolve o resumo",
    };
    emitir_contexto_adicional(
        "PostToolUse",
        &format!("📏 [watchdog vigia-saida] A saída do {nome_ferramenta} teve ~{}k caracteres e agora é relida em todo turno desta sessão. Da próxima vez, {como_filtrar}.", tamanho_saida / 1000),
    );
    0
}
