// Watchdog PostToolUse (Write | Edit | MultiEdit | Bash): o teste que o agente acabou de escrever cobre um
// risco de docs/riscos.md e não tem os sinais de teste sem valor (sem afirmação, afirmação trivial,
// desligado). Roda as regras do P5 de docs/regras-de-codigo.json só no arquivo editado; projeto que
// ainda não fechou as regras de código passa direto.

use crate::comum::*;
use crate::escrita_por_bash::arquivos_gravados_pela_ferramenta;
use crate::validador;
use serde_json::{json, Value};
use std::fs;

const NOME_WATCHDOG: &str = "vigia-teste";

pub fn rodar() -> i32 {
    let entrada_hook = ler_entrada_do_hook();
    if watchdogs_desligados() {
        return 0;
    }
    let diretorio_projeto = diretorio_do_projeto(&entrada_hook);
    // Edição pelo Bash (`sed -i`, heredoc, `python`) passa pela mesma conferência (ADR 0027).
    let arquivos_gravados = arquivos_gravados_pela_ferramenta(&entrada_hook, &diretorio_projeto);
    if arquivos_gravados.is_empty() {
        return 0;
    }
    let Ok(texto_config) = fs::read_to_string(diretorio_projeto.join("docs").join("regras-de-codigo.json")) else {
        return 0;
    };
    let Ok(config) = serde_json::from_str::<Value>(&texto_config) else {
        return 0;
    };
    let mut motivos: Vec<String> = Vec::new();
    for caminho_relativo in &arquivos_gravados {
        let violacoes = validador::conferir_arquivo_editado(&diretorio_projeto, &config, caminho_relativo);
        if violacoes.is_empty() {
            continue;
        }
        let achados: Vec<String> = violacoes
            .iter()
            .map(|violacao| match violacao.linha {
                Some(linha) => format!("- linha {linha}: {}", violacao.mensagem),
                None => format!("- {}", violacao.mensagem),
            })
            .collect();
        motivos.push(format!(
            "[watchdog {NOME_WATCHDOG}] \"{caminho_relativo}\" tem teste sem valor:\n{}\nTodo teste cobre um risco de docs/riscos.md e falha se o risco acontecer (regra 15, princípio P5; docs/agents/riscos.md). Corrija o teste ou apague-o. Risco novo só entra no registro com a origem aprovada pelo humano.",
            achados.join("\n")
        ));
    }
    if motivos.is_empty() {
        return 0;
    }
    let motivo = motivos.join("\n\n");
    if etapa_de_adocao(&diretorio_projeto) == "observacao" {
        emitir_contexto_adicional("PostToolUse", &motivo);
    } else {
        print!("{}", json!({ "decision": "block", "reason": motivo }));
    }
    0
}
