// Custo em tokens por issue: soma o uso de cada mensagem do modelo, de todas as sessões e subagentes
// em que a issue esteve ativa, em .claude/state/custos/issue-<n>.json. Regras: docs/agents/watchdogs.md
// Uso: .claude/watchdogs/rodar custo [<número da issue>]

use crate::comum::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

// Orçamento padrão por issue, em tokens equivalentes de entrada. O humano troca em
// .claude/metodo.json ("orcamentoPorIssue") depois de calibrar com as issues já medidas.
pub const ORCAMENTO_PADRAO_POR_ISSUE: u64 = 3_000_000;

// Peso de cada tipo de token em relação ao token de entrada, pela tabela de preços da Anthropic
// (iguais em todos os modelos atuais): escrita de cache 5 min 1,25x, 1 h 2x, leitura 0,1x, saída 5x.
const PESO_ESCRITA_CACHE_5M: f64 = 1.25;
const PESO_ESCRITA_CACHE_1H: f64 = 2.0;
const PESO_LEITURA_CACHE: f64 = 0.1;
const PESO_SAIDA: f64 = 5.0;

#[derive(Default, Clone, Copy)]
pub struct UsoDeTokens {
    pub entrada: u64,
    pub escrita_cache_5m: u64,
    pub escrita_cache_1h: u64,
    pub leitura_cache: u64,
    pub saida: u64,
}

impl UsoDeTokens {
    fn do_registro(uso: &Value) -> Self {
        let campo = |valor: &Value| valor.as_u64().unwrap_or(0);
        let escrita_total = campo(&uso["cache_creation_input_tokens"]);
        let escrita_1h = campo(&uso["cache_creation"]["ephemeral_1h_input_tokens"]).min(escrita_total);
        UsoDeTokens {
            entrada: campo(&uso["input_tokens"]),
            escrita_cache_5m: escrita_total - escrita_1h,
            escrita_cache_1h: escrita_1h,
            leitura_cache: campo(&uso["cache_read_input_tokens"]),
            saida: campo(&uso["output_tokens"]),
        }
    }

    pub fn equivalentes_de_entrada(&self) -> u64 {
        (self.entrada as f64
            + self.escrita_cache_5m as f64 * PESO_ESCRITA_CACHE_5M
            + self.escrita_cache_1h as f64 * PESO_ESCRITA_CACHE_1H
            + self.leitura_cache as f64 * PESO_LEITURA_CACHE
            + self.saida as f64 * PESO_SAIDA)
            .round() as u64
    }
}

// Lê o transcript a partir de onde a última contagem parou e devolve o uso por modelo e o novo ponto
// de parada. Só conta linhas completas; cada mensagem se repete em várias linhas com o mesmo id e o
// mesmo uso, então vale uma vez por id.
pub fn ler_uso_novo(caminho_transcript: &Path, ponto_de_parada: u64) -> (HashMap<String, UsoDeTokens>, u64) {
    let mut uso_por_modelo: HashMap<String, UsoDeTokens> = HashMap::new();
    let Ok(mut arquivo) = fs::File::open(caminho_transcript) else { return (uso_por_modelo, ponto_de_parada) };
    let tamanho_arquivo = arquivo.metadata().map(|dados| dados.len()).unwrap_or(0);
    // Transcript menor que o ponto de parada foi reescrito (compactação): recomeça do início.
    let inicio_leitura = if ponto_de_parada > tamanho_arquivo { 0 } else { ponto_de_parada };
    let mut trecho_novo = Vec::new();
    if arquivo.seek(SeekFrom::Start(inicio_leitura)).is_err() || arquivo.read_to_end(&mut trecho_novo).is_err() {
        return (uso_por_modelo, ponto_de_parada);
    }
    let Some(fim_da_ultima_linha) = trecho_novo.iter().rposition(|byte| *byte == b'\n') else {
        return (uso_por_modelo, inicio_leitura);
    };
    let mut uso_por_mensagem: HashMap<String, (String, UsoDeTokens)> = HashMap::new();
    for linha in String::from_utf8_lossy(&trecho_novo[..fim_da_ultima_linha]).split('\n') {
        if !linha.contains("\"usage\"") {
            continue;
        }
        let Ok(registro) = serde_json::from_str::<Value>(linha) else { continue };
        let mensagem = &registro["message"];
        if !mensagem["usage"].is_object() {
            continue;
        }
        let id_mensagem = texto(&mensagem["id"]).to_string();
        let chave_mensagem = if id_mensagem.is_empty() { format!("linha-{}", uso_por_mensagem.len()) } else { id_mensagem };
        let modelo = texto(&mensagem["model"]);
        let nome_modelo = if modelo.is_empty() { "desconhecido" } else { modelo };
        uso_por_mensagem.insert(chave_mensagem, (nome_modelo.to_string(), UsoDeTokens::do_registro(&mensagem["usage"])));
    }
    for (nome_modelo, uso_da_mensagem) in uso_por_mensagem.into_values() {
        let acumulado = uso_por_modelo.entry(nome_modelo).or_default();
        acumulado.entrada += uso_da_mensagem.entrada;
        acumulado.escrita_cache_5m += uso_da_mensagem.escrita_cache_5m;
        acumulado.escrita_cache_1h += uso_da_mensagem.escrita_cache_1h;
        acumulado.leitura_cache += uso_da_mensagem.leitura_cache;
        acumulado.saida += uso_da_mensagem.saida;
    }
    (uso_por_modelo, inicio_leitura + fim_da_ultima_linha as u64 + 1)
}

fn caminho_custo_da_issue(diretorio_projeto: &Path, numero_issue: i64) -> PathBuf {
    diretorio_projeto.join(".claude").join("state").join("custos").join(format!("issue-{numero_issue}.json"))
}

pub fn ler_custo_da_issue(diretorio_projeto: &Path, numero_issue: i64) -> Value {
    fs::read_to_string(caminho_custo_da_issue(diretorio_projeto, numero_issue))
        .ok()
        .and_then(|texto_custo| serde_json::from_str::<Value>(&texto_custo).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({ "numero": numero_issue, "tokensEquivalentes": 0, "porModelo": {}, "sessoes": [] }))
}

fn gravar_custo_da_issue(diretorio_projeto: &Path, numero_issue: i64, custo_issue: &Value) {
    let caminho_custo = caminho_custo_da_issue(diretorio_projeto, numero_issue);
    if let Some(pasta) = caminho_custo.parent() {
        let _ = fs::create_dir_all(pasta);
    }
    let _ = fs::write(&caminho_custo, serde_json::to_string_pretty(custo_issue).unwrap_or_default());
}

pub fn orcamento_por_issue(diretorio_projeto: &Path) -> u64 {
    fs::read_to_string(diretorio_projeto.join(".claude").join("metodo.json"))
        .ok()
        .and_then(|texto_metodo| serde_json::from_str::<Value>(&texto_metodo).ok())
        .and_then(|metodo| metodo["orcamentoPorIssue"].as_u64())
        .filter(|orcamento| *orcamento > 0)
        .unwrap_or(ORCAMENTO_PADRAO_POR_ISSUE)
}

pub fn em_milhares(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else {
        format!("{}k", (tokens as f64 / 1000.0).round() as u64)
    }
}

pub enum ResultadoDaContagem {
    SemIssueAtiva,
    DentroDoOrcamento,
    PassouDoOrcamentoAgora { numero_issue: i64, tokens_equivalentes: u64, orcamento: u64 },
}

// Soma o uso novo do transcript na issue ativa de quem está agindo (sessão ou subagente).
pub fn contar_no_transcript(entrada_hook: &Value, caminho_transcript: &str) -> ResultadoDaContagem {
    let diretorio_projeto = diretorio_do_projeto(entrada_hook);
    let id_sessao = texto(&entrada_hook["session_id"]);
    let mut estado_sessao = ler_estado_sessao(&diretorio_projeto, id_sessao);
    let issue_ativa = estado_sessao["issuesAtivas"][chave_do_agente(entrada_hook)].clone();
    let Some(numero_issue) = issue_ativa["numero"].as_i64() else { return ResultadoDaContagem::SemIssueAtiva };
    if caminho_transcript.is_empty() {
        return ResultadoDaContagem::DentroDoOrcamento;
    }

    // O ponto de parada fica por transcript: a sessão e cada subagente têm o seu arquivo.
    let ponto_de_parada = estado_sessao["custoLidoAte"][caminho_transcript].as_u64().unwrap_or(0);
    let (uso_por_modelo, novo_ponto_de_parada) = ler_uso_novo(Path::new(caminho_transcript), ponto_de_parada);
    if !estado_sessao["custoLidoAte"].is_object() {
        estado_sessao["custoLidoAte"] = json!({});
    }
    estado_sessao["custoLidoAte"][caminho_transcript] = novo_ponto_de_parada.into();
    gravar_estado_sessao(&diretorio_projeto, id_sessao, &estado_sessao);
    if uso_por_modelo.is_empty() {
        return ResultadoDaContagem::DentroDoOrcamento;
    }

    let mut custo_issue = ler_custo_da_issue(&diretorio_projeto, numero_issue);
    if let Some(titulo_issue) = issue_ativa["titulo"].as_str() {
        custo_issue["titulo"] = titulo_issue.into();
    }
    let tokens_antes = custo_issue["tokensEquivalentes"].as_u64().unwrap_or(0);
    let mut tokens_novos = 0;
    if !custo_issue["porModelo"].is_object() {
        custo_issue["porModelo"] = json!({});
    }
    for (nome_modelo, uso_novo) in &uso_por_modelo {
        tokens_novos += uso_novo.equivalentes_de_entrada();
        let registro_modelo = &mut custo_issue["porModelo"][nome_modelo];
        for (campo, valor_novo) in [
            ("entrada", uso_novo.entrada),
            ("escritaCache5m", uso_novo.escrita_cache_5m),
            ("escritaCache1h", uso_novo.escrita_cache_1h),
            ("leituraCache", uso_novo.leitura_cache),
            ("saida", uso_novo.saida),
            ("tokensEquivalentes", uso_novo.equivalentes_de_entrada()),
        ] {
            registro_modelo[campo] = (registro_modelo[campo].as_u64().unwrap_or(0) + valor_novo).into();
        }
    }
    let tokens_depois = tokens_antes + tokens_novos;
    custo_issue["tokensEquivalentes"] = tokens_depois.into();
    if !custo_issue["sessoes"].is_array() {
        custo_issue["sessoes"] = json!([]);
    }
    let sessoes_da_issue = custo_issue["sessoes"].as_array_mut().expect("sessoes é lista");
    if !sessoes_da_issue.iter().any(|sessao| texto(sessao) == id_sessao) {
        sessoes_da_issue.push(id_sessao.into());
    }
    custo_issue["atualizadoEm"] = agora_iso().into();
    gravar_custo_da_issue(&diretorio_projeto, numero_issue, &custo_issue);

    let orcamento = orcamento_por_issue(&diretorio_projeto);
    if tokens_antes < orcamento && tokens_depois >= orcamento {
        return ResultadoDaContagem::PassouDoOrcamentoAgora { numero_issue, tokens_equivalentes: tokens_depois, orcamento };
    }
    ResultadoDaContagem::DentroDoOrcamento
}

// Relatório para o humano e para a Evidence da PR.
pub fn rodar(argumentos: &[String]) -> i32 {
    let diretorio_projeto = diretorio_do_projeto(&Value::Null);
    let pasta_custos = diretorio_projeto.join(".claude").join("state").join("custos");
    let numeros_pedidos: Vec<i64> = match argumentos.first() {
        Some(numero_texto) => match numero_texto.trim_start_matches('#').parse::<i64>() {
            Ok(numero_issue) => vec![numero_issue],
            Err(_) => {
                eprintln!("uso: .claude/watchdogs/rodar custo [<número da issue>]");
                return 1;
            }
        },
        None => {
            let mut numeros_medidos: Vec<i64> = fs::read_dir(&pasta_custos)
                .into_iter()
                .flatten()
                .flatten()
                .filter_map(|entrada_pasta| {
                    let nome_arquivo = entrada_pasta.file_name().to_string_lossy().to_string();
                    nome_arquivo.strip_prefix("issue-")?.strip_suffix(".json")?.parse().ok()
                })
                .collect();
            numeros_medidos.sort_unstable();
            numeros_medidos
        }
    };
    if numeros_pedidos.is_empty() {
        println!("Nenhuma issue medida ainda. O custo é somado a cada fim de turno com uma issue ativa.");
        return 0;
    }
    let orcamento = orcamento_por_issue(&diretorio_projeto);
    for numero_issue in numeros_pedidos {
        let custo_issue = ler_custo_da_issue(&diretorio_projeto, numero_issue);
        let tokens_equivalentes = custo_issue["tokensEquivalentes"].as_u64().unwrap_or(0);
        let quantidade_sessoes = custo_issue["sessoes"].as_array().map_or(0, Vec::len);
        let percentual = tokens_equivalentes * 100 / orcamento;
        println!(
            "Issue #{numero_issue} {}: {} tokens equivalentes de entrada ({percentual}% do orçamento de {}), {quantidade_sessoes} sessão(ões)",
            texto(&custo_issue["titulo"]),
            em_milhares(tokens_equivalentes),
            em_milhares(orcamento)
        );
        for (nome_modelo, registro_modelo) in custo_issue["porModelo"].as_object().into_iter().flatten() {
            let campo = |nome: &str| em_milhares(registro_modelo[nome].as_u64().unwrap_or(0));
            println!(
                "  {nome_modelo}: {} equivalentes · entrada {} · escrita de cache {} (1 h {}) · leitura de cache {} · saída {}",
                campo("tokensEquivalentes"),
                campo("entrada"),
                em_milhares(registro_modelo["escritaCache5m"].as_u64().unwrap_or(0) + registro_modelo["escritaCache1h"].as_u64().unwrap_or(0)),
                campo("escritaCache1h"),
                campo("leituraCache"),
                campo("saida")
            );
        }
    }
    0
}
