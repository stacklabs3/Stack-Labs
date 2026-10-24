// CI local (ADR 0015 do método): roda na máquina, pelo pre-push ou à mão, o que o projeto decidiu em
// docs/regras-de-codigo.json, em três partes: as regras `comando` (lint e typecheck), o resto do
// validador e a revisão por modelo dos testes alterados. Reprova só pelas duas primeiras; a revisão é
// consultiva e falha aberta. Projeto que ainda não fechou as regras de código passa direto.
//
// Uso: .claude/watchdogs/rodar ci [--base <ref>]   (base padrão: origin/main)
// Saída: 0 aprovado, 1 violação, 2 configuração inválida. A última linha é "CI: APROVADO" ou "CI: REPROVADO".

use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::revisao_por_modelo::{self, Revisao};
use crate::validador::{self, ErroDeConfiguracao, Opcoes, Relatorio};

pub const BASE_PADRAO: &str = "origin/main";
const CAMINHO_CONFIG: &str = "docs/regras-de-codigo.json";
const CAMINHO_MD: &str = "docs/regras-de-codigo.md";

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
pub fn executar(raiz: &Path, base: &str, saida: &mut dyn Write) -> i32 {
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
    revisar_os_testes(raiz, &config, base, saida);

    let _ = writeln!(saida, "\nCI: {}", if aprovado { "APROVADO" } else { "REPROVADO" });
    if aprovado {
        0
    } else {
        1
    }
}

pub fn rodar(argumentos: &[String]) -> i32 {
    let base = match argumentos {
        [] => BASE_PADRAO.to_string(),
        [opcao, base] if opcao == "--base" => base.clone(),
        _ => {
            eprintln!("uso: rodar ci [--base <ref>]");
            return 2;
        }
    };
    let raiz = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    executar(&raiz, &base, &mut std::io::stdout())
}
