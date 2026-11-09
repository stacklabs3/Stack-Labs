// Watchdog PostToolUse (Write | Edit | MultiEdit | Bash): o código que o agente acabou de escrever segue as
// regras de código que têm forma fixa (ast-grep, termo-evitado, todo-sem-issue) de
// docs/regras-de-codigo.json, só no arquivo editado e fora do P5, que é do vigia-teste. A violação
// volta na hora, com a regra de onde nasceu, em vez de esperar o validador no commit ou o ci no push.
// Projeto que ainda não fechou as regras de código passa direto.

use crate::comum::*;
use crate::escrita_por_bash::arquivos_gravados_pela_ferramenta;
use crate::validador;
use serde_json::{json, Value};
use std::fs;

const NOME_WATCHDOG: &str = "vigia-codigo";

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
        let violacoes = validador::conferir_codigo_editado(&diretorio_projeto, &config, caminho_relativo);
        if violacoes.is_empty() {
            continue;
        }
        let achados: Vec<String> = violacoes
            .iter()
            .map(|achado| {
                let onde = achado.violacao.linha.map(|linha| format!("linha {linha}: ")).unwrap_or_default();
                format!(
                    "- {} ({}) {onde}{}. Regra: {}",
                    achado.id, achado.principio, achado.violacao.mensagem, achado.texto
                )
            })
            .collect();
        motivos.push(format!(
            "[watchdog {NOME_WATCHDOG}] \"{caminho_relativo}\" quebra regra de código:\n{}\nCorrija agora, no arquivo, do jeito que docs/regras-de-codigo.md pede (regra 19; o porquê e o exemplo estão na regra). Se a mudança sair da linha do contrato da fatia, atualize o contrato por escrito.",
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
