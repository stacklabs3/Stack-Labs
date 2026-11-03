//! Compara três formas de mostrar um teste à revisao-por-modelo (issue stacklabs1/Stack-Labs#53).
//!
//! - `atual`: o trecho que o watchdog envia hoje (cabeçalho colado, declaração e corpo).
//! - `auxiliares`: o trecho atual, mais as importações, as declarações de topo que ele usa (até 3
//!   níveis) e os ganchos (`beforeEach` e afins) dos `describe` que o envolvem. Sem o título do `describe`.
//! - `arquivo`: o arquivo de teste inteiro, numerado, com a linha do teste a julgar.
//!
//! Gabarito: cada teste ganha uma cópia com o RK trocado por outro risco do registro. A resposta
//! esperada é exercita=false. Chama o `claude -p` com as flags do watchdog, sem cache. Só JS e TS.

use regex::{Captures, Regex};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

// Copiados de .claude/watchdogs/src/revisao_por_modelo.rs. Se o watchdog mudar, atualize aqui.
const MODELO_PADRAO: &str = "claude-haiku-5-5";
const TETO_PADRAO_USD: &str = "0.05";
const LINHAS_MAXIMAS_DO_TRECHO: usize = 150;
const ESQUEMA: &str = r#"{"type":"object","properties":{"exercita":{"type":"boolean"},"motivo":{"type":"string"}},"required":["exercita","motivo"]}"#;
const INSTRUCAO_BASE: &str = "Você revisa um teste automatizado contra o risco que ele diz cobrir. O teste exercita o risco \
quando falharia se o risco acontecesse: ele monta o cenário do risco e afirma o resultado que o risco quebraria. \
Responda exercita=false quando o teste só cita o risco no nome ou num comentário, afirma outra coisa (um getter, \
o valor que acabou de montar, uma chamada de mock) ou passaria mesmo com o risco acontecendo. ";
const PEDIDO_DE_MOTIVO: &str = " Em motivo, uma ou duas frases em português que digam o que o teste afirma e o que faltaria.";
const VARIANTES: [&str; 3] = ["atual", "auxiliares", "arquivo"];

const AJUDA: &str = "uso: comparar-revisao [opções] <arquivo de teste>...

  --projeto <pasta>      raiz do projeto, onde fica docs/riscos.md (padrão: .)
  --riscos <caminho>     registro de riscos, relativo ao projeto (padrão: docs/riscos.md)
  --modelo <id>          padrão: o do watchdog
  --teto <usd>           teto por chamada (padrão: 0.05, o do watchdog)
  --limite <segundos>    por chamada (padrão: 120)
  --variantes <lista>    padrão: atual,auxiliares,arquivo
  --simultaneas <n>      chamadas ao mesmo tempo (padrão: 4)
  --semente <n>          escolha do RK trocado no gabarito (padrão: 53)
  --saida <pasta>        relativa ao projeto (padrão: .scratch/comparacao-revisao-por-modelo)
  --sem-gabarito         não gera as cópias com RK trocado
  --simular              só monta os prompts e mede o tamanho, sem chamar o modelo";

static DECLARACAO_JS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*(?:it|test)\b[\w.]*\s*\(").expect("regex válida"));
static GANCHO_JS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?:beforeEach|beforeAll|afterEach|afterAll)\s*\(").expect("regex válida"));
static DESCRIBE_JS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*describe\b[\w.]*\s*\(").expect("regex válida"));
static CITACAO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)rk(\d+)").expect("regex válida"));
static ID_NA_TABELA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^RK\d+$").expect("regex válida"));
static NOME_JS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\b(?:it|test)\b[\w.]*\s*\(\s*["'`]([^"'`]*)"#).expect("regex válida"));
static DECLARACAO_DE_TOPO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:export\s+)?(?:default\s+)?(?:async\s+)?(?:const|let|var|function\*?|class|type|interface|enum)\s+([A-Za-z_$][\w$]*)")
        .expect("regex válida")
});
static IDENTIFICADOR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[A-Za-z_$][\w$]*").expect("regex válida"));

struct Opcoes {
    projeto: PathBuf,
    riscos: String,
    modelo: String,
    teto_usd: String,
    limite: Duration,
    variantes: Vec<String>,
    simultaneas: usize,
    semente: u64,
    saida: String,
    com_gabarito: bool,
    simular: bool,
    arquivos: Vec<String>,
}

fn ler_opcoes() -> Result<Opcoes, String> {
    let mut opcoes = Opcoes {
        projeto: PathBuf::from("."),
        riscos: "docs/riscos.md".into(),
        modelo: MODELO_PADRAO.into(),
        teto_usd: TETO_PADRAO_USD.into(),
        limite: Duration::from_secs(120),
        variantes: VARIANTES.iter().map(|variante| variante.to_string()).collect(),
        simultaneas: 4,
        semente: 53,
        saida: ".scratch/comparacao-revisao-por-modelo".into(),
        com_gabarito: true,
        simular: false,
        arquivos: Vec::new(),
    };
    let mut argumentos = std::env::args().skip(1);
    while let Some(argumento) = argumentos.next() {
        let mut valor = || argumentos.next().ok_or(format!("{argumento} pede um valor"));
        match argumento.as_str() {
            "--projeto" => opcoes.projeto = PathBuf::from(valor()?),
            "--riscos" => opcoes.riscos = valor()?,
            "--modelo" => opcoes.modelo = valor()?,
            "--teto" => opcoes.teto_usd = valor()?,
            "--limite" => opcoes.limite = Duration::from_secs(valor()?.parse().map_err(|_| "--limite pede segundos")?),
            "--variantes" => {
                opcoes.variantes = valor()?.split(',').map(|variante| variante.trim().to_string()).collect();
                if let Some(desconhecida) = opcoes.variantes.iter().find(|variante| !VARIANTES.contains(&variante.as_str())) {
                    return Err(format!("variante desconhecida: {desconhecida}"));
                }
            }
            "--simultaneas" => opcoes.simultaneas = valor()?.parse().map_err(|_| "--simultaneas pede um número")?,
            "--semente" => opcoes.semente = valor()?.parse().map_err(|_| "--semente pede um número")?,
            "--saida" => opcoes.saida = valor()?,
            "--sem-gabarito" => opcoes.com_gabarito = false,
            "--simular" => opcoes.simular = true,
            "-h" | "--help" => return Err(AJUDA.into()),
            outro if outro.starts_with("--") => return Err(format!("opção desconhecida: {outro}\n\n{AJUDA}")),
            arquivo => opcoes.arquivos.push(arquivo.to_string()),
        }
    }
    if opcoes.arquivos.is_empty() {
        return Err(AJUDA.into());
    }
    opcoes.simultaneas = opcoes.simultaneas.max(1);
    Ok(opcoes)
}

fn recuo(linha: &str) -> usize {
    linha.chars().take_while(|letra| letra.is_whitespace()).count()
}

fn eh_cabecalho(linha: &str) -> bool {
    let aparada = linha.trim_start();
    ["//", "/*", "*", "#", "@"].iter().any(|prefixo| aparada.starts_with(prefixo))
}

// Mesma fronteira de riscos_citados (riscos.rs): "RK3", "rk3_x" e "TestRK3", mas não "work1".
fn citacao_valida(texto: &str, inicio: usize) -> bool {
    let anterior = texto[..inicio].chars().next_back();
    let maiuscula = texto[inicio..].starts_with('R');
    match anterior {
        None => true,
        Some(letra) => !letra.is_alphanumeric() || (maiuscula && letra.is_lowercase()),
    }
}

fn riscos_citados(texto: &str) -> Vec<String> {
    CITACAO
        .captures_iter(texto)
        .filter(|citacao| citacao_valida(texto, citacao.get(0).map_or(0, |achado| achado.start())))
        .map(|citacao| format!("RK{}", &citacao[1]))
        .collect()
}

fn ler_registro(caminho: &Path) -> Result<BTreeMap<String, String>, String> {
    let texto = fs::read_to_string(caminho).map_err(|erro| format!("não li {}: {erro}", caminho.display()))?;
    let mut registro = BTreeMap::new();
    for linha in texto.lines() {
        let partes: Vec<&str> = linha.split('|').collect();
        if partes.len() >= 3 && ID_NA_TABELA.is_match(partes[1].trim()) {
            registro.entry(partes[1].trim().to_string()).or_insert_with(|| linha.trim().to_string());
        }
    }
    Ok(registro)
}

fn inicio_com_cabecalho(linhas: &[String], mut indice: usize) -> usize {
    while indice > 0 && eh_cabecalho(&linhas[indice - 1]) {
        indice -= 1;
    }
    indice
}

// Índice (exclusivo) da linha que fecha o bloco aberto em `inicio`; sem chave, a linha que termina em ; ou ).
fn fim_do_bloco(linhas: &[String], inicio: usize) -> usize {
    let (mut profundidade, mut abriu) = (0i64, false);
    for (indice, linha) in linhas.iter().enumerate().skip(inicio) {
        for letra in linha.chars() {
            match letra {
                '{' => {
                    profundidade += 1;
                    abriu = true;
                }
                '}' => profundidade -= 1,
                _ => {}
            }
        }
        if abriu && profundidade <= 0 {
            return indice + 1;
        }
        if !abriu && indice > inicio && linha.trim_end().ends_with([';', ')']) {
            return indice + 1;
        }
    }
    linhas.len()
}

// Porta de trecho_do_teste (revisao_por_modelo.rs): devolve (início, fim). Sem chaves nas 6 primeiras
// linhas, corta por recuo como o watchdog, o que num it.each de tabela longa deixa o corpo de fora.
fn trecho_atual(linhas: &[String], indice: usize) -> (usize, usize) {
    let inicio = inicio_com_cabecalho(linhas, indice);
    let limite = (indice + LINHAS_MAXIMAS_DO_TRECHO).min(linhas.len());
    let tem_chaves = linhas[indice..limite.min(indice + 6)].iter().any(|linha| linha.contains('{'));
    if !tem_chaves {
        let recuo_da_declaracao = recuo(&linhas[indice]);
        let fim = linhas[indice + 1..limite]
            .iter()
            .position(|linha| !linha.trim().is_empty() && recuo(linha) <= recuo_da_declaracao)
            .map_or(limite, |deslocamento| indice + 1 + deslocamento);
        return (inicio, fim);
    }
    let (mut profundidade, mut abriu) = (0i64, false);
    for (posicao, linha) in linhas.iter().enumerate().take(limite).skip(indice) {
        for letra in linha.chars() {
            match letra {
                '{' => {
                    profundidade += 1;
                    abriu = true;
                }
                '}' => profundidade -= 1,
                _ => {}
            }
        }
        if abriu && profundidade <= 0 {
            return (inicio, posicao + 1);
        }
    }
    (inicio, limite)
}

// Linhas dos blocos que envolvem a linha `indice`, do mais interno para fora.
fn ancestrais(linhas: &[String], indice: usize) -> Vec<usize> {
    let (mut encontrados, mut nivel) = (Vec::new(), recuo(&linhas[indice]));
    for acima in (0..indice).rev() {
        if nivel == 0 {
            break;
        }
        let candidata = &linhas[acima];
        let aparada = candidata.trim_start();
        if aparada.is_empty() || aparada.starts_with(['}', ')', ']']) || recuo(candidata) >= nivel {
            continue;
        }
        nivel = recuo(candidata);
        encontrados.push(acima);
    }
    encontrados
}

// Mesma ideia de testes_do_arquivo (riscos.rs): declaração, cabeçalhos e blocos que envolvem.
fn riscos_do_teste(linhas: &[String], indice: usize) -> Vec<String> {
    let mut trechos = vec![linhas[indice].as_str()];
    let mut abaixo = indice;
    while abaixo + 1 < linhas.len() && eh_cabecalho(&linhas[abaixo]) {
        abaixo += 1;
        trechos.push(&linhas[abaixo]);
    }
    for posicao in std::iter::once(indice).chain(ancestrais(linhas, indice)) {
        trechos.push(&linhas[posicao]);
        trechos.extend(linhas[inicio_com_cabecalho(linhas, posicao)..posicao].iter().map(String::as_str));
    }
    let citados: BTreeSet<String> = trechos.iter().flat_map(|trecho| riscos_citados(trecho)).collect();
    citados.into_iter().collect()
}

fn juntar(linhas: &[String], inicio: usize, fim: usize) -> String {
    linhas[inicio..fim].join("\n")
}

fn contexto_auxiliar(linhas: &[String], indice: usize, inicio: usize, fim: usize) -> String {
    let envolventes: BTreeSet<usize> = ancestrais(linhas, indice).into_iter().collect();
    let ganchos: Vec<(usize, usize)> = (0..indice)
        .filter(|posicao| GANCHO_JS.is_match(&linhas[*posicao]))
        .filter(|posicao| ancestrais(linhas, *posicao).first().is_none_or(|dono| envolventes.contains(dono)))
        .map(|posicao| (inicio_com_cabecalho(linhas, posicao), fim_do_bloco(linhas, posicao)))
        .collect();
    let mut declaracoes: HashMap<String, (usize, usize)> = HashMap::new();
    for (posicao, linha) in linhas.iter().enumerate() {
        if recuo(linha) != 0 || DESCRIBE_JS.is_match(linha) {
            continue;
        }
        if let Some(achado) = DECLARACAO_DE_TOPO.captures(linha) {
            declaracoes.insert(achado[1].to_string(), (inicio_com_cabecalho(linhas, posicao), fim_do_bloco(linhas, posicao)));
        }
    }
    let mut escolhidas: BTreeSet<String> = BTreeSet::new();
    let mut a_olhar = juntar(linhas, inicio, fim);
    for (comeco, final_do_gancho) in &ganchos {
        a_olhar.push('\n');
        a_olhar.push_str(&juntar(linhas, *comeco, *final_do_gancho));
    }
    for _ in 0..3 {
        let novos: BTreeSet<String> = IDENTIFICADOR
            .find_iter(&a_olhar)
            .map(|achado| achado.as_str().to_string())
            .filter(|nome| declaracoes.contains_key(nome) && !escolhidas.contains(nome))
            .collect();
        if novos.is_empty() {
            break;
        }
        a_olhar = novos
            .iter()
            .map(|nome| juntar(linhas, declaracoes[nome].0, declaracoes[nome].1))
            .collect::<Vec<_>>()
            .join("\n");
        escolhidas.extend(novos);
    }
    let blocos: BTreeSet<(usize, usize)> = escolhidas.iter().map(|nome| declaracoes[nome]).collect();
    let mut partes = Vec::new();
    let importacoes: Vec<&str> = linhas
        .iter()
        .filter(|linha| linha.starts_with("import ") || (linha.starts_with("const ") && linha.contains("require(")))
        .map(String::as_str)
        .collect();
    if !importacoes.is_empty() {
        partes.push(format!("// importações do arquivo\n{}", importacoes.join("\n")));
    }
    partes.extend(blocos.iter().map(|(comeco, final_do_bloco)| juntar(linhas, *comeco, *final_do_bloco)));
    partes.extend(
        ganchos
            .iter()
            .map(|(comeco, final_do_gancho)| format!("// gancho do describe que envolve o teste\n{}", juntar(linhas, *comeco, *final_do_gancho))),
    );
    partes.join("\n\n")
}

fn montar_prompt(variante: &str, arquivo: &str, linhas_dos_riscos: &[String], linhas: &[String], indice: usize) -> String {
    let fim_da_instrucao = match variante {
        "atual" => "Julgue só pelo trecho abaixo.",
        "auxiliares" => "Julgue pelo teste e pelo contexto do arquivo abaixo dele.",
        _ => "Julgue o teste indicado, usando o resto do arquivo como contexto.",
    };
    let instrucao = format!("{INSTRUCAO_BASE}{fim_da_instrucao}{PEDIDO_DE_MOTIVO}");
    let riscos = linhas_dos_riscos.join("\n");
    let linha = indice + 1;
    let (inicio, fim) = trecho_atual(linhas, indice);
    let trecho = juntar(linhas, inicio, fim);
    let cabeca = format!("{instrucao}\n\nRisco citado (linha de docs/riscos.md):\n{riscos}\n\n");
    match variante {
        "atual" => format!("{cabeca}Teste ({arquivo}:{linha}):\n```\n{trecho}\n```"),
        "auxiliares" => {
            let contexto = contexto_auxiliar(linhas, indice, inicio, fim);
            let contexto = if contexto.is_empty() { "(nenhum auxiliar de fora)".to_string() } else { contexto };
            format!("{cabeca}Teste ({arquivo}:{linha}):\n```\n{trecho}\n```\n\nContexto do arquivo usado pelo teste:\n```\n{contexto}\n```")
        }
        _ => {
            let numerado: Vec<String> = linhas.iter().enumerate().map(|(posicao, texto)| format!("{:>4}| {texto}", posicao + 1)).collect();
            format!(
                "{cabeca}Teste a julgar: {arquivo}:{linha} (o bloco que começa na linha {linha}).\n\nArquivo inteiro:\n```\n{}\n```",
                numerado.join("\n")
            )
        }
    }
}

// Troca cada RK citado pelo novo, no arquivo todo, com a mesma fronteira de riscos_citados.
fn trocar_rk(linhas: &[String], citados: &[String], novo: &str) -> Vec<String> {
    let numeros: BTreeSet<&str> = citados.iter().map(|citado| &citado[2..]).collect();
    linhas
        .iter()
        .map(|linha| {
            CITACAO
                .replace_all(linha, |citacao: &Captures| {
                    let achado = citacao.get(0).expect("captura inteira");
                    if citacao_valida(linha, achado.start()) && numeros.contains(&citacao[1]) {
                        format!("{}{}", &achado.as_str()[..2], &novo[2..])
                    } else {
                        achado.as_str().to_string()
                    }
                })
                .into_owned()
        })
        .collect()
}

// Gerador simples e determinístico (xorshift), para o gabarito sair igual com a mesma semente.
struct Sorteio(u64);

impl Sorteio {
    fn escolher<'lista, Item>(&mut self, opcoes: &'lista [Item]) -> &'lista Item {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        &opcoes[(self.0 % opcoes.len() as u64) as usize]
    }
}

struct Pedido {
    variante: String,
    tipo: &'static str,
    arquivo: String,
    linha: usize,
    nome: String,
    riscos_originais: String,
    risco_no_prompt: String,
    prompt: String,
    arquivo_do_prompt: String,
}

#[derive(Default, Clone)]
struct Resposta {
    exercita: Option<bool>,
    motivo: String,
    erro: Option<String>,
    custo: Option<f64>,
}

fn perguntar(opcoes: &Opcoes, raiz: &Path, prompt: &str) -> Resposta {
    let falha = |erro: String| Resposta { erro: Some(erro), ..Resposta::default() };
    let processo = Command::new("claude")
        .args(["-p", "--setting-sources", "", "--strict-mcp-config", "--model", &opcoes.modelo, "--tools", ""])
        .args(["--no-session-persistence", "--output-format", "json", "--json-schema", ESQUEMA])
        .args(["--max-budget-usd", &opcoes.teto_usd, prompt])
        .env("STACKLABS_WATCHDOGS", "desligado")
        .current_dir(raiz)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut processo = match processo {
        Ok(processo) => processo,
        Err(erro) => return falha(format!("o claude não iniciou ({erro})")),
    };
    let prazo = Instant::now() + opcoes.limite;
    loop {
        match processo.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < prazo => std::thread::sleep(Duration::from_millis(100)),
            _ => {
                let _ = processo.kill();
                let _ = processo.wait();
                return falha(format!("sem resposta em {} s", opcoes.limite.as_secs()));
            }
        }
    }
    // A saída do claude -p em JSON é pequena; ler depois de sair não trava o cano.
    let saida = match processo.wait_with_output() {
        Ok(saida) => saida,
        Err(erro) => return falha(format!("não li a saída do claude ({erro})")),
    };
    let texto = String::from_utf8_lossy(&saida.stdout);
    let Ok(resposta) = serde_json::from_str::<Value>(texto.trim()) else {
        let resumo: String = format!("{texto} {}", String::from_utf8_lossy(&saida.stderr)).trim().chars().take(200).collect();
        return falha(format!("resposta fora de JSON: {resumo}"));
    };
    let custo = resposta["total_cost_usd"].as_f64();
    if resposta["is_error"].as_bool() == Some(true) {
        let motivo: String = resposta["result"].as_str().unwrap_or("erro sem mensagem").chars().take(200).collect();
        return Resposta { custo, ..falha(motivo) };
    }
    match resposta["structured_output"]["exercita"].as_bool() {
        Some(exercita) => Resposta {
            exercita: Some(exercita),
            motivo: resposta["structured_output"]["motivo"].as_str().unwrap_or("").trim().to_string(),
            erro: None,
            custo,
        },
        None => Resposta { custo, ..falha("resposta sem structured_output.exercita".into()) },
    }
}

fn celula_csv(valor: &str) -> String {
    if valor.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", valor.replace('"', "\"\""))
    } else {
        valor.to_string()
    }
}

fn linha_csv(celulas: &[String]) -> String {
    celulas.iter().map(|celula| celula_csv(celula)).collect::<Vec<_>>().join(",") + "\n"
}

fn veredito_em_texto(resposta: &Resposta) -> String {
    match resposta.exercita {
        Some(exercita) => exercita.to_string(),
        None => "erro".into(),
    }
}

// Um teste real na revisão humana: nome, riscos citados e o veredito de cada variante.
type TesteReal = (String, String, HashMap<String, Resposta>);

// Um caso a perguntar: o tipo (real ou gabarito), os riscos do prompt e as linhas do arquivo.
type Caso = (&'static str, Vec<String>, Vec<String>);

fn montar_pedidos(opcoes: &Opcoes, raiz: &Path, registro: &BTreeMap<String, String>) -> Result<Vec<Pedido>, String> {
    let mut sorteio = Sorteio(opcoes.semente.max(1));
    let mut pedidos = Vec::new();
    for arquivo in &opcoes.arquivos {
        let texto = fs::read_to_string(raiz.join(arquivo)).map_err(|erro| format!("não li {arquivo}: {erro}"))?;
        let linhas: Vec<String> = texto.lines().map(String::from).collect();
        for indice in (0..linhas.len()).filter(|indice| DECLARACAO_JS.is_match(&linhas[*indice])) {
            let citados: Vec<String> = riscos_do_teste(&linhas, indice).into_iter().filter(|citado| registro.contains_key(citado)).collect();
            if citados.is_empty() {
                continue;
            }
            let nome = NOME_JS.captures(&linhas[indice]).map(|achado| achado[1].to_string()).unwrap_or_default();
            let mut casos: Vec<Caso> = vec![("real", citados.clone(), linhas.clone())];
            let outros: Vec<&String> = registro.keys().filter(|risco| !citados.contains(risco)).collect();
            if opcoes.com_gabarito && !outros.is_empty() {
                let trocado = sorteio.escolher(&outros).to_string();
                casos.push(("gabarito", vec![trocado.clone()], trocar_rk(&linhas, &citados, &trocado)));
            }
            for (tipo, riscos, texto_do_caso) in casos {
                let linhas_dos_riscos: Vec<String> = riscos.iter().map(|risco| registro[risco].clone()).collect();
                for variante in &opcoes.variantes {
                    pedidos.push(Pedido {
                        variante: variante.clone(),
                        tipo,
                        arquivo: arquivo.clone(),
                        linha: indice + 1,
                        nome: nome.clone(),
                        riscos_originais: citados.join(" "),
                        risco_no_prompt: riscos.join(" "),
                        prompt: montar_prompt(variante, arquivo, &linhas_dos_riscos, &texto_do_caso, indice),
                        arquivo_do_prompt: String::new(),
                    });
                }
            }
        }
    }
    Ok(pedidos)
}

fn escrever(caminho: &Path, conteudo: &str) -> Result<(), String> {
    fs::write(caminho, conteudo).map_err(|erro| format!("não gravei {}: {erro}", caminho.display()))
}

fn rodar() -> Result<(), String> {
    let opcoes = ler_opcoes()?;
    let raiz = fs::canonicalize(&opcoes.projeto).map_err(|erro| format!("projeto {}: {erro}", opcoes.projeto.display()))?;
    let registro = ler_registro(&raiz.join(&opcoes.riscos))?;
    if registro.is_empty() {
        return Err(format!("nenhum RK em {}", raiz.join(&opcoes.riscos).display()));
    }
    let saida = raiz.join(&opcoes.saida);
    let pasta_dos_prompts = saida.join("prompts");
    fs::create_dir_all(&pasta_dos_prompts).map_err(|erro| format!("não criei {}: {erro}", pasta_dos_prompts.display()))?;

    let mut pedidos = montar_pedidos(&opcoes, &raiz, &registro)?;
    for (numero, pedido) in pedidos.iter_mut().enumerate() {
        pedido.arquivo_do_prompt = format!("{numero:04}-{}-{}.txt", pedido.variante, pedido.tipo);
        escrever(&pasta_dos_prompts.join(&pedido.arquivo_do_prompt), &pedido.prompt)?;
    }

    if opcoes.simular {
        for variante in &opcoes.variantes {
            let tamanhos: Vec<usize> = pedidos.iter().filter(|pedido| &pedido.variante == variante).map(|pedido| pedido.prompt.chars().count()).collect();
            if let Some(maximo) = tamanhos.iter().max() {
                println!("{variante:<11} {:>4} prompts, média {} caracteres, máximo {maximo}", tamanhos.len(), tamanhos.iter().sum::<usize>() / tamanhos.len());
            }
        }
        println!("prompts em {}", pasta_dos_prompts.display());
        return Ok(());
    }

    eprintln!("{} chamadas ao {}, {} por vez...", pedidos.len(), opcoes.modelo, opcoes.simultaneas);
    let mut respostas: Vec<Resposta> = Vec::with_capacity(pedidos.len());
    for (numero_do_lote, lote) in pedidos.chunks(opcoes.simultaneas).enumerate() {
        let respostas_do_lote: Vec<Resposta> = std::thread::scope(|escopo| {
            let tarefas: Vec<_> = lote.iter().map(|pedido| escopo.spawn(|| perguntar(&opcoes, &raiz, &pedido.prompt))).collect();
            tarefas
                .into_iter()
                .map(|tarefa| tarefa.join().unwrap_or_else(|_| Resposta { erro: Some("a chamada entrou em pânico".into()), ..Resposta::default() }))
                .collect()
        });
        respostas.extend(respostas_do_lote);
        eprintln!("  {} de {}", (numero_do_lote * opcoes.simultaneas + lote.len()), pedidos.len());
    }

    let mut resultados = linha_csv(
        &["variante", "tipo", "arquivo", "linha", "nome", "riscos_originais", "risco_no_prompt", "exercita", "motivo", "erro", "custo", "caracteres", "arquivo_do_prompt"]
            .map(String::from),
    );
    for (pedido, resposta) in pedidos.iter().zip(&respostas) {
        resultados += &linha_csv(&[
            pedido.variante.clone(),
            pedido.tipo.into(),
            pedido.arquivo.clone(),
            pedido.linha.to_string(),
            pedido.nome.clone(),
            pedido.riscos_originais.clone(),
            pedido.risco_no_prompt.clone(),
            veredito_em_texto(resposta),
            resposta.motivo.clone(),
            resposta.erro.clone().unwrap_or_default(),
            resposta.custo.map(|custo| format!("{custo:.6}")).unwrap_or_default(),
            pedido.prompt.chars().count().to_string(),
            pedido.arquivo_do_prompt.clone(),
        ]);
    }
    escrever(&saida.join("resultados.csv"), &resultados)?;

    // Para a revisão humana: cada teste real com o veredito de cada variante lado a lado.
    let mut por_teste: BTreeMap<(String, usize), TesteReal> = BTreeMap::new();
    for (pedido, resposta) in pedidos.iter().zip(&respostas).filter(|(pedido, _)| pedido.tipo == "real") {
        por_teste
            .entry((pedido.arquivo.clone(), pedido.linha))
            .or_insert_with(|| (pedido.nome.clone(), pedido.riscos_originais.clone(), HashMap::new()))
            .2
            .insert(pedido.variante.clone(), resposta.clone());
    }
    let mut cabecalho = vec!["arquivo".to_string(), "linha".into(), "nome".into(), "riscos".into()];
    cabecalho.extend(opcoes.variantes.iter().map(|variante| format!("{variante}_exercita")));
    cabecalho.extend(opcoes.variantes.iter().map(|variante| format!("{variante}_motivo")));
    cabecalho.extend(["humano_exercita (true/false)".to_string(), "nota".into()]);
    let mut revisao_humana = linha_csv(&cabecalho);
    for ((arquivo, linha), (nome, riscos, vereditos)) in &por_teste {
        let mut celulas = vec![arquivo.clone(), linha.to_string(), nome.clone(), riscos.clone()];
        celulas.extend(opcoes.variantes.iter().map(|variante| vereditos.get(variante).map(veredito_em_texto).unwrap_or_default()));
        celulas.extend(opcoes.variantes.iter().map(|variante| {
            vereditos.get(variante).map(|resposta| resposta.erro.clone().unwrap_or_else(|| resposta.motivo.clone())).unwrap_or_default()
        }));
        celulas.extend([String::new(), String::new()]);
        revisao_humana += &linha_csv(&celulas);
    }
    escrever(&saida.join("revisao-humana.csv"), &revisao_humana)?;

    let mut resumo = format!(
        "# Comparação da revisao-por-modelo\n\nModelo: `{}`. Teto por chamada: US$ {}. Sem cache.\n\n\
         | Variante | Testes reais | Suspeitos | Gabarito pego | Sem veredito | Custo médio (US$) | Custo máximo (US$) | Caracteres médios |\n\
         |---|---:|---:|---:|---:|---:|---:|---:|\n",
        opcoes.modelo, opcoes.teto_usd
    );
    for variante in &opcoes.variantes {
        let da_variante: Vec<(&Pedido, &Resposta)> = pedidos.iter().zip(&respostas).filter(|(pedido, _)| &pedido.variante == variante).collect();
        let reais = da_variante.iter().filter(|(pedido, _)| pedido.tipo == "real").count();
        let gabarito = da_variante.len() - reais;
        let suspeitos = da_variante.iter().filter(|(pedido, resposta)| pedido.tipo == "real" && resposta.exercita == Some(false)).count();
        let pegos = da_variante.iter().filter(|(pedido, resposta)| pedido.tipo == "gabarito" && resposta.exercita == Some(false)).count();
        let sem_veredito = da_variante.iter().filter(|(_, resposta)| resposta.erro.is_some()).count();
        let custos: Vec<f64> = da_variante.iter().filter_map(|(_, resposta)| resposta.custo).collect();
        let custo_medio = if custos.is_empty() { 0.0 } else { custos.iter().sum::<f64>() / custos.len() as f64 };
        let custo_maximo = custos.iter().cloned().fold(0.0, f64::max);
        let caracteres = da_variante.iter().map(|(pedido, _)| pedido.prompt.chars().count()).sum::<usize>() / da_variante.len().max(1);
        let _ = writeln!(
            resumo,
            "| {variante} | {reais} | {suspeitos} | {pegos} de {gabarito} | {sem_veredito} | {custo_medio:.4} | {custo_maximo:.4} | {caracteres} |"
        );
    }
    resumo += "\nPróximo passo: preencher `humano_exercita` em `revisao-humana.csv` e contar, por variante,\n\
               os suspeitos falsos (modelo false, humano true) e os aprovados errados (modelo true, humano false).\n\
               Um RK trocado que passa no gabarito pode ser um risco que o teste também cobre: confira o motivo.\n";
    escrever(&saida.join("resumo.md"), &resumo)?;
    print!("{resumo}");
    eprintln!("\narquivos em {}", saida.display());
    Ok(())
}

fn main() {
    if let Err(mensagem) = rodar() {
        eprintln!("{mensagem}");
        std::process::exit(2);
    }
}
