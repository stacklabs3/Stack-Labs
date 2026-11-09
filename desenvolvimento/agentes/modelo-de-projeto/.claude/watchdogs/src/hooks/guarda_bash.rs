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

// Um comando do shell em trechos (separados por &&, ||, ;, & e quebra de linha fora de aspas), cada
// trecho em comandos do cano (| fora de aspas) e cada comando em palavras, já sem as aspas. O que está
// entre aspas é argumento: `gh issue comment --body "cat src/a.ts"` não lê código, e o `\|` de um
// padrão do grep não parte o cano. O corpo de um heredoc e o comentário ficam de fora.
pub(crate) fn trechos_do_shell(comando: &str) -> Vec<Vec<Vec<String>>> {
    let mut trechos: Vec<Vec<Vec<String>>> = Vec::new();
    let mut cano: Vec<Vec<String>> = Vec::new();
    let mut palavras: Vec<String> = Vec::new();
    let mut palavra = String::new();
    let mut palavra_aberta = false;
    let mut fins_de_heredoc: Vec<String> = Vec::new();
    let letras: Vec<char> = comando.chars().collect();
    let mut posicao = 0;
    let fechar_palavra = |palavra: &mut String, palavra_aberta: &mut bool, palavras: &mut Vec<String>| {
        if *palavra_aberta {
            palavras.push(std::mem::take(palavra));
            *palavra_aberta = false;
        }
    };
    let fechar_comando = |palavras: &mut Vec<String>, cano: &mut Vec<Vec<String>>| {
        if !palavras.is_empty() {
            cano.push(std::mem::take(palavras));
        }
    };
    let fechar_trecho = |cano: &mut Vec<Vec<String>>, trechos: &mut Vec<Vec<Vec<String>>>| {
        if !cano.is_empty() {
            trechos.push(std::mem::take(cano));
        }
    };
    while posicao < letras.len() {
        let letra = letras[posicao];
        match letra {
            '\'' => {
                palavra_aberta = true;
                posicao += 1;
                while posicao < letras.len() && letras[posicao] != '\'' {
                    palavra.push(letras[posicao]);
                    posicao += 1;
                }
            }
            '"' => {
                palavra_aberta = true;
                posicao += 1;
                while posicao < letras.len() && letras[posicao] != '"' {
                    if letras[posicao] == '\\' && posicao + 1 < letras.len() {
                        posicao += 1;
                    }
                    palavra.push(letras[posicao]);
                    posicao += 1;
                }
            }
            '\\' => {
                palavra_aberta = true;
                if let Some(proxima) = letras.get(posicao + 1).filter(|proxima| **proxima != '\n') {
                    palavra.push(*proxima);
                }
                posicao += 1;
            }
            '#' if !palavra_aberta => {
                while posicao + 1 < letras.len() && letras[posicao + 1] != '\n' {
                    posicao += 1;
                }
            }
            ' ' | '\t' => fechar_palavra(&mut palavra, &mut palavra_aberta, &mut palavras),
            '|' | ';' | '&' | '\n' => {
                fechar_palavra(&mut palavra, &mut palavra_aberta, &mut palavras);
                let dobrado = letras.get(posicao + 1) == Some(&letra) && letra != '\n';
                // `2>&1` e `&>` são redirecionamento, não separador.
                let redirecionamento = letra == '&' && (letras.get(posicao.wrapping_sub(1)) == Some(&'>') || letras.get(posicao + 1) == Some(&'>'));
                if redirecionamento {
                    palavra.push(letra);
                    palavra_aberta = true;
                    posicao += 1;
                    continue;
                }
                fechar_comando(&mut palavras, &mut cano);
                if letra != '|' || dobrado {
                    fechar_trecho(&mut cano, &mut trechos);
                }
                if dobrado {
                    posicao += 1;
                }
                if letra == '\n' {
                    // As linhas do heredoc são texto que vai para o comando, não comandos.
                    for fim_do_heredoc in std::mem::take(&mut fins_de_heredoc) {
                        while posicao + 1 < letras.len() {
                            let fim_da_linha = letras[posicao + 1..]
                                .iter()
                                .position(|letra_da_linha| *letra_da_linha == '\n')
                                .map(|tamanho| posicao + 1 + tamanho)
                                .unwrap_or(letras.len());
                            let linha: String = letras[posicao + 1..fim_da_linha].iter().collect();
                            posicao = fim_da_linha;
                            if linha.trim() == fim_do_heredoc {
                                break;
                            }
                        }
                    }
                }
            }
            '<' if letras.get(posicao + 1) == Some(&'<') && letras.get(posicao + 2) != Some(&'<') => {
                fechar_palavra(&mut palavra, &mut palavra_aberta, &mut palavras);
                posicao += 2;
                if letras.get(posicao) == Some(&'-') {
                    posicao += 1;
                }
                while letras.get(posicao) == Some(&' ') {
                    posicao += 1;
                }
                let mut fim_do_heredoc = String::new();
                while posicao < letras.len() && !matches!(letras[posicao], ' ' | '\t' | '\n' | ';' | '|' | '&' | '>' | ')') {
                    if !matches!(letras[posicao], '\'' | '"' | '\\') {
                        fim_do_heredoc.push(letras[posicao]);
                    }
                    posicao += 1;
                }
                palavras.push(format!("<<{fim_do_heredoc}"));
                if !fim_do_heredoc.is_empty() {
                    fins_de_heredoc.push(fim_do_heredoc);
                }
                continue;
            }
            outra_letra => {
                palavra_aberta = true;
                palavra.push(outra_letra);
            }
        }
        posicao += 1;
    }
    fechar_palavra(&mut palavra, &mut palavra_aberta, &mut palavras);
    fechar_comando(&mut palavras, &mut cano);
    fechar_trecho(&mut cano, &mut trechos);
    trechos
}

// A consulta de um trecho que chama o `rodar contexto`, sem --limite e sem o resto do cano.
fn consulta_do_trecho(trecho: &[Vec<String>]) -> Option<String> {
    let palavras = trecho.first()?;
    let posicao_do_rodar = palavras.iter().position(|palavra| palavra.ends_with(".claude/watchdogs/rodar"))?;
    let antes_do_rodar_so_sh = palavras[..posicao_do_rodar].iter().all(|palavra| palavra == "sh");
    if !antes_do_rodar_so_sh || palavras.get(posicao_do_rodar + 1).map(String::as_str) != Some("contexto") {
        return None;
    }
    let mut partes_da_consulta = Vec::new();
    let mut proxima_eh_limite = false;
    for palavra in &palavras[posicao_do_rodar + 2..] {
        if palavra.contains('>') || palavra.starts_with('<') {
            break;
        }
        if proxima_eh_limite {
            proxima_eh_limite = false;
        } else if palavra == "--limite" {
            proxima_eh_limite = true;
        } else {
            partes_da_consulta.push(palavra.as_str());
        }
    }
    let consulta = partes_da_consulta.join(" ");
    (!consulta.is_empty()).then_some(consulta)
}

// Opções que levam o valor na palavra seguinte, por comando: o valor não é caminho.
fn opcao_leva_valor(verbo: &str, opcao: &str) -> bool {
    let opcoes_com_valor: &[&str] = match verbo {
        "grep" | "egrep" => &[
            "-A",
            "-B",
            "-C",
            "-m",
            "-e",
            "-f",
            "--regexp",
            "--file",
            "--max-count",
            "--include",
            "--exclude",
            "--exclude-dir",
        ],
        "rg" => &[
            "-A",
            "-B",
            "-C",
            "-m",
            "-e",
            "-f",
            "-g",
            "-t",
            "-T",
            "--regexp",
            "--file",
            "--glob",
            "--type",
            "--type-not",
            "--max-count",
        ],
        "sed" => &["-e", "-f", "--expression", "--file"],
        "head" | "tail" => &["-n", "-c", "--lines", "--bytes"],
        _ => &[],
    };
    opcoes_com_valor.contains(&opcao)
}

// O padrão vem numa opção (`grep -e x`, `sed -e s/a/b/`, `sed -f roteiro`): aí não há padrão posicional.
fn padrao_veio_em_opcao(verbo: &str, opcao: &str) -> bool {
    matches!(verbo, "grep" | "egrep" | "rg" | "sed")
        && (matches!(opcao, "-e" | "-f" | "--regexp" | "--file" | "--expression")
            || opcao.starts_with("--regexp=")
            || opcao.starts_with("--expression=")
            || (opcao.starts_with("-e") && opcao.len() > 2 && verbo != "sed"))
}

// O primeiro caminho que exige consulta entre os que um cat, sed ou grep do trecho lê. Só conta caminho
// de arquivo ou pasta que existe no disco, dentro do projeto e fora do .gitignore: o padrão do grep e o
// script do sed são o primeiro argumento e nunca contam, e `git log | grep x` lê do cano.
fn codigo_lido_no_trecho(trecho: &[Vec<String>], pasta_base: &Path, diretorio_projeto: &Path, ajustes: &AjustesDoProjeto) -> Option<String> {
    let verbo_de_leitura = regex(r"^(cat|head|tail|less|sed|e?grep|rg)$");
    for (posicao_no_cano, palavras_do_comando) in trecho.iter().enumerate() {
        let Some(verbo) = palavras_do_comando.first().map(String::as_str).filter(|verbo| verbo_de_leitura.is_match(verbo)) else {
            continue;
        };
        let argumentos: Vec<&str> = palavras_do_comando[1..]
            .iter()
            .map(String::as_str)
            .take_while(|palavra| !palavra.contains('>') && !palavra.starts_with('<'))
            .collect();
        let eh_recursivo = verbo == "rg"
            || argumentos
                .iter()
                .any(|argumento| regex(r"^(-[a-zA-Z]*[rR][a-zA-Z]*|--recursive)$").is_match(argumento));
        let mut espera_padrao = matches!(verbo, "grep" | "egrep" | "rg" | "sed");
        let mut posicionais: Vec<&str> = Vec::new();
        let mut pular_proxima = false;
        let mut fim_das_opcoes = false;
        for argumento in &argumentos {
            if pular_proxima {
                pular_proxima = false;
                continue;
            }
            if !fim_das_opcoes && *argumento == "--" {
                fim_das_opcoes = true;
                continue;
            }
            if !fim_das_opcoes && argumento.starts_with('-') && argumento.len() > 1 {
                if padrao_veio_em_opcao(verbo, argumento) {
                    espera_padrao = false;
                }
                pular_proxima = opcao_leva_valor(verbo, argumento);
                continue;
            }
            if espera_padrao {
                espera_padrao = false;
                continue;
            }
            posicionais.push(argumento);
        }
        let mut caminhos_lidos: Vec<PathBuf> = posicionais
            .iter()
            .map(|argumento| argumento.split('*').next().unwrap_or(""))
            .filter(|caminho| !caminho.is_empty())
            .map(|caminho| pasta_base.join(caminho))
            .filter(|caminho| caminho.exists())
            .collect();
        if posicionais.is_empty() && eh_recursivo && posicao_no_cano == 0 && pasta_base.is_dir() {
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
            if caminho_exige_consulta(&caminho_com_barra, ajustes) && !ignorado_pelo_git(diretorio_projeto, &caminho_com_barra) {
                return Some(caminho_com_barra);
            }
        }
    }
    None
}

// node_modules/, build/ e o que mais o .gitignore esconde não é código do projeto.
fn ignorado_pelo_git(diretorio_projeto: &Path, caminho_relativo: &str) -> bool {
    !caminho_relativo.is_empty()
        && std::process::Command::new("git")
            .args(["check-ignore", "-q", "--", caminho_relativo.trim_end_matches('/')])
            .current_dir(diretorio_projeto)
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|situacao| situacao.success())
            .unwrap_or(false)
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
    // A atualização dos watchdogs é do agente, mas só da fonte oficial e nunca para trás (ADR 0024).
    if regex(r"--permitir-regressao\b").is_match(&comando) {
        bloquear("--permitir-regressao volta o método a uma versão anterior e afrouxa os watchdogs. Só um humano roda o setup-stacklabs com essa opção.");
    }
    if regex(r"\b(STACKLABS_ORIGEM|MATT_ORIGEM)\b").is_match(&comando) {
        bloquear("o setup-stacklabs só instala da fonte oficial quando quem roda é o agente. Uma fonte local poderia trazer watchdogs alterados: só um humano usa STACKLABS_ORIGEM ou MATT_ORIGEM.");
    }
    if regex(r"gh\s+[^;&|\n]*--add-label[=\s]+[^;&|\n]*altera-watchdogs").is_match(&comando) {
        bloquear("a label altera-watchdogs é o aval de que a mudança é segura: quem a aplica é um humano ou o --concluir-pr do setup-stacklabs, depois de conferir os arquivos com a fonte oficial.");
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
    let mut pasta_do_trecho = pasta_de_trabalho.clone();
    for trecho in &trechos_do_shell(&comando) {
        let eh_cd = trecho.len() == 1 && trecho[0].len() == 2 && trecho[0][0] == "cd";
        if eh_cd {
            pasta_do_trecho = pasta_do_trecho.join(&trecho[0][1]);
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
