// Watchdog UserPromptSubmit: mede a janela de contexto e cobra o /nova-sessao na smart zone; em
// sessão de grilling, relembra a linguagem simples a cada resposta do humano; com a issue ativa acima
// do orçamento, relembra o custo.

use crate::comum::*;
use crate::custo::{em_milhares, ler_custo_da_issue, orcamento_por_issue};
use regex::Regex;

pub fn rodar() -> i32 {
    let entrada_hook = ler_entrada_do_hook();
    if watchdogs_desligados() {
        return 0;
    }
    let mut avisos_do_prompt: Vec<String> = Vec::new();

    let tokens_de_contexto = medir_tokens_de_contexto(texto(&entrada_hook["transcript_path"]));
    let milhares_de_tokens = (tokens_de_contexto as f64 / 1000.0).round() as u64;
    let limite_em_milhares = LIMITE_DURO_CONTEXTO / 1000;
    if tokens_de_contexto >= LIMITE_DURO_CONTEXTO {
        avisos_do_prompt.push(format!("🛑 [watchdog vigia-prompt] Contexto em ~{milhares_de_tokens}k tokens: passou da smart zone ({limite_em_milhares}k). REGRA DURA: a próxima ação é /nova-sessao, e o trabalho continua numa janela nova com o kickoff. Edições no repositório estão bloqueadas. Avise o usuário."));
    } else if tokens_de_contexto >= LIMITE_AVISO_CONTEXTO {
        avisos_do_prompt.push(format!("⚠️ [watchdog vigia-prompt] Contexto em ~{milhares_de_tokens}k tokens, perto da smart zone ({limite_em_milhares}k). Feche a etapa atual e planeje o /nova-sessao. Não comece uma fase nova nesta sessão."));
    }

    // Grilling digitado como /grill-me, /wayfinder etc. não passa pelo Skill tool: marca aqui.
    let diretorio_projeto = diretorio_do_projeto(&entrada_hook);
    let id_sessao = texto(&entrada_hook["session_id"]);
    let mut estado_sessao = ler_estado_sessao(&diretorio_projeto, id_sessao);
    let grilling_ativo = estado_sessao["grillingAtivo"].as_bool().unwrap_or(false);
    let comando_digitado = Regex::new(r"^\s*(/\S+)")
        .expect("regex válida")
        .captures(texto(&entrada_hook["prompt"]))
        .map(|partes| partes[1].to_string());
    if !grilling_ativo && comando_digitado.is_some_and(|comando| nome_da_skill_de_grilling(&comando).is_some()) {
        estado_sessao["grillingAtivo"] = true.into();
        gravar_estado_sessao(&diretorio_projeto, id_sessao, &estado_sessao);
    }
    if let Some(numero_issue) = estado_sessao["issuesAtivas"]["principal"]["numero"].as_i64() {
        let tokens_da_issue = ler_custo_da_issue(&diretorio_projeto, numero_issue)["tokensEquivalentes"].as_u64().unwrap_or(0);
        let orcamento = orcamento_por_issue(&diretorio_projeto);
        if tokens_da_issue >= orcamento {
            avisos_do_prompt.push(format!("💸 [watchdog vigia-custo] A issue #{numero_issue} já custou {} tokens equivalentes de entrada (orçamento {}). Leia trechos, filtre saídas, delegue buscas a subagente com modelo barato e não abra frente nova; o humano decide se a issue segue ou se fatia.", em_milhares(tokens_da_issue), em_milhares(orcamento)));
        }
    }
    if estado_sessao["grillingAtivo"].as_bool().unwrap_or(false) {
        avisos_do_prompt.push(REGRA_LINGUAGEM_GRILLING.to_string());
    }

    if !avisos_do_prompt.is_empty() {
        emitir_contexto_adicional("UserPromptSubmit", &avisos_do_prompt.join("\n\n"));
    }
    0
}
