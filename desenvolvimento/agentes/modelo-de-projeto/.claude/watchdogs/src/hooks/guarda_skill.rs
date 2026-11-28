// Watchdog PreToolUse (Skill): barra a skill do projeto que uma cópia pessoal de mesmo nome e
// conteúdo diferente encobre, porque a sessão rodaria a pessoal (ADR 0028). Na observação, só avisa.

use crate::comum::*;
use crate::homonimas::{como_resolver, skill_esta_encoberta};

pub fn rodar() -> i32 {
    let entrada_hook = ler_entrada_do_hook();
    if watchdogs_desligados() {
        return 0;
    }
    let diretorio_projeto = diretorio_do_projeto(&entrada_hook);
    let Some(nome_da_skill) = skill_esta_encoberta(&diretorio_projeto, texto(&entrada_hook["tool_input"]["skill"])) else {
        return 0;
    };
    let motivo = format!(
        "a skill /{nome_da_skill} rodaria a cópia pessoal, não a do projeto. {}",
        como_resolver(std::slice::from_ref(&nome_da_skill))
    );
    if etapa_de_adocao(&diretorio_projeto) == "observacao" {
        emitir_contexto_adicional("PreToolUse", &format!("⚠️ [watchdog guarda-skill] {motivo}"));
        return 0;
    }
    bloquear_ferramenta("guarda-skill", &motivo);
}
