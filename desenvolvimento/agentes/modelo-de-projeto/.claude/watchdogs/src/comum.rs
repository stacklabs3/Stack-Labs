// Utilitários compartilhados pelos watchdogs do workflow wayfinder / Matt Pocock.
// Regras e motivação: docs/agents/watchdogs.md

use regex::Regex;
use serde_json::{json, Map, Value};
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// Smart zone calibrada para modelos de contexto longo: no aviso, feche a etapa (o orquestrador de
// issues já passa a rodada adiante aqui); no limite duro, a próxima ação é /nova-sessao.
pub const LIMITE_AVISO_CONTEXTO: u64 = 450_000;
pub const LIMITE_DURO_CONTEXTO: u64 = 550_000;

// Labels de triagem que tornam uma issue implementável (docs/agents/triage-labels.md).
pub const LABELS_IMPLEMENTAVEIS: [&str; 2] = ["ready-for-agent", "ready-for-human"];

// Branches descartáveis do próprio workflow: /prototype e tickets de pesquisa do wayfinder.
pub const PREFIXOS_BRANCH_DESCARTAVEL: [&str; 2] = ["prototype/", "research/"];

// Mapas iniciais da casa (docs/agents/mapas-iniciais.md): label mapa-inicial:<tipo> + wayfinder:map.
pub const MAPAS_INICIAIS: [&str; 5] = ["prototipo", "arquitetura", "fases", "regras-de-codigo", "infra"];

// Mapas iniciais que precisam estar fechados para uma issue de implementação ser ativada.
pub const MAPAS_INICIAIS_FECHADOS_PARA_IMPLEMENTAR: [&str; 4] = ["prototipo", "arquitetura", "fases", "regras-de-codigo"];

// Tipos de ticket do wayfinder cuja resolução exige a seção "## Fontes" (pesquisa antes de recomendar).
pub const TIPOS_QUE_EXIGEM_FONTES: [&str; 3] = ["grilling", "task", "research"];

// Skills que abrem uma sessão de grilling. Chamadas pelo Skill tool ou digitadas como /<nome>.
pub const SKILLS_DE_GRILLING: [&str; 4] = ["grilling", "grill-me", "grill-with-docs", "wayfinder"];

// Linguagem das perguntas ao humano no grilling (AGENTS.md, regra 10). Injetada quando o grilling
// começa e a cada resposta do humano, porque a skill sozinha deixa a pergunta crescer.
pub const REGRA_LINGUAGEM_GRILLING: &str = "🗣️ [watchdog linguagem-grilling] Perguntas do grilling em linguagem simples:
- Curta: título de até 8 palavras e corpo de até 3 linhas. Sem parágrafo de contexto, sem repetir o que o humano já disse.
- Simples: palavras do dia a dia. Termo técnico só se for inevitável, explicado em meia linha.
- Com exemplo real do cenário deste projeto: uma tela, um usuário, um dado ou um caso que existe aqui, mostrando o que muda com cada opção. Nada de exemplo genérico (\"imagine um e-commerce\").
- Opções em lista A/B/C, uma linha cada. Recomendação em uma frase, com o porquê e a fonte.
Ex.: ❓ Q1 - Quem pode cancelar um pedido? A) Só o cliente, até o envio. B) Cliente e atendente. Ex.: a Ana pede às 10h, o pacote sai às 14h; às 15h ela quer cancelar. Em A, não pode; em B, o atendente cancela. ➡️ A: o fluxo de devolução já cobre depois do envio (docs/research/devolucao.md).";

pub fn nome_da_skill_de_grilling(texto_ou_nome: &str) -> Option<&'static str> {
    let sem_barra = texto_ou_nome.trim().trim_start_matches('/');
    let primeira_palavra = sem_barra.split_whitespace().next().unwrap_or("");
    let nome_limpo = primeira_palavra.rsplit(':').next().unwrap_or("");
    SKILLS_DE_GRILLING.iter().copied().find(|skill| *skill == nome_limpo)
}

// Caminhos que nenhum agente edita: são os próprios watchdogs e o estado deles.
pub const CAMINHOS_PROTEGIDOS: [&str; 7] = [
    ".claude/watchdogs/",
    ".claude/githooks/",
    ".claude/agents/",
    ".claude/metodo.json",
    ".claude/settings.json",
    ".claude/state/",
    ".github/workflows/vigia-do-metodo.yml",
];

// Onde o git procura os hooks deste repo (pre-push que protege a main).
// Título do comentário da issue que publica o contrato da fatia (regra 24, docs/agents/contrato-da-fatia.md).
pub const MARCADOR_CONTRATO: &str = "## Contrato da fatia";

pub const CAMINHO_GITHOOKS: &str = ".claude/githooks";

// Como o agente ativa uma issue: o guarda-bash reconhece este comando antes de ele rodar.
pub const COMANDO_ATIVAR_ISSUE: &str = ".claude/watchdogs/rodar ativar-issue";

// Caminhos de planejamento e documentação, editáveis sem issue ativa.
pub const CAMINHOS_LIVRES_SEM_ISSUE: [&str; 12] = [
    "AGENTS.md",
    "CLAUDE.md",
    "GLOSSARY.md",
    "GLOSSARY-MAP.md",
    "CONTEXT.md",
    "CONTEXT-MAP.md",
    "README.md",
    ".gitignore",
    "docs/",
    ".scratch/",
    "prototipos/",
    ".claude/",
];

// Etapa de adoção do método (projeto que já existia antes dele; docs/agents/adocao.md), gravada em
// .claude/metodo.json. Sem o arquivo, o projeto nasceu no método: etapa "completo".
//   observacao   — só protegem a si mesmos e a main; o resto apenas avisa
//   planejamento — valem as regras do wayfinder, das fontes e da ativação de issue
//   completo     — tudo, inclusive código só com issue ativa
pub const ETAPAS_DE_ADOCAO: [&str; 3] = ["observacao", "planejamento", "completo"];

pub struct MetodoDoProjeto {
    pub etapa_de_adocao: String,
    pub adotado_em: Option<String>,
}

pub fn ler_metodo_do_projeto(diretorio_projeto: &Path) -> MetodoDoProjeto {
    let caminho_metodo = diretorio_projeto.join(".claude").join("metodo.json");
    let Ok(texto_metodo) = fs::read_to_string(&caminho_metodo) else {
        return MetodoDoProjeto {
            etapa_de_adocao: "completo".into(),
            adotado_em: None,
        };
    };
    let metodo: Value = serde_json::from_str(&texto_metodo).unwrap_or(Value::Null);
    let etapa_lida = metodo["etapaDeAdocao"].as_str().unwrap_or("");
    let etapa_valida = if ETAPAS_DE_ADOCAO.contains(&etapa_lida) { etapa_lida } else { "completo" };
    MetodoDoProjeto {
        etapa_de_adocao: etapa_valida.into(),
        adotado_em: metodo["adotadoEm"].as_str().map(String::from),
    }
}

// Ajustes do projeto aos watchdogs, em .claude/metodo.json ("ajustes"; docs/agents/watchdogs.md).
// Só o humano grava o arquivo. Valor ausente ou inválido fica no padrão do método.
pub struct AjustesDoProjeto {
    pub caminhos_livres: Vec<String>,
    pub branches_descartaveis: Vec<String>,
    pub limite_aviso_contexto: u64,
    pub limite_duro_contexto: u64,
}

pub fn ler_ajustes_do_projeto(diretorio_projeto: &Path) -> AjustesDoProjeto {
    let metodo: Value = fs::read_to_string(diretorio_projeto.join(".claude").join("metodo.json"))
        .ok()
        .and_then(|texto_metodo| serde_json::from_str(&texto_metodo).ok())
        .unwrap_or(Value::Null);
    let ajustes = &metodo["ajustes"];
    let lista = |campo: &str| -> Vec<String> {
        ajustes[campo]
            .as_array()
            .map(|itens| {
                itens
                    .iter()
                    .filter_map(Value::as_str)
                    .filter(|item| !item.is_empty())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    };
    let aviso_lido = ajustes["contexto"]["aviso"].as_u64();
    let duro_lido = ajustes["contexto"]["duro"].as_u64();
    let (limite_aviso_contexto, limite_duro_contexto) = match (aviso_lido, duro_lido) {
        (Some(aviso), Some(duro)) if aviso > 0 && aviso < duro => (aviso, duro),
        _ => (LIMITE_AVISO_CONTEXTO, LIMITE_DURO_CONTEXTO),
    };
    AjustesDoProjeto {
        caminhos_livres: lista("caminhosLivres"),
        branches_descartaveis: lista("branchesDescartaveis"),
        limite_aviso_contexto,
        limite_duro_contexto,
    }
}

pub fn etapa_de_adocao(diretorio_projeto: &Path) -> String {
    ler_metodo_do_projeto(diretorio_projeto).etapa_de_adocao
}

pub fn watchdogs_desligados() -> bool {
    std::env::var("STACKLABS_WATCHDOGS").map(|valor| valor == "desligado").unwrap_or(false)
}

pub fn ler_entrada_do_hook() -> Value {
    let mut texto_entrada = String::new();
    let _ = std::io::stdin().read_to_string(&mut texto_entrada);
    if texto_entrada.trim().is_empty() {
        return Value::Object(Map::new());
    }
    serde_json::from_str(&texto_entrada).unwrap_or_else(|_| Value::Object(Map::new()))
}

pub fn texto(valor: &Value) -> &str {
    valor.as_str().unwrap_or("")
}

pub fn diretorio_do_projeto(entrada_hook: &Value) -> PathBuf {
    if let Ok(diretorio) = std::env::var("CLAUDE_PROJECT_DIR") {
        if !diretorio.is_empty() {
            return PathBuf::from(diretorio);
        }
    }
    match entrada_hook["cwd"].as_str() {
        Some(cwd) if !cwd.is_empty() => PathBuf::from(cwd),
        _ => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    }
}

// Exit 2 em PreToolUse impede a ferramenta; o stderr volta para o agente como motivo.
pub fn bloquear_ferramenta(nome_watchdog: &str, motivo: &str) -> ! {
    eprintln!("[watchdog {nome_watchdog}] BLOQUEADO — {motivo}\n(Regras: AGENTS.md · Watchdogs: docs/agents/watchdogs.md)");
    std::process::exit(2);
}

pub fn emitir_contexto_adicional(nome_evento: &str, texto_contexto: &str) {
    let saida = json!({ "hookSpecificOutput": { "hookEventName": nome_evento, "additionalContext": texto_contexto } });
    print!("{saida}");
}

// ---------- estado por sessão ----------

fn caminho_estado_sessao(diretorio_projeto: &Path, id_sessao: &str) -> PathBuf {
    diretorio_projeto
        .join(".claude")
        .join("state")
        .join("sessoes")
        .join(format!("{id_sessao}.json"))
}

// O estado fica como JSON livre para preservar campos que outra versão dos watchdogs gravou.
pub fn ler_estado_sessao(diretorio_projeto: &Path, id_sessao: &str) -> Value {
    let caminho_estado = caminho_estado_sessao(diretorio_projeto, id_sessao);
    let mut estado_sessao = fs::read_to_string(&caminho_estado)
        .ok()
        .and_then(|texto_estado| serde_json::from_str::<Value>(&texto_estado).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({ "issuesAtivas": {}, "ticketsWayfinderResolvidos": [] }));
    let objeto = estado_sessao.as_object_mut().expect("estado é objeto");
    // Estado gravado antes da issue por agente: a issue única era da sessão principal.
    if !objeto.get("issuesAtivas").is_some_and(Value::is_object) {
        let issue_antiga = objeto.get("issueAtiva").cloned().filter(|issue| !issue.is_null());
        let issues_ativas = issue_antiga.map(|issue| json!({ "principal": issue })).unwrap_or_else(|| json!({}));
        objeto.insert("issuesAtivas".into(), issues_ativas);
    }
    objeto.remove("issueAtiva");
    if !objeto.get("ticketsWayfinderResolvidos").is_some_and(Value::is_array) {
        objeto.insert("ticketsWayfinderResolvidos".into(), json!([]));
    }
    estado_sessao
}

// Quem está agindo: um subagente (agent_id, na orquestração) ou a própria sessão. Cada um tem a sua
// issue ativa, para que o orquestrador dispare um implementador por issue na mesma sessão.
pub fn chave_do_agente(entrada_hook: &Value) -> String {
    match entrada_hook["agent_id"].as_str() {
        Some(id_agente) if !id_agente.is_empty() => id_agente.to_string(),
        _ => "principal".into(),
    }
}

pub fn gravar_estado_sessao(diretorio_projeto: &Path, id_sessao: &str, estado_sessao: &Value) {
    let caminho_estado = caminho_estado_sessao(diretorio_projeto, id_sessao);
    if let Some(pasta) = caminho_estado.parent() {
        let _ = fs::create_dir_all(pasta);
    }
    let _ = fs::write(&caminho_estado, serde_json::to_string_pretty(estado_sessao).unwrap_or_default());
}

// ---------- git ----------

fn executar_git(diretorio: &Path, argumentos_git: &[&str]) -> Option<String> {
    let saida = Command::new("git")
        .args(argumentos_git)
        .current_dir(diretorio)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    saida.status.success().then(|| String::from_utf8_lossy(&saida.stdout).trim().to_string())
}

pub fn caminho_hooks_do_git(diretorio_projeto: &Path) -> Option<String> {
    executar_git(diretorio_projeto, &["config", "--get", "core.hooksPath"])
}

pub fn branch_atual(diretorio: &Path) -> Option<String> {
    executar_git(diretorio, &["symbolic-ref", "--short", "HEAD"])
}

pub fn repositorio_tem_commits(diretorio: &Path) -> bool {
    executar_git(diretorio, &["rev-parse", "--verify", "--quiet", "HEAD"]).is_some()
}

pub fn branch_eh_principal(nome_branch: Option<&str>) -> bool {
    matches!(nome_branch, Some("main") | Some("master"))
}

pub fn branch_eh_descartavel(nome_branch: Option<&str>, ajustes: &AjustesDoProjeto) -> bool {
    nome_branch.is_some_and(|nome| {
        PREFIXOS_BRANCH_DESCARTAVEL.iter().any(|prefixo| nome.starts_with(prefixo))
            || ajustes.branches_descartaveis.iter().any(|prefixo| nome.starts_with(prefixo.as_str()))
    })
}

// ---------- GitHub ----------

pub fn repositorio_github(diretorio_projeto: &Path) -> Option<String> {
    let url_remoto = executar_git(diretorio_projeto, &["remote", "get-url", "origin"]).unwrap_or_default();
    let padrao_remoto = Regex::new(r"github\.com[:/]([^/]+)/(.+?)(\.git)?$").expect("regex válida");
    padrao_remoto.captures(&url_remoto).map(|partes| format!("{}/{}", &partes[1], &partes[2]))
}

// Roda o gh com limite de 15 s. O erro traz a primeira linha útil, como a mensagem do Node trazia.
pub fn executar_gh(diretorio_projeto: &Path, argumentos_gh: &[&str]) -> Result<String, String> {
    let mut processo = Command::new("gh")
        .args(argumentos_gh)
        .current_dir(diretorio_projeto)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| "spawnSync gh ENOENT".to_string())?;
    let inicio = Instant::now();
    loop {
        match processo.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if inicio.elapsed() > Duration::from_secs(15) => {
                let _ = processo.kill();
                return Err("spawnSync gh ETIMEDOUT".into());
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(erro) => return Err(erro.to_string()),
        }
    }
    let saida = processo.wait_with_output().map_err(|erro| erro.to_string())?;
    if !saida.status.success() {
        return Err(format!("Command failed: gh {}", argumentos_gh.join(" ")));
    }
    Ok(String::from_utf8_lossy(&saida.stdout).trim().to_string())
}

pub struct Issue {
    pub numero: i64,
    pub titulo: String,
    pub estado: String,
    pub corpo: String,
    pub labels: Vec<String>,
    pub responsaveis: Vec<String>,
    pub quantidade_comentarios: i64,
}

pub fn buscar_issue(diretorio_projeto: &Path, numero_issue: i64) -> Result<Issue, String> {
    let nome_repositorio = repositorio_github(diretorio_projeto).unwrap_or_else(|| "null".into());
    let texto_issue = executar_gh(diretorio_projeto, &["api", &format!("repos/{nome_repositorio}/issues/{numero_issue}")])?;
    let issue_bruta: Value = serde_json::from_str(&texto_issue).map_err(|erro| erro.to_string())?;
    let nomes = |lista: &Value, campo: &str| -> Vec<String> {
        lista
            .as_array()
            .map(|itens| itens.iter().filter_map(|item| item[campo].as_str().map(String::from)).collect())
            .unwrap_or_default()
    };
    Ok(Issue {
        numero: issue_bruta["number"].as_i64().unwrap_or(numero_issue),
        titulo: texto(&issue_bruta["title"]).to_string(),
        estado: texto(&issue_bruta["state"]).to_string(),
        corpo: texto(&issue_bruta["body"]).to_string(),
        labels: nomes(&issue_bruta["labels"], "name"),
        responsaveis: nomes(&issue_bruta["assignees"], "login"),
        quantidade_comentarios: issue_bruta["comments"].as_i64().unwrap_or(0),
    })
}

pub fn tipo_wayfinder(labels_issue: &[String]) -> Option<String> {
    labels_issue.iter().find_map(|label| label.strip_prefix("wayfinder:").map(String::from))
}

// Mapa dono de um ticket: sub-issue nativa primeiro, depois a convenção "Part of #N" do corpo.
pub fn numero_do_mapa(diretorio_projeto: &Path, ticket_wayfinder: &Issue) -> Option<i64> {
    let nome_repositorio = repositorio_github(diretorio_projeto).unwrap_or_else(|| "null".into());
    let rota_do_pai = format!("repos/{nome_repositorio}/issues/{}/parent", ticket_wayfinder.numero);
    if let Ok(numero_pai) = executar_gh(diretorio_projeto, &["api", &rota_do_pai, "--jq", ".number"]) {
        if let Ok(numero) = numero_pai.parse::<i64>() {
            if numero != 0 {
                return Some(numero);
            }
        }
    }
    let padrao_pai = Regex::new(r"(?i)Part of #(\d+)").expect("regex válida");
    padrao_pai.captures(&ticket_wayfinder.corpo).and_then(|partes| partes[1].parse().ok())
}

// Corpos dos comentários de uma issue, do mais antigo ao mais novo.
pub fn buscar_corpos_dos_comentarios(diretorio_projeto: &Path, numero_issue: i64) -> Result<Vec<String>, String> {
    let nome_repositorio = repositorio_github(diretorio_projeto).unwrap_or_else(|| "null".into());
    let rota = format!("repos/{nome_repositorio}/issues/{numero_issue}/comments");
    let corpos_json = executar_gh(diretorio_projeto, &["api", "--paginate", &rota, "--jq", "[.[].body]"])?;
    // --paginate emite um array JSON por página.
    let mut corpos = Vec::new();
    for pagina_json in corpos_json.lines().filter(|linha| !linha.is_empty()) {
        let pagina: Value = serde_json::from_str(pagina_json).map_err(|erro| erro.to_string())?;
        corpos.extend(pagina.as_array().into_iter().flatten().map(|corpo| texto(corpo).to_string()));
    }
    Ok(corpos)
}

pub struct MapaInicial {
    pub tipo: &'static str,
    pub numero: Option<i64>,
    pub titulo: String,
    pub estado: String,
}

// Situação de cada mapa inicial; numero None quando falta.
pub fn buscar_mapas_iniciais(diretorio_projeto: &Path) -> Result<Vec<MapaInicial>, String> {
    MAPAS_INICIAIS
        .iter()
        .map(|tipo_mapa| {
            let label_do_tipo = format!("mapa-inicial:{tipo_mapa}");
            let argumentos = [
                "issue",
                "list",
                "--label",
                &label_do_tipo,
                "--label",
                "wayfinder:map",
                "--state",
                "all",
                "--json",
                "number,title,state",
            ];
            let mapas_encontrados: Value = serde_json::from_str(&executar_gh(diretorio_projeto, &argumentos)?).map_err(|erro| erro.to_string())?;
            let mapa_do_tipo = &mapas_encontrados[0];
            Ok(match mapa_do_tipo["number"].as_i64() {
                None => MapaInicial {
                    tipo: tipo_mapa,
                    numero: None,
                    titulo: String::new(),
                    estado: String::new(),
                },
                Some(numero) => MapaInicial {
                    tipo: tipo_mapa,
                    numero: Some(numero),
                    titulo: texto(&mapa_do_tipo["title"]).to_string(),
                    estado: texto(&mapa_do_tipo["state"]).to_lowercase(),
                },
            })
        })
        .collect()
}

// "## Fontes" seguida de ao menos um link http(s) ou um caminho em docs/research/.
pub fn texto_tem_fontes(texto_resolucao: &str) -> bool {
    let cabecalho_fontes = Regex::new(r"(?im)^#{2,3}\s*Fontes\b.*$").expect("regex válida");
    let Some(encontrado) = cabecalho_fontes.find(texto_resolucao) else {
        return false;
    };
    let texto_apos_fontes = &texto_resolucao[encontrado.end()..];
    let proximo_titulo = Regex::new(r"(?m)^#{1,2}\s").expect("regex válida");
    let secao_fontes = proximo_titulo
        .find(texto_apos_fontes)
        .map(|titulo| &texto_apos_fontes[..titulo.start()])
        .unwrap_or(texto_apos_fontes);
    Regex::new(r"https?://\S+|docs/research/\S+").expect("regex válida").is_match(secao_fontes)
}

// ---------- caminhos ----------

// Resolve "." e ".." sem tocar o disco, como o path.resolve do Node.
fn normalizar(caminho: &Path) -> PathBuf {
    let mut normalizado = PathBuf::new();
    for componente in caminho.components() {
        match componente {
            Component::ParentDir => {
                normalizado.pop();
            }
            Component::CurDir => {}
            outro => normalizado.push(outro),
        }
    }
    normalizado
}

pub fn caminho_relativo_ao_projeto(diretorio_projeto: &Path, caminho_arquivo: &str) -> Option<String> {
    let base = normalizar(&std::env::current_dir().unwrap_or_default().join(diretorio_projeto));
    let caminho_absoluto = normalizar(&base.join(caminho_arquivo));
    let caminho_relativo = caminho_absoluto.strip_prefix(&base).ok()?; // fora do repositório
    Some(caminho_relativo.to_string_lossy().replace('\\', "/"))
}

pub fn caminho_eh_protegido(caminho_relativo: &str) -> bool {
    CAMINHOS_PROTEGIDOS
        .iter()
        .any(|protegido| caminho_relativo == protegido.trim_end_matches('/') || caminho_relativo.starts_with(protegido))
}

pub fn caminho_eh_livre_sem_issue(caminho_relativo: &str, ajustes: &AjustesDoProjeto) -> bool {
    // Handoffs de sessão satélite (/voltar-a-principal): HANDOFF-<projeto>.md na raiz do repositório.
    let handoff_na_raiz = caminho_relativo.starts_with("HANDOFF-")
        && caminho_relativo.ends_with(".md")
        && !caminho_relativo.contains('/')
        && caminho_relativo.len() > "HANDOFF-.md".len();
    handoff_na_raiz
        || CAMINHOS_LIVRES_SEM_ISSUE
            .iter()
            .copied()
            .chain(ajustes.caminhos_livres.iter().map(String::as_str))
            .any(|livre| {
                if livre.ends_with('/') {
                    caminho_relativo.starts_with(livre)
                } else {
                    caminho_relativo == livre
                }
            })
}

// ---------- contexto ----------

// Tokens de entrada da última resposta do modelo, lidos do transcript (JSONL).
pub fn medir_tokens_de_contexto(caminho_transcript: &str) -> u64 {
    let Ok(mut arquivo) = fs::File::open(caminho_transcript) else { return 0 };
    let tamanho_arquivo = arquivo.metadata().map(|dados| dados.len()).unwrap_or(0);
    let tamanho_leitura = tamanho_arquivo.min(4 * 1024 * 1024);
    let mut final_do_arquivo = vec![0u8; tamanho_leitura as usize];
    if arquivo.seek(SeekFrom::Start(tamanho_arquivo - tamanho_leitura)).is_err() || arquivo.read_exact(&mut final_do_arquivo).is_err() {
        return 0;
    }
    for linha in String::from_utf8_lossy(&final_do_arquivo).split('\n').rev() {
        if !linha.contains("\"usage\"") {
            continue;
        }
        let Ok(registro) = serde_json::from_str::<Value>(linha) else { continue }; // linha truncada no início do buffer
        let uso = &registro["message"]["usage"];
        if !uso.is_object() {
            continue;
        }
        let campo = |nome: &str| uso[nome].as_u64().unwrap_or(0);
        return campo("input_tokens") + campo("cache_creation_input_tokens") + campo("cache_read_input_tokens");
    }
    0
}

// Data e hora em UTC no formato do toISOString do JavaScript (2026-10-23T18:00:00.000Z).
pub fn agora_iso() -> String {
    let duracao = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let segundos_totais = duracao.as_secs() as i64;
    let (dias, segundos_do_dia) = (segundos_totais.div_euclid(86_400), segundos_totais.rem_euclid(86_400));
    // Algoritmo de dias para data civil (Howard Hinnant).
    let deslocado = dias + 719_468;
    let era = deslocado.div_euclid(146_097);
    let dia_da_era = deslocado - era * 146_097;
    let ano_da_era = (dia_da_era - dia_da_era / 1460 + dia_da_era / 36_524 - dia_da_era / 146_096) / 365;
    let dia_do_ano = dia_da_era - (365 * ano_da_era + ano_da_era / 4 - ano_da_era / 100);
    let mes_deslocado = (5 * dia_do_ano + 2) / 153;
    let dia = dia_do_ano - (153 * mes_deslocado + 2) / 5 + 1;
    let mes = if mes_deslocado < 10 { mes_deslocado + 3 } else { mes_deslocado - 9 };
    let ano = ano_da_era + era * 400 + i64::from(mes <= 2);
    format!(
        "{ano:04}-{mes:02}-{dia:02}T{:02}:{:02}:{:02}.{:03}Z",
        segundos_do_dia / 3600,
        segundos_do_dia % 3600 / 60,
        segundos_do_dia % 60,
        duracao.subsec_millis()
    )
}
