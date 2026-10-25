// Watchdog Stop e SubagentStop: soma o uso de tokens do turno na issue ativa e, quando ela passa do
// orçamento, avisa o humano. Não bloqueia: quem decide continuar, fatiar ou trocar de modelo é ele.

use crate::comum::*;
use crate::custo::{contar_no_transcript, em_milhares, ResultadoDaContagem};
use serde_json::json;

pub fn rodar() -> i32 {
    let entrada_hook = ler_entrada_do_hook();
    if watchdogs_desligados() {
        return 0;
    }
    // No SubagentStop, o uso do subagente está no transcript dele, não no da sessão.
    let transcript_do_subagente = texto(&entrada_hook["agent_transcript_path"]);
    let caminho_transcript = if transcript_do_subagente.is_empty() { texto(&entrada_hook["transcript_path"]) } else { transcript_do_subagente };
    if let ResultadoDaContagem::PassouDoOrcamentoAgora { numero_issue, tokens_equivalentes, orcamento } =
        contar_no_transcript(&entrada_hook, caminho_transcript)
    {
        let aviso = format!("💸 [watchdog vigia-custo] A issue #{numero_issue} passou do orçamento: {} tokens equivalentes de entrada (orçamento {}). Veja o detalhe com `sh .claude/watchdogs/rodar custo {numero_issue}` e decida: seguir, fatiar a issue ou trocar de modelo.", em_milhares(tokens_equivalentes), em_milhares(orcamento));
        print!("{}", json!({ "systemMessage": aviso }));
    }
    0
}
