//! Instala ou atualiza o método StackLabs num projeto: modelo-de-projeto, skills da casa e as skills
//! publicadas do Matt Pocock (sempre da versão mais nova do repositório dele).
//!
//! Uso: sh rodar [--destino <pasta>] [--versao <x.y.z>] [--simular] [--sem-labels] [--atualizar-watchdogs] [--adocao]
//!   --destino              raiz do projeto (padrão: a raiz do git da pasta atual)
//!   --versao               instala a versão marcada (tag v<x.y.z> do Stack-Labs) em vez da main
//!   --simular              mostra o que faria, sem escrever nada
//!   --sem-labels           não cria as labels do método no GitHub
//!   --atualizar-watchdogs  sobrescreve watchdogs que já existem. SÓ HUMANO (o guarda-bash barra o agente)
//!   --adocao               projeto que já existia: cria .claude/metodo.json na etapa de observação (/adotar-metodo-stack-labs)
//!
//! Fontes (opcional, para usar um clone local em vez de baixar):
//!   STACKLABS_ORIGEM=<clone do Stack-Labs>   MATT_ORIGEM=<clone de mattpocock/skills>

use serde_json::{json, Value};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

/// Arquivos que o projeto pode ter com conteúdo próprio: nunca sobrescreve, deixa o novo ao lado.
const ARQUIVOS_PARA_MESCLAR: [&str; 4] = [
    "AGENTS.md",
    "CLAUDE.md",
    ".gitignore",
    ".claude/settings.json",
];
/// Seção do `AGENTS.md` que pertence ao projeto: o instalador a carrega de uma versão para a outra.
const SECAO_DO_PROJETO: &str = "## Regras deste projeto";
/// Infraestrutura dos watchdogs: só se sobrescreve com --atualizar-watchdogs.
const CAMINHOS_DOS_WATCHDOGS: [&str; 5] = [
    ".claude/watchdogs",
    ".claude/githooks",
    ".claude/agents",
    ".claude/settings.json",
    ".github/workflows/vigia-do-metodo.yml",
];
/// Watchdogs antigos (em node), que a versão em Rust de `.claude/watchdogs/` substitui.
const WATCHDOGS_ANTIGOS: [&str; 2] = [".claude/hooks", ".claude/validador"];
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
    atualizar_watchdogs: bool,
    adocao: bool,
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
            "--atualizar-watchdogs" => opcoes.atualizar_watchdogs = true,
            "--adocao" => opcoes.adocao = true,
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

/// Apaga a compilação anterior dos watchdogs do projeto. A cópia guarda as datas da origem (num clone
/// local, mais antigas que a compilação), e tanto o `rodar` quanto o cargo decidem recompilar pela
/// data: o cargo religaria o artefato velho, e o relatório diria COMPILADO com o binário velho rodando
/// (e o `rodar` o guardaria sob o hash do fonte novo). Sem `target/`, o cargo compila do zero ou o
/// `rodar` reaproveita o binário guardado pelo hash do fonte.
fn apagar_compilacao_antiga(destino: &Path) {
    let _ = fs::remove_dir_all(
        destino
            .join(".claude")
            .join("watchdogs")
            .join(PASTA_DE_COMPILACAO),
    );
}

/// Compila os watchdogs do projeto uma vez, para a primeira sessão não esperar a compilação.
fn compilar_watchdogs(destino: &Path) -> String {
    let lancador = destino.join(".claude").join("watchdogs").join("rodar");
    let execucao = Command::new("sh")
        .arg(&lancador)
        .arg("--version")
        .current_dir(destino)
        .stdin(Stdio::null())
        .output();
    match execucao {
        Ok(saida) if saida.status.success() => "COMPILADO .claude/watchdogs".to_string(),
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
            format!("FALHA     .claude/watchdogs não compilou (sh .claude/watchdogs/rodar --version saiu com {codigo}): {motivo}")
        }
        Err(erro) => format!("FALHA     .claude/watchdogs não compilou (sh .claude/watchdogs/rodar --version): {erro}"),
    }
}

// ---------- instalação ----------

fn instalar(opcoes: &Opcoes) -> Resultado<()> {
    conferir_cargo()?;
    let destino = match &opcoes.destino {
        Some(pasta) => resolver(pasta),
        None => resolver(&git(&["rev-parse", "--show-toplevel"]).map_err(|_| {
            "a pasta atual não é um repositório git: rode dentro do projeto ou passe --destino <pasta>"
                .to_string()
        })?),
    };
    let pasta_temporaria = PastaTemporaria::criar()?;
    let mut relatorio: Vec<String> = Vec::new();
    let simular = opcoes.simular;

    let buscar_origem = |clone_local: Option<String>,
                         url_do_repositorio: &str,
                         nome_da_pasta: &str,
                         tag: Option<String>|
     -> Resultado<PathBuf> {
        if let Some(clone) = clone_local.filter(|clone| !clone.is_empty()) {
            return Ok(resolver(&clone));
        }
        let pasta_do_clone = pasta_temporaria.0.join(nome_da_pasta);
        let pasta_em_texto = texto(&pasta_do_clone);
        let mut argumentos = vec!["clone", "-q", "--depth", "1"];
        if let Some(tag) = &tag {
            argumentos.extend(["--branch", tag.as_str()]);
        }
        argumentos.extend([url_do_repositorio, pasta_em_texto.as_str()]);
        git(&argumentos)?;
        Ok(pasta_do_clone)
    };
    let origem_stacklabs = buscar_origem(
        std::env::var("STACKLABS_ORIGEM").ok(),
        "https://github.com/stacklabs1/Stack-Labs",
        "stacklabs",
        opcoes.versao.as_ref().map(|versao| format!("v{versao}")),
    )?;
    let origem_matt = buscar_origem(
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
    let aplica_watchdogs = !ja_tem_watchdogs || opcoes.atualizar_watchdogs;

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
    for caminho_relativo in listar_arquivos(&pasta_do_modelo)? {
        let arquivo_do_modelo = pasta_do_modelo.join(&caminho_relativo);
        let arquivo_do_destino = destino.join(&caminho_relativo);
        if arquivos_iguais(&arquivo_do_modelo, &arquivo_do_destino) {
            continue;
        }

        if caminho_eh_watchdog(&caminho_relativo) && !aplica_watchdogs {
            relatorio.push(format!(
                "PULADO    {caminho_relativo} (watchdog ausente ou desatualizado: um humano roda com --atualizar-watchdogs)"
            ));
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
        }
        relatorio.push(format!("COPIADO   {caminho_relativo}"));
    }
    if copiou_watchdogs && !simular {
        apagar_compilacao_antiga(&destino);
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
    {
        if aplica_watchdogs {
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
        } else {
            relatorio.push(
                "PRE-PUSH  inativo: um humano roda git config core.hooksPath .claude/githooks"
                    .to_string(),
            );
        }
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
    println!("\nMétodo StackLabs em {destino_em_texto}");
    let versao_do_metodo_em_texto = versao_do_metodo
        .as_ref()
        .map_or("sem versão".to_string(), |versao| format!("v{versao}"));
    println!("Fontes: Stack-Labs {versao_do_metodo_em_texto} ({commit_stacklabs}) · mattpocock/skills {commit_matt} (v{versao_em_texto})");
    if simular {
        println!("(simulação: nada foi escrito)");
    }
    if relatorio.is_empty() {
        println!("Nada a fazer: o projeto já está em dia.");
    } else {
        println!("{}", relatorio.join("\n"));
    }
    Ok(())
}

fn main() {
    let argumentos: Vec<String> = std::env::args().skip(1).collect();
    let resultado = ler_argumentos(&argumentos).and_then(|opcoes| instalar(&opcoes));
    if let Err(mensagem) = resultado {
        eprintln!("{mensagem}");
        std::process::exit(1);
    }
}
