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

use crate::riscos;

// Expressões usadas por nome declarado ou por linha: compiladas uma vez só.
static CAMEL_CASE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([a-z0-9])([A-Z])").expect("regex válida"));
static SEPARADORES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\s_\-]+").expect("regex válida"));
// "let mut" é só de Rust; em JS "let mut = 1" continua declarando "mut" (o "mut" opcional volta atrás).
static DECLARACAO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:function|class|const|let(?:\s+mut)?|var|def|type|interface|enum|struct|fn|func|record|trait|object)\s+([A-Za-zÀ-ÿ_][A-Za-z0-9_À-ÿ]*)")
        .expect("regex válida")
});
// Rust tem declarações próprias (static, mod, union, macro_rules!) e "mut" antes do nome. Fica numa
// regex separada para que "static" não vire declaração em Java e TypeScript ("static void" daria "void").
static DECLARACAO_RUST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:const|let(?:\s+mut)?|static(?:\s+mut)?|fn|struct|enum|union|trait|type|mod|macro_rules!)\s+(?:r#)?([A-Za-zÀ-ÿ_][A-Za-z0-9_À-ÿ]*)")
        .expect("regex válida")
});
// Dart declara com final, late e o tipo antes do nome (final Usuario usuario), e tem mixin, extension e
// typedef. O tipo opcional só vale depois de final/const/var/late, para "class A extends B" dar "A".
static DECLARACAO_DART: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(?:(?:class|mixin|enum|typedef|extension(?:\s+type)?)\s+|(?:final|const|var|late(?:\s+final)?)\s+(?:[A-Za-z_][\w.]*(?:<[^=;()]*>)?\??\s+)?)([A-Za-zÀ-ÿ_][A-Za-z0-9_À-ÿ]*)",
    )
    .expect("regex válida")
});
// ID de prática da linguagem, fora dos 22 princípios: TS (TypeScript), JV (Java), RS (Rust), DT (Dart).
static PRATICA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?:TS|JV|RS|DT)\d+$").expect("regex válida"));
static ID_DE_REGRA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bR\d+\b").expect("regex válida"));

pub const PRINCIPIOS: [&str; 22] = [
    "E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8", "P1", "P2", "P3", "P4", "P5", "P6", "P7", "P8", "P9", "S", "O", "L", "I", "D",
];
pub const DECISOES: [&str; 4] = ["adotado", "adaptado", "afastado", "garantido pelo lint"];
// Princípios que a constituição torna regra dura: o projeto adapta, nunca afasta (E1: regra 20; P5: regra 15).
pub const OBRIGATORIOS: [&str; 2] = ["E1", "P5"];
// Princípio que só conta como decidido se a coluna "Onde" cita uma regra com esta checagem.
pub const CHECAGEM_EXIGIDA: [(&str, &str); 1] = [("P5", "teste-cobre-risco")];
pub const TIPOS: [&str; 10] = [
    "termo-evitado",
    "teste-cobre-risco",
    "teste-de-contrato-por-port",
    "lint-cobre",
    "todo-sem-issue",
    "mensagem-de-commit",
    "comando",
    "ast-grep",
    "revisor",
    "revisao-por-modelo",
];
// Tipo que só aconselha (ADR 0015): roda no ci, nunca reprova e fica fora da cobertura dos 22.
pub const TIPO_CONSULTIVO: &str = "revisao-por-modelo";

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
// Regra de prática da linguagem (TS<n>, JV<n>, RS<n>, DT<n>) fica fora dos 22: mora numa tabela de práticas
// no mesmo formato (| RS1 | adotado | R12 |) e só a prática que tem regra no .json é exigida.

pub struct LinhaDeCobertura {
    pub decisao: String,
    pub onde_ou_porque: String,
}

pub fn eh_pratica(principio: &str) -> bool {
    PRATICA.is_match(principio)
}

// Linhas de tabela cuja primeira célula é um dos 22 princípios ou uma prática da linguagem.
pub fn ler_tabela_de_cobertura(texto_md: &str) -> Vec<(String, LinhaDeCobertura)> {
    let mut linhas_da_tabela: Vec<(String, LinhaDeCobertura)> = Vec::new();
    for linha in texto_md.lines() {
        let partes: Vec<&str> = linha.split('|').collect();
        if partes.len() < 5 {
            continue;
        }
        let celulas: Vec<&str> = partes[1..partes.len() - 1].iter().map(|celula| celula.trim()).collect();
        if !PRINCIPIOS.contains(&celulas[0]) && !eh_pratica(celulas[0]) {
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
    pub texto: String,
    pub verificacao: Map<String, Value>,
}

// Confere os IDs de regra citados numa linha decidida: existem no .json e nascem daquele princípio.
fn conferir_citacoes<'a>(principio: &str, linha: &'a LinhaDeCobertura, regras: &[Regra], violacoes: &mut Vec<Violacao>) -> Vec<&'a str> {
    let ids_na_linha: Vec<&str> = ID_DE_REGRA.find_iter(&linha.onde_ou_porque).map(|encontrado| encontrado.as_str()).collect();
    for id_citado in &ids_na_linha {
        match regras.iter().find(|regra| regra.id == *id_citado) {
            None => violacoes.push(Violacao::so_mensagem(format!(
                "{principio} cita {id_citado}, que não está em regras-de-codigo.json"
            ))),
            Some(regra_citada) if regra_citada.principio != principio => violacoes.push(Violacao::so_mensagem(format!(
                "{id_citado} nasce de {} no .json, mas a tabela a cita em {principio}",
                regra_citada.principio
            ))),
            Some(_) => {}
        }
    }
    ids_na_linha
}

fn decisao_invalida(principio: &str, linha: &LinhaDeCobertura) -> Option<Violacao> {
    (!DECISOES.contains(&linha.decisao.as_str()))
        .then(|| Violacao::so_mensagem(format!("{principio}: decisão \"{}\" não é uma de: {}", linha.decisao, DECISOES.join(", "))))
}

pub fn checar_cobertura(texto_md: &str, regras: &[Regra]) -> Vec<Violacao> {
    let mut violacoes = Vec::new();
    let tabela = ler_tabela_de_cobertura(texto_md);
    // Citação na linha de um dos 22 não vale para regra de prática, e vice-versa.
    let mut citadas_nos_22: HashSet<String> = HashSet::new();
    let mut citadas_nas_praticas: HashSet<String> = HashSet::new();
    for principio in PRINCIPIOS {
        let Some((_, linha)) = tabela.iter().find(|(nome, _)| nome == principio) else {
            violacoes.push(Violacao::so_mensagem(format!("{principio} sem linha na tabela de cobertura")));
            continue;
        };
        if let Some(violacao) = decisao_invalida(principio, linha) {
            violacoes.push(violacao);
            continue;
        }
        if linha.decisao == "afastado" {
            if OBRIGATORIOS.contains(&principio) {
                violacoes.push(Violacao::so_mensagem(format!(
                    "{principio} é regra dura da constituição: adapte, mas não afaste"
                )));
            } else if linha.onde_ou_porque.is_empty() {
                violacoes.push(Violacao::so_mensagem(format!("{principio} afastado sem o porquê")));
            }
            continue;
        }
        let ids_na_linha = conferir_citacoes(principio, linha, regras, &mut violacoes);
        if ids_na_linha.is_empty() {
            violacoes.push(Violacao::so_mensagem(format!(
                "{principio} {} sem regra (R<n>) na coluna \"Onde\"",
                linha.decisao
            )));
        }
        for (principio_exigente, checagem) in CHECAGEM_EXIGIDA.iter().filter(|(exigente, _)| *exigente == principio) {
            let tem_a_checagem = regras
                .iter()
                .any(|regra| ids_na_linha.contains(&regra.id.as_str()) && regra.verificacao.get("tipo").and_then(Value::as_str) == Some(checagem));
            if !tem_a_checagem {
                violacoes.push(Violacao::so_mensagem(format!(
                    "{principio_exigente} precisa citar uma regra com a checagem {checagem} (regra da constituição)"
                )));
            }
        }
        citadas_nos_22.extend(ids_na_linha.into_iter().map(String::from));
    }
    for (pratica, linha) in tabela.iter().filter(|(nome, _)| eh_pratica(nome)) {
        if let Some(violacao) = decisao_invalida(pratica, linha) {
            violacoes.push(violacao);
            continue;
        }
        if linha.decisao != "afastado" {
            let ids_na_linha = conferir_citacoes(pratica, linha, regras, &mut violacoes);
            citadas_nas_praticas.extend(ids_na_linha.into_iter().map(String::from));
        }
    }
    for regra in regras
        .iter()
        .filter(|regra| regra.verificacao.get("tipo").and_then(Value::as_str) != Some(TIPO_CONSULTIVO))
    {
        if eh_pratica(&regra.principio) {
            if !citadas_nas_praticas.contains(&regra.id) {
                violacoes.push(Violacao::so_mensagem(format!(
                    "{} nasce da prática {} e nenhuma linha da tabela de práticas a cita",
                    regra.id, regra.principio
                )));
            }
        } else if !citadas_nos_22.contains(&regra.id) {
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
    // Falso no hook vigia-teste, que confere só o arquivo editado: risco sem teste é do validador.
    pub conferir_o_registro_inteiro: bool,
    // Cada analysis_options.yaml por pasta reprova uma vez só, mesmo com várias regras lint-cobre.
    pub analysis_options_ja_mostrados: HashSet<String>,
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

// "{dir_de_teste}": o primeiro segmento "src/main/" da pasta vira "src/test/".
fn pasta_de_teste(pasta: &str) -> String {
    let com_barra = format!("{pasta}/");
    let inicio = com_barra
        .match_indices("src/main/")
        .map(|(posicao, _)| posicao)
        .find(|posicao| *posicao == 0 || com_barra[..*posicao].ends_with('/'));
    match inicio {
        Some(posicao) => format!("{}src/test/{}", &com_barra[..posicao], &com_barra[posicao + "src/main/".len()..])
            .trim_end_matches('/')
            .to_string(),
        None => pasta.to_string(),
    }
}

// ---------- lint-cobre: a regra delegada está ligada, não só escrita, na configuração ----------

// Resultado de uma regra num arquivo de configuração. A ordem é a de prioridade: entre vários
// arquivos, vale o maior (Coberta em qualquer um basta).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Cobertura {
    Ausente,
    Comentada,
    // O nível ou valor que a deixa sem bloquear: "allow", "warn", "off", false, severity "ignore"...
    Desligada(String),
    Ilegivel(String),
    Coberta,
}

#[derive(Clone, Copy, PartialEq)]
enum Comentarios {
    // # fora de string
    Toml,
    // <!-- -->
    Xml,
    // // e /* */ fora de string
    Json,
    // // e /* */ fora de string, e # no começo da linha ou depois de espaço
    Generico,
}

// Separa o texto em (código, comentários). Cada parte tem uma letra por letra do original (o que é da
// outra vira espaço) e as quebras de linha ficam nas duas, para as linhas continuarem batendo.
fn separar_comentarios(texto: &str, estilo: Comentarios) -> (String, String) {
    let letras: Vec<char> = texto.chars().collect();
    let (mut codigo, mut comentarios) = (String::with_capacity(texto.len()), String::with_capacity(texto.len()));
    let comeca = |indice: usize, marca: &str| {
        marca
            .chars()
            .enumerate()
            .all(|(deslocamento, letra)| letras.get(indice + deslocamento) == Some(&letra))
    };
    let aspas_do_estilo: &[char] = match estilo {
        Comentarios::Toml => &['"', '\''],
        Comentarios::Xml => &[],
        Comentarios::Json => &['"'],
        Comentarios::Generico => &['"', '\'', '`'],
    };
    let mut aspa_aberta: Option<char> = None;
    let mut indice = 0;
    while indice < letras.len() {
        let letra = letras[indice];
        if let Some(aspa) = aspa_aberta {
            codigo.push(letra);
            comentarios.push(if letra == '\n' { '\n' } else { ' ' });
            if letra == '\\' && aspa != '\'' && indice + 1 < letras.len() {
                indice += 1;
                codigo.push(letras[indice]);
                comentarios.push(if letras[indice] == '\n' { '\n' } else { ' ' });
            } else if letra == aspa || (letra == '\n' && aspa != '`') {
                aspa_aberta = None;
            }
            indice += 1;
            continue;
        }
        if aspas_do_estilo.contains(&letra) {
            aspa_aberta = Some(letra);
            codigo.push(letra);
            comentarios.push(' ');
            indice += 1;
            continue;
        }
        let depois_de_espaco = indice == 0 || letras[indice - 1].is_whitespace();
        let fim_do_comentario: Option<(usize, &str)> = match estilo {
            Comentarios::Xml if comeca(indice, "<!--") => Some((4, "-->")),
            Comentarios::Json | Comentarios::Generico if comeca(indice, "/*") => Some((2, "*/")),
            Comentarios::Json | Comentarios::Generico if comeca(indice, "//") => Some((2, "\n")),
            Comentarios::Toml if letra == '#' => Some((1, "\n")),
            Comentarios::Generico if letra == '#' && depois_de_espaco => Some((1, "\n")),
            _ => None,
        };
        let Some((tamanho_da_abertura, fecho)) = fim_do_comentario else {
            codigo.push(letra);
            comentarios.push(if letra == '\n' { '\n' } else { ' ' });
            indice += 1;
            continue;
        };
        // O comentário vai até o fecho (a quebra de linha fica fora dele, nas duas partes).
        let mut fim = indice + tamanho_da_abertura;
        while fim < letras.len() && !comeca(fim, fecho) {
            fim += 1;
        }
        if fecho != "\n" {
            fim = (fim + fecho.chars().count()).min(letras.len());
        }
        for &letra_do_comentario in &letras[indice..fim] {
            codigo.push(if letra_do_comentario == '\n' { '\n' } else { ' ' });
            comentarios.push(letra_do_comentario);
        }
        indice = fim;
    }
    (codigo, comentarios)
}

fn letra_de_identificador(letra: char) -> bool {
    letra.is_alphanumeric() || matches!(letra, '_' | '-' | '$' | '@' | '/')
}

// Posições (em bytes) onde o nome aparece inteiro: sem letra de identificador colada antes ou depois.
// Assim "panic" não casa com "panic_in_result_fn" nem "max-depth" com "max-depth-x".
fn ocorrencias_inteiras(texto: &str, nome: &str) -> Vec<usize> {
    ocorrencias_com_fronteira(texto, nome, letra_de_identificador)
}

fn ocorrencias_com_fronteira(texto: &str, nome: &str, letra_de_identificador: fn(char) -> bool) -> Vec<usize> {
    if nome.is_empty() {
        return Vec::new();
    }
    texto
        .match_indices(nome)
        .map(|(posicao, _)| posicao)
        .filter(|posicao| {
            let antes = texto[..*posicao].chars().next_back();
            let depois = texto[posicao + nome.len()..].chars().next();
            !antes.is_some_and(letra_de_identificador) && !depois.is_some_and(letra_de_identificador)
        })
        .collect()
}

// Só escolhe a mensagem (as duas reprovam), então aceita o nome no fim de um caminho comentado
// (<!-- <rule ref=".../GodClass"/> -->).
fn comentada_ou_ausente(comentarios: &str, nome: &str) -> Cobertura {
    let letra_de_nome = |letra: char| letra.is_alphanumeric() || matches!(letra, '_' | '-' | '$');
    if ocorrencias_com_fronteira(comentarios, nome, letra_de_nome).is_empty() {
        Cobertura::Ausente
    } else {
        Cobertura::Comentada
    }
}

// Um analysis_options.yaml vale para a própria pasta e cala o lint dali (visto com Dart 3.12.2): dentro
// da pasta de um config declarado e fora da lista, é falso aprovado (ADR 0021). Só conta o versionado:
// o ignorado pelo git e o ainda sem commit não aparecem no git ls-files --cached.
fn analysis_options_por_pasta(raiz: &Path, caminhos_de_config: &[String]) -> Vec<Violacao> {
    const ARQUIVO_DE_OPCOES: &str = "analysis_options.yaml";
    let versionados = git(raiz, &["ls-files", "--cached"]).unwrap_or_default();
    let sem_ponto = |caminho: &str| caminho.trim_start_matches("./").to_string();
    let declarados: Vec<String> = caminhos_de_config.iter().map(|caminho| sem_ponto(caminho)).collect();
    let pastas_declaradas: Vec<&str> = declarados
        .iter()
        .filter_map(|caminho| caminho.strip_suffix(ARQUIVO_DE_OPCOES))
        .filter(|pasta| pasta.is_empty() || pasta.ends_with('/'))
        .collect();
    versionados
        .lines()
        .map(String::from)
        .filter(|caminho| caminho.ends_with(&format!("/{ARQUIVO_DE_OPCOES}")) && !declarados.contains(caminho))
        .filter(|caminho| pastas_declaradas.iter().any(|pasta| caminho.starts_with(pasta)))
        .map(|caminho| {
            Violacao::em(
                &caminho,
                None,
                format!("este {ARQUIVO_DE_OPCOES} cala o lint da pasta dele: apague-o ou declare-o em \"config\""),
            )
        })
        .collect()
}

// Escolhe a leitura pela extensão do arquivo de configuração.
fn cobertura_de_lint(caminho: &str, texto: &str, nome: &str) -> Cobertura {
    let nome_do_arquivo = Path::new(caminho)
        .file_name()
        .map(|nome| nome.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let extensao = Path::new(caminho)
        .extension()
        .map(|extensao| extensao.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match extensao.as_str() {
        "toml" => cobertura_toml(texto, nome, nome_do_arquivo.ends_with("clippy.toml")),
        "xml" => cobertura_xml(texto, nome),
        "json" | "jsonc" => cobertura_json(texto, nome),
        "yaml" | "yml" if nome_do_arquivo.starts_with("analysis_options") => cobertura_analysis_options(texto, nome),
        _ => cobertura_generica(texto, nome),
    }
}

// Cargo.toml e clippy.toml: a regra está em [lints.<grupo>] ou [workspace.lints.<grupo>] com nível
// "deny" ou "forbid" (texto ou { level = ..., priority = ... }). "clippy::x" procura só em
// [lints.clippy]; "x" em qualquer grupo. No clippy.toml, uma chave de primeiro nível (limite como
// too-many-lines-threshold) conta se não for false.
fn cobertura_toml(texto: &str, nome: &str, eh_clippy_toml: bool) -> Cobertura {
    let (_, comentarios) = separar_comentarios(texto, Comentarios::Toml);
    let tabela: toml::Table = match texto.parse() {
        Ok(tabela) => tabela,
        Err(erro_toml) => return Cobertura::Ilegivel(format!("não é TOML válido ({})", erro_toml.message())),
    };
    let (grupo_pedido, chave) = match nome.split_once("::") {
        Some((grupo, chave)) => (Some(grupo), chave),
        None => (None, nome),
    };
    let mut melhor = Cobertura::Ausente;
    let tabelas_de_lints = [tabela.get("lints"), tabela.get("workspace").and_then(|workspace| workspace.get("lints"))];
    for lints in tabelas_de_lints.into_iter().flatten().filter_map(toml::Value::as_table) {
        for (grupo, regras_do_grupo) in lints {
            if grupo_pedido.is_some_and(|pedido| pedido != grupo) {
                continue;
            }
            let Some(valor) = regras_do_grupo.as_table().and_then(|regras| regras.get(chave)) else {
                continue;
            };
            let nivel = match valor {
                toml::Value::String(nivel) => nivel.as_str(),
                toml::Value::Table(detalhe) => detalhe.get("level").and_then(toml::Value::as_str).unwrap_or(""),
                _ => "",
            };
            let cobertura = if nivel == "deny" || nivel == "forbid" {
                Cobertura::Coberta
            } else {
                Cobertura::Desligada(format!("\"{nivel}\""))
            };
            melhor = melhor.max(cobertura);
        }
    }
    if eh_clippy_toml {
        if let Some(valor) = tabela.get(nome) {
            melhor = melhor.max(if valor.as_bool() == Some(false) {
                Cobertura::Desligada("false".into())
            } else {
                Cobertura::Coberta
            });
        }
    }
    if melhor == Cobertura::Ausente {
        return comentada_ou_ausente(&comentarios, chave);
    }
    melhor
}

static MODULO_OU_PROPRIEDADE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<(/?)(module|property)\b([^>]*?)(/?)>").expect("regex válida"));
static ATRIBUTO_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\bname\s*=\s*"([^"]*)""#).expect("regex válida"));
static ATRIBUTO_VALUE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\bvalue\s*=\s*"([^"]*)""#).expect("regex válida"));
static REGRA_DO_PMD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<rule\b[^>]*\bref\s*=\s*"([^"]*)""#).expect("regex válida"));
static EXCLUSAO_DO_PMD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<exclude\b[^>]*\bname\s*=\s*"([^"]*)""#).expect("regex válida"));
static FLAG_DO_ERROR_PRONE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"-Xep:([A-Za-z0-9_]+)(?::([A-Za-z]+))?").expect("regex válida"));

fn atributo(regex_do_atributo: &Regex, atributos: &str) -> String {
    regex_do_atributo.captures(atributos).map(|achado| achado[1].to_string()).unwrap_or_default()
}

// Checkstyle, PMD e pom.xml com Error Prone, sem os comentários <!-- -->:
// - Checkstyle: <module name="X"> (ou XCheck, ou o nome qualificado) sem severity "ignore" dentro dele;
// - PMD: <rule ref=".../X"/>, desde que nenhum <exclude name="X"/> a tire;
// - Error Prone: -Xep:X:ERROR (warning não quebra o build). O nome pode vir como "-Xep:X:ERROR" ou "X:ERROR".
// Outro XML: o nome inteiro fora de comentário.
fn cobertura_xml(texto: &str, nome: &str) -> Cobertura {
    let (codigo, comentarios) = separar_comentarios(texto, Comentarios::Xml);
    let sem_prefixo = nome.strip_prefix("-Xep:").unwrap_or(nome);
    let base = [":ERROR", ":WARN", ":OFF"]
        .iter()
        .find_map(|nivel| sem_prefixo.strip_suffix(nivel))
        .unwrap_or(sem_prefixo);
    let (eh_checkstyle, eh_pmd, eh_error_prone) = (codigo.contains("<module"), codigo.contains("<rule"), codigo.contains("-Xep:"));
    if !(eh_checkstyle || eh_pmd || eh_error_prone) {
        return if ocorrencias_inteiras(&codigo, nome).is_empty() {
            comentada_ou_ausente(&comentarios, nome)
        } else {
            Cobertura::Coberta
        };
    }
    let mut melhor = Cobertura::Ausente;
    if eh_checkstyle {
        let casa_o_modulo = |nome_do_modulo: &str| {
            let curto = nome_do_modulo.rsplit('.').next().unwrap_or(nome_do_modulo);
            curto == base || curto.strip_suffix("Check") == Some(base)
        };
        // Pilha dos <module> abertos: Some(severidade ignorada?) quando é o módulo procurado.
        let mut pilha: Vec<Option<bool>> = Vec::new();
        for etiqueta in MODULO_OU_PROPRIEDADE.captures_iter(&codigo) {
            let (fecha, tipo, atributos, auto_fecha) = (&etiqueta[1] == "/", &etiqueta[2], &etiqueta[3], &etiqueta[4] == "/");
            match (tipo, fecha) {
                ("module", false) => {
                    let alvo = casa_o_modulo(&atributo(&ATRIBUTO_NAME, atributos));
                    if auto_fecha {
                        if alvo {
                            melhor = melhor.max(Cobertura::Coberta);
                        }
                    } else {
                        pilha.push(alvo.then_some(false));
                    }
                }
                ("module", true) => {
                    if let Some(Some(ignorada)) = pilha.pop() {
                        melhor = melhor.max(if ignorada {
                            Cobertura::Desligada("severity \"ignore\"".into())
                        } else {
                            Cobertura::Coberta
                        });
                    }
                }
                ("property", false) => {
                    if let Some(Some(ignorada)) = pilha.last_mut() {
                        if atributo(&ATRIBUTO_NAME, atributos) == "severity" && atributo(&ATRIBUTO_VALUE, atributos) == "ignore" {
                            *ignorada = true;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    if eh_pmd {
        let excluida = EXCLUSAO_DO_PMD.captures_iter(&codigo).any(|exclusao| &exclusao[1] == base);
        let referida = REGRA_DO_PMD
            .captures_iter(&codigo)
            .any(|regra| &regra[1] == base || regra[1].ends_with(&format!("/{base}")));
        if excluida {
            melhor = melhor.max(Cobertura::Desligada("<exclude>".into()));
        } else if referida {
            melhor = melhor.max(Cobertura::Coberta);
        }
    }
    if eh_error_prone {
        for flag in FLAG_DO_ERROR_PRONE.captures_iter(&codigo).filter(|flag| &flag[1] == base) {
            melhor = melhor.max(match flag.get(2).map(|nivel| nivel.as_str()) {
                Some("ERROR") => Cobertura::Coberta,
                Some(nivel) => Cobertura::Desligada(format!("-Xep:{base}:{nivel}")),
                None => Cobertura::Desligada(format!("-Xep:{base} sem :ERROR")),
            });
        }
    }
    if melhor == Cobertura::Ausente {
        return comentada_ou_ausente(&comentarios, base);
    }
    melhor
}

// tsconfig.json, .eslintrc.json: sem comentários nem vírgula sobrando, a regra ou flag é uma chave
// (em qualquer nível) cujo valor não é false, "off", 0 nem uma lista que começa por eles.
fn cobertura_json(texto: &str, nome: &str) -> Cobertura {
    let (codigo, comentarios) = separar_comentarios(texto, Comentarios::Json);
    let sem_virgula_sobrando = tirar_virgulas_sobrando(&codigo);
    let valor: Value = match serde_json::from_str(&sem_virgula_sobrando) {
        Ok(valor) => valor,
        Err(erro_json) => return Cobertura::Ilegivel(format!("não é JSON válido ({erro_json})")),
    };
    let mut achados = Vec::new();
    juntar_valores_da_chave(&valor, nome, &mut achados);
    let desliga = |valor: &Value| matches!(valor, Value::Bool(false)) || valor.as_str() == Some("off") || valor.as_f64() == Some(0.0);
    let mut melhor = Cobertura::Ausente;
    for achado in achados {
        let primeiro = achado.as_array().and_then(|lista| lista.first()).unwrap_or(achado);
        melhor = melhor.max(if desliga(primeiro) {
            Cobertura::Desligada(primeiro.to_string())
        } else {
            Cobertura::Coberta
        });
    }
    if melhor == Cobertura::Ausente {
        return comentada_ou_ausente(&comentarios, nome);
    }
    melhor
}

fn juntar_valores_da_chave<'a>(valor: &'a Value, chave: &str, achados: &mut Vec<&'a Value>) {
    match valor {
        Value::Object(objeto) => {
            for (nome, filho) in objeto {
                if nome == chave {
                    achados.push(filho);
                }
                juntar_valores_da_chave(filho, chave, achados);
            }
        }
        Value::Array(lista) => lista.iter().for_each(|filho| juntar_valores_da_chave(filho, chave, achados)),
        _ => {}
    }
}

// JSONC aceita vírgula antes de } e ]; o serde_json não. Fora de string, ela some.
fn tirar_virgulas_sobrando(codigo: &str) -> String {
    let letras: Vec<char> = codigo.chars().collect();
    let mut resultado = String::with_capacity(codigo.len());
    let mut dentro_de_string = false;
    let mut indice = 0;
    while indice < letras.len() {
        let letra = letras[indice];
        if dentro_de_string {
            if letra == '\\' && indice + 1 < letras.len() {
                resultado.push(letra);
                indice += 1;
                resultado.push(letras[indice]);
                indice += 1;
                continue;
            }
            dentro_de_string = letra != '"';
        } else if letra == '"' {
            dentro_de_string = true;
        } else if letra == ','
            && letras[indice + 1..]
                .iter()
                .find(|seguinte| !seguinte.is_whitespace())
                .is_some_and(|seguinte| matches!(seguinte, '}' | ']'))
        {
            indice += 1;
            continue;
        }
        resultado.push(letra);
        indice += 1;
    }
    resultado
}

// Valor desligado logo depois do nome, na mesma linha: "max-depth": "off", unwrap_used = "allow", x: false.
static VALOR_DESLIGADO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^["'`]?\s*[:=,]?\s*\[?\s*("off"|'off'|"allow"|'allow'|false\b|0\b)"#).expect("regex válida"));

// Outros arquivos (eslint.config.js, build.gradle): o nome inteiro fora de comentário (//, /* */, #),
// sem um valor desligado logo depois dele na mesma linha. É compatibilidade: para o ESLint, o que
// prova a regra ligada é o comando com eslint --print-config.
fn sem_aspas(texto: &str) -> &str {
    texto.trim().trim_matches(|letra| letra == '"' || letra == '\'')
}

// analysis_options.yaml do Dart: a regra está ligada em linter: rules: (lista "- x" ou mapa "x: true")
// ou sobe para error/warning em analyzer: errors:; "x: false" e "x: ignore" a desligam. Opção de
// analyzer: language: (strict-casts...) conta com true. Regra que vem só do include: (preset) não
// aparece no arquivo e não conta: escreva-a, como no ESLint.
fn cobertura_analysis_options(texto: &str, nome: &str) -> Cobertura {
    let (codigo, comentarios) = separar_comentarios(texto, Comentarios::Toml);
    let (mut na_lista, mut no_mapa, mut severidade, mut na_linguagem): (bool, Option<String>, Option<String>, Option<String>) = (false, None, None, None);
    let mut caminho: Vec<(usize, String)> = Vec::new();
    for linha in codigo.lines() {
        let conteudo = linha.trim();
        if conteudo.is_empty() {
            continue;
        }
        let recuo_da_linha = linha.len() - linha.trim_start().len();
        while caminho.last().is_some_and(|(recuo_da_chave, _)| *recuo_da_chave >= recuo_da_linha) {
            caminho.pop();
        }
        let chaves: Vec<&str> = caminho.iter().map(|(_, chave)| chave.as_str()).collect();
        if let Some(item) = conteudo.strip_prefix("- ") {
            na_lista |= chaves == ["linter", "rules"] && sem_aspas(item) == nome;
            continue;
        }
        let Some((chave, valor)) = conteudo.split_once(':') else { continue };
        let (chave, valor) = (sem_aspas(chave), sem_aspas(valor).to_lowercase());
        if valor.is_empty() {
            caminho.push((recuo_da_linha, chave.to_string()));
            continue;
        }
        if chave != nome {
            continue;
        }
        match chaves.as_slice() {
            ["linter", "rules"] => no_mapa = Some(valor),
            ["analyzer", "errors"] => severidade = Some(valor),
            ["analyzer", "language"] => na_linguagem = Some(valor),
            _ => {}
        }
    }
    let desligada = [&no_mapa, &severidade, &na_linguagem]
        .into_iter()
        .flatten()
        .find(|valor| matches!(valor.as_str(), "false" | "ignore"));
    if let Some(valor) = desligada {
        return Cobertura::Desligada(format!("\"{valor}\""));
    }
    let ligada =
        na_lista || no_mapa.as_deref() == Some("true") || na_linguagem.as_deref() == Some("true") || matches!(severidade.as_deref(), Some("error" | "warning"));
    if ligada {
        return Cobertura::Coberta;
    }
    if let Some(valor) = severidade.or(no_mapa).or(na_linguagem) {
        return Cobertura::Desligada(format!("\"{valor}\""));
    }
    comentada_ou_ausente(&comentarios, nome)
}

fn cobertura_generica(texto: &str, nome: &str) -> Cobertura {
    let (codigo, comentarios) = separar_comentarios(texto, Comentarios::Generico);
    let mut melhor = Cobertura::Ausente;
    for posicao in ocorrencias_inteiras(&codigo, nome) {
        let resto_da_linha = codigo[posicao + nome.len()..].split('\n').next().unwrap_or("");
        melhor = melhor.max(match VALOR_DESLIGADO.captures(resto_da_linha) {
            Some(desligado) => Cobertura::Desligada(desligado[1].to_string()),
            None => Cobertura::Coberta,
        });
    }
    if melhor == Cobertura::Ausente {
        return comentada_ou_ausente(&comentarios, nome);
    }
    melhor
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
                let declaracao: &Regex = if caminho_relativo.ends_with(".rs") {
                    &DECLARACAO_RUST
                } else if caminho_relativo.ends_with(".dart") {
                    &DECLARACAO_DART
                } else {
                    &DECLARACAO
                };
                for (indice, texto_da_linha) in ler_texto(&raiz, &caminho_relativo).split('\n').enumerate() {
                    for declarado in declaracao.captures_iter(texto_da_linha) {
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
        // teste na pasta das ports não contam como port. "{dir_de_teste}" é o "{dir}" com o primeiro
        // "src/main/" trocado por "src/test/" (layout Maven/Gradle); sem "src/main/", fica igual ao "{dir}".
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
                    let caminho_do_contrato = modelo_do_contrato
                        .replace("{dir_de_teste}", &pasta_de_teste(&pasta))
                        .replace("{dir}", &pasta)
                        .replace("{nome}", &nome_da_port);
                    (!raiz.join(&caminho_do_contrato).exists())
                        .then(|| Violacao::em(caminho_da_port, None, format!("port sem teste de contrato em {caminho_do_contrato}")))
                })
                .collect()
        }

        // P5: todo teste cita um risco do registro, todo risco citado existe e todo risco que não está
        // pendente tem teste. O registro e os testes se leem inteiros: risco sem teste aparece mesmo
        // quando a mudança é em outro lugar.
        "teste-cobre-risco" => {
            let caminho_do_registro = parametro_texto(parametros, "riscos").unwrap_or("docs/riscos.md");
            if !raiz.join(caminho_do_registro).exists() {
                return Ok(vec![Violacao::so_mensagem(format!(
                    "registro de riscos {caminho_do_registro} não existe (docs/agents/riscos.md diz como nasce)"
                ))]);
            }
            let declaracao_configurada = match parametro_texto(parametros, "declaracao") {
                Some(padrao) => Some(
                    Regex::new(padrao).map_err(|erro_regex| ErroDeConfiguracao(format!("teste-cobre-risco: regex de declaracao inválida: {erro_regex}")))?,
                ),
                None => None,
            };
            let mut padroes_de_teste = lista_de_texto(parametros.get("testes").unwrap_or(&Value::Null));
            if padroes_de_teste.is_empty() {
                padroes_de_teste = riscos::TESTES_PADRAO.iter().map(|padrao| padrao.to_string()).collect();
            }
            let mut padroes_ignorados = lista_de_texto(&contexto.projeto["ignorar"]);
            padroes_ignorados.extend(lista_de_texto(parametros.get("ignorar").unwrap_or(&Value::Null)));
            let (globs_de_teste, globs_ignorados) = (Globs::de(&padroes_de_teste), Globs::de(&padroes_ignorados));
            let eh_teste = |caminho: &&String| globs_de_teste.casa(caminho) && !globs_ignorados.casa(caminho);

            let (registro, problemas) = riscos::ler_registro(&ler_texto(&raiz, caminho_do_registro));
            let mut violacoes: Vec<Violacao> = if contexto.conferir_o_registro_inteiro {
                problemas
                    .into_iter()
                    .map(|problema| Violacao::em(caminho_do_registro, Some(problema.linha), problema.mensagem))
                    .collect()
            } else {
                Vec::new()
            };
            let mut citados_em_algum_teste: HashSet<String> = HashSet::new();
            let lidos: Vec<String> = if contexto.conferir_o_registro_inteiro {
                contexto.arquivos.iter().filter(eh_teste).cloned().collect()
            } else {
                Vec::new()
            };
            let universo = contexto.alterados.as_ref().unwrap_or(&contexto.arquivos);
            let conferidos: HashSet<&String> = universo.iter().filter(eh_teste).collect();
            for caminho_relativo in lidos.iter().chain(conferidos.iter().copied().filter(|caminho| !lidos.contains(caminho))) {
                let Some(declaracao) = declaracao_configurada.as_ref().or_else(|| riscos::declaracao_padrao(caminho_relativo)) else {
                    continue;
                };
                for teste in riscos::testes_do_arquivo(&ler_texto(&raiz, caminho_relativo), declaracao) {
                    citados_em_algum_teste.extend(teste.riscos.iter().cloned());
                    if !conferidos.contains(caminho_relativo) {
                        continue;
                    }
                    if teste.riscos.is_empty() {
                        violacoes.push(Violacao::em(
                            caminho_relativo,
                            Some(teste.linha),
                            format!("o teste não cita o risco que cobre (RK<n> de {caminho_do_registro}); sem risco, o teste não entra"),
                        ));
                    }
                    for citado in teste.riscos.iter().filter(|citado| !registro.iter().any(|risco| &risco.id == *citado)) {
                        violacoes.push(Violacao::em(
                            caminho_relativo,
                            Some(teste.linha),
                            format!("o teste cita {citado}, que não está em {caminho_do_registro}"),
                        ));
                    }
                }
            }
            if contexto.conferir_o_registro_inteiro {
                for risco in registro.iter().filter(|risco| !risco.pendente && !citados_em_algum_teste.contains(&risco.id)) {
                    violacoes.push(Violacao::em(
                        caminho_do_registro,
                        Some(risco.linha),
                        format!(
                            "{} não tem teste que o cite; escreva o teste ou marque pendente #<issue que vai cobri-lo>",
                            risco.id
                        ),
                    ));
                }
            }
            violacoes
        }

        // "garantido pelo lint": a configuração do lint liga as regras que o projeto delegou a ele.
        // Falha fechado: regra desligada, comentada ou só parecida com outra não conta (veja cobertura_de_lint).
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
            let configs: Vec<(&String, String)> = existentes.iter().map(|caminho| (*caminho, ler_texto(&raiz, caminho))).collect();
            let mut violacoes = analysis_options_por_pasta(&raiz, &caminhos_de_config);
            violacoes.retain(|violacao| {
                let caminho_do_arquivo = violacao.arquivo.clone().unwrap_or_default();
                contexto.analysis_options_ja_mostrados.insert(caminho_do_arquivo)
            });
            for nome_da_regra in lista_de_texto(&parametros["regras"]) {
                // Coberta em qualquer arquivo basta; senão, vale o motivo mais informativo entre eles.
                let mut melhor = (Cobertura::Ausente, existentes[0]);
                for (caminho, texto) in &configs {
                    let cobertura = cobertura_de_lint(caminho, texto, &nome_da_regra);
                    if cobertura > melhor.0 {
                        melhor = (cobertura, *caminho);
                    }
                }
                let (cobertura, arquivo) = melhor;
                let motivo = match cobertura {
                    Cobertura::Coberta => continue,
                    Cobertura::Ausente => "não está configurada".to_string(),
                    Cobertura::Comentada => "só aparece comentada".to_string(),
                    Cobertura::Desligada(nivel) => format!("está configurada como {nivel}, que não bloqueia"),
                    Cobertura::Ilegivel(erro_de_leitura) => format!("não foi conferida: {erro_de_leitura}"),
                };
                violacoes.push(Violacao::em(arquivo, None, format!("a regra de lint \"{nome_da_regra}\" {motivo}")));
            }
            violacoes
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
                            "ast-grep não encontrado ({binario}); instale-o no CI: npm i -g @ast-grep/cli@0.51.0 ou cargo install ast-grep --version 0.51.0 --locked"
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

        // Consultiva: quem a roda é o ci (revisao_por_modelo.rs); o validador nunca chama o modelo.
        TIPO_CONSULTIVO => Vec::new(),

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
        if !PRINCIPIOS.contains(&principio) && !eh_pratica(principio) {
            return erro(format!(
                "{id}: princípio \"{}\" não existe (use um dos 22 princípios, {}, ou uma prática da linguagem: TS<n>, JV<n>, RS<n> ou DT<n>)",
                if regra["principio"].is_null() { "undefined" } else { principio },
                PRINCIPIOS.join(", ")
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
        if tipo == TIPO_CONSULTIVO && verificacao.get("consultivo") != Some(&Value::Bool(true)) {
            return erro(format!(
                "{id}: \"{TIPO_CONSULTIVO}\" só existe consultiva (ADR 0015): declare \"consultivo\": true"
            ));
        }
        regras.push(Regra {
            id: id.into(),
            principio: principio.into(),
            texto: regra["texto"].as_str().unwrap_or("").into(),
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
        conferir_o_registro_inteiro: true,
        analysis_options_ja_mostrados: HashSet::new(),
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

// Hook vigia-teste: as regras do P5 que leem código de teste (teste-cobre-risco e ast-grep) rodam só no
// arquivo que o agente acabou de editar. Configuração inválida e ast-grep ausente ficam para o validador.
pub fn conferir_arquivo_editado(raiz: &Path, config: &Value, caminho_relativo: &str) -> Vec<Violacao> {
    let Ok(regras) = ler_regras(config) else {
        return Vec::new();
    };
    let mut contexto = Contexto {
        raiz: raiz.to_path_buf(),
        projeto: config["projeto"].clone(),
        base: None,
        avisos: Vec::new(),
        arquivos: Vec::new(),
        alterados: Some(vec![caminho_relativo.to_string()]),
        conferir_o_registro_inteiro: false,
        analysis_options_ja_mostrados: HashSet::new(),
    };
    let mut violacoes = Vec::new();
    for regra in regras.iter().filter(|regra| regra.principio == "P5") {
        let tipo = regra.verificacao.get("tipo").and_then(Value::as_str).unwrap_or("");
        if tipo != "teste-cobre-risco" && tipo != "ast-grep" {
            continue;
        }
        let mut parametros = regra.verificacao.clone();
        parametros.remove("tipo");
        if let Ok(achadas) = rodar_checagem(tipo, &parametros, &mut contexto) {
            violacoes.extend(achadas.into_iter().filter(|violacao| violacao.arquivo.is_some()));
        }
    }
    violacoes
}

// Hook vigia-codigo: as regras que leem código fora do P5 e têm forma fixa (ast-grep, termo-evitado,
// todo-sem-issue) rodam só no arquivo que o agente acabou de editar, cada violação com a regra de onde
// nasceu. Comando (lint, typecheck) e checagens do repositório inteiro ficam para o validador e o ci.
pub const TIPOS_RAPIDOS_NO_HOOK: [&str; 3] = ["ast-grep", "termo-evitado", "todo-sem-issue"];

pub struct ViolacaoDeRegra {
    pub id: String,
    pub principio: String,
    pub texto: String,
    pub violacao: Violacao,
}

pub fn conferir_codigo_editado(raiz: &Path, config: &Value, caminho_relativo: &str) -> Vec<ViolacaoDeRegra> {
    let Ok(regras) = ler_regras(config) else {
        return Vec::new();
    };
    let mut contexto = Contexto {
        raiz: raiz.to_path_buf(),
        projeto: config["projeto"].clone(),
        base: None,
        avisos: Vec::new(),
        arquivos: Vec::new(),
        alterados: Some(vec![caminho_relativo.to_string()]),
        conferir_o_registro_inteiro: false,
        analysis_options_ja_mostrados: HashSet::new(),
    };
    let mut violacoes = Vec::new();
    for regra in regras.iter().filter(|regra| regra.principio != "P5") {
        let tipo = regra.verificacao.get("tipo").and_then(Value::as_str).unwrap_or("");
        if !TIPOS_RAPIDOS_NO_HOOK.contains(&tipo) {
            continue;
        }
        let mut parametros = regra.verificacao.clone();
        parametros.remove("tipo");
        if let Ok(achadas) = rodar_checagem(tipo, &parametros, &mut contexto) {
            violacoes.extend(
                achadas
                    .into_iter()
                    .filter(|violacao| violacao.arquivo.is_some())
                    .map(|violacao| ViolacaoDeRegra {
                        id: regra.id.clone(),
                        principio: regra.principio.clone(),
                        texto: regra.texto.clone(),
                        violacao,
                    }),
            );
        }
    }
    violacoes
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

// As linhas de uma regra no relatório: o rótulo com ✅ ou ❌ e cada violação embaixo.
pub fn formatar_resultado(resultado: &Resultado) -> Vec<String> {
    let mut linhas = Vec::new();
    let rotulo = format!("{} ({}) {}", resultado.id, resultado.principio, resultado.tipo);
    if resultado.tipo == "revisor" {
        linhas.push(format!("👁  {rotulo}: fica com o revisor"));
        return linhas;
    }
    if resultado.tipo == TIPO_CONSULTIVO {
        linhas.push(format!("💬 {rotulo}: consultiva, roda no ci"));
        return linhas;
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
    linhas
}

pub fn formatar(relatorio: &Relatorio) -> String {
    let mut linhas = vec![format!("Validador de regras de código: {}", relatorio.escopo)];
    for resultado in &relatorio.resultados {
        linhas.extend(formatar_resultado(resultado));
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
