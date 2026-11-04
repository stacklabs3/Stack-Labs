//! Instala ou atualiza o método StackLabs num projeto: modelo-de-projeto, skills da casa e as skills
//! publicadas do Matt Pocock (sempre da versão mais nova do repositório dele).
//!
//! Uso: sh rodar [--destino <pasta>] [--versao <x.y.z>] [--simular] [--sem-labels] [--adocao] [--permitir-regressao]
//!      sh rodar --conferir-skill | --verificar | --concluir-pr <número>
//!   --destino             raiz do projeto (padrão: a raiz do git da pasta atual)
//!   --versao              instala a versão marcada (tag v<x.y.z> do Stack-Labs) em vez da main
//!   --simular             mostra o que faria, sem escrever nada
//!   --sem-labels          não cria as labels do método no GitHub
//!   --adocao              projeto que já existia: cria .claude/metodo.json na etapa de observação (/adotar-metodo-stack-labs)
//!   --permitir-regressao  aceita --versao anterior à instalada. SÓ HUMANO (o guarda-bash barra o agente)
//!   --conferir-skill      atualiza as skills de máquina a partir da fonte oficial
//!   --verificar           confere a instalação do projeto; sai com 1 se algo reprovar
//!   --concluir-pr         prova que a PR só traz o que a fonte oficial manda, rotula, espera o CI e faz o merge;
//!                         sai com 3 se a PR ficar para um humano
//!
//! Fontes (opcional, para usar um clone local em vez de baixar):
//!   STACKLABS_ORIGEM=<clone do Stack-Labs>   MATT_ORIGEM=<clone de mattpocock/skills>

use serde_json::{json, Value};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

/// Arquivos que o projeto pode ter com conteúdo próprio: nunca sobrescreve, deixa o novo ao lado.
const ARQUIVOS_PARA_MESCLAR: [&str; 2] = ["AGENTS.md", "CLAUDE.md"];
/// Modelos antigos que o instalador agora mescla sozinho: a sobra `.stacklabs-novo` deles sai.
const ARQUIVOS_MESCLADOS_PELO_INSTALADOR: [&str; 2] = [".gitignore", ".claude/settings.json"];
/// Skills que moram na máquina e se atualizam pelo --conferir-skill.
const SKILLS_DE_MAQUINA: [&str; 3] = ["setup-stacklabs", "adotar-metodo-stack-labs", "auditar-metodo"];
const URL_DO_STACKLABS: &str = "https://github.com/stacklabs1/Stack-Labs";
const LABEL_ALTERA_WATCHDOGS: &str = "altera-watchdogs";
/// Saída do --concluir-pr quando a PR não pode entrar sem uma pessoa.
const CODIGO_FICA_PARA_HUMANO: i32 = 3;
/// Seção do `AGENTS.md` que pertence ao projeto: o instalador a carrega de uma versão para a outra.
const SECAO_DO_PROJETO: &str = "## Regras deste projeto";
/// Infraestrutura dos watchdogs: entra inteira ou não entra (a fonte precisa compilar antes).
const CAMINHOS_DOS_WATCHDOGS: [&str; 5] = [
    ".claude/watchdogs",
    ".claude/githooks",
    ".claude/agents",
    ".claude/settings.json",
    ".github/workflows/vigia-do-metodo.yml",
];
/// Watchdogs antigos (em node), que a versão em Rust de `.claude/watchdogs/` substitui.
const WATCHDOGS_ANTIGOS: [&str; 2] = [".claude/hooks", ".claude/validador"];
/// Pastas que ficam idênticas às do modelo: arquivo que saiu da fonte sai do projeto.
const PASTAS_ESPELHADAS: [&str; 2] = [".claude/watchdogs", ".claude/githooks"];
/// Pasta de artefatos de compilação do cargo: nunca é copiada, em nenhum nível.
/// Skills do Matt que o método libera para o agente rodar a cadeia sozinho: sai a linha `disable-model-invocation`.
const SKILLS_SEM_TRAVA: [&str; 5] = ["wayfinder", "handoff", "to-spec", "to-tickets", "implement"];
/// Âncora do `implement-spec` do Matt: o remendo entra logo depois do parágrafo que a começa.
const ANCORA_DO_IMPLEMENT_SPEC: &str = "Communication to and from subagents should be sparse.";
const REMENDO_DO_IMPLEMENT_SPEC: &str = "**Implementer subagents** run **one at a time**, never in parallel: this machine has no swap, and two builds at once kill every process, the agent's shell included. (Remendo do método StackLabs sobre o upstream, aplicado pelo `/setup-stacklabs`.)";

const PASTA_DE_COMPILACAO: &str = "target";
const LABELS_DO_METODO: [&str; 17] = [
    "wayfinder:map",
    "wayfinder:research",
    "wayfinder:prototype",
    "wayfinder:grilling",
    "wayfinder:task",
    "mapa-inicial:prototipo",
    "mapa-inicial:arquitetura",
    "mapa-inicial:fases",
    "mapa-inicial:regras-de-codigo",
    "mapa-inicial:infra",
    "needs-triage",
    "needs-info",
    "ready-for-agent",
    "ready-for-human",
    "wontfix",
    "spec",
    "altera-watchdogs",
];

type Resultado<T> = Result<T, String>;

#[derive(Default)]
struct Opcoes {
    destino: Option<String>,
    versao: Option<String>,
    simular: bool,
    sem_labels: bool,
    adocao: bool,
    permitir_regressao: bool,
    conferir_skill: bool,
    verificar: bool,
    concluir_pr: Option<u64>,
    pasta_das_skills: Option<String>,
}

// ---------- utilitários ----------

fn ler_argumentos(argumentos_da_linha: &[String]) -> Resultado<Opcoes> {
    let mut opcoes = Opcoes::default();
    let mut argumentos = argumentos_da_linha.iter();
    while let Some(argumento) = argumentos.next() {
        match argumento.as_str() {
            "--destino" => opcoes.destino = argumentos.next().cloned(),
            "--versao" => {
                opcoes.versao = Some(
                    argumentos
                        .next()
                        .map(|versao| versao.trim_start_matches('v').to_string())
                        .ok_or("--versao precisa de um valor, como 1.0.0")?,
                )
            }
            "--simular" => opcoes.simular = true,
            "--sem-labels" => opcoes.sem_labels = true,
            "--adocao" => opcoes.adocao = true,
            "--permitir-regressao" => opcoes.permitir_regressao = true,
            "--conferir-skill" => opcoes.conferir_skill = true,
            "--verificar" => opcoes.verificar = true,
            "--concluir-pr" => {
                opcoes.concluir_pr = Some(
                    argumentos
                        .next()
                        .and_then(|numero| numero.trim_start_matches('#').parse().ok())
                        .ok_or("--concluir-pr precisa do número da PR")?,
                )
            }
            "--pasta-das-skills" => opcoes.pasta_das_skills = argumentos.next().cloned(),
            _ => return Err(format!("opção desconhecida: {argumento}")),
        }
    }
    Ok(opcoes)
}

fn git(argumentos: &[&str]) -> Resultado<String> {
    let saida = Command::new("git")
        .args(argumentos)
        .stdin(Stdio::null())
        .output()
        .map_err(|erro| format!("git {}: {erro}", argumentos.join(" ")))?;
    if !saida.status.success() {
        return Err(format!(
            "Command failed: git {}\n{}",
            argumentos.join(" "),
            String::from_utf8_lossy(&saida.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&saida.stdout).trim().to_string())
}

fn git_ou_nulo(argumentos: &[&str]) -> Option<String> {
    git(argumentos).ok()
}

fn texto(caminho: &Path) -> String {
    caminho.to_string_lossy().into_owned()
}

/// Caminho absoluto e normalizado, sem resolver links (como o `path.resolve` do node).
fn resolver(caminho: &str) -> PathBuf {
    let caminho = Path::new(caminho);
    let absoluto = if caminho.is_absolute() {
        caminho.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(caminho)
    };
    let mut normalizado = PathBuf::new();
    for componente in absoluto.components() {
        match componente {
            Component::CurDir => {}
            Component::ParentDir => {
                normalizado.pop();
            }
            outro => normalizado.push(outro.as_os_str()),
        }
    }
    normalizado
}

/// Compara como o `sort()` do JavaScript (unidades UTF-16).
fn ordem_do_javascript(primeiro: &str, segundo: &str) -> std::cmp::Ordering {
    primeiro.encode_utf16().cmp(segundo.encode_utf16())
}

/// Caminhos relativos de todos os arquivos de uma pasta, em ordem, com "/" como separador.
/// Pastas `target/` (compilação do cargo) ficam de fora.
fn listar_arquivos(pasta_raiz: &Path) -> Resultado<Vec<String>> {
    fn visitar(pasta_atual: &Path, prefixo: &str, arquivos: &mut Vec<String>) -> Resultado<()> {
        let entradas =
            fs::read_dir(pasta_atual).map_err(|erro| format!("{}: {erro}", texto(pasta_atual)))?;
        for entrada in entradas {
            let entrada = entrada.map_err(|erro| erro.to_string())?;
            let nome = entrada.file_name().to_string_lossy().into_owned();
            let caminho_relativo = if prefixo.is_empty() {
                nome.clone()
            } else {
                format!("{prefixo}/{nome}")
            };
            let eh_pasta = entrada
                .file_type()
                .map(|tipo| tipo.is_dir())
                .unwrap_or(false);
            if eh_pasta {
                if nome != PASTA_DE_COMPILACAO {
                    visitar(&entrada.path(), &caminho_relativo, arquivos)?;
                }
            } else {
                arquivos.push(caminho_relativo);
            }
        }
        Ok(())
    }
    let mut arquivos = Vec::new();
    visitar(pasta_raiz, "", &mut arquivos)?;
    arquivos.sort_by(|primeiro, segundo| ordem_do_javascript(primeiro, segundo));
    Ok(arquivos)
}

fn arquivos_iguais(caminho_a: &Path, caminho_b: &Path) -> bool {
    if !fs::metadata(caminho_b)
        .map(|dados| dados.is_file())
        .unwrap_or(false)
    {
        return false;
    }
    match (fs::read(caminho_a), fs::read(caminho_b)) {
        (Ok(conteudo_a), Ok(conteudo_b)) => conteudo_a == conteudo_b,
        _ => false,
    }
}

fn pastas_iguais(pasta_a: &Path, pasta_b: &Path) -> Resultado<bool> {
    if !pasta_b.exists() {
        return Ok(false);
    }
    let arquivos_de_a = listar_arquivos(pasta_a)?;
    let arquivos_de_b = listar_arquivos(pasta_b)?;
    Ok(arquivos_de_a == arquivos_de_b
        && arquivos_de_a.iter().all(|caminho_relativo| {
            arquivos_iguais(
                &pasta_a.join(caminho_relativo),
                &pasta_b.join(caminho_relativo),
            )
        }))
}

/// Aplica no `SKILL.md` copiado os remendos do método às skills do Matt (nada a fazer nas demais).
fn remendar_skill_do_matt(pasta_da_skill: &Path) -> Resultado<()> {
    let nome = nome_da_pasta(pasta_da_skill);
    let arquivo = pasta_da_skill.join("SKILL.md");
    if SKILLS_SEM_TRAVA.contains(&nome.as_str()) {
        let conteudo =
            fs::read_to_string(&arquivo).map_err(|erro| format!("{}: {erro}", texto(&arquivo)))?;
        let sem_trava: String = conteudo
            .split_inclusive('\n')
            .filter(|linha| !linha.starts_with("disable-model-invocation:"))
            .collect();
        fs::write(&arquivo, sem_trava).map_err(|erro| format!("{}: {erro}", texto(&arquivo)))?;
    } else if nome == "implement-spec" {
        let conteudo =
            fs::read_to_string(&arquivo).map_err(|erro| format!("{}: {erro}", texto(&arquivo)))?;
        if conteudo.contains(REMENDO_DO_IMPLEMENT_SPEC) {
            return Ok(());
        }
        let Some(inicio) = conteudo.find(ANCORA_DO_IMPLEMENT_SPEC) else {
            return Err(format!(
                "{}: a âncora do remendo (\"{ANCORA_DO_IMPLEMENT_SPEC}\") sumiu do upstream; atualize o remendo em main.rs",
                texto(&arquivo)
            ));
        };
        let fim_do_paragrafo = conteudo[inicio..]
            .find("\n\n")
            .map_or(conteudo.len(), |posicao| inicio + posicao + 2);
        let mut remendado = conteudo[..fim_do_paragrafo].to_string();
        if !remendado.ends_with("\n\n") {
            remendado.push_str("\n\n");
        }
        remendado.push_str(REMENDO_DO_IMPLEMENT_SPEC);
        remendado.push_str("\n\n");
        remendado.push_str(&conteudo[fim_do_paragrafo..]);
        fs::write(&arquivo, remendado).map_err(|erro| format!("{}: {erro}", texto(&arquivo)))?;
    }
    Ok(())
}

fn caminho_eh_watchdog(caminho_relativo: &str) -> bool {
    CAMINHOS_DOS_WATCHDOGS.iter().any(|caminho_watchdog| {
        caminho_relativo == *caminho_watchdog
            || caminho_relativo.starts_with(&format!("{caminho_watchdog}/"))
    })
}

/// O `AGENTS.md` novo do modelo com a seção do projeto do antigo no lugar da seção do modelo.
/// `None` quando o antigo não tem a seção: aí a mesclagem é à mão.
fn mesclar_agents(modelo_novo: &str, antigo: &str) -> Option<String> {
    let secao_do_projeto = &antigo[antigo.find(SECAO_DO_PROJETO)?..];
    let base = match modelo_novo.find(SECAO_DO_PROJETO) {
        Some(inicio) => &modelo_novo[..inicio],
        None => modelo_novo,
    };
    Some(format!("{}\n\n{}", base.trim_end(), secao_do_projeto))
}

fn comando_eh_do_metodo(comando: &str) -> bool {
    [".claude/watchdogs/", ".claude/hooks/", ".claude/validador/"]
        .iter()
        .any(|caminho| comando.contains(caminho))
}

/// O `settings.json` do projeto com os hooks do método trocados pelos do modelo. O que é do projeto
/// (`permissions`, `env`, hook próprio) fica, e o hook próprio entra depois dos do modelo.
fn mesclar_settings(modelo: &Value, projeto: &Value) -> Value {
    let mut mesclado = projeto.clone();
    let mut hooks_mesclados = modelo["hooks"].as_object().cloned().unwrap_or_default();
    if let Some(hooks_do_projeto) = projeto["hooks"].as_object() {
        for (evento, grupos_do_projeto) in hooks_do_projeto {
            for grupo in grupos_do_projeto.as_array().into_iter().flatten() {
                let hooks_proprios: Vec<Value> = grupo["hooks"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|hook| !comando_eh_do_metodo(hook["command"].as_str().unwrap_or("")))
                    .cloned()
                    .collect();
                if hooks_proprios.is_empty() {
                    continue;
                }
                let mut grupo_proprio = grupo.clone();
                grupo_proprio["hooks"] = Value::Array(hooks_proprios);
                let grupos_do_evento = hooks_mesclados.entry(evento.clone()).or_insert_with(|| json!([]));
                if let Some(lista_de_grupos) = grupos_do_evento.as_array_mut() {
                    lista_de_grupos.push(grupo_proprio);
                }
            }
        }
    }
    if !hooks_mesclados.is_empty() || projeto.get("hooks").is_some() {
        mesclado["hooks"] = Value::Object(hooks_mesclados);
    }
    mesclado
}

/// As linhas do modelo que faltam no `.gitignore` do projeto.
fn linhas_que_faltam(modelo: &str, projeto: &str) -> Vec<String> {
    modelo
        .lines()
        .filter(|linha| !linha.trim().is_empty() && !projeto.lines().any(|existente| existente == *linha))
        .map(str::to_string)
        .collect()
}

/// `[4, 1, 0]` de `4.1.0`, para comparar versões número a número.
fn numeros_da_versao(versao: &str) -> Vec<u64> {
    versao.split('.').filter_map(|numero| numero.parse().ok()).collect()
}

fn criar_pasta(pasta: &Path) -> Resultado<()> {
    fs::create_dir_all(pasta).map_err(|erro| format!("{}: {erro}", texto(pasta)))
}

fn remover_pasta(pasta: &Path) -> Resultado<()> {
    match fs::symlink_metadata(pasta) {
        Err(_) => Ok(()),
        Ok(dados) if dados.is_dir() => {
            fs::remove_dir_all(pasta).map_err(|erro| format!("{}: {erro}", texto(pasta)))
        }
        Ok(_) => fs::remove_file(pasta).map_err(|erro| format!("{}: {erro}", texto(pasta))),
    }
}

fn copiar_arquivo(origem: &Path, destino: &Path) -> Resultado<()> {
    fs::copy(origem, destino)
        .map(|_| ())
        .map_err(|erro| format!("{} -> {}: {erro}", texto(origem), texto(destino)))
}

/// Copia o arquivo mantendo o modo (executável) e as datas de acesso e de modificação.
fn copiar_arquivo_com_datas(origem: &Path, destino: &Path) -> Resultado<()> {
    copiar_arquivo(origem, destino)?;
    let dados = fs::metadata(origem).map_err(|erro| erro.to_string())?;
    let mut datas = fs::FileTimes::new();
    if let Ok(acesso) = dados.accessed() {
        datas = datas.set_accessed(acesso);
    }
    if let Ok(modificacao) = dados.modified() {
        datas = datas.set_modified(modificacao);
    }
    fs::OpenOptions::new()
        .write(true)
        .open(destino)
        .and_then(|arquivo| arquivo.set_times(datas))
        .map_err(|erro| format!("{}: {erro}", texto(destino)))
}

fn marcar_como_recem_alterado(caminho: &Path) -> Resultado<()> {
    fs::OpenOptions::new()
        .write(true)
        .open(caminho)
        .and_then(|arquivo| arquivo.set_modified(SystemTime::now()))
        .map_err(|erro| format!("{}: {erro}", texto(caminho)))
}

/// Copia a pasta inteira, menos as pastas `target/`.
fn copiar_pasta(origem: &Path, destino: &Path) -> Resultado<()> {
    criar_pasta(destino)?;
    let entradas = fs::read_dir(origem).map_err(|erro| format!("{}: {erro}", texto(origem)))?;
    for entrada in entradas {
        let entrada = entrada.map_err(|erro| erro.to_string())?;
        let tipo = entrada.file_type().map_err(|erro| erro.to_string())?;
        let alvo = destino.join(entrada.file_name());
        if tipo.is_dir() {
            if entrada.file_name() != PASTA_DE_COMPILACAO {
                copiar_pasta(&entrada.path(), &alvo)?;
            }
        } else if tipo.is_symlink() {
            copiar_link(&entrada.path(), &alvo)?;
        } else {
            copiar_arquivo(&entrada.path(), &alvo)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn copiar_link(origem: &Path, destino: &Path) -> Resultado<()> {
    let apontado = fs::read_link(origem).map_err(|erro| erro.to_string())?;
    std::os::unix::fs::symlink(apontado, destino)
        .map_err(|erro| format!("{}: {erro}", texto(destino)))
}

#[cfg(not(unix))]
fn copiar_link(origem: &Path, destino: &Path) -> Resultado<()> {
    copiar_arquivo(origem, destino)
}

fn subpastas(pasta: &Path) -> Resultado<Vec<PathBuf>> {
    let entradas = fs::read_dir(pasta).map_err(|erro| format!("{}: {erro}", texto(pasta)))?;
    let mut pastas: Vec<PathBuf> = entradas
        .filter_map(|entrada| entrada.ok())
        .filter(|entrada| {
            entrada
                .file_type()
                .map(|tipo| tipo.is_dir())
                .unwrap_or(false)
        })
        .map(|entrada| entrada.path())
        .collect();
    pastas.sort_by(|primeira, segunda| primeira.file_name().cmp(&segunda.file_name()));
    Ok(pastas)
}

fn nome_da_pasta(pasta: &Path) -> String {
    pasta
        .file_name()
        .map(|nome| nome.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn ler_json(caminho: &Path) -> Resultado<Value> {
    let conteudo =
        fs::read_to_string(caminho).map_err(|erro| format!("{}: {erro}", texto(caminho)))?;
    serde_json::from_str(&conteudo).map_err(|erro| format!("{}: {erro}", texto(caminho)))
}

fn escrever_json(caminho: &Path, valor: &Value) -> Resultado<()> {
    let conteudo = serde_json::to_string_pretty(valor).map_err(|erro| erro.to_string())? + "\n";
    fs::write(caminho, conteudo).map_err(|erro| format!("{}: {erro}", texto(caminho)))
}

fn lista_de_textos(valor: &Value, campo: &str, arquivo: &str) -> Resultado<Vec<String>> {
    valor[campo]
        .as_array()
        .map(|itens| {
            itens
                .iter()
                .map(|item| {
                    item.as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| item.to_string())
                })
                .collect()
        })
        .ok_or_else(|| format!("{arquivo}: o campo {campo} não é uma lista"))
}

/// Data e hora em UTC no formato do `toISOString()` (2026-01-31T12:00:00.000Z).
fn agora_em_iso() -> String {
    let milissegundos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duracao| duracao.as_millis())
        .unwrap_or(0) as i64;
    let dias = milissegundos.div_euclid(86_400_000);
    let resto = milissegundos.rem_euclid(86_400_000);
    // Conversão de dias desde 1970 em data civil (algoritmo de Howard Hinnant).
    let dias_desde_a_era = dias + 719_468;
    let era = dias_desde_a_era.div_euclid(146_097);
    let dia_da_era = dias_desde_a_era.rem_euclid(146_097);
    let ano_da_era =
        (dia_da_era - dia_da_era / 1460 + dia_da_era / 36_524 - dia_da_era / 146_096) / 365;
    let dia_do_ano = dia_da_era - (365 * ano_da_era + ano_da_era / 4 - ano_da_era / 100);
    let mes_deslocado = (5 * dia_do_ano + 2) / 153;
    let dia = dia_do_ano - (153 * mes_deslocado + 2) / 5 + 1;
    let mes = if mes_deslocado < 10 {
        mes_deslocado + 3
    } else {
        mes_deslocado - 9
    };
    let ano = ano_da_era + era * 400 + i64::from(mes <= 2);
    format!(
        "{ano:04}-{mes:02}-{dia:02}T{:02}:{:02}:{:02}.{:03}Z",
        resto / 3_600_000,
        resto / 60_000 % 60,
        resto / 1000 % 60,
        resto % 1000
    )
}

/// Pasta temporária apagada ao sair, com ou sem erro.
struct PastaTemporaria(PathBuf);

impl PastaTemporaria {
    fn criar() -> Resultado<Self> {
        let marca = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duracao| duracao.as_nanos())
            .unwrap_or(0);
        let pasta =
            std::env::temp_dir().join(format!("setup-stacklabs-{}-{marca}", std::process::id()));
        fs::create_dir_all(&pasta).map_err(|erro| format!("{}: {erro}", texto(&pasta)))?;
        Ok(PastaTemporaria(pasta))
    }
}

impl Drop for PastaTemporaria {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn conferir_cargo() -> Resultado<()> {
    let cargo_responde = Command::new("cargo")
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    if cargo_responde {
        Ok(())
    } else {
        Err("cargo não encontrado: instale o Rust (https://rustup.rs) e rode de novo.".to_string())
    }
}

/// Roda `sh <lancador> --version`, que compila os watchdogs se preciso. O erro já vem pronto para o relatório.
fn compilar_com_lancador(lancador: &Path, pasta_de_trabalho: &Path) -> Result<(), String> {
    let execucao = Command::new("sh")
        .arg(lancador)
        .arg("--version")
        .current_dir(pasta_de_trabalho)
        .stdin(Stdio::null())
        .output();
    match execucao {
        Ok(saida) if saida.status.success() => Ok(()),
        Ok(saida) => {
            let erro = String::from_utf8_lossy(&saida.stderr);
            // A primeira linha de erro do cargo diz a causa; sem ela, a última linha da saída de erro.
            let motivo = erro
                .lines()
                .find(|linha| linha.trim_start().to_lowercase().starts_with("error"))
                .or_else(|| erro.lines().rev().find(|linha| !linha.trim().is_empty()))
                .unwrap_or("")
                .trim();
            let codigo = saida.status.code().map(|codigo| codigo.to_string()).unwrap_or_else(|| "sinal".to_string());
            Err(format!("FALHA     .claude/watchdogs não compilou (sh .claude/watchdogs/rodar --version saiu com {codigo}): {motivo}"))
        }
        Err(erro) => Err(format!("FALHA     .claude/watchdogs não compilou (sh .claude/watchdogs/rodar --version): {erro}")),
    }
}

/// Compila os watchdogs do projeto uma vez, para a primeira sessão não esperar a compilação.
fn compilar_watchdogs(destino: &Path) -> String {
    let lancador = destino.join(".claude").join("watchdogs").join("rodar");
    match compilar_com_lancador(&lancador, destino) {
        Ok(()) => "COMPILADO .claude/watchdogs".to_string(),
        Err(linha_de_falha) => linha_de_falha,
    }
}

/// Compila a fonte numa pasta à parte antes de tocar no projeto: um watchdog que não compila, instalado,
/// trava o agente em todo hook, e ele não pode consertar a pasta protegida.
fn fonte_dos_watchdogs_compila(pasta_do_modelo: &Path, pasta_temporaria: &Path) -> Result<(), String> {
    let pasta_de_preparo = pasta_temporaria.join("preparo-dos-watchdogs");
    copiar_pasta(&pasta_do_modelo.join(".claude").join("watchdogs"), &pasta_de_preparo).map_err(|erro| format!("FALHA     {erro}"))?;
    compilar_com_lancador(&pasta_de_preparo.join("rodar"), &pasta_de_preparo)
        .map_err(|linha_de_falha| format!("{linha_de_falha}; os watchdogs do projeto ficaram como estavam"))
}

fn buscar_origem(pasta_temporaria: &Path, clone_local: Option<String>, url_do_repositorio: &str, nome_da_pasta: &str, tag: Option<String>) -> Resultado<PathBuf> {
    if let Some(clone) = clone_local.filter(|clone| !clone.is_empty()) {
        return Ok(resolver(&clone));
    }
    let pasta_do_clone = pasta_temporaria.join(nome_da_pasta);
    let pasta_em_texto = texto(&pasta_do_clone);
    let mut argumentos = vec!["clone", "-q", "--depth", "1"];
    if let Some(tag) = &tag {
        argumentos.extend(["--branch", tag.as_str()]);
    }
    argumentos.extend([url_do_repositorio, pasta_em_texto.as_str()]);
    git(&argumentos)?;
    Ok(pasta_do_clone)
}

fn destino_das_opcoes(opcoes: &Opcoes) -> Resultado<PathBuf> {
    match &opcoes.destino {
        Some(pasta) => Ok(resolver(pasta)),
        None => Ok(resolver(&git(&["rev-parse", "--show-toplevel"]).map_err(|_| {
            "a pasta atual não é um repositório git: rode dentro do projeto ou passe --destino <pasta>".to_string()
        })?)),
    }
}

// ---------- instalação ----------

/// O relatório de uma instalação e as linhas de cabeçalho que dizem de onde ela veio.
struct ResultadoDaInstalacao {
    cabecalho: String,
    relatorio: Vec<String>,
}

fn instalar(opcoes: &Opcoes) -> Resultado<ResultadoDaInstalacao> {
    conferir_cargo()?;
    let destino = destino_das_opcoes(opcoes)?;
    let pasta_temporaria = PastaTemporaria::criar()?;
    let mut relatorio: Vec<String> = Vec::new();
    let simular = opcoes.simular;

    let origem_stacklabs = buscar_origem(
        &pasta_temporaria.0,
        std::env::var("STACKLABS_ORIGEM").ok(),
        URL_DO_STACKLABS,
        "stacklabs",
        opcoes.versao.as_ref().map(|versao| format!("v{versao}")),
    )?;
    let origem_matt = buscar_origem(
        &pasta_temporaria.0,
        std::env::var("MATT_ORIGEM").ok(),
        "https://github.com/mattpocock/skills",
        "matt",
        None,
    )?;
    let versao_do_metodo =
        fs::read_to_string(origem_stacklabs.join("desenvolvimento").join("VERSAO"))
            .ok()
            .map(|versao| versao.trim().to_string())
            .filter(|versao| !versao.is_empty());
    if let (Some(pedida), Some(encontrada)) = (&opcoes.versao, &versao_do_metodo) {
        if pedida != encontrada {
            return Err(format!(
                "pediu a versão {pedida}, mas a fonte do Stack-Labs está na {encontrada}"
            ));
        }
    }
    if let (Some(pedida), false) = (&opcoes.versao, opcoes.permitir_regressao) {
        let arquivo_de_origem = destino.join(".claude").join("skills").join("ORIGEM.json");
        if arquivo_de_origem.exists() {
            let origem_instalada = ler_json(&arquivo_de_origem)?;
            let instalada = origem_instalada["stacklabs"]["versao"].as_str().unwrap_or("");
            if numeros_da_versao(pedida) < numeros_da_versao(instalada) {
                return Err(format!(
                    "a versão {pedida} é anterior à {instalada}, já instalada: voltar afrouxa os watchdogs. Só um humano volta, com --permitir-regressao"
                ));
            }
        }
    }
    let pasta_do_modelo = origem_stacklabs
        .join("desenvolvimento")
        .join("agentes")
        .join("modelo-de-projeto");
    let pasta_das_skills_da_casa = origem_stacklabs
        .join("desenvolvimento")
        .join("agentes")
        .join("skills");
    let pasta_das_skills_do_projeto = destino.join(".claude").join("skills");

    // ---------- 0. adoção: a etapa vem antes dos watchdogs, para eles já nascerem nela ----------
    let arquivo_do_metodo = destino.join(".claude").join("metodo.json");
    if opcoes.adocao && !arquivo_do_metodo.exists() {
        if !simular {
            criar_pasta(&destino.join(".claude"))?;
            escrever_json(
                &arquivo_do_metodo,
                &json!({ "etapaDeAdocao": "observacao", "adotadoEm": agora_em_iso() }),
            )?;
        }
        relatorio.push("ADOÇÃO    .claude/metodo.json criado na etapa observacao".to_string());
    }

    // ---------- 1. modelo-de-projeto ----------
    let ja_tem_watchdogs = destino.join(".claude").join("watchdogs").exists()
        || WATCHDOGS_ANTIGOS
            .iter()
            .any(|caminho| destino.join(caminho).exists());
    let watchdogs_mudam = !pastas_iguais(
        &pasta_do_modelo.join(".claude").join("watchdogs"),
        &destino.join(".claude").join("watchdogs"),
    )?;
    let mut aplica_watchdogs = true;
    if ja_tem_watchdogs && watchdogs_mudam && !simular {
        if let Err(linha_de_falha) = fonte_dos_watchdogs_compila(&pasta_do_modelo, &pasta_temporaria.0) {
            relatorio.push(linha_de_falha);
            aplica_watchdogs = false;
        }
    }

    // Os watchdogs antigos em node saem quando os novos entram.
    if aplica_watchdogs {
        for caminho_antigo in WATCHDOGS_ANTIGOS {
            let pasta_antiga = destino.join(caminho_antigo);
            if pasta_antiga.exists() {
                if !simular {
                    remover_pasta(&pasta_antiga)?;
                }
                relatorio.push(format!(
                    "REMOVIDA  {caminho_antigo} (watchdogs antigos: agora em .claude/watchdogs)"
                ));
            }
        }
    }

    let mut copiou_watchdogs = false;
    let arquivos_do_modelo = listar_arquivos(&pasta_do_modelo)?;
    for caminho_relativo in &arquivos_do_modelo {
        let caminho_relativo = caminho_relativo.clone();
        let arquivo_do_modelo = pasta_do_modelo.join(&caminho_relativo);
        let arquivo_do_destino = destino.join(&caminho_relativo);
        if arquivos_iguais(&arquivo_do_modelo, &arquivo_do_destino) {
            continue;
        }

        if caminho_eh_watchdog(&caminho_relativo) && !aplica_watchdogs {
            continue;
        }
        if caminho_relativo == "AGENTS.md" && arquivo_do_destino.exists() {
            let mesclado = mesclar_agents(
                &fs::read_to_string(&arquivo_do_modelo).map_err(|erro| erro.to_string())?,
                &fs::read_to_string(&arquivo_do_destino).map_err(|erro| erro.to_string())?,
            );
            if let Some(mesclado) = mesclado {
                if fs::read_to_string(&arquivo_do_destino).ok().as_deref() != Some(&mesclado) {
                    if !simular {
                        fs::write(&arquivo_do_destino, &mesclado)
                            .map_err(|erro| format!("AGENTS.md: {erro}"))?;
                    }
                    relatorio.push(format!(
                        "MESCLADO  AGENTS.md (modelo novo com a seção {SECAO_DO_PROJETO} do projeto)"
                    ));
                }
                continue;
            }
        }
        if caminho_relativo == ".claude/settings.json" && arquivo_do_destino.exists() {
            let (modelo, projeto) = match (ler_json(&arquivo_do_modelo), ler_json(&arquivo_do_destino)) {
                (Ok(modelo), Ok(projeto)) => (modelo, projeto),
                (Err(erro), _) | (_, Err(erro)) => {
                    relatorio.push(format!("FALHA     .claude/settings.json não é um JSON válido ({erro}): um humano conserta e roda de novo"));
                    continue;
                }
            };
            let mesclado = mesclar_settings(&modelo, &projeto);
            if mesclado != projeto {
                if !simular {
                    escrever_json(&arquivo_do_destino, &mesclado)?;
                }
                relatorio.push("MESCLADO  .claude/settings.json (hooks do modelo; o resto do projeto fica)".to_string());
            }
            continue;
        }
        if caminho_relativo == ".gitignore" && arquivo_do_destino.exists() {
            let conteudo_do_projeto = fs::read_to_string(&arquivo_do_destino).map_err(|erro| format!(".gitignore: {erro}"))?;
            let conteudo_do_modelo = fs::read_to_string(&arquivo_do_modelo).map_err(|erro| format!(".gitignore: {erro}"))?;
            let novas_linhas = linhas_que_faltam(&conteudo_do_modelo, &conteudo_do_projeto);
            if !novas_linhas.is_empty() {
                if !simular {
                    let separador = if conteudo_do_projeto.is_empty() || conteudo_do_projeto.ends_with('\n') { "" } else { "\n" };
                    let acrescentado = format!("{conteudo_do_projeto}{separador}{}\n", novas_linhas.join("\n"));
                    fs::write(&arquivo_do_destino, acrescentado).map_err(|erro| format!(".gitignore: {erro}"))?;
                }
                relatorio.push(format!("MESCLADO  .gitignore ({} linhas novas)", novas_linhas.len()));
            }
            continue;
        }
        if ARQUIVOS_PARA_MESCLAR.contains(&caminho_relativo.as_str()) && arquivo_do_destino.exists()
        {
            if !simular {
                copiar_arquivo(
                    &arquivo_do_modelo,
                    Path::new(&format!("{}.stacklabs-novo", texto(&arquivo_do_destino))),
                )?;
            }
            relatorio.push(format!(
                "MESCLAR   {caminho_relativo} (modelo novo em {caminho_relativo}.stacklabs-novo)"
            ));
            continue;
        }
        if !simular {
            if let Some(pasta) = arquivo_do_destino.parent() {
                criar_pasta(pasta)?;
            }
            copiar_arquivo_com_datas(&arquivo_do_modelo, &arquivo_do_destino)?;
        }
        if caminho_relativo.starts_with(".claude/watchdogs/") {
            copiou_watchdogs = true;
            if !simular {
                // Fonte com data antiga passaria por "já compilado" no `rodar` e no cargo, e o binário velho ficaria valendo.
                marcar_como_recem_alterado(&arquivo_do_destino)?;
            }
        }
        relatorio.push(format!("COPIADO   {caminho_relativo}"));
    }
    if aplica_watchdogs {
        for pasta_espelhada in PASTAS_ESPELHADAS {
            if !destino.join(pasta_espelhada).exists() {
                continue;
            }
            for arquivo_na_pasta in listar_arquivos(&destino.join(pasta_espelhada))? {
                let caminho_relativo = format!("{pasta_espelhada}/{arquivo_na_pasta}");
                if !arquivos_do_modelo.contains(&caminho_relativo) {
                    if !simular {
                        remover_pasta(&destino.join(&caminho_relativo))?;
                    }
                    if pasta_espelhada == ".claude/watchdogs" {
                        copiou_watchdogs = true;
                    }
                    relatorio.push(format!("REMOVIDO  {caminho_relativo} (saiu da fonte)"));
                }
            }
        }
    }
    for arquivo_mesclado in ARQUIVOS_MESCLADOS_PELO_INSTALADOR {
        let sobra = destino.join(format!("{arquivo_mesclado}.stacklabs-novo"));
        if sobra.exists() {
            if !simular {
                remover_pasta(&sobra)?;
            }
            relatorio.push(format!("REMOVIDO  {arquivo_mesclado}.stacklabs-novo (agora o instalador mescla)"));
        }
    }
    if copiou_watchdogs && !simular {
        relatorio.push(compilar_watchdogs(&destino));
    }
    if !simular {
        criar_pasta(&destino.join("docs").join("adr"))?;
        criar_pasta(&destino.join("docs").join("research"))?;
    }

    // ---------- 2. skills ----------
    let plugin_do_matt = ler_json(&origem_matt.join(".claude-plugin").join("plugin.json"))?;
    let pastas_do_matt: Vec<PathBuf> =
        lista_de_textos(&plugin_do_matt, "skills", "plugin.json do Matt")?
            .iter()
            .map(|caminho_skill| resolver(&texto(&origem_matt.join(caminho_skill))))
            .collect();
    let nomes_do_matt: Vec<String> = pastas_do_matt
        .iter()
        .map(|pasta| nome_da_pasta(pasta))
        .collect();

    // Da casa: as recomendadas que não são do Matt e as skills próprias do método (as demais pastas de skills/).
    let mut pastas_da_casa = subpastas(&pasta_das_skills_da_casa.join("recomendada"))?;
    pastas_da_casa.extend(subpastas(&pasta_das_skills_da_casa)?);
    pastas_da_casa.retain(|pasta| {
        let nome = nome_da_pasta(pasta);
        nome != "recomendada" && !nomes_do_matt.contains(&nome)
    });
    let nomes_da_casa: Vec<String> = pastas_da_casa
        .iter()
        .map(|pasta| nome_da_pasta(pasta))
        .collect();

    // Remove o que a instalação anterior trouxe e a fonte não tem mais.
    let arquivo_de_origem = pasta_das_skills_do_projeto.join("ORIGEM.json");
    if arquivo_de_origem.exists() {
        let origem_anterior = ler_json(&arquivo_de_origem)?;
        let versao_anterior = origem_anterior["stacklabs"]["versao"]
            .as_str()
            .map(str::to_string);
        if versao_anterior != versao_do_metodo {
            relatorio.push(format!(
                "VERSÃO    método {} → {} (o que mudou: desenvolvimento/CHANGELOG.md)",
                versao_anterior.as_deref().unwrap_or("sem versão"),
                versao_do_metodo.as_deref().unwrap_or("sem versão"),
            ));
        }
        let mut nomes_anteriores =
            lista_de_textos(&origem_anterior, "skillsDoMatt", "ORIGEM.json")?;
        nomes_anteriores.extend(lista_de_textos(
            &origem_anterior,
            "skillsDaCasa",
            "ORIGEM.json",
        )?);
        for nome_anterior in nomes_anteriores {
            let pasta_anterior = pasta_das_skills_do_projeto.join(&nome_anterior);
            if !nomes_do_matt.contains(&nome_anterior)
                && !nomes_da_casa.contains(&nome_anterior)
                && pasta_anterior.exists()
            {
                if !simular {
                    remover_pasta(&pasta_anterior)?;
                }
                relatorio.push(format!("REMOVIDA  skill {nome_anterior} (saiu da fonte)"));
            }
        }
    }

    let pasta_de_preparo =
        std::env::temp_dir().join(format!("stacklabs-skills-{}", std::process::id()));
    let mut copiar_skill = |pasta_de_origem: &Path, rotulo: &str| -> Resultado<()> {
        let nome = nome_da_pasta(pasta_de_origem);
        let pasta_do_destino = pasta_das_skills_do_projeto.join(&nome);
        // A skill do Matt passa pelos remendos do método antes de ser comparada e instalada.
        let pasta_pronta = if rotulo == "Matt" {
            let pasta_remendada = pasta_de_preparo.join(&nome);
            remover_pasta(&pasta_remendada)?;
            copiar_pasta(pasta_de_origem, &pasta_remendada)?;
            remendar_skill_do_matt(&pasta_remendada)?;
            pasta_remendada
        } else {
            pasta_de_origem.to_path_buf()
        };
        if pastas_iguais(&pasta_pronta, &pasta_do_destino)? {
            return Ok(());
        }
        if !simular {
            remover_pasta(&pasta_do_destino)?;
            copiar_pasta(&pasta_pronta, &pasta_do_destino)?;
        }
        relatorio.push(format!("SKILL     {nome} ({rotulo})"));
        Ok(())
    };
    for pasta_skill in &pastas_do_matt {
        copiar_skill(pasta_skill, "Matt")?;
    }
    for pasta_skill in &pastas_da_casa {
        copiar_skill(pasta_skill, "casa")?;
    }

    remover_pasta(&pasta_de_preparo)?;

    let commit_stacklabs = git_ou_nulo(&[
        "-C",
        &texto(&origem_stacklabs),
        "rev-parse",
        "--short",
        "HEAD",
    ])
    .filter(|commit| !commit.is_empty())
    .unwrap_or_else(|| "desconhecido".to_string());
    let commit_matt = git_ou_nulo(&["-C", &texto(&origem_matt), "rev-parse", "--short", "HEAD"])
        .filter(|commit| !commit.is_empty())
        .unwrap_or_else(|| "desconhecido".to_string());
    let versao_do_matt = plugin_do_matt.get("version").cloned();
    if !simular {
        let mut dados_do_matt =
            json!({ "repositorio": "mattpocock/skills", "commit": commit_matt });
        if let Some(versao) = &versao_do_matt {
            dados_do_matt["versao"] = versao.clone();
        }
        let mut dados_do_stacklabs =
            json!({ "repositorio": "stacklabs1/Stack-Labs", "commit": commit_stacklabs });
        if let Some(versao) = &versao_do_metodo {
            dados_do_stacklabs["versao"] = json!(versao);
        }
        escrever_json(
            &arquivo_de_origem,
            &json!({
                "instaladoEm": agora_em_iso(),
                "stacklabs": dados_do_stacklabs,
                "matt": dados_do_matt,
                "skillsDoMatt": nomes_do_matt,
                "skillsDaCasa": nomes_da_casa,
            }),
        )?;
    }

    // ---------- 3. pre-push e labels ----------
    let destino_em_texto = texto(&destino);
    if git_ou_nulo(&["-C", &destino_em_texto, "config", "--get", "core.hooksPath"]).as_deref()
        != Some(".claude/githooks")
        && aplica_watchdogs
    {
        if !simular {
            git(&[
                "-C",
                &destino_em_texto,
                "config",
                "core.hooksPath",
                ".claude/githooks",
            ])?;
        }
        relatorio.push("PRE-PUSH  ativado (core.hooksPath)".to_string());
    }

    if !opcoes.sem_labels {
        // Label que falha não desfaz o que já foi instalado: vira FALHA no relatório.
        let mut falha_das_labels: Option<String> = None;
        if !simular {
            for label in LABELS_DO_METODO {
                let criacao_da_label = Command::new("gh")
                    .args(["label", "create", label, "--force"])
                    .current_dir(&destino)
                    .stdin(Stdio::null())
                    .output();
                let motivo = match criacao_da_label {
                    Err(erro) => Some(erro.to_string()),
                    Ok(saida) if !saida.status.success() => {
                        Some(String::from_utf8_lossy(&saida.stderr).trim().to_string())
                    }
                    Ok(_) => None,
                };
                if let Some(motivo) = motivo {
                    falha_das_labels = Some(format!("gh label create {label}: {motivo}"));
                    break;
                }
            }
        }
        relatorio.push(match falha_das_labels {
            Some(motivo) => format!(
                "FALHA     labels do método não criadas ({motivo}); com o origin no GitHub e o gh autenticado, rode de novo"
            ),
            None => format!(
                "LABELS    as {} do método conferidas no GitHub",
                LABELS_DO_METODO.len()
            ),
        });
    }

    // ---------- relatório ----------
    let versao_em_texto = match &versao_do_matt {
        None => "undefined".to_string(),
        Some(Value::String(versao)) => versao.clone(),
        Some(versao) => versao.to_string(),
    };
    let versao_do_metodo_em_texto = versao_do_metodo
        .as_ref()
        .map_or("sem versão".to_string(), |versao| format!("v{versao}"));
    let cabecalho = format!(
        "\nMétodo StackLabs em {destino_em_texto}\nFontes: Stack-Labs {versao_do_metodo_em_texto} ({commit_stacklabs}) · mattpocock/skills {commit_matt} (v{versao_em_texto})"
    );
    Ok(ResultadoDaInstalacao { cabecalho, relatorio })
}

fn imprimir_instalacao(opcoes: &Opcoes) -> Resultado<()> {
    let instalacao = instalar(opcoes)?;
    println!("{}", instalacao.cabecalho);
    if opcoes.simular {
        println!("(simulação: nada foi escrito)");
    }
    if instalacao.relatorio.is_empty() {
        println!("Nada a fazer: o projeto já está em dia.");
    } else {
        println!("{}", instalacao.relatorio.join("\n"));
    }
    Ok(())
}

// ---------- skill atualizada ----------

/// Troca o conteúdo da pasta pelo da fonte, mantendo `target/` para não recompilar do zero.
fn substituir_mantendo_compilacao(pasta_de_origem: &Path, pasta_do_destino: &Path) -> Resultado<()> {
    if pasta_do_destino.exists() {
        let entradas = fs::read_dir(pasta_do_destino).map_err(|erro| format!("{}: {erro}", texto(pasta_do_destino)))?;
        for entrada in entradas {
            let entrada = entrada.map_err(|erro| erro.to_string())?;
            if entrada.file_name() != PASTA_DE_COMPILACAO {
                remover_pasta(&entrada.path())?;
            }
        }
    }
    copiar_pasta(pasta_de_origem, pasta_do_destino)
}

fn conferir_skills(opcoes: &Opcoes) -> Resultado<()> {
    // O binário mora em <skills>/setup-stacklabs/target/release/instalar.
    let pasta_das_skills = match &opcoes.pasta_das_skills {
        Some(pasta) => resolver(pasta),
        None => std::env::current_exe()
            .map_err(|erro| erro.to_string())?
            .ancestors()
            .nth(4)
            .map(Path::to_path_buf)
            .ok_or("não achei a pasta das skills a partir do binário")?,
    };
    let raiz_do_git = git_ou_nulo(&["-C", &texto(&pasta_das_skills), "rev-parse", "--show-toplevel"]);
    if raiz_do_git.is_some_and(|raiz| Path::new(&raiz).join("desenvolvimento").join("VERSAO").exists()) {
        println!("SKILL É A FONTE: rodando de um clone do Stack-Labs, nada a atualizar.");
        return Ok(());
    }
    let pasta_temporaria = PastaTemporaria::criar()?;
    let origem = match buscar_origem(&pasta_temporaria.0, std::env::var("STACKLABS_ORIGEM").ok(), URL_DO_STACKLABS, "stacklabs", None) {
        Ok(origem) => origem,
        Err(erro) => {
            let primeira_linha = erro.lines().last().unwrap_or("").trim();
            println!("SKILL NÃO CONFERIDA: sem acesso à fonte oficial ({primeira_linha}).");
            return Ok(());
        }
    };
    let versao = fs::read_to_string(origem.join("desenvolvimento").join("VERSAO")).unwrap_or_default();
    let versao = versao.trim();
    let pasta_das_skills_da_fonte = origem.join("desenvolvimento").join("agentes").join("skills");
    let mut atualizadas = Vec::new();
    for nome_da_skill in SKILLS_DE_MAQUINA {
        let pasta_de_origem = pasta_das_skills_da_fonte.join(nome_da_skill);
        let pasta_do_destino = pasta_das_skills.join(nome_da_skill);
        if !pastas_iguais(&pasta_de_origem, &pasta_do_destino)? {
            substituir_mantendo_compilacao(&pasta_de_origem, &pasta_do_destino)?;
            atualizadas.push(nome_da_skill);
        }
    }
    if atualizadas.is_empty() {
        println!("SKILL EM DIA (método v{versao}).");
    }
    for nome_da_skill in atualizadas {
        println!("SKILL ATUALIZADA {nome_da_skill} (método v{versao}).");
    }
    Ok(())
}

// ---------- verificação ----------

fn comando_passa(programa: &str, argumentos: &[&str], pasta: &Path) -> bool {
    Command::new(programa)
        .args(argumentos)
        .current_dir(pasta)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn problemas_das_skills(destino: &Path) -> Vec<String> {
    let pasta_das_skills = destino.join(".claude").join("skills");
    let mut problemas = Vec::new();
    match subpastas(&pasta_das_skills) {
        Ok(pastas) => {
            for pasta_da_skill in pastas {
                let nome_da_skill = nome_da_pasta(&pasta_da_skill);
                if nome_da_skill != "prototipo-html" && !pasta_da_skill.join("SKILL.md").exists() {
                    problemas.push(format!(".claude/skills/{nome_da_skill} não tem SKILL.md"));
                }
            }
        }
        Err(erro) => problemas.push(erro),
    }
    match ler_json(&pasta_das_skills.join("ORIGEM.json")) {
        Ok(origem) => {
            for campo in ["skillsDoMatt", "skillsDaCasa"] {
                if origem[campo].as_array().is_none_or(|skills| skills.is_empty()) {
                    problemas.push(format!("ORIGEM.json não lista {campo}"));
                }
            }
        }
        Err(erro) => problemas.push(erro),
    }
    problemas
}

/// As reprovações da instalação do projeto; vazio quer dizer que passou.
fn verificar_instalacao(destino: &Path) -> Vec<String> {
    let mut reprovacoes = Vec::new();
    let lancador = ".claude/watchdogs/rodar";
    if !comando_passa("sh", &[lancador, "--version"], destino) {
        reprovacoes.push(format!("sh {lancador} --version falhou: os watchdogs não compilam"));
    }
    if !comando_passa("sh", &["-n", ".claude/githooks/pre-push"], destino) {
        reprovacoes.push(".claude/githooks/pre-push ausente ou com erro de sintaxe".to_string());
    }
    for caminho_antigo in WATCHDOGS_ANTIGOS {
        if destino.join(caminho_antigo).exists() {
            reprovacoes.push(format!("{caminho_antigo} ainda existe"));
        }
    }
    if git_ou_nulo(&["-C", &texto(destino), "config", "--get", "core.hooksPath"]).as_deref() != Some(".claude/githooks") {
        reprovacoes.push("core.hooksPath não aponta para .claude/githooks".to_string());
    }
    reprovacoes.extend(problemas_das_skills(destino));
    if destino.join("docs").join("regras-de-codigo.json").exists() {
        let validacao = Command::new("sh").args([lancador, "validar"]).current_dir(destino).stdin(Stdio::null()).output();
        let aprovado = validacao.is_ok_and(|saida| String::from_utf8_lossy(&saida.stdout).contains("VALIDADOR: APROVADO"));
        if !aprovado {
            reprovacoes.push(format!("o validador reprovou: rode sh {lancador} validar"));
        }
    }
    reprovacoes
}

fn imprimir_verificacao(opcoes: &Opcoes) -> Resultado<()> {
    let reprovacoes = verificar_instalacao(&destino_das_opcoes(opcoes)?);
    if reprovacoes.is_empty() {
        println!("VERIFICADO: a instalação passou em todas as conferências.");
        return Ok(());
    }
    for reprovacao in &reprovacoes {
        println!("REPROVADO {reprovacao}");
    }
    Err(format!("{} conferências reprovaram", reprovacoes.len()))
}

// ---------- PR de atualização ----------

fn gh(argumentos: &[&str], pasta: &Path) -> Resultado<String> {
    let saida = Command::new("gh")
        .args(argumentos)
        .current_dir(pasta)
        .stdin(Stdio::null())
        .output()
        .map_err(|erro| format!("gh {}: {erro}", argumentos.join(" ")))?;
    if !saida.status.success() {
        return Err(format!("gh {}: {}", argumentos.join(" "), String::from_utf8_lossy(&saida.stderr).trim()));
    }
    Ok(String::from_utf8_lossy(&saida.stdout).trim().to_string())
}

/// Espera o CI da PR. O GitHub pode levar uns segundos para registrar os checks depois do push ou da label.
fn esperar_ci(numero_em_texto: &str, destino: &Path) -> Resultado<()> {
    const TENTATIVAS: u32 = 6;
    const ESPERA_ENTRE_TENTATIVAS: std::time::Duration = std::time::Duration::from_secs(20);
    let espera = if std::env::var("STACKLABS_SEM_ESPERA").is_ok() { std::time::Duration::ZERO } else { ESPERA_ENTRE_TENTATIVAS };
    let mut ultimo_erro = String::new();
    for _ in 0..TENTATIVAS {
        std::thread::sleep(espera);
        match gh(&["pr", "checks", numero_em_texto, "--watch"], destino) {
            Ok(_) => return Ok(()),
            Err(erro) if erro.contains("no checks reported") => ultimo_erro = erro,
            Err(erro) => return Err(erro),
        }
    }
    Err(ultimo_erro)
}

/// O que impede a PR de entrar sem uma pessoa. Tudo o que ela muda precisa ser o que a fonte oficial manda.
fn impedimentos_da_pr(opcoes: &Opcoes, destino: &Path, numero_em_texto: &str) -> Resultado<Vec<String>> {
    let mut impedimentos = Vec::new();
    let destino_em_texto = texto(destino);
    let dados_da_pr: Value = serde_json::from_str(&gh(
        &["pr", "view", numero_em_texto, "--json", "state,headRefName,headRefOid,baseRefName"],
        destino,
    )?)
    .map_err(|erro| erro.to_string())?;
    if dados_da_pr["state"] != "OPEN" {
        impedimentos.push("a PR não está aberta".to_string());
    }
    if !dados_da_pr["headRefName"].as_str().unwrap_or("").starts_with("chore/") {
        impedimentos.push("a branch da PR não é chore/*".to_string());
    }
    if dados_da_pr["headRefOid"].as_str() != git_ou_nulo(&["-C", &destino_em_texto, "rev-parse", "HEAD"]).as_deref() {
        impedimentos.push("o HEAD local não é o commit da PR".to_string());
    }
    if git_ou_nulo(&["-C", &destino_em_texto, "status", "--porcelain"]).is_none_or(|estado| !estado.is_empty()) {
        impedimentos.push("há mudanças fora do commit".to_string());
    }

    // Instalar de novo por cima não pode achar nada: o commit já é o que a fonte manda.
    let simulacao = instalar(&Opcoes { destino: opcoes.destino.clone(), simular: true, sem_labels: true, ..Opcoes::default() })?;
    impedimentos.extend(simulacao.relatorio.iter().map(|linha| format!("difere da fonte oficial: {linha}")));
    impedimentos.extend(verificar_instalacao(destino));

    let base = format!("origin/{}", dados_da_pr["baseRefName"].as_str().unwrap_or("main"));
    git(&["-C", &destino_em_texto, "fetch", "-q", "origin"])?;
    let arquivo_na_base = |caminho: &str| git_ou_nulo(&["-C", &destino_em_texto, "show", &format!("{base}:{caminho}")]);
    if arquivo_na_base(".claude/skills/ORIGEM.json").is_none() {
        impedimentos.push("é uma instalação nova, não uma atualização".to_string());
    }

    let pasta_temporaria = PastaTemporaria::criar()?;
    let origem = buscar_origem(&pasta_temporaria.0, std::env::var("STACKLABS_ORIGEM").ok(), URL_DO_STACKLABS, "stacklabs", None)?;
    let pasta_do_modelo = origem.join("desenvolvimento").join("agentes").join("modelo-de-projeto");
    let arquivos_do_modelo = listar_arquivos(&pasta_do_modelo)?;
    let arquivos_da_pr = gh(&["pr", "diff", numero_em_texto, "--name-only"], destino)?;
    for arquivo_da_pr in arquivos_da_pr.lines().filter(|linha| !linha.is_empty()) {
        let veio_do_instalador = arquivos_do_modelo.iter().any(|arquivo| arquivo == arquivo_da_pr)
            || arquivo_da_pr.starts_with(".claude/skills/")
            || WATCHDOGS_ANTIGOS.iter().chain(PASTAS_ESPELHADAS.iter()).any(|pasta| arquivo_da_pr.starts_with(&format!("{pasta}/")));
        if !veio_do_instalador {
            impedimentos.push(format!("{arquivo_da_pr} não vem do instalador"));
        }
        if arquivo_da_pr == "CLAUDE.md" {
            impedimentos.push("CLAUDE.md foi mesclado à mão".to_string());
        }
    }

    // A parte que é do projeto não muda numa atualização.
    let agents_atual = fs::read_to_string(destino.join("AGENTS.md")).unwrap_or_default();
    let secao_de = |conteudo: &str| conteudo.find(SECAO_DO_PROJETO).map(|inicio| conteudo[inicio..].trim().to_string());
    if arquivo_na_base("AGENTS.md").and_then(|conteudo| secao_de(&conteudo)) != secao_de(&agents_atual) {
        impedimentos.push(format!("a seção {SECAO_DO_PROJETO} do AGENTS.md mudou ou foi criada à mão"));
    }
    let settings_na_base = arquivo_na_base(".claude/settings.json").unwrap_or_else(|| "{}".to_string());
    let settings_esperado = serde_json::from_str::<Value>(&settings_na_base)
        .ok()
        .zip(ler_json(&pasta_do_modelo.join(".claude").join("settings.json")).ok())
        .map(|(projeto, modelo)| mesclar_settings(&modelo, &projeto));
    if settings_esperado != ler_json(&destino.join(".claude").join("settings.json")).ok() {
        impedimentos.push(".claude/settings.json traz mais do que os hooks do modelo".to_string());
    }
    Ok(impedimentos)
}

fn concluir_pr(opcoes: &Opcoes, numero_da_pr: u64) -> Resultado<()> {
    let destino = destino_das_opcoes(opcoes)?;
    let numero_em_texto = numero_da_pr.to_string();
    let ficar_para_humano = |motivos: &[String]| -> ! {
        println!("PR #{numero_da_pr} FICA PARA UM HUMANO:");
        for motivo in motivos {
            println!("- {motivo}");
        }
        std::process::exit(CODIGO_FICA_PARA_HUMANO);
    };
    let impedimentos = impedimentos_da_pr(opcoes, &destino, &numero_em_texto)?;
    if !impedimentos.is_empty() {
        ficar_para_humano(&impedimentos);
    }
    let arquivos_da_pr = gh(&["pr", "diff", &numero_em_texto, "--name-only"], &destino)?;
    if arquivos_da_pr.lines().any(caminho_eh_watchdog) {
        gh(&["pr", "edit", &numero_em_texto, "--add-label", LABEL_ALTERA_WATCHDOGS], &destino)?;
        println!("LABEL     {LABEL_ALTERA_WATCHDOGS}: os watchdogs da PR são idênticos aos da fonte oficial");
    }
    if let Err(erro) = esperar_ci(&numero_em_texto, &destino) {
        ficar_para_humano(&[format!("o CI não ficou verde: {erro}")]);
    }
    if let Err(erro) = gh(&["pr", "merge", &numero_em_texto, "--merge"], &destino) {
        ficar_para_humano(&[format!("o merge falhou: {erro}")]);
    }
    println!("MESCLADA  PR #{numero_da_pr}");
    Ok(())
}

fn main() {
    let argumentos: Vec<String> = std::env::args().skip(1).collect();
    let resultado = ler_argumentos(&argumentos).and_then(|opcoes| {
        if opcoes.conferir_skill {
            conferir_skills(&opcoes)
        } else if opcoes.verificar {
            imprimir_verificacao(&opcoes)
        } else if let Some(numero_da_pr) = opcoes.concluir_pr {
            concluir_pr(&opcoes, numero_da_pr)
        } else {
            imprimir_instalacao(&opcoes)
        }
    });
    if let Err(mensagem) = resultado {
        eprintln!("{mensagem}");
        std::process::exit(1);
    }
}
