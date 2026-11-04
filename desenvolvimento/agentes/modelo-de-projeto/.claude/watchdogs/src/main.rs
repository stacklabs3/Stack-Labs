// Ponto de entrada único: .claude/watchdogs/rodar <subcomando> [argumentos].

use watchdogs::{ativar_issue, ci, contexto, custo, hooks, pre_push, trava, validador};

const USO: &str = "uso: rodar hook <nome> | ativar-issue <n> | pre-push <remoto> <url> | validar [opções] | ci [--base <ref>] | custo [<n>] | contexto <consulta> | pesado <comando> | --version";

fn main() {
    let argumentos: Vec<String> = std::env::args().skip(1).collect();
    let resto = argumentos.get(1..).unwrap_or_default();
    let codigo_de_saida = match argumentos.first().map(String::as_str) {
        Some("hook") => hooks::rodar(resto.first().map(String::as_str).unwrap_or("")),
        Some("ativar-issue") => ativar_issue::rodar(resto),
        Some("pre-push") => pre_push::rodar(resto),
        Some("validar") => validador::rodar(resto),
        Some("ci") => ci::rodar(resto),
        Some("custo") => custo::rodar(resto),
        Some("contexto") => contexto::rodar(resto),
        Some("pesado") => trava::rodar(resto),
        Some("--version") => {
            println!("watchdogs {}", env!("CARGO_PKG_VERSION"));
            0
        }
        _ => {
            eprintln!("{USO}");
            2
        }
    };
    std::process::exit(codigo_de_saida);
}
