// Validador de regras de código (ADR 0010 do método): lê docs/regras-de-codigo.json, roda a
// verificação de cada regra e confere a cobertura dos 22 princípios em docs/regras-de-codigo.md.
// O motor é do método e fica protegido; o projeto refina só o JSON e os arquivos que ele aponta.
//
// Uso: .claude/watchdogs/rodar validar [--config <json>] [--md <md>] [--base <ref>] [--completo] [--so-cobertura] [--json]
// Com --base, lê só os arquivos alterados desde a base (CI e revisão); sem ela, ou com --completo, o
// repositório inteiro (auditoria). Saída: 0 aprovado, 1 violação, 2 configuração inválida. A última
// linha é "VALIDADOR: APROVADO" ou "VALIDADOR: REPROVADO".

use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::LazyLock;

// Expressões usadas por nome declarado ou por linha: compiladas uma vez só.
static CAMEL_CASE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([a-z0-9])([A-Z])").expect("regex válida"));
static SEPARADORES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\s_\-]+").expect("regex válida"));
static DECLARACAO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:function|class|const|let|var|def|type|interface|enum|struct|fn|func|record|trait|object)\s+([A-Za-zÀ-ÿ_][A-Za-z0-9_À-ÿ]*)")
        .expect("regex válida")
});

pub const PRINCIPIOS: [&str; 22] = [
    "E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8", "P1", "P2", "P3", "P4", "P5", "P6", "P7", "P8", "P9", "S", "O", "L", "I", "D",
];
pub const DECISOES: [&str; 4] = ["adotado", "adaptado", "afastado", "garantido pelo lint"];
pub const TIPOS: [&str; 8] = [
    "termo-evitado",
    "teste-de-contrato-por-port",
    "lint-cobre",
    "todo-sem-issue",
    "mensagem-de-commit",
    "comando",
    "ast-grep",
    "revisor",
];

#[derive(Debug, Clone, PartialEq)]
pub struct Violacao {
    pub arquivo: Option<String>,
    pub linha: Option<u64>,
    pub mensagem: String,
}

impl Violacao {
    fn so_mensagem(mensagem: impl Into<String>) -> Self {
        Violacao {
            arquivo: None,
            linha: None,
            mensagem: mensagem.into(),
        }
    }
    fn em(arquivo: &str, linha: Option<u64>, mensagem: impl Into<String>) -> Self {
        Violacao {
            arquivo: Some(arquivo.to_string()),
            linha,
            mensagem: mensagem.into(),
        }
    }
}

#[derive(Debug)]
pub struct ErroDeConfiguracao(pub String);

fn erro<T>(mensagem: impl Into<String>) -> Result<T, ErroDeConfiguracao> {
    Err(ErroDeConfiguracao(mensagem.into()))
}

// ---------- arquivos e globs ----------

pub fn glob_para_regex(padrao_glob: &str) -> Regex {
    let caracteres: Vec<char> = padrao_glob.chars().collect();
    let mut expressao = String::from("^");
    let mut indice = 0;
    while indice < caracteres.len() {
        let caractere = caracteres[indice];
        if caractere == '*' && caracteres.get(indice + 1) == Some(&'*') {
            let seguido_de_barra = caracteres.get(indice + 2) == Some(&'/');
            expressao.push_str(if seguido_de_barra { "(?:.*/)?" } else { ".*" });
            indice += if seguido_de_barra { 3 } else { 2 };
            continue;
        }
        match caractere {
            '*' => expressao.push_str("[^/]*"),
            '?' => expressao.push_str("[^/]"),
            '{' => {
                let fim = caracteres[indice..]
                    .iter()
                    .position(|letra| *letra == '}')
                    .map(|deslocamento| indice + deslocamento)
                    .unwrap_or(caracteres.len() - 1);
                let opcoes: String = caracteres[indice + 1..fim].iter().collect();
                expressao.push_str(&format!("(?:{})", opcoes.split(',').map(regex::escape).collect::<Vec<_>>().join("|")));
                indice = fim;
            }
            outro => expressao.push_str(&regex::escape(&outro.to_string())),
        }
        indice += 1;
    }
    expressao.push('$');
    Regex::new(&expressao).unwrap_or_else(|_| Regex::new("^$").expect("regex válida"))
}

pub fn casa_algum(caminho_relativo: &str, padroes_glob: &[String]) -> bool {
    padroes_glob.iter().any(|padrao| glob_para_regex(padrao).is_match(caminho_relativo))
}

// Globs compilados uma vez, para filtrar listas longas de arquivos.
struct Globs(Vec<Regex>);

impl Globs {
    fn de(padroes_glob: &[String]) -> Self {
        Globs(padroes_glob.iter().map(|padrao| glob_para_regex(padrao)).collect())
    }
    fn casa(&self, caminho_relativo: &str) -> bool {
        self.0.iter().any(|regex_do_glob| regex_do_glob.is_match(caminho_relativo))
    }
}

fn git(raiz: &Path, argumentos: &[&str]) -> Option<String> {
    let saida = Command::new("git")
        .args(argumentos)
        .current_dir(raiz)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    saida.status.success().then(|| String::from_utf8_lossy(&saida.stdout).to_string())
}

fn linhas_existentes(raiz: &Path, saida: &str) -> Vec<String> {
    saida
        .lines()
        .filter(|linha| !linha.is_empty() && raiz.join(linha).exists())
        .map(String::from)
        .collect()
}

fn listar_arquivos_do_repositorio(raiz: &Path) -> Result<Vec<String>, ErroDeConfiguracao> {
    match git(raiz, &["ls-files", "--cached", "--others", "--exclude-standard"]) {
        Some(saida) => Ok(linhas_existentes(raiz, &saida)),
        None => erro("o validador roda dentro de um repositório git (git ls-files falhou)"),
    }
}

// Arquivos criados ou mudados desde a base, commitados ou não. None quando a base não existe.
pub fn arquivos_alterados_desde(raiz: &Path, base: &str) -> Option<Vec<String>> {
    git(raiz, &["rev-parse", "--verify", "--quiet", base])?;
    let intervalo = format!("{base}...HEAD");
    let listas = [
        git(raiz, &["diff", "--name-only", "--diff-filter=ACMR", &intervalo]).unwrap_or_default(),
        git(raiz, &["diff", "--name-only", "--diff-filter=ACMR", "HEAD"]).unwrap_or_default(),
        git(raiz, &["ls-files", "--others", "--exclude-standard"]).unwrap_or_default(),
    ];
    let mut vistos = HashSet::new();
    Some(
        linhas_existentes(raiz, &listas.join("\n"))
            .into_iter()
            .filter(|caminho| vistos.insert(caminho.clone()))
            .collect(),
    )
}

fn ler_texto(raiz: &Path, caminho_relativo: &str) -> String {
    fs::read_to_string(raiz.join(caminho_relativo)).unwrap_or_default()
}

// ---------- cobertura: os 22 princípios decididos no .md e ligados às regras do .json ----------

pub struct LinhaDeCobertura {
    pub decisao: String,
    pub onde_ou_porque: String,
}

pub fn ler_tabela_de_cobertura(texto_md: &str) -> Vec<(String, LinhaDeCobertura)> {
    let mut linhas_da_tabela: Vec<(String, LinhaDeCobertura)> = Vec::new();
    for linha in texto_md.lines() {
        let partes: Vec<&str> = linha.split('|').collect();
        if partes.len() < 5 {
            continue;
        }
        let celulas: Vec<&str> = partes[1..partes.len() - 1].iter().map(|celula| celula.trim()).collect();
        if !PRINCIPIOS.contains(&celulas[0]) {
            continue;
        }
        let nova_linha = LinhaDeCobertura {
            decisao: celulas[1].to_lowercase(),
            onde_ou_porque: celulas[2].to_string(),
        };
        match linhas_da_tabela.iter_mut().find(|(principio, _)| principio == celulas[0]) {
            Some((_, existente)) => *existente = nova_linha,
            None => linhas_da_tabela.push((celulas[0].to_string(), nova_linha)),
        }
    }
    linhas_da_tabela
}

pub struct Regra {
    pub id: String,
    pub principio: String,
    pub verificacao: Map<String, Value>,
}

pub fn checar_cobertura(texto_md: &str, regras: &[Regra]) -> Vec<Violacao> {
    let mut violacoes = Vec::new();
    let tabela = ler_tabela_de_cobertura(texto_md);
    let ids_das_regras: HashSet<&str> = regras.iter().map(|regra| regra.id.as_str()).collect();
    let mut regras_citadas: HashSet<String> = HashSet::new();
    let id_de_regra = Regex::new(r"\bR\d+\b").expect("regex válida");
    for principio in PRINCIPIOS {
        let Some((_, linha)) = tabela.iter().find(|(nome, _)| nome == principio) else {
            violacoes.push(Violacao::so_mensagem(format!("{principio} sem linha na tabela de cobertura")));
            continue;
        };
        if !DECISOES.contains(&linha.decisao.as_str()) {
            violacoes.push(Violacao::so_mensagem(format!(
                "{principio}: decisão \"{}\" não é uma de: {}",
                linha.decisao,
                DECISOES.join(", ")
            )));
            continue;
        }
        if linha.decisao == "afastado" {
            if linha.onde_ou_porque.is_empty() {
                violacoes.push(Violacao::so_mensagem(format!("{principio} afastado sem o porquê")));
            }
            continue;
        }
        let ids_na_linha: Vec<&str> = id_de_regra.find_iter(&linha.onde_ou_porque).map(|encontrado| encontrado.as_str()).collect();
        if ids_na_linha.is_empty() {
            violacoes.push(Violacao::so_mensagem(format!(
                "{principio} {} sem regra (R<n>) na coluna \"Onde\"",
                linha.decisao
            )));
        }
        for id_citado in ids_na_linha {
            regras_citadas.insert(id_citado.to_string());
            if !ids_das_regras.contains(id_citado) {
                violacoes.push(Violacao::so_mensagem(format!(
                    "{principio} cita {id_citado}, que não está em regras-de-codigo.json"
                )));
            } else if let Some(regra_citada) = regras.iter().find(|regra| regra.id == id_citado).filter(|regra| regra.principio != principio) {
                violacoes.push(Violacao::so_mensagem(format!(
                    "{id_citado} nasce de {} no .json, mas a tabela a cita em {principio}",
                    regra_citada.principio
                )));
            }
        }
    }
    for regra in regras {
        if !regras_citadas.contains(&regra.id) {
            violacoes.push(Violacao::so_mensagem(format!(
                "{} está no .json e nenhuma linha da tabela de cobertura a cita",
                regra.id
            )));
        }
    }
    violacoes
}

// ---------- catálogo de checagens ----------

pub struct Contexto {
    pub raiz: PathBuf,
    pub projeto: Value,
    pub base: Option<String>,
    pub avisos: Vec<String>,
    pub arquivos: Vec<String>,
    pub alterados: Option<Vec<String>>,
}

fn lista_de_texto(valor: &Value) -> Vec<String> {
    match valor {
        Value::String(texto) => vec![texto.clone()],
        Value::Array(itens) => itens.iter().filter_map(|item| item.as_str().map(String::from)).collect(),
        _ => Vec::new(),
    }
}

fn exigir(parametros: &Map<String, Value>, nomes: &[&str], tipo: &str) -> Result<(), ErroDeConfiguracao> {
    for nome in nomes {
        if !parametros.contains_key(*nome) {
            return erro(format!("verificação \"{tipo}\" exige o parâmetro \"{nome}\""));
        }
    }
    Ok(())
}

// Os arquivos de código que a checagem lê: todos, ou só os alterados no modo incremental.
fn arquivos_de_codigo(contexto: &Contexto, parametros: &Map<String, Value>) -> Vec<String> {
    let mut padroes_de_codigo = lista_de_texto(parametros.get("arquivos").unwrap_or(&Value::Null));
    if padroes_de_codigo.is_empty() {
        padroes_de_codigo = lista_de_texto(&contexto.projeto["codigo"]);
    }
    if padroes_de_codigo.is_empty() {
        padroes_de_codigo = vec!["**".into()];
    }
    let mut padroes_ignorados = lista_de_texto(&contexto.projeto["ignorar"]);
    padroes_ignorados.extend(lista_de_texto(parametros.get("ignorar").unwrap_or(&Value::Null)));
    let (globs_de_codigo, globs_ignorados) = (Globs::de(&padroes_de_codigo), Globs::de(&padroes_ignorados));
    let universo = contexto.alterados.as_ref().unwrap_or(&contexto.arquivos);
    universo
        .iter()
        .filter(|caminho| globs_de_codigo.casa(caminho) && !globs_ignorados.casa(caminho))
        .cloned()
        .collect()
}

// Tira acentos e põe em minúsculas, para comparar termos do glossário com nomes de código.
fn sem_acento(texto: &str) -> String {
    texto
        .chars()
        .map(|letra| match letra {
            'á' | 'à' | 'â' | 'ã' | 'ä' | 'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
            'ç' | 'Ç' => 'c',
            'ñ' | 'Ñ' => 'n',
            outra => outra,
        })
        .collect::<String>()
        .to_lowercase()
}

fn quebrar_identificador(identificador: &str) -> Vec<String> {
    let separado = CAMEL_CASE.replace_all(identificador, "$1 $2").to_string();
    SEPARADORES.split(&separado).filter(|parte| !parte.is_empty()).map(sem_acento).collect()
}

pub fn termos_evitados_do_glossario(texto_glossario: &str) -> Vec<String> {
    let linha_evitar = Regex::new(r"^_(?:Evitar|Avoid)_:\s*(.+)$").expect("regex válida");
    let entre_parenteses = Regex::new(r"\(.*?\)").expect("regex válida");
    let mut termos = Vec::new();
    for linha in texto_glossario.lines() {
        let Some(encontrado) = linha_evitar.captures(linha) else { continue };
        for termo_bruto in encontrado[1].split(',') {
            let termo = entre_parenteses.replace_all(termo_bruto, "").replace(['"', '“', '”'], "").trim().to_string();
            if termo.chars().count() >= 3 {
                termos.push(termo);
            }
        }
    }
    termos
}

fn parametro_texto<'a>(parametros: &'a Map<String, Value>, nome: &str) -> Option<&'a str> {
    parametros.get(nome).and_then(Value::as_str)
}

fn escapar_para_shell(caminho: &str) -> String {
    format!("'{}'", caminho.replace('\'', "'\\''"))
}

fn rodar_no_shell(linha_de_comando: &str, raiz: &Path) -> std::io::Result<std::process::Output> {
    if cfg!(windows) {
        Command::new("cmd").args(["/C", linha_de_comando]).current_dir(raiz).output()
    } else {
        Command::new("sh").args(["-c", linha_de_comando]).current_dir(raiz).output()
    }
}

fn ultimas_linhas(texto: &str, quantidade: usize) -> String {
    let linhas: Vec<&str> = texto.trim().split('\n').collect();
    linhas[linhas.len().saturating_sub(quantidade)..].join("\n")
}

pub fn rodar_checagem(tipo: &str, parametros: &Map<String, Value>, contexto: &mut Contexto) -> Result<Vec<Violacao>, ErroDeConfiguracao> {
    let raiz = contexto.raiz.clone();
    Ok(match tipo {
        // E1: nenhum nome declarado usa um termo que o GLOSSARY.md manda evitar.
        "termo-evitado" => {
            let caminho_do_glossario = parametro_texto(parametros, "glossario").unwrap_or("GLOSSARY.md");
            if !raiz.join(caminho_do_glossario).exists() {
                return Ok(vec![Violacao::so_mensagem(format!("glossário {caminho_do_glossario} não existe"))]);
            }
            let ignorados: HashSet<String> = lista_de_texto(parametros.get("ignorarTermos").unwrap_or(&Value::Null))
                .iter()
                .map(|termo| sem_acento(termo))
                .collect();
            let termos: Vec<(String, Vec<String>)> = termos_evitados_do_glossario(&ler_texto(&raiz, caminho_do_glossario))
                .into_iter()
                .filter(|termo| !ignorados.contains(&sem_acento(termo)))
                .map(|termo| {
                    let partes = quebrar_identificador(&termo);
                    (termo, partes)
                })
                .collect();
            let mut violacoes = Vec::new();
            for caminho_relativo in arquivos_de_codigo(contexto, parametros) {
                let nome_do_arquivo = Path::new(&caminho_relativo)
                    .file_stem()
                    .map(|nome| nome.to_string_lossy().to_string())
                    .unwrap_or_default();
                let mut nomes: Vec<(String, Option<u64>)> = vec![(nome_do_arquivo, None)];
                for (indice, texto_da_linha) in ler_texto(&raiz, &caminho_relativo).split('\n').enumerate() {
                    for declarado in DECLARACAO.captures_iter(texto_da_linha) {
                        nomes.push((declarado[1].to_string(), Some(indice as u64 + 1)));
                    }
                }
                for (nome, linha) in nomes {
                    let partes_do_nome = quebrar_identificador(&nome);
                    for (termo, partes) in &termos {
                        let contem = (0..partes_do_nome.len()).any(|inicio| {
                            partes
                                .iter()
                                .enumerate()
                                .all(|(deslocamento, parte)| partes_do_nome.get(inicio + deslocamento) == Some(parte))
                        });
                        if contem && !partes.is_empty() {
                            violacoes.push(Violacao::em(
                                &caminho_relativo,
                                linha,
                                format!("\"{nome}\" usa \"{termo}\", que o glossário manda evitar"),
                            ));
                        }
                    }
                }
            }
            violacoes
        }

        // L: toda port tem um teste de contrato. "{dir}" e "{nome}" vêm do arquivo da port; arquivos de
        // teste na pasta das ports não contam como port.
        "teste-de-contrato-por-port" => {
            exigir(parametros, &["ports", "contrato"], tipo)?;
            let padroes_das_ports = lista_de_texto(&parametros["ports"]);
            let mut nao_sao_ports = lista_de_texto(parametros.get("ignorar").unwrap_or(&Value::Null));
            if nao_sao_ports.is_empty() {
                nao_sao_ports = vec!["**/*.test.*".into(), "**/*.spec.*".into()];
            }
            let modelo_do_contrato = parametro_texto(parametros, "contrato").unwrap_or("");
            let (globs_das_ports, globs_que_nao_sao_ports) = (Globs::de(&padroes_das_ports), Globs::de(&nao_sao_ports));
            let universo = contexto.alterados.as_ref().unwrap_or(&contexto.arquivos);
            universo
                .iter()
                .filter(|caminho| globs_das_ports.casa(caminho) && !globs_que_nao_sao_ports.casa(caminho))
                .filter_map(|caminho_da_port| {
                    let caminho = Path::new(caminho_da_port);
                    let pasta = caminho
                        .parent()
                        .map(|pasta| pasta.to_string_lossy().to_string())
                        .filter(|pasta| !pasta.is_empty())
                        .unwrap_or_else(|| ".".into());
                    let nome_da_port = caminho.file_stem().map(|nome| nome.to_string_lossy().to_string()).unwrap_or_default();
                    let caminho_do_contrato = modelo_do_contrato.replace("{dir}", &pasta).replace("{nome}", &nome_da_port);
                    (!raiz.join(&caminho_do_contrato).exists())
                        .then(|| Violacao::em(caminho_da_port, None, format!("port sem teste de contrato em {caminho_do_contrato}")))
                })
                .collect()
        }

        // "garantido pelo lint": a configuração do lint tem as regras que o projeto delegou a ele.
        "lint-cobre" => {
            exigir(parametros, &["config", "regras"], tipo)?;
            let caminhos_de_config = lista_de_texto(&parametros["config"]);
            let existentes: Vec<&String> = caminhos_de_config.iter().filter(|caminho| raiz.join(caminho).exists()).collect();
            if existentes.is_empty() {
                return Ok(vec![Violacao::so_mensagem(format!(
                    "nenhuma configuração de lint encontrada em {}",
                    caminhos_de_config.join(", ")
                ))]);
            }
            let texto_das_configs = existentes.iter().map(|caminho| ler_texto(&raiz, caminho)).collect::<Vec<_>>().join("\n");
            lista_de_texto(&parametros["regras"])
                .into_iter()
                .filter(|nome_da_regra| !texto_das_configs.contains(nome_da_regra.as_str()))
                .map(|nome_da_regra| Violacao::em(existentes[0], None, format!("a regra de lint \"{nome_da_regra}\" não está configurada")))
                .collect()
        }

        // E6: TODO e FIXME só com a issue que vai resolvê-los.
        "todo-sem-issue" => {
            let mut marcadores = lista_de_texto(parametros.get("marcadores").unwrap_or(&Value::Null));
            if marcadores.is_empty() {
                marcadores = vec!["TODO".into(), "FIXME".into()];
            }
            let padrao_de_marcador = Regex::new(&format!(
                r"\b(?:{})\b",
                marcadores.iter().map(|marcador| regex::escape(marcador)).collect::<Vec<_>>().join("|")
            ))
            .expect("regex válida");
            let padrao_de_issue = Regex::new(parametro_texto(parametros, "issue").unwrap_or(r"#\d+|/issues/\d+"))
                .map_err(|erro_regex| ErroDeConfiguracao(format!("todo-sem-issue: regex de issue inválida: {erro_regex}")))?;
            let mut violacoes = Vec::new();
            for caminho_relativo in arquivos_de_codigo(contexto, parametros) {
                for (indice, texto_da_linha) in ler_texto(&raiz, &caminho_relativo).split('\n').enumerate() {
                    if padrao_de_marcador.is_match(texto_da_linha) && !padrao_de_issue.is_match(texto_da_linha) {
                        violacoes.push(Violacao::em(&caminho_relativo, Some(indice as u64 + 1), "marcador sem a issue que o resolve"));
                    }
                }
            }
            violacoes
        }

        // P9: o título de cada commit desde a base segue o formato escolhido.
        "mensagem-de-commit" => {
            exigir(parametros, &["padrao"], tipo)?;
            let base = contexto
                .base
                .clone()
                .or_else(|| parametro_texto(parametros, "base").map(String::from))
                .unwrap_or_else(|| "origin/main".into());
            if git(&raiz, &["rev-parse", "--verify", "--quiet", &base]).is_none() {
                contexto
                    .avisos
                    .push(format!("mensagem-de-commit: a base {base} não existe; commits não conferidos"));
                return Ok(Vec::new());
            }
            let padrao = parametro_texto(parametros, "padrao").unwrap_or("");
            let padrao_do_titulo = Regex::new(padrao).map_err(|erro_regex| ErroDeConfiguracao(format!("mensagem-de-commit: regex inválida: {erro_regex}")))?;
            let historico = git(&raiz, &["log", "--no-merges", "--format=%h%x09%s", &format!("{base}..HEAD")]).unwrap_or_default();
            historico
                .lines()
                .filter(|linha| !linha.is_empty())
                .filter_map(|linha| linha.split_once('\t'))
                .filter(|(_, titulo)| !padrao_do_titulo.is_match(titulo))
                .map(|(hash, titulo)| Violacao::so_mensagem(format!("commit {hash} \"{titulo}\" fora do formato {padrao}")))
                .collect()
        }

        // Qualquer ferramenta (lint, dependency-cruiser, jscpd, typecheck): saída diferente de 0 é
        // violação. "{arquivos}" no comando vira a lista dos arquivos de código que a checagem lê.
        "comando" => {
            exigir(parametros, &["executar"], tipo)?;
            let executar = parametro_texto(parametros, "executar").unwrap_or("");
            let mut linha_de_comando = executar.to_string();
            if linha_de_comando.contains("{arquivos}") {
                let arquivos = arquivos_de_codigo(contexto, parametros);
                if arquivos.is_empty() {
                    return Ok(Vec::new());
                }
                linha_de_comando = linha_de_comando.replace(
                    "{arquivos}",
                    &arquivos.iter().map(|caminho| escapar_para_shell(caminho)).collect::<Vec<_>>().join(" "),
                );
            }
            match rodar_no_shell(&linha_de_comando, &raiz) {
                Err(_) => vec![Violacao::so_mensagem(format!("ferramenta não encontrada: {executar}"))],
                Ok(saida) if saida.status.success() => Vec::new(),
                Ok(saida) if saida.status.code() == Some(127) => vec![Violacao::so_mensagem(format!("ferramenta não encontrada: {executar}"))],
                Ok(saida) => {
                    let tudo = format!("{}{}", String::from_utf8_lossy(&saida.stdout), String::from_utf8_lossy(&saida.stderr));
                    let codigo = saida.status.code().map(|codigo| codigo.to_string()).unwrap_or_else(|| "null".into());
                    vec![Violacao::so_mensagem(format!(
                        "\"{executar}\" saiu com {codigo}:\n{}",
                        ultimas_linhas(&tudo, 15)
                    ))]
                }
            }
        }

        // Padrões de código (TypeScript, JavaScript, Python, Go, Java, Rust e outras), com regras que o
        // projeto escreve a partir dos modelos. O ast-grep usa o .gitignore e o "files"/"ignores" da regra.
        "ast-grep" => {
            exigir(parametros, &["regras"], tipo)?;
            let binario = parametro_texto(parametros, "binario")
                .map(String::from)
                .or_else(|| std::env::var("AST_GREP").ok().filter(|valor| !valor.is_empty()))
                .unwrap_or_else(|| "ast-grep".into());
            let alvos = if contexto.alterados.is_some() {
                arquivos_de_codigo(contexto, parametros)
            } else {
                let alvos_configurados = lista_de_texto(parametros.get("alvos").unwrap_or(&Value::Null));
                if alvos_configurados.is_empty() {
                    vec![".".into()]
                } else {
                    alvos_configurados
                }
            };
            if alvos.is_empty() {
                return Ok(Vec::new());
            }
            let mut violacoes = Vec::new();
            for arquivo_de_regras in lista_de_texto(&parametros["regras"]) {
                let saida = Command::new(&binario)
                    .args(["scan", "--rule", &arquivo_de_regras, "--json=stream"])
                    .args(&alvos)
                    .current_dir(&raiz)
                    .output();
                let saida = match saida {
                    Err(_) => {
                        violacoes.push(Violacao::so_mensagem(format!(
                            "ast-grep não encontrado ({binario}); instale-o no CI: npm i -g @ast-grep/cli ou cargo install ast-grep"
                        )));
                        continue;
                    }
                    Ok(saida) => saida,
                };
                let achados: Vec<Value> = String::from_utf8_lossy(&saida.stdout)
                    .lines()
                    .filter(|linha| !linha.is_empty())
                    .filter_map(|linha| serde_json::from_str(linha).ok())
                    .collect();
                if !saida.status.success() && achados.is_empty() {
                    let resumo = String::from_utf8_lossy(&saida.stderr).trim().split('\n').take(6).collect::<Vec<_>>().join(" ");
                    violacoes.push(Violacao::so_mensagem(format!("ast-grep falhou com {arquivo_de_regras}: {resumo}")));
                    continue;
                }
                // O ast-grep varre em paralelo: a ordem dos achados muda de uma execução para outra.
                let mut achados = achados;
                achados.sort_by_key(|achado| {
                    (
                        achado["file"].as_str().unwrap_or("").to_string(),
                        achado["range"]["start"]["line"].as_u64().unwrap_or(0),
                    )
                });
                for achado in achados {
                    let mensagem = achado["message"]
                        .as_str()
                        .filter(|mensagem| !mensagem.is_empty())
                        .or_else(|| achado["ruleId"].as_str())
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    let linha = achado["range"]["start"]["line"].as_u64().map(|linha| linha + 1);
                    violacoes.push(Violacao::em(achado["file"].as_str().unwrap_or(""), linha, mensagem));
                }
            }
            violacoes
        }

        // Princípios de julgamento: o validador só os lista; quem decide é o revisor da orquestração.
        "revisor" => Vec::new(),

        outro => return erro(format!("tipo de verificação \"{outro}\" não existe no catálogo ({})", TIPOS.join(", "))),
    })
}

// ---------- execução ----------

pub struct Resultado {
    pub id: String,
    pub principio: String,
    pub tipo: String,
    pub violacoes: Vec<Violacao>,
}

pub struct Relatorio {
    pub aprovado: bool,
    pub escopo: String,
    pub resultados: Vec<Resultado>,
    pub avisos: Vec<String>,
}

pub struct Opcoes<'a> {
    pub base: Option<&'a str>,
    pub completo: bool,
    pub so_cobertura: bool,
}

pub fn ler_regras(config: &Value) -> Result<Vec<Regra>, ErroDeConfiguracao> {
    if config["versao"].as_i64() != Some(1) {
        return erro(format!(
            "versão {} não suportada (esperado 1)",
            if config["versao"].is_null() {
                "undefined".into()
            } else {
                config["versao"].to_string()
            }
        ));
    }
    let Some(lista) = config["regras"].as_array() else {
        return erro("\"regras\" precisa ser uma lista");
    };
    let id_valido = Regex::new(r"^R\d+$").expect("regex válida");
    let mut regras = Vec::new();
    for regra in lista {
        let id = regra["id"].as_str().unwrap_or("");
        if !id_valido.is_match(id) {
            return erro(format!(
                "regra sem id no formato R<n>: {}",
                regra.to_string().chars().take(80).collect::<String>()
            ));
        }
        let principio = regra["principio"].as_str().unwrap_or("");
        if !PRINCIPIOS.contains(&principio) {
            return erro(format!(
                "{id}: princípio \"{}\" não existe",
                if regra["principio"].is_null() { "undefined" } else { principio }
            ));
        }
        let tipo = regra["verificacao"]["tipo"].as_str().unwrap_or("");
        if !TIPOS.contains(&tipo) {
            return erro(format!(
                "{id}: tipo de verificação \"{}\" não existe no catálogo ({})",
                if tipo.is_empty() { "undefined" } else { tipo },
                TIPOS.join(", ")
            ));
        }
        let verificacao = regra["verificacao"].as_object().cloned().unwrap_or_default();
        regras.push(Regra {
            id: id.into(),
            principio: principio.into(),
            verificacao,
        });
    }
    Ok(regras)
}

pub fn validar(raiz: &Path, config: &Value, texto_md: &str, opcoes: &Opcoes) -> Result<Relatorio, ErroDeConfiguracao> {
    let regras = ler_regras(config)?;
    let mut contexto = Contexto {
        raiz: raiz.to_path_buf(),
        projeto: config["projeto"].clone(),
        base: opcoes.base.map(String::from),
        avisos: Vec::new(),
        arquivos: if opcoes.so_cobertura {
            Vec::new()
        } else {
            listar_arquivos_do_repositorio(raiz)?
        },
        alterados: None,
    };
    if let (Some(base), false, false) = (opcoes.base, opcoes.completo, opcoes.so_cobertura) {
        contexto.alterados = arquivos_alterados_desde(raiz, base);
        if contexto.alterados.is_none() {
            contexto.avisos.push(format!("a base {base} não existe; o repositório inteiro foi lido"));
        }
    }
    let mut resultados = vec![Resultado {
        id: "cobertura".into(),
        principio: "todos".into(),
        tipo: "cobertura".into(),
        violacoes: checar_cobertura(texto_md, &regras),
    }];
    if !opcoes.so_cobertura {
        for regra in &regras {
            let tipo = regra.verificacao.get("tipo").and_then(Value::as_str).unwrap_or("").to_string();
            let mut parametros = regra.verificacao.clone();
            parametros.remove("tipo");
            let violacoes = rodar_checagem(&tipo, &parametros, &mut contexto)?;
            resultados.push(Resultado {
                id: regra.id.clone(),
                principio: regra.principio.clone(),
                tipo,
                violacoes,
            });
        }
    }
    let aprovado = resultados.iter().all(|resultado| resultado.violacoes.is_empty());
    let escopo = match (&contexto.alterados, opcoes.base) {
        (Some(alterados), Some(base)) => format!("{} arquivo(s) alterado(s) desde {base}", alterados.len()),
        _ => "repositório inteiro".into(),
    };
    Ok(Relatorio {
        aprovado,
        escopo,
        resultados,
        avisos: contexto.avisos,
    })
}

pub fn relatorio_em_json(relatorio: &Relatorio) -> Value {
    let resultados: Vec<Value> = relatorio
        .resultados
        .iter()
        .map(|resultado| {
            let violacoes: Vec<Value> = resultado
                .violacoes
                .iter()
                .map(|violacao| {
                    let mut objeto = Map::new();
                    if let Some(arquivo) = &violacao.arquivo {
                        objeto.insert("arquivo".into(), arquivo.clone().into());
                    }
                    if let Some(linha) = violacao.linha {
                        objeto.insert("linha".into(), linha.into());
                    }
                    objeto.insert("mensagem".into(), violacao.mensagem.clone().into());
                    Value::Object(objeto)
                })
                .collect();
            json!({ "id": resultado.id, "principio": resultado.principio, "tipo": resultado.tipo, "violacoes": violacoes })
        })
        .collect();
    json!({ "aprovado": relatorio.aprovado, "escopo": relatorio.escopo, "resultados": resultados, "avisos": relatorio.avisos })
}

pub fn formatar(relatorio: &Relatorio) -> String {
    let mut linhas = vec![format!("Validador de regras de código: {}", relatorio.escopo)];
    for resultado in &relatorio.resultados {
        let rotulo = format!("{} ({}) {}", resultado.id, resultado.principio, resultado.tipo);
        if resultado.tipo == "revisor" {
            linhas.push(format!("👁  {rotulo}: fica com o revisor"));
            continue;
        }
        let quantidade = resultado.violacoes.len();
        linhas.push(format!(
            "{} {rotulo}{}",
            if quantidade > 0 { "❌" } else { "✅" },
            if quantidade > 0 {
                format!(": {quantidade} violação(ões)")
            } else {
                String::new()
            }
        ));
        for violacao in &resultado.violacoes {
            let local = match (&violacao.arquivo, violacao.linha) {
                (Some(arquivo), Some(linha)) => format!("{arquivo}:{linha}  "),
                (Some(arquivo), None) => format!("{arquivo}  "),
                _ => String::new(),
            };
            linhas.push(format!("   {local}{}", violacao.mensagem.replace('\n', "\n      ")));
        }
    }
    for aviso in &relatorio.avisos {
        linhas.push(format!("⚠️  {aviso}"));
    }
    linhas.push(format!("VALIDADOR: {}", if relatorio.aprovado { "APROVADO" } else { "REPROVADO" }));
    linhas.join("\n")
}

fn executar(argumentos: &[String]) -> Result<i32, ErroDeConfiguracao> {
    let (mut caminho_config, mut caminho_md) = ("docs/regras-de-codigo.json".to_string(), "docs/regras-de-codigo.md".to_string());
    let (mut base, mut completo, mut so_cobertura, mut em_json) = (None, false, false, false);
    let mut indice = 0;
    while indice < argumentos.len() {
        let valor_seguinte = |indice: usize| {
            argumentos
                .get(indice + 1)
                .cloned()
                .ok_or_else(|| ErroDeConfiguracao(format!("{} exige um valor", argumentos[indice])))
        };
        match argumentos[indice].as_str() {
            "--config" => {
                caminho_config = valor_seguinte(indice)?;
                indice += 1;
            }
            "--md" => {
                caminho_md = valor_seguinte(indice)?;
                indice += 1;
            }
            "--base" => {
                base = Some(valor_seguinte(indice)?);
                indice += 1;
            }
            "--completo" => completo = true,
            "--so-cobertura" => so_cobertura = true,
            "--json" => em_json = true,
            outro => return erro(format!("argumento desconhecido: {outro}")),
        }
        indice += 1;
    }
    let raiz = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for caminho in [&caminho_config, &caminho_md] {
        if !raiz.join(caminho).exists() {
            return erro(format!("{caminho} não existe (o mapa de Regras de código gera os dois arquivos)"));
        }
    }
    let config: Value = serde_json::from_str(&ler_texto(&raiz, &caminho_config))
        .map_err(|erro_json| ErroDeConfiguracao(format!("{caminho_config} não é JSON válido: {erro_json}")))?;
    let relatorio = validar(
        &raiz,
        &config,
        &ler_texto(&raiz, &caminho_md),
        &Opcoes {
            base: base.as_deref(),
            completo,
            so_cobertura,
        },
    )?;
    if em_json {
        println!("{}", serde_json::to_string_pretty(&relatorio_em_json(&relatorio)).unwrap_or_default());
    } else {
        println!("{}", formatar(&relatorio));
    }
    Ok(if relatorio.aprovado { 0 } else { 1 })
}

pub fn rodar(argumentos: &[String]) -> i32 {
    match executar(argumentos) {
        Ok(codigo) => codigo,
        Err(ErroDeConfiguracao(mensagem)) => {
            eprintln!("Configuração inválida: {mensagem}");
            println!("VALIDADOR: REPROVADO");
            2
        }
    }
}
