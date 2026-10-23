// Watchdog PostToolUse (Write | Edit | MultiEdit): o teste que o agente acabou de escrever cobre um
// risco de docs/riscos.md e não tem os sinais de teste sem valor (sem afirmação, afirmação trivial,
// desligado). Roda as regras do P5 de docs/regras-de-codigo.json só no arquivo editado; projeto que
// ainda não fechou as regras de código passa direto.

use crate::comum::*;
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
    let caminho_arquivo = texto(&entrada_hook["tool_input"]["file_path"]);
    let Some(caminho_relativo) = caminho_relativo_ao_projeto(&diretorio_projeto, caminho_arquivo) else {
        return 0;
    };
    let Ok(texto_config) = fs::read_to_string(diretorio_projeto.join("docs").join("regras-de-codigo.json")) else {
        return 0;
    };
    let Ok(config) = serde_json::from_str::<Value>(&texto_config) else {
        return 0;
    };
    let violacoes = validador::conferir_arquivo_editado(&diretorio_projeto, &config, &caminho_relativo);
    if violacoes.is_empty() {
        return 0;
    }
    let achados: Vec<String> = violacoes
        .iter()
        .map(|violacao| match violacao.linha {
            Some(linha) => format!("- linha {linha}: {}", violacao.mensagem),
            None => format!("- {}", violacao.mensagem),
        })
        .collect();
    let motivo = format!(
        "[watchdog {NOME_WATCHDOG}] \"{caminho_relativo}\" tem teste sem valor:\n{}\nTodo teste cobre um risco de docs/riscos.md e falha se o risco acontecer (regra 15, princípio P5; docs/agents/riscos.md). Corrija o teste ou apague-o. Risco novo só entra no registro com a origem aprovada pelo humano.",
        achados.join("\n")
    );
    if etapa_de_adocao(&diretorio_projeto) == "observacao" {
        emitir_contexto_adicional("PostToolUse", &motivo);
    } else {
        print!("{}", json!({ "decision": "block", "reason": motivo }));
    }
    0
}
