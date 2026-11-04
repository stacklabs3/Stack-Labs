// Git pre-push: a main só recebe merge de PR no GitHub, nunca push direto. Depois, o CI local
// (ci.rs) contra <remoto>/main: lint, typecheck ou validador vermelho barram o push; na etapa de
// observação, só avisam. O CI roda como comando pesado (trava.rs), sem a revisão por modelo, e não
// roda quando o mesmo conteúdo já foi aprovado contra a mesma base.
// Ativação (uma vez por clone): git config core.hooksPath .claude/githooks
// Escape humano para emergências: STACKLABS_PERMITIR_PUSH_MAIN=1 git push ...

use crate::ci;
use crate::comum::etapa_de_adocao;
use std::io::Read;
use std::path::PathBuf;

pub fn rodar(argumentos: &[String]) -> i32 {
    let nome_do_remoto = argumentos.first().map(String::as_str).unwrap_or("origin");
    if std::env::var("STACKLABS_PERMITIR_PUSH_MAIN").as_deref() == Ok("1") {
        eprintln!("[pre-push] STACKLABS_PERMITIR_PUSH_MAIN=1: push para a main liberado por um humano.");
        return 0;
    }
    // Cada linha do stdin: <ref local> <sha local> <ref remota> <sha remoto>
    let mut linhas_do_push = String::new();
    let _ = std::io::stdin().read_to_string(&mut linhas_do_push);
    for linha_do_push in linhas_do_push.lines().filter(|linha| !linha.is_empty()) {
        let ref_remota = linha_do_push.split(' ').nth(2).unwrap_or("");
        if let Some(branch_destino) = ref_remota.strip_prefix("refs/heads/").filter(|branch| *branch == "main" || *branch == "master") {
            eprintln!(
                "[pre-push] BLOQUEADO — push direto para \"{branch_destino}\" em \"{nome_do_remoto}\".\nA {branch_destino} só recebe merge de PR. Envie a branch da issue e abra o PR:\n  git push -u {nome_do_remoto} <sua-branch> && gh pr create\n(Regra 17 do AGENTS.md · docs/agents/watchdogs.md)"
            );
            return 1;
        }
    }
    let raiz = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let base = format!("{nome_do_remoto}/main");
    if let Some(chave) = ci::chave_do_conteudo(&raiz, &base).filter(|chave| ci::ja_aprovado(&raiz, chave)) {
        eprintln!(
            "[pre-push] este conteúdo já passou no CI local contra {base} (chave {}); push sem rodar de novo.",
            &chave[..12]
        );
        return 0;
    }
    let argumentos_do_ci = ["--base".to_string(), base.clone(), "--sem-revisao".to_string()];
    if ci::rodar_como_pesado(&raiz, &argumentos_do_ci, crate::trava::Saida::NoErro) == 0 {
        return 0;
    }
    if etapa_de_adocao(&raiz) == "observacao" {
        eprintln!("[pre-push] o CI local reprovou; na etapa de observação isso só avisa. (docs/agents/adocao.md)");
        return 0;
    }
    eprintln!(
        "[pre-push] BLOQUEADO — o CI local reprovou (lint, typecheck ou validador, acima). Corrija e suba de novo.\nPara ver de novo: sh .claude/watchdogs/rodar ci --base {base} (docs/agents/validador.md)"
    );
    1
}
