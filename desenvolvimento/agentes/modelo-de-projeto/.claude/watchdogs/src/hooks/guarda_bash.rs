// Watchdog PreToolUse (Bash).
// W1 protege os watchdogs · W4 ativação de issue (com os mapas iniciais) · W5/W6 regras do /wayfinder
// · W7 nada de commit na main · W9 resolução cita as fontes (pesquisa antes de recomendar)
// · W10 registra `rodar contexto` e barra cat, sed e grep em código antes dele (ADR 0022).

use super::guarda_contexto::{agente_ja_consultou, caminho_exige_consulta, motivo_sem_consulta, registrar_consulta};
use crate::comum::*;
use regex::Regex;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const NOME_WATCHDOG: &str = "guarda-bash";

fn regex(padrao: &str) -> Regex {
    Regex::new(padrao).expect("regex válida")
}

fn bloquear(motivo: &str) -> ! {
    bloquear_ferramenta(NOME_WATCHDOG, motivo)
}

fn primeira_linha(mensagem: &str) -> &str {
    mensagem.lines().next().unwrap_or("")
}

fn buscar_issue_ou_bloquear(diretorio_projeto: &Path, numero_issue: i64) -> Issue {
    buscar_issue(diretorio_projeto, numero_issue).unwrap_or_else(|erro| {
        bloquear(&format!(
            "não consegui consultar a issue #{numero_issue} no GitHub ({}). Os watchdogs falham fechados: verifique o gh auth/rede.",
            primeira_linha(&erro)
        ))
    })
}

// Números das issues logo depois de um subcomando do gh (aceita "42", "#42" e URL da issue).
fn numeros_apos_subcomando(comando: &str, padrao_subcomando: &Regex) -> Vec<i64> {
    let numero_no_final = regex(r#"(\d+)["']?$"#);
    padrao_subcomando
        .captures_iter(comando)
        .filter_map(|partes| numero_no_final.captures(&partes[1]).and_then(|numero| numero[1].parse().ok()))
        .collect()
}

fn comando_reivindica_no_mesmo_passo(comando: &str, numero_ticket: i64) -> bool {
    regex(&format!(r"gh\s+issue\s+edit\s+#?{numero_ticket}\b[^;&|\n]*--add-assignee")).is_match(comando)
}

fn caminho_entre_aspas(partes: &regex::Captures) -> String {
    partes
        .get(2)
        .or_else(|| partes.get(3))
        .or_else(|| partes.get(4))
        .map(|parte| parte.as_str().to_string())
        .unwrap_or_default()
}

// A consulta de um trecho que chama o `rodar contexto`, sem aspas, sem --limite e sem o resto do cano.
fn consulta_do_trecho(trecho: &str) -> Option<String> {
    let partes = regex(r#"^\s*(?:sh\s+)?(?:"[^"]*|\S*)\.claude/watchdogs/rodar"?\s+contexto\s+([^|]+)"#).captures(trecho)?;
    let mut palavras = Vec::new();
    let mut proxima_eh_limite = false;
    for palavra in partes[1].split_whitespace() {
        if palavra.contains('>') {
            break;
        }
        if proxima_eh_limite {
            proxima_eh_limite = false;
        } else if palavra == "--limite" {
            proxima_eh_limite = true;
        } else {
            palavras.push(palavra.trim_matches(['"', '\'']));
        }
    }
    let consulta = palavras.join(" ");
    (!consulta.is_empty()).then_some(consulta)
}

// O primeiro caminho que exige consulta entre os que um cat, sed ou grep do trecho lê. Só conta palavra
// que existe no disco: o padrão do grep e o script do sed não existem, e `git log | grep x` lê do cano.
fn codigo_lido_no_trecho(trecho: &str, pasta_base: &Path, diretorio_projeto: &Path, ajustes: &AjustesDoProjeto) -> Option<String> {
    let verbo_de_leitura = regex(r"^(cat|head|tail|less|sed|e?grep|rg)$");
    for (posicao_no_cano, comando_do_cano) in trecho.split('|').enumerate() {
        let mut palavras = comando_do_cano.split_whitespace();
        let Some(verbo) = palavras.next().filter(|verbo| verbo_de_leitura.is_match(verbo)) else {
            continue;
        };
        let argumentos: Vec<&str> = palavras.take_while(|palavra| !palavra.contains('>') && !palavra.starts_with("<<")).collect();
        let eh_recursivo = verbo == "rg"
            || argumentos
                .iter()
                .any(|argumento| regex(r"^(-[a-zA-Z]*[rR][a-zA-Z]*|--recursive)$").is_match(argumento));
        let mut caminhos_lidos: Vec<PathBuf> = argumentos
            .iter()
            .filter(|argumento| !argumento.starts_with('-'))
            .map(|argumento| argumento.trim_matches(['"', '\'']).split('*').next().unwrap_or(""))
            .filter(|caminho| !caminho.is_empty())
            .map(|caminho| pasta_base.join(caminho))
            .filter(|caminho| caminho.exists())
            .collect();
        if caminhos_lidos.is_empty() && eh_recursivo && posicao_no_cano == 0 {
            caminhos_lidos.push(pasta_base.to_path_buf());
        }
        for caminho_lido in caminhos_lidos {
            let Some(caminho_relativo) = caminho_relativo_ao_projeto(diretorio_projeto, &caminho_lido.to_string_lossy()) else {
                continue;
            };
            let caminho_com_barra = if caminho_lido.is_dir() && !caminho_relativo.is_empty() {
                format!("{caminho_relativo}/")
            } else {
                caminho_relativo
            };
            if caminho_exige_consulta(&caminho_com_barra, ajustes) {
                return Some(caminho_com_barra);
            }
        }
    }
    None
}

pub fn rodar() -> i32 {
    let entrada_hook = ler_entrada_do_hook();
    if watchdogs_desligados() {
        return 0;
    }
    let diretorio_projeto = diretorio_do_projeto(&entrada_hook);
    let id_sessao = texto(&entrada_hook["session_id"]).to_string();
    let comando = texto(&entrada_hook["tool_input"]["command"]).to_string();
    let separador_de_trechos = regex(r"&&|\|\||;|\n");
    let trechos_do_comando: Vec<&str> = separador_de_trechos.split(&comando).collect();
    let pasta_de_trabalho: PathBuf = match entrada_hook["cwd"].as_str() {
        Some(cwd) if !cwd.is_empty() => PathBuf::from(cwd),
        _ => diretorio_projeto.clone(),
    };

    // ---------- W1: os watchdogs não se desligam por Bash ----------
    let sem_redirecionamento_inofensivo = regex(r"\d*>\s*/dev/null")
        .replace_all(&regex(r"\d*>&\d+").replace_all(&comando, ""), "")
        .to_string();
    let caminho_protegido = regex(r"\.claude/(watchdogs|githooks|agents|metodo\.json|settings\.json|state)|\.github/workflows/vigia-do-metodo\.yml");
    let verbo_mutante =
        regex(r"(>|\brm\b|\bmv\b|\bcp\b|sed\s+-i|perl\s+-[a-z]*i|\btee\b|\bchmod\b|\btruncate\b|\bln\b|\bunlink\b|git\s+(checkout|restore|rm|stash)\b)");
    if separador_de_trechos
        .split(&sem_redirecionamento_inofensivo)
        .any(|trecho| caminho_protegido.is_match(trecho) && verbo_mutante.is_match(trecho))
    {
        bloquear("o comando altera .claude/watchdogs, .claude/githooks, .claude/agents, .claude/metodo.json, .claude/settings.json, .claude/state ou o vigia-do-metodo do CI. Só um humano altera a infraestrutura dos watchdogs.");
    }

    // ---------- W8: o pre-push que protege a main não se desliga ----------
    if regex(r"\bgit\b[^;&|\n]*--no-verify\b").is_match(&comando) {
        bloquear("--no-verify pula os hooks do git (inclusive o pre-push que protege a main). Proibido para o agente.");
    }
    if regex(r"(?i)core\.hooksPath").is_match(&comando) && !regex(r"(?i)git\s+config\s+(--get\s+)?core\.hooksPath\s*($|[;&|\n])").is_match(&comando) {
        bloquear("alterar core.hooksPath desliga o pre-push que protege a main. Só um humano altera.");
    }
    if regex(r"--atualizar-watchdogs\b").is_match(&comando) {
        bloquear("--atualizar-watchdogs sobrescreve os watchdogs do projeto. Só um humano roda o setup-stacklabs com essa opção.");
    }
    if comando.contains("STACKLABS_PERMITIR_PUSH_MAIN") {
        bloquear("STACKLABS_PERMITIR_PUSH_MAIN é o escape humano do pre-push. O agente não usa.");
    }
    if regex(r"\bgit\s+push\b[^;&|\n]*(--force\b|--force-with-lease\b|\s-f\b)").is_match(&comando) {
        bloquear("push forçado é proibido para o agente. Peça ao usuário.");
    }

    // Em observação (adoção do método num projeto existente), daqui para baixo só se avisa: nada bloqueia.
    if etapa_de_adocao(&diretorio_projeto) == "observacao" {
        return 0;
    }

    // ---------- W10: consulta de contexto antes de ler código ----------
    // Cada trecho vale na ordem: `rodar contexto x && cat x` passa, `cat x && rodar contexto x` não.
    let chave_do_leitor = chave_do_agente(&entrada_hook);
    let ajustes_do_projeto = ler_ajustes_do_projeto(&diretorio_projeto);
    let cd_inicial = regex(r#"^\s*cd\s+("([^"]+)"|'([^']+)'|(\S+))\s*$"#);
    let mut pasta_do_trecho = pasta_de_trabalho.clone();
    for trecho in &trechos_do_comando {
        if let Some(partes) = cd_inicial.captures(trecho) {
            pasta_do_trecho = pasta_do_trecho.join(caminho_entre_aspas(&partes));
        } else if let Some(consulta) = consulta_do_trecho(trecho) {
            registrar_consulta(&diretorio_projeto, &id_sessao, &chave_do_leitor, &consulta);
        } else if !agente_ja_consultou(&ler_estado_sessao(&diretorio_projeto, &id_sessao), &chave_do_leitor) {
            if let Some(codigo_lido) = codigo_lido_no_trecho(trecho, &pasta_do_trecho, &diretorio_projeto, &ajustes_do_projeto) {
                bloquear(&motivo_sem_consulta(&codigo_lido));
            }
        }
    }

    // ---------- W4: ativação de issue (uma issue por sessão) ----------
    let mut numeros_ativados: Vec<i64> = Vec::new();
    for partes in regex(r"ativar-issue(?:\.mjs)?\s+#?(\d+)").captures_iter(&comando) {
        let numero: i64 = partes[1].parse().unwrap_or(0);
        if !numeros_ativados.contains(&numero) {
            numeros_ativados.push(numero);
        }
    }
    if numeros_ativados.len() > 1 {
        let lista = numeros_ativados.iter().map(|numero| format!("#{numero}")).collect::<Vec<_>>().join(", ");
        bloquear(&format!("o comando ativa mais de uma issue ({lista}). Uma sessão trabalha uma issue só."));
    }
    if let [numero_issue] = numeros_ativados[..] {
        let mut estado_sessao = ler_estado_sessao(&diretorio_projeto, &id_sessao);
        let chave_agente = chave_do_agente(&entrada_hook);
        let numero_ja_ativo = estado_sessao["issuesAtivas"][&chave_agente]["numero"].as_i64();
        if let Some(numero_anterior) = numero_ja_ativo.filter(|numero| *numero != numero_issue) {
            bloquear(&if chave_agente == "principal" {
                format!("esta sessão já trabalha na issue #{numero_anterior}. Contexto limpo por issue: termine, rode /nova-sessao e cole o kickoff numa janela nova para a #{numero_issue}.")
            } else {
                format!(
                    "este subagente já trabalha na issue #{numero_anterior}. Um implementador por issue: o orquestrador dispara outro para a #{numero_issue}."
                )
            });
        }
        let issue_candidata = buscar_issue_ou_bloquear(&diretorio_projeto, numero_issue);
        if issue_candidata.estado != "open" {
            bloquear(&format!("a issue #{numero_issue} está fechada."));
        }
        if tipo_wayfinder(&issue_candidata.labels).is_some() {
            bloquear(&format!(
                "#{numero_issue} é um ticket do wayfinder: decide, não implementa. Feche o mapa e passe por /to-spec → /to-tickets."
            ));
        }
        if issue_candidata.labels.iter().any(|label| label == "spec") {
            bloquear(&format!(
                "#{numero_issue} é um spec: ele é o pai das fatias, não uma delas. Fatie com /to-tickets e implemente as sub-issues."
            ));
        }
        if !issue_candidata.labels.iter().any(|label| LABELS_IMPLEMENTAVEIS.contains(&label.as_str())) {
            bloquear(&format!(
                "a issue #{numero_issue} não tem {}. Ela precisa sair do /to-tickets ou passar pelo /triage antes.",
                LABELS_IMPLEMENTAVEIS.join(" nem ")
            ));
        }
        // W4b — mapas iniciais: os cinco existem, e os que decidem desenho, stack, fases e regras de código estão fechados.
        let mapas_iniciais = buscar_mapas_iniciais(&diretorio_projeto).unwrap_or_else(|erro| {
            bloquear(&format!(
                "não consegui consultar os mapas iniciais no GitHub ({}). Os watchdogs falham fechados.",
                primeira_linha(&erro)
            ))
        });
        let mapas_faltando: Vec<&str> = mapas_iniciais.iter().filter(|mapa| mapa.numero.is_none()).map(|mapa| mapa.tipo).collect();
        if !mapas_faltando.is_empty() {
            bloquear(&format!("faltam mapas iniciais: {}. Todo projeto abre os cinco antes de implementar (docs/agents/mapas-iniciais.md). Abra com /wayfinder e as labels wayfinder:map + mapa-inicial:<tipo>.", mapas_faltando.join(", ")));
        }
        let mapas_ainda_abertos: Vec<String> = mapas_iniciais
            .iter()
            .filter(|mapa| MAPAS_INICIAIS_FECHADOS_PARA_IMPLEMENTAR.contains(&mapa.tipo) && mapa.estado != "closed")
            .map(|mapa| format!("\"{}\"", mapa.titulo))
            .collect();
        if !mapas_ainda_abertos.is_empty() {
            bloquear(&format!("os mapas iniciais {} ainda estão abertos. Implementação só começa com o desenho inicial do protótipo aprovado e a arquitetura, as fases e as regras de código decididas (o mapa de Infra não trava).", mapas_ainda_abertos.join(", ")));
        }
        estado_sessao["issuesAtivas"][&chave_agente] = json!({ "numero": issue_candidata.numero, "titulo": issue_candidata.titulo, "ativadaEm": agora_iso() });
        gravar_estado_sessao(&diretorio_projeto, &id_sessao, &estado_sessao);
    }

    // ---------- W6: reivindicar o ticket antes de trabalhar nele ----------
    for numero_ticket in numeros_apos_subcomando(&comando, &regex(r"gh\s+issue\s+comment\s+([^\s;&|]+)")) {
        let ticket_comentado = buscar_issue_ou_bloquear(&diretorio_projeto, numero_ticket);
        match tipo_wayfinder(&ticket_comentado.labels).as_deref() {
            None | Some("map") => continue,
            _ => {}
        }
        if ticket_comentado.responsaveis.is_empty() && !comando_reivindica_no_mesmo_passo(&comando, numero_ticket) {
            bloquear(&format!(
                "o ticket \"{}\" (#{numero_ticket}) não foi reivindicado. Primeira escrita da sessão: gh issue edit {numero_ticket} --add-assignee @me",
                ticket_comentado.titulo
            ));
        }
    }

    // ---------- W5: fechar ticket do wayfinder ----------
    let mut numeros_fechados = numeros_apos_subcomando(&comando, &regex(r"gh\s+issue\s+close\s+([^\s;&|]+)"));
    numeros_fechados.extend(
        regex(r#"issues/(\d+)\b[^;&|\n]*state[=:]\s*["']?closed"#)
            .captures_iter(&comando)
            .filter_map(|partes| partes[1].parse::<i64>().ok()),
    );

    for numero_ticket in numeros_fechados {
        let ticket_fechado = buscar_issue_ou_bloquear(&diretorio_projeto, numero_ticket);
        let tipo_ticket = match tipo_wayfinder(&ticket_fechado.labels) {
            None => continue,
            Some(tipo) if tipo == "map" => continue,
            Some(tipo) => tipo,
        };
        let padrao_do_fechamento = regex(&format!(r"gh\s+issue\s+close\s+\S*{numero_ticket}\b"));
        let trecho_do_fechamento = trechos_do_comando
            .iter()
            .copied()
            .find(|trecho| padrao_do_fechamento.is_match(trecho))
            .unwrap_or("");
        let fora_de_escopo = regex(r#"(?i)(--reason|-r)\s+["']?not[ _]planned"#).is_match(trecho_do_fechamento);

        if ticket_fechado.responsaveis.is_empty() && !comando_reivindica_no_mesmo_passo(&comando, numero_ticket) {
            bloquear(&format!(
                "o ticket \"{}\" (#{numero_ticket}) nunca foi reivindicado (sem assignee). Reivindique antes de resolver.",
                ticket_fechado.titulo
            ));
        }
        if fora_de_escopo {
            continue; // fora de escopo: não é passo da rota, não conta no limite
        }

        let tem_comentario_de_resolucao = ticket_fechado.quantidade_comentarios > 0
            || regex(r"(--comment|-c)\s").is_match(trecho_do_fechamento)
            || regex(&format!(r"gh\s+issue\s+comment\s+#?{numero_ticket}\b")).is_match(&comando);
        if !tem_comentario_de_resolucao {
            bloquear(&format!(
                "o ticket \"{}\" (#{numero_ticket}) seria fechado sem comentário de resolução. Poste a resposta antes (gh issue comment ou --comment).",
                ticket_fechado.titulo
            ));
        }

        // W9 — pesquisa antes de recomendar: a resolução termina com "## Fontes" e ao menos um link.
        if TIPOS_QUE_EXIGEM_FONTES.contains(&tipo_ticket.as_str()) {
            let corpos_dos_comentarios = buscar_corpos_dos_comentarios(&diretorio_projeto, numero_ticket).unwrap_or_else(|erro| {
                bloquear(&format!(
                    "não consegui ler os comentários de \"{}\" (#{numero_ticket}) no GitHub ({}). Os watchdogs falham fechados.",
                    ticket_fechado.titulo,
                    primeira_linha(&erro)
                ))
            });
            let textos_dos_arquivos_de_corpo: Vec<String> = regex(r#"--body-file\s+("([^"]+)"|'([^']+)'|(\S+))"#)
                .captures_iter(&comando)
                .map(|partes| caminho_entre_aspas(&partes))
                .filter(|caminho_corpo| caminho_corpo != "-")
                .filter_map(|caminho_corpo| std::fs::read_to_string(pasta_de_trabalho.join(caminho_corpo)).ok())
                .collect();
            let resolucao_tem_fontes = texto_tem_fontes(&comando)
                || textos_dos_arquivos_de_corpo.iter().any(|texto_corpo| texto_tem_fontes(texto_corpo))
                || corpos_dos_comentarios.iter().any(|corpo| texto_tem_fontes(corpo));
            if !resolucao_tem_fontes {
                bloquear(&format!("o ticket \"{}\" (#{numero_ticket}) seria fechado sem \"## Fontes\" no comentário de resolução. Nenhuma recomendação sem pesquisa: acrescente a seção com ao menos um link para docs/research/ ou para a fonte primária (docs/agents/pesquisa-antes-de-recomendar.md).", ticket_fechado.titulo));
            }
        }

        let mut estado_sessao = ler_estado_sessao(&diretorio_projeto, &id_sessao);
        let resolvidos: Vec<Value> = estado_sessao["ticketsWayfinderResolvidos"].as_array().cloned().unwrap_or_default();
        let ja_registrado = resolvidos.iter().any(|ticket| ticket["numero"].as_i64() == Some(numero_ticket));
        let decisoes_ja_resolvidas: Vec<&Value> = resolvidos
            .iter()
            .filter(|ticket| {
                let tipo_anterior = texto(&ticket["tipo"]);
                tipo_anterior != "research" && !(tipo_ticket == "prototype" && tipo_anterior == "prototype")
            })
            .collect();
        if !ja_registrado && tipo_ticket != "research" && !decisoes_ja_resolvidas.is_empty() {
            let ticket_anterior = decisoes_ja_resolvidas[0];
            bloquear(&format!(
                "esta sessão já resolveu \"{}\" (#{}). O /wayfinder resolve UM ticket por sessão (pesquisa é a exceção). Abra uma sessão nova para \"{}\".",
                texto(&ticket_anterior["titulo"]),
                ticket_anterior["numero"],
                ticket_fechado.titulo
            ));
        }
        if !ja_registrado {
            let novo_ticket = json!({ "numero": numero_ticket, "titulo": ticket_fechado.titulo, "tipo": tipo_ticket, "ponteiroNoMapaVerificado": false });
            if let Some(lista) = estado_sessao["ticketsWayfinderResolvidos"].as_array_mut() {
                lista.push(novo_ticket);
            }
            gravar_estado_sessao(&diretorio_projeto, &id_sessao, &estado_sessao);
        }
    }

    // ---------- W7: main só recebe merge ----------
    // A branch é a da pasta onde o commit roda (worktree de agente, `git -C`, `cd`), não a do checkout principal.
    if regex(r"\bgit\s+(?:-C\s+\S+\s+)?commit\b").is_match(&comando) {
        let pasta_do_commit = if let Some(partes) = regex(r#"\bgit\s+-C\s+("([^"]+)"|'([^']+)'|(\S+))\s+commit\b"#).captures(&comando) {
            PathBuf::from(caminho_entre_aspas(&partes))
        } else if let Some(partes) = regex(r#"^\s*cd\s+("([^"]+)"|'([^']+)'|(\S+))\s*(?:&&|;)"#).captures(&comando) {
            PathBuf::from(caminho_entre_aspas(&partes))
        } else {
            pasta_de_trabalho.clone()
        };
        let nome_branch = branch_atual(&pasta_do_commit);
        if branch_eh_principal(nome_branch.as_deref()) && repositorio_tem_commits(&pasta_do_commit) {
            bloquear(&format!(
                "commit direto na \"{}\" é proibido. Use a branch da issue (issue/<n>-<slug>), prototype/<nome> ou research/<nome>.",
                nome_branch.unwrap_or_default()
            ));
        }
    }
    0
}
