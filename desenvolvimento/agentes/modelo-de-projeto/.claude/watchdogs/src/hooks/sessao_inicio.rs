// Watchdog SessionStart: verifica o setup e reancora a constituição no início de toda sessão
// (inclusive depois de /clear e /compact, quando as regras somem da memória de trabalho).

use crate::comum::*;

const ARQUIVOS_DO_SETUP: [&str; 13] = [
    "AGENTS.md",
    "docs/agents/issue-tracker.md",
    "docs/agents/triage-labels.md",
    "docs/agents/domain.md",
    "docs/agents/mapas-iniciais.md",
    "docs/agents/pesquisa-antes-de-recomendar.md",
    "docs/agents/roteiro-de-prototipo.md",
    "docs/agents/caminho-ate-ready-for-agent.md",
    "docs/agents/orquestracao.md",
    "docs/agents/watchdogs.md",
    "docs/agents/adocao.md",
    "docs/agents/principios-de-codigo.md",
    "docs/agents/validador.md",
];

pub fn rodar() -> i32 {
    let entrada_hook = ler_entrada_do_hook();
    let diretorio_projeto = diretorio_do_projeto(&entrada_hook);
    if watchdogs_desligados() {
        emitir_contexto_adicional(
            "SessionStart",
            "⚠️ Watchdogs DESLIGADOS (STACKLABS_WATCHDOGS=desligado). As regras de AGENTS.md continuam valendo; só não há bloqueio automático.",
        );
        return 0;
    }

    let arquivos_faltando: Vec<&str> = ARQUIVOS_DO_SETUP
        .iter()
        .copied()
        .filter(|arquivo| !diretorio_projeto.join(arquivo).exists())
        .collect();

    let mapas_abertos = match executar_gh(
        &diretorio_projeto,
        &[
            "issue",
            "list",
            "--label",
            "wayfinder:map",
            "--state",
            "open",
            "--json",
            "number,title",
            "--jq",
            r#".[] | "- \(.title) (#\(.number))""#,
        ],
    ) {
        Ok(lista) if lista.is_empty() => "nenhum".to_string(),
        Ok(lista) => lista,
        Err(_) => "falha ao consultar o GitHub (gh auth/rede)".to_string(),
    };

    let situacao_dos_mapas_iniciais = match buscar_mapas_iniciais(&diretorio_projeto) {
        Err(_) => "falha ao consultar o GitHub (gh auth/rede)".to_string(),
        Ok(mapas) => mapas
            .iter()
            .map(|mapa| match mapa.numero {
                None => format!("- ❌ {}: NÃO EXISTE. Abra com /wayfinder (docs/agents/mapas-iniciais.md).", mapa.tipo),
                Some(numero) => {
                    let precisa_fechar = MAPAS_INICIAIS_FECHADOS_PARA_IMPLEMENTAR.contains(&mapa.tipo) && mapa.estado != "closed";
                    let situacao = if mapa.estado == "closed" { "fechado" } else { "aberto" };
                    format!(
                        "- {} {}: \"{}\" (#{numero}, {situacao})",
                        if precisa_fechar { "⏳" } else { "✅" },
                        mapa.tipo,
                        mapa.titulo
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
    };

    let estado_sessao = ler_estado_sessao(&diretorio_projeto, texto(&entrada_hook["session_id"]));
    let issue_principal = &estado_sessao["issuesAtivas"]["principal"];
    let metodo = ler_metodo_do_projeto(&diretorio_projeto);

    let mut linhas_contexto = vec![
        "## Watchdogs do método StackLabs (wayfinder / Matt Pocock) ativos".to_string(),
        "AGENTS.md é a constituição deste repositório: regras DURAS, não sugestões. Releia antes de agir.".to_string(),
        if arquivos_faltando.is_empty() {
            "✅ Setup do issue tracker (GitHub) presente.".to_string()
        } else {
            format!(
                "❌ Setup incompleto, faltam: {}. Rode /setup-stacklabs antes de qualquer fluxo.",
                arquivos_faltando.join(", ")
            )
        },
        if caminho_hooks_do_git(&diretorio_projeto).as_deref() == Some(CAMINHO_GITHOOKS) {
            "✅ pre-push da main ativo.".to_string()
        } else {
            format!("❌ pre-push da main INATIVO neste clone. Peça ao usuário para rodar: git config core.hooksPath {CAMINHO_GITHOOKS}")
        },
        format!("Branch atual: {}", branch_atual(&diretorio_projeto).unwrap_or_else(|| "(desconhecida)".into())),
    ];
    if metodo.etapa_de_adocao != "completo" {
        linhas_contexto.push(format!(
            "🔄 Adoção do método em andamento (desde {}): etapa \"{}\". O que ainda não bloqueia também vale; o próximo passo está em /adotar-metodo-stack-labs (docs/agents/adocao.md).",
            metodo.adotado_em.as_deref().unwrap_or("null"),
            metodo.etapa_de_adocao
        ));
    }
    linhas_contexto.push(format!(
        "Issue ativa nesta sessão: {}",
        if issue_principal.is_object() {
            format!("#{} {}", issue_principal["numero"], texto(&issue_principal["titulo"]))
        } else {
            "nenhuma (editar código de produção está bloqueado)".to_string()
        }
    ));
    linhas_contexto.push(format!(
        "Mapas iniciais (ativar issue exige os cinco e {} fechados):\n{situacao_dos_mapas_iniciais}",
        MAPAS_INICIAIS_FECHADOS_PARA_IMPLEMENTAR.join(" + ")
    ));
    linhas_contexto.push(format!("Mapas do wayfinder abertos:\n{mapas_abertos}"));
    linhas_contexto.push("Grilling em todo ticket de decisão: o humano responde. Pergunta curta, em linguagem simples, com um exemplo real deste projeto. Nenhuma recomendação sem pesquisa: cite docs/research/ ou a fonte primária; sem fonte, rode /research antes.".to_string());
    linhas_contexto.push("Rota: ideia grande → /wayfinder (1 ticket por sessão) → /to-spec → /to-tickets → /orquestrar-issues (implementa, revisa e confere, uma issue por vez). Trabalho novo, fase nova ou janela cheia → /nova-sessao (janela nova). Assunto paralelo → sessão satélite e /voltar-a-principal.".to_string());
    emitir_contexto_adicional("SessionStart", &linhas_contexto.join("\n"));
    0
}
