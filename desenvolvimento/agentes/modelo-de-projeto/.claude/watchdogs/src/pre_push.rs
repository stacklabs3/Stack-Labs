// Git pre-push: a main só recebe merge de PR no GitHub, nunca push direto. Depois, o CI local
// (ci.rs) contra a branch padrão do remoto (<remoto>/HEAD; sem ela, <remoto>/main): lint, typecheck ou validador vermelho barram o push; na etapa de
// observação, só avisam. O CI roda como comando pesado (trava.rs), sem a revisão por modelo, e não
// roda quando o mesmo conteúdo já foi aprovado contra a mesma base.
// Ativação (uma vez por clone): git config core.hooksPath .claude/githooks
// Escape humano para emergências: STACKLABS_PERMITIR_PUSH_MAIN=1 git push ...

use crate::ci;
use crate::comum::{base_do_remoto, etapa_de_adocao, executar_git};
use std::io::Read;
use std::path::PathBuf;

// O git passa o nome do remoto, ou a URL quando o push é por URL (`git push https://… <branch>`). Por URL,
// a base vira o remoto configurado com aquela URL, e sem nenhum, o `origin`: `https://…/HEAD` não existe.
pub fn remoto_do_push(raiz: &std::path::Path, nome_ou_url: &str) -> String {
    let remotos = executar_git(raiz, &["remote"]).unwrap_or_default();
    if remotos.lines().any(|remoto| remoto == nome_ou_url) {
        return nome_ou_url.to_string();
    }
    let sem_git = |url: &str| url.trim_end_matches('/').trim_end_matches(".git").to_string();
    remotos
        .lines()
        .find(|remoto| executar_git(raiz, &["remote", "get-url", remoto]).is_some_and(|url| sem_git(&url) == sem_git(nome_ou_url)))
        .unwrap_or("origin")
        .to_string()
}

pub fn rodar(argumentos: &[String]) -> i32 {
    let raiz = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let nome_do_remoto = remoto_do_push(&raiz, argumentos.first().map(String::as_str).unwrap_or("origin"));
    let nome_do_remoto = nome_do_remoto.as_str();
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
    let base = base_do_remoto(&raiz, nome_do_remoto);
    // O cache vale para o conteúdo do HEAD: só quando tudo o que sobe é exatamente o HEAD.
    let shas_enviados: Vec<&str> = linhas_do_push
        .lines()
        .filter_map(|linha| linha.split(' ').nth(1))
        .filter(|sha| !sha.is_empty() && sha.chars().any(|letra| letra != '0'))
        .collect();
    let sobe_so_o_head =
        executar_git(&raiz, &["rev-parse", "HEAD"]).is_some_and(|head| !shas_enviados.is_empty() && shas_enviados.iter().all(|sha| *sha == head));
    let aprovado_antes = ci::chave_do_conteudo(&raiz, &base).filter(|chave| sobe_so_o_head && ci::ja_aprovado(&raiz, chave));
    if let Some(chave) = aprovado_antes {
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
