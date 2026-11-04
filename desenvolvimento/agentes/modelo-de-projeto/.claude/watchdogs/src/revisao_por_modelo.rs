// Revisão por modelo (ADR 0015 do método): um modelo barato lê cada teste alterado desde a base, com a
// linha do risco que ele cita em docs/riscos.md, e diz se o teste exercita o risco ou só o cita. É
// consultiva: lista os suspeitos para o revisor e nunca reprova. Falha aberta: sem `claude` no PATH,
// sem rede ou sem login, avisa e segue. O veredito fica em cache em .claude/state/revisao-de-testes/,
// pela soma do prompt (teste, risco e instrução) e do modelo: repetir não chama de novo.
//
// Chama o `claude -p` com o login de quem roda, sem chave de API (`--bare` exigiria uma). O filho não
// carrega as configurações do projeto nem os watchdogs (recursão), não usa ferramentas e tem teto de gasto.

use regex::Regex;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use crate::riscos;
use crate::validador::casa_algum;

pub const MODELO_PADRAO: &str = "claude-haiku-5-5";
const TETO_PADRAO_USD: &str = "0.05";
const LIMITE_PADRAO_SEGUNDOS: u64 = 120;
// Cada `claude -p` é um processo Node de ~220 MB (medido, research/2026-11-04-memoria-e-tempo-das-validacoes.md):
// um por vez, dentro da vaga do ci.
const CHAMADAS_SIMULTANEAS: usize = 1;
const LINHAS_MAXIMAS_DO_TRECHO: usize = 150;
const PASTA_DO_CACHE: &str = ".claude/state/revisao-de-testes";
const ESQUEMA: &str = r#"{"type":"object","properties":{"exercita":{"type":"boolean"},"motivo":{"type":"string"}},"required":["exercita","motivo"]}"#;
const INSTRUCAO: &str = "Você revisa um teste automatizado contra o risco que ele diz cobrir. O teste exercita o risco \
quando falharia se o risco acontecesse: ele monta o cenário do risco e afirma o resultado que o risco quebraria. \
Responda exercita=false quando o teste só cita o risco no nome ou num comentário, afirma outra coisa (um getter, \
o valor que acabou de montar, uma chamada de mock) ou passaria mesmo com o risco acontecendo. Julgue só pelo \
trecho abaixo. Em motivo, uma ou duas frases em português que digam o que o teste afirma e o que faltaria.";

static NOME_DO_TESTE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\bfn\s+(\w+)|\b(?:it|test|testWidgets|goldenTest|patrolTest)\b[\w.]*\s*\(\s*["'`]([^"'`]*)|\bdef\s+(\w+)|\bfunc\s+(\w+)|\bvoid\s+(\w+)|\bfun\s+`?([^`(]+)"#)
        .expect("regex válida")
});

pub struct Veredito {
    pub exercita: bool,
    pub motivo: String,
}

pub struct TesteRevisado {
    pub arquivo: String,
    pub linha: u64,
    pub nome: String,
    pub riscos: Vec<String>,
    pub veredito: Veredito,
}

#[derive(Default)]
pub struct Revisao {
    pub modelo: String,
    pub revisados: Vec<TesteRevisado>,
    pub chamadas_ao_modelo: usize,
    pub vindos_do_cache: usize,
    pub sem_veredito: usize,
    pub avisos: Vec<String>,
}

impl Revisao {
    pub fn suspeitos(&self) -> impl Iterator<Item = &TesteRevisado> {
        self.revisados.iter().filter(|revisado| !revisado.veredito.exercita)
    }
}

// Um teste a revisar: onde está, o que cita e o prompt pronto (a chave do cache sai dele).
struct Pedido {
    arquivo: String,
    linha: u64,
    nome: String,
    riscos: Vec<String>,
    prompt: String,
    chave: String,
}

fn lista_de_texto(valor: Option<&Value>) -> Vec<String> {
    match valor {
        Some(Value::String(texto)) => vec![texto.clone()],
        Some(Value::Array(itens)) => itens.iter().filter_map(|item| item.as_str().map(String::from)).collect(),
        _ => Vec::new(),
    }
}

fn recuo(linha: &str) -> usize {
    linha.chars().take_while(|letra| letra.is_whitespace()).count()
}

fn eh_cabecalho(linha: &str) -> bool {
    let aparada = linha.trim_start();
    ["//", "/*", "*", "#", "@"].iter().any(|prefixo| aparada.starts_with(prefixo))
}

// Um it.each(tabela)(nome, corpo) só acaba na linha em que todo parêntese, colchete e chave já fechou:
// a tabela pode ter chaves próprias ou ser longa demais para o corpo aparecer nas primeiras linhas.
fn fim_da_tabela(linhas: &[&str], indice_da_declaracao: usize, limite: usize) -> usize {
    let mut profundidade: i64 = 0;
    for (indice, linha) in linhas.iter().enumerate().take(limite).skip(indice_da_declaracao) {
        for letra in linha.chars() {
            match letra {
                '(' | '[' | '{' => profundidade += 1,
                ')' | ']' | '}' => profundidade -= 1,
                _ => {}
            }
        }
        if profundidade <= 0 {
            return indice + 1;
        }
    }
    limite
}

// O teste inteiro: o cabeçalho colado em cima, a declaração e o corpo, até a chave que fecha o bloco
// aberto na declaração ou, sem chaves (Python), até a linha com recuo igual ou menor.
fn trecho_do_teste(linhas: &[&str], indice_da_declaracao: usize) -> String {
    let mut inicio = indice_da_declaracao;
    while inicio > 0 && eh_cabecalho(linhas[inicio - 1]) {
        inicio -= 1;
    }
    let limite = (indice_da_declaracao + LINHAS_MAXIMAS_DO_TRECHO).min(linhas.len());
    if linhas[indice_da_declaracao].contains(".each(") {
        return linhas[inicio..fim_da_tabela(linhas, indice_da_declaracao, limite)].join("\n");
    }
    let tem_chaves = linhas[indice_da_declaracao..limite.min(indice_da_declaracao + 6)]
        .iter()
        .any(|linha| linha.contains('{'));
    let mut fim = limite;
    if tem_chaves {
        let mut profundidade: i64 = 0;
        let mut abriu = false;
        for (indice, linha) in linhas.iter().enumerate().take(limite).skip(indice_da_declaracao) {
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
                fim = indice + 1;
                break;
            }
        }
    } else {
        let recuo_da_declaracao = recuo(linhas[indice_da_declaracao]);
        fim = linhas[indice_da_declaracao + 1..limite]
            .iter()
            .position(|linha| !linha.trim().is_empty() && recuo(linha) <= recuo_da_declaracao)
            .map(|deslocamento| indice_da_declaracao + 1 + deslocamento)
            .unwrap_or(limite);
    }
    linhas[inicio..fim].join("\n")
}

fn nome_do_teste(trecho: &str) -> String {
    NOME_DO_TESTE
        .captures(trecho)
        .and_then(|achado| achado.iter().skip(1).flatten().next().map(|nome| nome.as_str().trim().to_string()))
        .unwrap_or_default()
}

fn montar_prompt(arquivo: &str, linha: u64, linhas_dos_riscos: &[String], trecho: &str) -> String {
    format!(
        "{INSTRUCAO}\n\nRisco citado (linha de docs/riscos.md):\n{}\n\nTeste ({arquivo}:{linha}):\n```\n{trecho}\n```",
        linhas_dos_riscos.join("\n")
    )
}

fn chave_do_cache(prompt: &str, modelo: &str) -> String {
    let mut soma = Sha256::new();
    soma.update(modelo.as_bytes());
    soma.update([0]);
    soma.update(prompt.as_bytes());
    soma.finalize().iter().map(|byte| format!("{byte:02x}")).collect()
}

fn caminho_no_cache(raiz: &Path, chave: &str) -> PathBuf {
    raiz.join(PASTA_DO_CACHE).join(format!("{chave}.json"))
}

fn ler_do_cache(raiz: &Path, chave: &str) -> Option<Veredito> {
    let gravado: Value = serde_json::from_str(&fs::read_to_string(caminho_no_cache(raiz, chave)).ok()?).ok()?;
    Some(Veredito {
        exercita: gravado["exercita"].as_bool()?,
        motivo: gravado["motivo"].as_str().unwrap_or("").to_string(),
    })
}

fn gravar_no_cache(raiz: &Path, pedido: &Pedido, modelo: &str, veredito: &Veredito) {
    let caminho = caminho_no_cache(raiz, &pedido.chave);
    if let Some(pasta) = caminho.parent() {
        let _ = fs::create_dir_all(pasta);
    }
    let gravado = json!({
        "teste": format!("{}:{}", pedido.arquivo, pedido.linha),
        "riscos": pedido.riscos,
        "modelo": modelo,
        "exercita": veredito.exercita,
        "motivo": veredito.motivo,
    });
    let _ = fs::write(caminho, serde_json::to_string_pretty(&gravado).unwrap_or_default());
}

fn ler_ate_o_fim(cano: Option<impl Read + Send + 'static>) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut texto_lido = String::new();
        if let Some(mut cano_aberto) = cano {
            let _ = cano_aberto.read_to_string(&mut texto_lido);
        }
        texto_lido
    })
}

// Uma chamada ao `claude -p`. Erro volta como texto curto, para virar aviso.
fn perguntar_ao_modelo(raiz: &Path, modelo: &str, teto_usd: &str, limite: Duration, prompt: &str) -> Result<Veredito, String> {
    let mut processo = Command::new("claude")
        .args(["-p", "--setting-sources", "", "--strict-mcp-config", "--model", modelo, "--tools", ""])
        .args(["--no-session-persistence", "--output-format", "json", "--json-schema", ESQUEMA])
        .args(["--max-budget-usd", teto_usd, prompt])
        .env("STACKLABS_WATCHDOGS", "desligado")
        .current_dir(raiz)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|erro_ao_iniciar| match erro_ao_iniciar.kind() {
            std::io::ErrorKind::NotFound => "o claude não está no PATH".to_string(),
            _ => format!("o claude não iniciou ({erro_ao_iniciar})"),
        })?;
    // Os dois canos se leem enquanto o filho roda, para ele nunca travar com um deles cheio.
    let leitor_da_saida = ler_ate_o_fim(processo.stdout.take());
    let leitor_do_erro = ler_ate_o_fim(processo.stderr.take());
    let prazo = Instant::now() + limite;
    let estado_final = loop {
        match processo.try_wait() {
            Ok(Some(estado)) => break estado,
            Ok(None) if Instant::now() < prazo => std::thread::sleep(Duration::from_millis(50)),
            _ => {
                let _ = processo.kill();
                let _ = processo.wait();
                return Err(format!("o claude não respondeu em {} s", limite.as_secs()));
            }
        }
    };
    let texto_da_saida = leitor_da_saida.join().unwrap_or_default();
    let erro_do_claude = leitor_do_erro.join().unwrap_or_default();
    let resposta: Value = serde_json::from_str(texto_da_saida.trim()).map_err(|_| {
        let resumo = format!("{texto_da_saida} {erro_do_claude}").trim().chars().take(200).collect::<String>();
        format!("o claude saiu com {} sem a resposta em JSON: {resumo}", estado_final.code().unwrap_or(-1))
    })?;
    if resposta["is_error"].as_bool() == Some(true) {
        let motivo = resposta["result"].as_str().unwrap_or("erro sem mensagem");
        return Err(format!("o claude respondeu com erro: {}", motivo.chars().take(200).collect::<String>()));
    }
    let exercita = resposta["structured_output"]["exercita"]
        .as_bool()
        .ok_or("a resposta do claude veio sem structured_output.exercita")?;
    Ok(Veredito {
        exercita,
        motivo: resposta["structured_output"]["motivo"].as_str().unwrap_or("").trim().to_string(),
    })
}

fn montar_pedidos(raiz: &Path, projeto: &Value, parametros: &Map<String, Value>, alterados: &[String], modelo: &str) -> Vec<Pedido> {
    let caminho_do_registro = parametros.get("riscos").and_then(Value::as_str).unwrap_or("docs/riscos.md");
    let texto_do_registro = fs::read_to_string(raiz.join(caminho_do_registro)).unwrap_or_default();
    let linhas_do_registro: Vec<&str> = texto_do_registro.lines().collect();
    let (registro, _) = riscos::ler_registro(&texto_do_registro);
    let declaracao_configurada = parametros.get("declaracao").and_then(Value::as_str).and_then(|padrao| Regex::new(padrao).ok());
    let mut padroes_de_teste = lista_de_texto(parametros.get("testes"));
    if padroes_de_teste.is_empty() {
        padroes_de_teste = riscos::TESTES_PADRAO.iter().map(|padrao| padrao.to_string()).collect();
    }
    let mut padroes_ignorados = lista_de_texto(projeto.get("ignorar"));
    padroes_ignorados.extend(lista_de_texto(parametros.get("ignorar")));
    let mut pedidos = Vec::new();
    for arquivo in alterados
        .iter()
        .filter(|caminho| casa_algum(caminho, &padroes_de_teste) && !casa_algum(caminho, &padroes_ignorados))
    {
        let Some(declaracao) = declaracao_configurada.as_ref().or_else(|| riscos::declaracao_padrao(arquivo)) else {
            continue;
        };
        let texto_do_arquivo = fs::read_to_string(raiz.join(arquivo)).unwrap_or_default();
        let linhas_do_arquivo: Vec<&str> = texto_do_arquivo.lines().collect();
        for teste in riscos::testes_do_arquivo(&texto_do_arquivo, declaracao) {
            // Teste sem risco ou com risco fora do registro já reprova no teste-cobre-risco.
            let linhas_dos_riscos: Vec<String> = teste
                .riscos
                .iter()
                .filter_map(|citado| registro.iter().find(|risco| &risco.id == citado))
                .map(|risco| linhas_do_registro[risco.linha as usize - 1].trim().to_string())
                .collect();
            if linhas_dos_riscos.is_empty() {
                continue;
            }
            let trecho = trecho_do_teste(&linhas_do_arquivo, teste.linha as usize - 1);
            let prompt = montar_prompt(arquivo, teste.linha, &linhas_dos_riscos, &trecho);
            pedidos.push(Pedido {
                arquivo: arquivo.clone(),
                linha: teste.linha,
                nome: nome_do_teste(&trecho),
                riscos: teste.riscos.clone(),
                chave: chave_do_cache(&prompt, modelo),
                prompt,
            });
        }
    }
    pedidos
}

// Revisa os testes de `alterados` com a regra `revisao-por-modelo` (parâmetros sem o "tipo").
pub fn revisar(raiz: &Path, projeto: &Value, parametros: &Map<String, Value>, alterados: &[String]) -> Revisao {
    let modelo = parametros.get("modelo").and_then(Value::as_str).unwrap_or(MODELO_PADRAO).to_string();
    let teto_usd = match parametros.get("tetoUsd") {
        Some(Value::Number(valor)) => valor.to_string(),
        Some(Value::String(valor)) => valor.clone(),
        _ => TETO_PADRAO_USD.to_string(),
    };
    let limite = Duration::from_secs(parametros.get("limiteSegundos").and_then(Value::as_u64).unwrap_or(LIMITE_PADRAO_SEGUNDOS));
    let mut revisao = Revisao {
        modelo: modelo.clone(),
        ..Revisao::default()
    };
    let mut a_perguntar = Vec::new();
    for pedido in montar_pedidos(raiz, projeto, parametros, alterados, &modelo) {
        match ler_do_cache(raiz, &pedido.chave) {
            Some(veredito) => {
                revisao.vindos_do_cache += 1;
                revisao.revisados.push(revisado(&pedido, veredito));
            }
            None => a_perguntar.push(pedido),
        }
    }
    let mut avisos_vistos = HashSet::new();
    for lote in a_perguntar.chunks(CHAMADAS_SIMULTANEAS) {
        let respostas: Vec<Result<Veredito, String>> = std::thread::scope(|escopo| {
            let tarefas: Vec<_> = lote
                .iter()
                .map(|pedido| escopo.spawn(|| perguntar_ao_modelo(raiz, &modelo, &teto_usd, limite, &pedido.prompt)))
                .collect();
            tarefas
                .into_iter()
                .map(|tarefa| tarefa.join().unwrap_or_else(|_| Err("a chamada ao claude entrou em pânico".into())))
                .collect()
        });
        for (pedido, resposta) in lote.iter().zip(respostas) {
            revisao.chamadas_ao_modelo += 1;
            match resposta {
                Ok(veredito) => {
                    gravar_no_cache(raiz, pedido, &modelo, &veredito);
                    revisao.revisados.push(revisado(pedido, veredito));
                }
                Err(motivo_da_falha) => {
                    revisao.sem_veredito += 1;
                    if avisos_vistos.insert(motivo_da_falha.clone()) {
                        revisao.avisos.push(motivo_da_falha);
                    }
                }
            }
        }
        // Sem o claude no PATH, as próximas chamadas falhariam igual.
        if revisao.avisos.iter().any(|aviso| aviso.contains("não está no PATH")) {
            revisao.sem_veredito += a_perguntar.len() - revisao.chamadas_ao_modelo;
            break;
        }
    }
    revisao
        .revisados
        .sort_by(|primeiro, segundo| (&primeiro.arquivo, primeiro.linha).cmp(&(&segundo.arquivo, segundo.linha)));
    revisao
}

fn revisado(pedido: &Pedido, veredito: Veredito) -> TesteRevisado {
    TesteRevisado {
        arquivo: pedido.arquivo.clone(),
        linha: pedido.linha,
        nome: pedido.nome.clone(),
        riscos: pedido.riscos.clone(),
        veredito,
    }
}

#[cfg(test)]
mod testes_do_trecho {
    use super::trecho_do_teste;

    fn trecho_de(arquivo: &str) -> String {
        let linhas: Vec<&str> = arquivo.lines().collect();
        trecho_do_teste(&linhas, 0)
    }

    #[test]
    fn it_each_de_tabela_longa_leva_o_corpo() {
        let arquivo = "  it.each([\n    [1],\n    [2],\n    [3],\n    [4],\n    [5],\n    [6],\n  ])(\"caso %s\", (valor) => {\n    expect(valor).toBe(1);\n  });\n\n  it(\"outro\", () => {});\n";
        let trecho = trecho_de(arquivo);
        assert!(trecho.contains("expect(valor)"), "{trecho}");
        assert!(!trecho.contains("outro"), "{trecho}");
    }

    #[test]
    fn it_each_com_chave_na_tabela_leva_o_corpo() {
        let arquivo = "  it.each([\n    [\"a\", { sexta: [{ abre: \"18:00\" }] }],\n  ])(\"recusa %s\", (nome, periodos) => {\n    expect(periodos).toBeNull();\n  });\n\n  it(\"outro\", () => {});\n";
        let trecho = trecho_de(arquivo);
        assert!(trecho.contains("expect(periodos)"), "{trecho}");
        assert!(!trecho.contains("outro"), "{trecho}");
    }

    #[test]
    fn teste_comum_para_na_chave_que_fecha() {
        let trecho = trecho_de("it(\"um\", () => {\n  expect(1).toBe(1);\n});\nit(\"outro\", () => {});\n");
        assert_eq!(trecho, "it(\"um\", () => {\n  expect(1).toBe(1);\n});");
    }
}
