// Watchdog PreToolUse (Write | Edit | NotebookEdit).
// W1 protege os próprios watchdogs · W2 smart zone · W3 código só com issue ativa, fora da main ·
// W4 código só depois do contrato da fatia publicado na issue (regra 24).

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
    let ajustes = ler_ajustes_do_projeto(&diretorio_projeto);
    let tokens_de_contexto = medir_tokens_de_contexto(texto(&entrada_hook["transcript_path"]));
    if tokens_de_contexto >= ajustes.limite_duro_contexto {
        let milhares_de_tokens = (tokens_de_contexto as f64 / 1000.0).round() as u64;
        bloquear_ferramenta(NOME_WATCHDOG, &format!("contexto em ~{milhares_de_tokens}k tokens, acima da smart zone ({}k). Rode /nova-sessao (se precisar de documento, ele vai para o tmp do SO, fora do repositório) e continue numa janela nova com o kickoff.", ajustes.limite_duro_contexto / 1000));
    }

    // Na adoção do método, código sem issue ativa só passa a ser barrado na etapa "completo".
    if etapa_de_adocao(&diretorio_projeto) != "completo" || caminho_eh_livre_sem_issue(&caminho_relativo, &ajustes) {
        return 0;
    }
    let nome_branch = branch_atual(&diretorio_projeto);
    if branch_eh_descartavel(nome_branch.as_deref(), &ajustes) {
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

    // W4 — o contrato da fatia vem antes da primeira linha de código. Uma consulta ao GitHub por
    // sessão e por subagente: achado, fica no estado; o ciclo de correção o reencontra na issue.
    let id_sessao = texto(&entrada_hook["session_id"]);
    let chave_agente = chave_do_agente(&entrada_hook);
    let numero_issue = issue_ativa["numero"].as_i64().unwrap_or(0);
    let mut estado_sessao = ler_estado_sessao(&diretorio_projeto, id_sessao);
    if estado_sessao["contratos"][&chave_agente].as_i64() == Some(numero_issue) {
        return 0;
    }
    match buscar_corpos_dos_comentarios(&diretorio_projeto, numero_issue) {
        Ok(corpos) if corpos.iter().any(|corpo| corpo.contains(MARCADOR_CONTRATO)) => {
            estado_sessao["contratos"][&chave_agente] = serde_json::json!(numero_issue);
            gravar_estado_sessao(&diretorio_projeto, id_sessao, &estado_sessao);
        }
        Ok(_) => bloquear_ferramenta(
            NOME_WATCHDOG,
            &format!(
                "editar \"{caminho_relativo}\" é código, e a issue #{numero_issue} ainda não tem o contrato da fatia. Antes do código, publique o comentário \"{MARCADOR_CONTRATO}\" na issue (gh issue comment {numero_issue} --body-file <arquivo>), como manda docs/agents/contrato-da-fatia.md: o que o diff toca e o vizinho que imita, as seams e os riscos RK<n>, as regras de código que o alcançam, os termos do glossário, as ADRs e os critérios de aceite."
            ),
        ),
        Err(erro) => bloquear_ferramenta(
            NOME_WATCHDOG,
            &format!("não deu para ler os comentários da issue #{numero_issue} para achar o contrato da fatia ({erro}). Confira o gh auth status e a rede."),
        ),
    }
    0
}
