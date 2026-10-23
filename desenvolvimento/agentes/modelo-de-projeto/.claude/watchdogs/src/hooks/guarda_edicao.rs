// Watchdog PreToolUse (Write | Edit | NotebookEdit).
// W1 protege os próprios watchdogs · W2 smart zone · W3 código só com issue ativa, fora da main.

use crate::comum::*;

const NOME_WATCHDOG: &str = "guarda-edicao";

pub fn rodar() -> i32 {
    let entrada_hook = ler_entrada_do_hook();
    if watchdogs_desligados() {
        return 0;
    }
    let diretorio_projeto = diretorio_do_projeto(&entrada_hook);
    let parametros_ferramenta = &entrada_hook["tool_input"];
    let caminho_arquivo = parametros_ferramenta["file_path"]
        .as_str()
        .or_else(|| parametros_ferramenta["notebook_path"].as_str())
        .unwrap_or("");
    if caminho_arquivo.is_empty() {
        return 0;
    }
    let Some(caminho_relativo) = caminho_relativo_ao_projeto(&diretorio_projeto, caminho_arquivo) else {
        return 0; // fora do repo (ex.: documento de handoff no tmp)
    };

    // W1 — os watchdogs não podem ser desligados pelo agente que eles vigiam.
    if caminho_eh_protegido(&caminho_relativo) {
        bloquear_ferramenta(
            NOME_WATCHDOG,
            &format!("\"{caminho_relativo}\" é infraestrutura dos watchdogs. Só um humano altera esse caminho. Peça ao usuário."),
        );
    }

    // W2 — passou da smart zone: nada de seguir editando com o raciocínio degradado.
    let tokens_de_contexto = medir_tokens_de_contexto(texto(&entrada_hook["transcript_path"]));
    if tokens_de_contexto >= LIMITE_DURO_CONTEXTO {
        let milhares_de_tokens = (tokens_de_contexto as f64 / 1000.0).round() as u64;
        bloquear_ferramenta(NOME_WATCHDOG, &format!("contexto em ~{milhares_de_tokens}k tokens, acima da smart zone ({}k). Rode /nova-sessao (se precisar de documento, ele vai para o tmp do SO, fora do repositório) e continue numa janela nova com o kickoff.", LIMITE_DURO_CONTEXTO / 1000));
    }

    // Na adoção do método, código sem issue ativa só passa a ser barrado na etapa "completo".
    if etapa_de_adocao(&diretorio_projeto) != "completo" || caminho_eh_livre_sem_issue(&caminho_relativo) {
        return 0;
    }
    let nome_branch = branch_atual(&diretorio_projeto);
    if branch_eh_descartavel(nome_branch.as_deref()) {
        return 0; // prototype/* e research/* são descartáveis
    }

    // W3 — código de produção nasce de uma issue (to-tickets / triage) numa sessão limpa.
    let estado_sessao = ler_estado_sessao(&diretorio_projeto, texto(&entrada_hook["session_id"]));
    let issue_ativa = &estado_sessao["issuesAtivas"][chave_do_agente(&entrada_hook)];
    if !issue_ativa.is_object() {
        bloquear_ferramenta(
            NOME_WATCHDOG,
            &[
                format!("editar \"{caminho_relativo}\" é código de produção e exige uma issue ativa nesta sessão."),
                format!("Ative com: {COMANDO_ATIVAR_ISSUE} <número>  (issue aberta com label ready-for-agent ou ready-for-human)."),
                "Ainda não há issue? O caminho é /wayfinder → /to-spec → /to-tickets (ou /triage para pedidos externos).".to_string(),
            ]
            .join("\n"),
        );
    }
    if branch_eh_principal(nome_branch.as_deref()) && repositorio_tem_commits(&diretorio_projeto) {
        let numero_issue = &issue_ativa["numero"];
        bloquear_ferramenta(
            NOME_WATCHDOG,
            &format!(
                "issue #{numero_issue} ativa, mas a branch é \"{}\". Crie a branch da issue antes: git switch -c issue/{numero_issue}-<slug>",
                nome_branch.unwrap_or_default()
            ),
        );
    }
    0
}
