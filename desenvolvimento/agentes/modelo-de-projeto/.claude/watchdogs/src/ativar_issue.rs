// Ativa uma issue para esta sessão. Quem valida e grava é o watchdog guarda-bash, que intercepta
// este comando ANTES de ele rodar: se você está lendo a saída abaixo, a issue passou.
// Uso: .claude/watchdogs/rodar ativar-issue <número>

use crate::comum::{texto, COMANDO_ATIVAR_ISSUE};
use serde_json::Value;
use std::process::Command;

pub fn rodar(argumentos: &[String]) -> i32 {
    let numero_issue = argumentos.first().map(|numero| numero.trim_start_matches('#').to_string()).unwrap_or_default();
    if numero_issue.is_empty() || !numero_issue.chars().all(|caractere| caractere.is_ascii_digit()) {
        eprintln!("uso: {COMANDO_ATIVAR_ISSUE} <número da issue>");
        return 1;
    }
    let saida = Command::new("gh").args(["issue", "view", &numero_issue, "--json", "title,labels,url"]).output();
    let issue_ativada: Value = match saida {
        Ok(saida) if saida.status.success() => serde_json::from_slice(&saida.stdout).unwrap_or(Value::Null),
        Ok(saida) => {
            eprintln!("gh issue view {numero_issue} falhou: {}", String::from_utf8_lossy(&saida.stderr).trim());
            return 1;
        }
        Err(erro) => {
            eprintln!("não consegui rodar o gh: {erro}");
            return 1;
        }
    };
    let labels: Vec<&str> = issue_ativada["labels"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|label| texto(&label["name"]))
        .collect();
    println!("Issue #{numero_issue} ativa nesta sessão.");
    println!("{}", texto(&issue_ativada["title"]));
    println!("Labels: {}", labels.join(", "));
    println!("{}", texto(&issue_ativada["url"]));
    println!("Próximo passo: git switch -c issue/{numero_issue}-<slug> e então /implement com a issue (e o spec pai).");
    0
}
