// CI local (ADR 0015 do método): roda na máquina, pelo pre-push ou à mão, o que o projeto decidiu em
// docs/regras-de-codigo.json, em três partes: as regras `comando` (lint e typecheck), o resto do
// validador e a revisão por modelo dos testes alterados. Reprova só pelas duas primeiras; a revisão é
// consultiva e falha aberta. Projeto que ainda não fechou as regras de código passa direto.
//
// O ci roda como comando pesado (trava.rs): uma vaga e o limite de memória.
// Aprovado com a árvore limpa, ele grava a aprovação pela chave do conteúdo (árvore do HEAD, commit
// da base, regras e o hash do binário dos watchdogs); o pre-push que encontra a chave não roda de novo.
//
// Uso: .claude/watchdogs/rodar ci [--base <ref>] [--sem-revisao]   (base padrão: origin/main)
// Saída: 0 aprovado, 1 violação, 2 configuração inválida. A última linha é "CI: APROVADO" ou "CI: REPROVADO".

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::comum::executar_git;
use crate::revisao_por_modelo::{self, Revisao};
use crate::trava;
use crate::validador::{self, ErroDeConfiguracao, Opcoes, Relatorio};

pub const BASE_PADRAO: &str = "origin/main";
const CAMINHO_CONFIG: &str = "docs/regras-de-codigo.json";
const CAMINHO_MD: &str = "docs/regras-de-codigo.md";
const PASTA_DE_APROVADOS: &str = ".claude/state/ci-aprovado";

pub struct OpcoesDoCi<'a> {
    pub base: &'a str,
    // A revisão por modelo é consultiva e a parte mais lenta: o pre-push a pula.
    pub revisar: bool,
}

// Chave do conteúdo que o ci confere. None com a árvore suja (arquivo não commitado ou não
// rastreado também entra no ci) ou sem a base: aí não há cache.
pub fn chave_do_conteudo(raiz: &Path, base: &str) -> Option<String> {
    let pendencias = executar_git(raiz, &["status", "--porcelain", "--untracked-files=normal"])?;
    if !pendencias.is_empty() {
        return None;
    }
    let arvore = executar_git(raiz, &["rev-parse", "HEAD^{tree}"])?;
    let commit_da_base = executar_git(raiz, &["rev-parse", "--verify", "--quiet", &format!("{base}^{{commit}}")])?;
    // A versão do pacote não muda entre versões do método; o binário muda. Watchdogs atualizados com
    // regras mais rígidas não reaproveitam a aprovação dada pelos antigos.
    let hash_do_binario = hash_do_proprio_binario()?;
    let mut soma = Sha256::new();
    for parte in [
        "ci-v2",
        hash_do_binario,
        &arvore,
        &commit_da_base,
        &fs::read_to_string(raiz.join(CAMINHO_CONFIG)).unwrap_or_default(),
        &fs::read_to_string(raiz.join(CAMINHO_MD)).unwrap_or_default(),
    ] {
        soma.update(parte.as_bytes());
        soma.update([0]);
    }
    Some(format!("{:x}", soma.finalize()))
}

// Calculado uma vez por processo: o pre-push e o ci leem a chave mais de uma vez.
fn hash_do_proprio_binario() -> Option<&'static str> {
    static HASH: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    HASH.get_or_init(|| {
        let binario = fs::read(std::env::current_exe().ok()?).ok()?;
        Some(format!("{:x}", Sha256::digest(&binario)))
    })
    .as_deref()
}

pub fn ja_aprovado(raiz: &Path, chave: &str) -> bool {
    raiz.join(PASTA_DE_APROVADOS).join(chave).is_file()
}

fn gravar_aprovacao(raiz: &Path, chave: &str, base: &str) {
    let pasta = raiz.join(PASTA_DE_APROVADOS);
    if fs::create_dir_all(&pasta).is_ok() {
        let _ = fs::write(pasta.join(chave), format!("aprovado contra {base}\n"));
    }
}

fn ler_configuracao(raiz: &Path) -> Result<(Value, String), ErroDeConfiguracao> {
    let texto_config = fs::read_to_string(raiz.join(CAMINHO_CONFIG)).unwrap_or_default();
    let config = serde_json::from_str(&texto_config).map_err(|erro_json| ErroDeConfiguracao(format!("{CAMINHO_CONFIG} não é JSON válido: {erro_json}")))?;
    let texto_md = fs::read_to_string(raiz.join(CAMINHO_MD))
        .map_err(|_| ErroDeConfiguracao(format!("{CAMINHO_MD} não existe (o mapa de Regras de código gera os dois arquivos)")))?;
    Ok((config, texto_md))
}

fn escrever_revisao(saida: &mut dyn Write, id_da_regra: &str, revisao: &Revisao) {
    let _ = writeln!(
        saida,
        "{id_da_regra} com {}: {} teste(s) revisado(s), {} chamada(s) ao modelo, {} do cache",
        revisao.modelo,
        revisao.revisados.len(),
        revisao.chamadas_ao_modelo,
        revisao.vindos_do_cache
    );
    for suspeito in revisao.suspeitos() {
        let _ = writeln!(
            saida,
            "⚠️  {}:{} {} ({}): não exercita o risco que cita. {}",
            suspeito.arquivo,
            suspeito.linha,
            suspeito.nome,
            suspeito.riscos.join(", "),
            suspeito.veredito.motivo
        );
    }
    for aviso in &revisao.avisos {
        let _ = writeln!(saida, "⚠️  revisão por modelo incompleta: {aviso}. O ci segue sem ela.");
    }
    if revisao.sem_veredito > 0 {
        let _ = writeln!(saida, "   {} teste(s) ficaram sem veredito.", revisao.sem_veredito);
    }
    if revisao.suspeitos().next().is_some() {
        let _ = writeln!(
            saida,
            "   Consultivo: o revisor confirma ou descarta cada suspeito (regra 15, docs/agents/riscos.md)."
        );
    }
}

fn revisar_os_testes(raiz: &Path, config: &Value, base: &str, saida: &mut dyn Write) {
    let regras_de_revisao: Vec<&Value> = config["regras"]
        .as_array()
        .map(|regras| {
            regras
                .iter()
                .filter(|regra| regra["verificacao"]["tipo"].as_str() == Some("revisao-por-modelo"))
                .collect()
        })
        .unwrap_or_default();
    if regras_de_revisao.is_empty() {
        let _ = writeln!(saida, "nenhuma regra revisao-por-modelo em {CAMINHO_CONFIG}.");
        return;
    }
    let Some(alterados) = validador::arquivos_alterados_desde(raiz, base) else {
        let _ = writeln!(saida, "⚠️  a base {base} não existe; a revisão por modelo não rodou.");
        return;
    };
    for regra in regras_de_revisao {
        let mut parametros = regra["verificacao"].as_object().cloned().unwrap_or_default();
        parametros.remove("tipo");
        let revisao = revisao_por_modelo::revisar(raiz, &config["projeto"], &parametros, &alterados);
        escrever_revisao(saida, regra["id"].as_str().unwrap_or(""), &revisao);
    }
}

// O CI inteiro numa pasta de projeto, escrevendo o relatório em `saida`. Devolve o código de saída.
pub fn executar(raiz: &Path, opcoes_do_ci: &OpcoesDoCi, saida: &mut dyn Write) -> i32 {
    let base = opcoes_do_ci.base;
    // A chave se lê antes: um arquivo criado durante o ci não pode entrar numa aprovação velha.
    let chave = chave_do_conteudo(raiz, base);
    if !raiz.join(CAMINHO_CONFIG).exists() {
        let _ = writeln!(
            saida,
            "[ci] {CAMINHO_CONFIG} não existe: nada a conferir ainda (o mapa de Regras de código o cria).\nCI: APROVADO"
        );
        return 0;
    }
    let _ = writeln!(saida, "[ci] base {base}");
    let relatorio = ler_configuracao(raiz).and_then(|(config, texto_md)| {
        let opcoes = Opcoes {
            base: Some(base),
            completo: false,
            so_cobertura: false,
        };
        validador::validar(raiz, &config, &texto_md, &opcoes).map(|relatorio| (config, relatorio))
    });
    let (config, relatorio) = match relatorio {
        Ok(lido) => lido,
        Err(ErroDeConfiguracao(mensagem)) => {
            let _ = writeln!(saida, "Configuração inválida: {mensagem}\nCI: REPROVADO");
            return 2;
        }
    };
    let aprovado = relatorio.aprovado;
    let (comandos, demais): (Vec<_>, Vec<_>) = relatorio.resultados.into_iter().partition(|resultado| resultado.tipo == "comando");

    let _ = writeln!(saida, "\n[ci] 1/3 lint e typecheck (regras comando)");
    if comandos.is_empty() {
        let _ = writeln!(saida, "nenhuma regra comando em {CAMINHO_CONFIG}.");
    }
    for resultado in &comandos {
        let _ = writeln!(saida, "{}", validador::formatar_resultado(resultado).join("\n"));
    }

    let _ = writeln!(saida, "\n[ci] 2/3 validador");
    let validador_aprovado = demais.iter().all(|resultado| resultado.violacoes.is_empty());
    let do_validador = Relatorio {
        aprovado: validador_aprovado,
        escopo: relatorio.escopo,
        resultados: demais,
        avisos: relatorio.avisos,
    };
    let _ = writeln!(saida, "{}", validador::formatar(&do_validador));

    let _ = writeln!(saida, "\n[ci] 3/3 revisão por modelo dos testes alterados (consultiva)");
    if opcoes_do_ci.revisar {
        revisar_os_testes(raiz, &config, base, saida);
    } else {
        let _ = writeln!(saida, "pulada aqui; rode sh .claude/watchdogs/rodar ci para vê-la.");
    }

    if let (true, Some(chave)) = (aprovado, &chave) {
        if chave_do_conteudo(raiz, base).as_ref() == Some(chave) {
            gravar_aprovacao(raiz, chave, base);
        }
    }
    let _ = writeln!(saida, "\nCI: {}", if aprovado { "APROVADO" } else { "REPROVADO" });
    if aprovado {
        0
    } else {
        1
    }
}

pub fn rodar(argumentos: &[String]) -> i32 {
    let mut base = BASE_PADRAO.to_string();
    let mut revisar = true;
    let mut restantes = argumentos.iter();
    while let Some(argumento) = restantes.next() {
        match (argumento.as_str(), restantes.clone().next()) {
            ("--base", Some(valor)) => {
                base = valor.clone();
                restantes.next();
            }
            ("--sem-revisao", _) => revisar = false,
            _ => {
                eprintln!("uso: rodar ci [--base <ref>] [--sem-revisao]");
                return 2;
            }
        }
    }
    let raiz = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if trava::projeto_de_fora().is_none() {
        return rodar_como_pesado(&raiz, argumentos, trava::Saida::Herdada);
    }
    executar(&raiz, &OpcoesDoCi { base: &base, revisar }, &mut std::io::stdout())
}

// O próprio binário, de novo, como comando pesado: pega uma vaga e o limite de memória. Não pega a
// trava do projeto: o ci só lê o código, e um dev server de pé no projeto não pode segurar o push.
pub fn rodar_como_pesado(raiz: &Path, argumentos_do_ci: &[String], saida: trava::Saida) -> i32 {
    let Ok(binario) = std::env::current_exe() else {
        eprintln!("[ci] não achei o próprio binário para rodar como comando pesado.");
        return 2;
    };
    let argumentos: Vec<String> = [binario.to_string_lossy().into_owned(), "ci".into()]
        .into_iter()
        .chain(argumentos_do_ci.iter().cloned())
        .collect();
    trava::executar(&argumentos, raiz, saida, trava::Travar::SoVaga)
}
