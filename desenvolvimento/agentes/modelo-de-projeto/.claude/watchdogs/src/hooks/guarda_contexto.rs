// Watchdog PreToolUse (Read | Grep | Glob): código só se lê depois de `rodar contexto` (ADR 0022).
// Uma consulta por sessão e por subagente libera a leitura; resposta vazia também libera. A primeira
// leitura fora do que a consulta citou recebe um aviso, sem bloquear.
// O guarda-bash registra a consulta e usa as mesmas regras para cat, sed e grep.

use crate::comum::*;
use crate::contexto::{arquivos_citados, montar_contextos, LIMITE_PADRAO_POR_SECAO};
use serde_json::{json, Value};
use std::path::Path;

const NOME_WATCHDOG: &str = "guarda-contexto";

pub const COMANDO_CONTEXTO: &str = "sh .claude/watchdogs/rodar contexto";

// Documento e configuração não são código: passam em qualquer pasta.
const EXTENSOES_LIVRES: [&str; 11] = ["md", "txt", "json", "yaml", "yml", "toml", "lock", "ini", "cfg", "conf", "env"];

fn nome_tem_extensao_livre(caminho: &str) -> bool {
    let nome_arquivo = caminho.rsplit('/').next().unwrap_or(caminho);
    let extensao = nome_arquivo.rsplit_once('.').map(|(_, extensao)| extensao).unwrap_or("");
    EXTENSOES_LIVRES.contains(&extensao) || nome_arquivo.contains(".config.") || nome_arquivo.starts_with('.')
}

// `caminho_relativo` vazio é a raiz do projeto; pasta chega terminada em "/". O .claude/ é a
// infraestrutura do método (skills, watchdogs, agentes), não código do projeto: a consulta não o explica.
pub fn caminho_exige_consulta(caminho_relativo: &str, ajustes: &AjustesDoProjeto) -> bool {
    caminho_relativo.is_empty()
        || !(caminho_relativo.starts_with(".claude/") || caminho_eh_livre_sem_issue(caminho_relativo, ajustes) || nome_tem_extensao_livre(caminho_relativo))
}

pub fn agente_ja_consultou(estado_sessao: &Value, chave_agente: &str) -> bool {
    estado_sessao["consultasDeContexto"][chave_agente].is_object()
}

pub fn motivo_sem_consulta(caminho_relativo: &str) -> String {
    let consulta_sugerida = if caminho_relativo.is_empty() || caminho_relativo.ends_with('/') {
        "<símbolo | termo | caminho | #issue>"
    } else {
        caminho_relativo
    };
    format!(
        "ler \"{}\" exige a consulta de contexto antes. Rode: {COMANDO_CONTEXTO} {consulta_sugerida}\nLeia a resposta e abra só os trechos citados; vazia ou insuficiente, leia direto.",
        if caminho_relativo.is_empty() { "." } else { caminho_relativo }
    )
}

// Soma a consulta ao que o agente já consultou. O aviso volta a valer para a nova lista de citados.
pub fn registrar_consulta(diretorio_projeto: &Path, id_sessao: &str, chave_agente: &str, consulta: &str) {
    let citados_agora: Vec<String> = montar_contextos(diretorio_projeto, consulta, LIMITE_PADRAO_POR_SECAO)
        .iter()
        .flat_map(arquivos_citados)
        .collect();
    let mut estado_sessao = ler_estado_sessao(diretorio_projeto, id_sessao);
    let registro_anterior = &estado_sessao["consultasDeContexto"][chave_agente];
    let mut consultas: Vec<Value> = registro_anterior["consultas"].as_array().cloned().unwrap_or_default();
    let mut citados: Vec<Value> = registro_anterior["citados"].as_array().cloned().unwrap_or_default();
    consultas.push(json!(consulta));
    for arquivo_citado in citados_agora {
        if !citados.iter().any(|citado| citado.as_str() == Some(arquivo_citado.as_str())) {
            citados.push(json!(arquivo_citado));
        }
    }
    estado_sessao["consultasDeContexto"][chave_agente] = json!({ "consultas": consultas, "citados": citados, "avisado": false });
    gravar_estado_sessao(diretorio_projeto, id_sessao, &estado_sessao);
}

// Alvo da ferramenta, relativo ao projeto. None: fora do repositório.
fn alvo_da_ferramenta(diretorio_projeto: &Path, parametros_ferramenta: &Value) -> Option<String> {
    let caminho_pedido = parametros_ferramenta["file_path"]
        .as_str()
        .or_else(|| parametros_ferramenta["path"].as_str())
        .unwrap_or("");
    let caminho_relativo = caminho_relativo_ao_projeto(diretorio_projeto, caminho_pedido)?;
    let eh_pasta = !caminho_relativo.is_empty() && diretorio_projeto.join(&caminho_relativo).is_dir();
    Some(if eh_pasta { format!("{caminho_relativo}/") } else { caminho_relativo })
}

// O Glob sem `path` procura a partir da parte fixa do padrão: "docs/**/*.md" fica em docs/.
fn prefixo_fixo_do_padrao(padrao_glob: &str) -> &str {
    let antes_do_curinga = padrao_glob.split(['*', '?', '[', '{']).next().unwrap_or("");
    antes_do_curinga.rsplit_once('/').map(|(pasta, _)| pasta).unwrap_or("")
}

pub fn rodar() -> i32 {
    let entrada_hook = ler_entrada_do_hook();
    if watchdogs_desligados() {
        return 0;
    }
    let diretorio_projeto = diretorio_do_projeto(&entrada_hook);
    if etapa_de_adocao(&diretorio_projeto) == "observacao" {
        return 0;
    }
    let parametros_ferramenta = &entrada_hook["tool_input"];
    let padrao_glob = texto(&parametros_ferramenta["pattern"]);
    let eh_glob = texto(&entrada_hook["tool_name"]) == "Glob";
    let filtro_de_arquivos = if eh_glob { padrao_glob } else { texto(&parametros_ferramenta["glob"]) };
    if !filtro_de_arquivos.is_empty() && nome_tem_extensao_livre(filtro_de_arquivos) {
        return 0;
    }
    let Some(mut caminho_relativo) = alvo_da_ferramenta(&diretorio_projeto, parametros_ferramenta) else {
        return 0;
    };
    if eh_glob && caminho_relativo.is_empty() {
        let prefixo = prefixo_fixo_do_padrao(padrao_glob);
        if !prefixo.is_empty() {
            caminho_relativo = format!("{prefixo}/");
        }
    }
    let ajustes = ler_ajustes_do_projeto(&diretorio_projeto);
    if !caminho_exige_consulta(&caminho_relativo, &ajustes) {
        return 0;
    }

    let id_sessao = texto(&entrada_hook["session_id"]);
    let chave_agente = chave_do_agente(&entrada_hook);
    let mut estado_sessao = ler_estado_sessao(&diretorio_projeto, id_sessao);
    if !agente_ja_consultou(&estado_sessao, &chave_agente) {
        bloquear_ferramenta(NOME_WATCHDOG, &motivo_sem_consulta(&caminho_relativo));
    }

    // Resposta vazia não cita nada: a leitura direta é o caminho previsto, sem aviso.
    let registro = &estado_sessao["consultasDeContexto"][&chave_agente];
    let citados = registro["citados"].as_array().cloned().unwrap_or_default();
    let leitura_de_arquivo = !parametros_ferramenta["file_path"].is_null();
    let fora_dos_citados = !citados.is_empty() && !citados.iter().any(|citado| citado.as_str() == Some(caminho_relativo.as_str()));
    if leitura_de_arquivo && fora_dos_citados && registro["avisado"] != json!(true) {
        estado_sessao["consultasDeContexto"][&chave_agente]["avisado"] = json!(true);
        gravar_estado_sessao(&diretorio_projeto, id_sessao, &estado_sessao);
        emitir_contexto_adicional(
            "PreToolUse",
            &format!("🪙 [watchdog {NOME_WATCHDOG}] \"{caminho_relativo}\" está fora do que a consulta citou. Se a resposta foi insuficiente, siga; senão, volte aos trechos citados."),
        );
    }
    0
}
