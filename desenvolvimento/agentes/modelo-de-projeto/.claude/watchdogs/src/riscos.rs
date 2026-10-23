// Registro de riscos (docs/riscos.md) e os testes que citam cada risco: base da checagem
// teste-cobre-risco do validador e do hook vigia-teste (princípio P5, regra 15 da constituição).
// Todo teste nasce de um risco que o humano aprovou; o teste o cita como RK<n> no nome, num
// comentário, atributo ou anotação logo acima, ou num bloco que o contém (describe, mod, classe).

use regex::Regex;
use std::sync::LazyLock;

static ID_NA_TABELA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^RK\d+$").expect("regex válida"));
static ORIGEM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)#\d+|https?://|\badr/|\bADR\s*\d+").expect("regex válida"));
static PENDENTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\bpendente\b").expect("regex válida"));
static ISSUE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"#\d+|/issues/\d+").expect("regex válida"));
static CITACAO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)rk(\d+)").expect("regex válida"));

// Declaração de teste por extensão: a linha que abre o teste, ou o atributo e a anotação dele.
static TESTE_JS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*(?:it|test)\b[\w.]*\s*\(").expect("regex válida"));
static TESTE_RUST: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*#\[(?:\w+::)*test\]").expect("regex válida"));
static TESTE_JVM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*@(?:Test|ParameterizedTest|RepeatedTest|TestFactory|TestTemplate)\b").expect("regex válida"));
static TESTE_PYTHON: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*(?:async\s+)?def\s+test").expect("regex válida"));
static TESTE_GO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^func\s+(?:Test|Fuzz)\w*\(").expect("regex válida"));

pub const TESTES_PADRAO: [&str; 12] = [
    "**/*.test.*",
    "**/*.spec.*",
    "**/*_test.*",
    "**/test_*.py",
    "**/tests/**",
    "**/test/**",
    "**/__tests__/**",
    "**/*.rs",
    "**/*Test.java",
    "**/*Tests.java",
    "**/*IT.java",
    "**/*Test.kt",
];

pub struct Risco {
    pub id: String,
    pub linha: u64,
    pub pendente: bool,
}

pub struct ProblemaNoRegistro {
    pub linha: u64,
    pub mensagem: String,
}

// Linhas de tabela cuja primeira célula é RK<n>. Cada risco cita a origem (a issue, o ticket ou a ADR
// em que o humano o aprovou); o que ainda não tem teste leva "pendente #<issue que vai cobri-lo>".
pub fn ler_registro(texto_md: &str) -> (Vec<Risco>, Vec<ProblemaNoRegistro>) {
    let (mut riscos, mut problemas): (Vec<Risco>, Vec<ProblemaNoRegistro>) = (Vec::new(), Vec::new());
    for (indice, linha) in texto_md.lines().enumerate() {
        let partes: Vec<&str> = linha.split('|').collect();
        if partes.len() < 3 {
            continue;
        }
        let celulas: Vec<&str> = partes[1..partes.len() - 1].iter().map(|celula| celula.trim()).collect();
        if !ID_NA_TABELA.is_match(celulas[0]) {
            continue;
        }
        let (id, numero_da_linha) = (celulas[0].to_string(), indice as u64 + 1);
        let mut problema = |mensagem: String| {
            problemas.push(ProblemaNoRegistro {
                linha: numero_da_linha,
                mensagem,
            })
        };
        if riscos.iter().any(|risco: &Risco| risco.id == id) {
            problema(format!("{id} aparece duas vezes no registro"));
        }
        let resto = &celulas[1..];
        if !resto.iter().any(|celula| ORIGEM.is_match(celula) && !PENDENTE.is_match(celula)) {
            problema(format!("{id} sem origem: cite a issue, o ticket ou a ADR em que o humano aprovou o risco"));
        }
        let celula_pendente = resto.iter().find(|celula| PENDENTE.is_match(celula));
        if celula_pendente.is_some_and(|celula| !ISSUE.is_match(celula)) {
            problema(format!("{id} pendente sem a issue que vai cobri-lo (pendente #<n>)"));
        }
        riscos.push(Risco {
            id,
            linha: numero_da_linha,
            pendente: celula_pendente.is_some(),
        });
    }
    (riscos, problemas)
}

// IDs citados num trecho, normalizados para RK<n>. Aceita "RK3", "rk3_recusa" e "TestRK3Recusa", mas
// não o "rk" no meio de uma palavra ("work1", "MARK2").
pub fn riscos_citados(trecho: &str) -> Vec<String> {
    let letras: Vec<char> = trecho.chars().collect();
    let mut citados = Vec::new();
    for citacao in CITACAO.captures_iter(trecho) {
        let inicio = citacao.get(0).map(|encontrado| encontrado.start()).unwrap_or(0);
        let posicao = trecho[..inicio].chars().count();
        let anterior = posicao.checked_sub(1).map(|indice| letras[indice]);
        let em_maiusculas = letras.get(posicao) == Some(&'R');
        let fronteira = match anterior {
            None => true,
            Some(letra) => !letra.is_alphanumeric() || (em_maiusculas && letra.is_lowercase()),
        };
        if fronteira {
            citados.push(format!("RK{}", &citacao[1]));
        }
    }
    citados
}

pub fn declaracao_padrao(caminho_relativo: &str) -> Option<&'static Regex> {
    let extensao = caminho_relativo.rsplit_once('.').map(|(_, extensao)| extensao).unwrap_or("");
    match extensao {
        "ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs" => Some(&TESTE_JS),
        "rs" => Some(&TESTE_RUST),
        "java" | "kt" => Some(&TESTE_JVM),
        "py" => Some(&TESTE_PYTHON),
        "go" => Some(&TESTE_GO),
        _ => None,
    }
}

pub struct TesteDeclarado {
    pub linha: u64,
    pub riscos: Vec<String>,
}

fn recuo(linha: &str) -> usize {
    linha
        .chars()
        .take_while(|letra| letra.is_whitespace())
        .map(|letra| if letra == '\t' { 4 } else { 1 })
        .sum()
}

// Comentário, atributo, anotação ou decorador: o que fica colado em cima de um teste ou de um bloco.
fn eh_cabecalho(linha: &str) -> bool {
    let aparada = linha.trim_start();
    ["//", "/*", "*", "#", "@"].iter().any(|prefixo| aparada.starts_with(prefixo))
}

fn cabecalho_acima(linhas: &[&str], indice: usize) -> Vec<String> {
    linhas[..indice]
        .iter()
        .rev()
        .take_while(|linha| eh_cabecalho(linha))
        .map(|linha| linha.to_string())
        .collect()
}

// Cada teste do arquivo, com os riscos que ele cita: na declaração, nas linhas de atributo até o nome
// (Rust, Java), no cabeçalho logo acima e nas linhas que o contêm, achadas pelo recuo menor.
pub fn testes_do_arquivo(texto: &str, declaracao: &Regex) -> Vec<TesteDeclarado> {
    let linhas: Vec<&str> = texto.lines().collect();
    let mut testes = Vec::new();
    for (indice, linha) in linhas.iter().enumerate() {
        if !declaracao.is_match(linha) {
            continue;
        }
        let mut trechos: Vec<String> = vec![linha.to_string()];
        let mut abaixo = indice;
        while eh_cabecalho(linhas[abaixo]) && abaixo + 1 < linhas.len() {
            abaixo += 1;
            trechos.push(linhas[abaixo].to_string());
        }
        trechos.extend(cabecalho_acima(&linhas, indice));
        let mut nivel = recuo(linha);
        for acima in (0..indice).rev() {
            if nivel == 0 {
                break;
            }
            let candidata = linhas[acima];
            let aparada = candidata.trim_start();
            if aparada.is_empty() || aparada.starts_with(['}', ')', ']']) || recuo(candidata) >= nivel {
                continue;
            }
            nivel = recuo(candidata);
            trechos.push(candidata.to_string());
            trechos.extend(cabecalho_acima(&linhas, acima));
        }
        let mut riscos: Vec<String> = trechos.iter().flat_map(|trecho| riscos_citados(trecho)).collect();
        riscos.sort();
        riscos.dedup();
        testes.push(TesteDeclarado {
            linha: indice as u64 + 1,
            riscos,
        });
    }
    testes
}
