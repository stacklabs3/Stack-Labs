// Git pre-push: a main só recebe merge de PR no GitHub, nunca push direto.
// Ativação (uma vez por clone): git config core.hooksPath .claude/githooks
// Escape humano para emergências: STACKLABS_PERMITIR_PUSH_MAIN=1 git push ...

use std::io::Read;

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
    0
}
