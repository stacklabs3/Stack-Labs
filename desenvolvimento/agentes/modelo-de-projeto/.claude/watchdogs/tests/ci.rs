// Testes do subcomando ci e da revisão por modelo: cada um roda o binário num repositório git de
// verdade, com um `claude` falso no PATH que devolve o JSON do `claude -p` e anota cada chamada.

use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const REGISTRO: &str = "| Risco | O que dá errado | Dano | Onde | Origem | Situação |\n|---|---|---|---|---|---|\n\
| RK1 | Frete grátis cobrado | cliente perdido | frete | #12 | |\n\
| RK2 | Cupom vencido aceito | margem perdida | cupom | #13 | |\n";

const TESTE_DO_FRETE: &str = "#[test]\nfn rk1_frete_gratis_acima_de_cem() {\n    assert_eq!(frete(150), 0);\n}\n";
const TESTE_DO_CUPOM: &str =
    "#[test]\nfn rk2_recusa_cupom_vencido() {\n    let cupom = Cupom::novo(\"X\", data(2026, 1, 1));\n    assert_eq!(cupom.codigo(), \"X\");\n}\n";

// O claude falso responde pelo nome do teste no prompt: o do cupom não exercita o risco.
const CLAUDE_QUE_RESPONDE: &str = r#"#!/bin/sh
echo chamada >> "$(dirname "$0")/chamadas"
case "$*" in
  *rk2_recusa_cupom_vencido*) echo '{"type":"result","is_error":false,"structured_output":{"exercita":false,"motivo":"só confere o código do cupom, não a recusa"}}' ;;
  *) echo '{"type":"result","is_error":false,"structured_output":{"exercita":true,"motivo":"afirma o frete zero"}}' ;;
esac
"#;

struct Projeto {
    raiz: PathBuf,
    pasta_do_claude: PathBuf,
}

impl Projeto {
    // Repositório com a main na base e a branch da issue com os dois testes novos.
    fn novo(regras: serde_json::Value) -> Self {
        // O relógio do macOS tem resolução de microssegundo: o contador separa testes paralelos.
        static PROXIMO: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let ordem = PROXIMO.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let sufixo = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let raiz = std::env::temp_dir().join(format!("ci-{}-{ordem}-{sufixo}", std::process::id()));
        let pasta_do_claude = raiz.join(".claude-falso");
        fs::create_dir_all(&pasta_do_claude).unwrap();
        let projeto = Projeto { raiz, pasta_do_claude };
        projeto.escrever(".gitignore", ".claude-falso/\n.claude/state/\n");
        projeto.escrever("docs/riscos.md", REGISTRO);
        projeto.escrever("docs/regras-de-codigo.json", &json!({ "versao": 1, "regras": regras }).to_string());
        projeto.escrever("docs/regras-de-codigo.md", &tabela_de_cobertura());
        for argumentos in [
            &["init", "-q", "-b", "main"][..],
            &["config", "user.email", "t@t"],
            &["config", "user.name", "t"],
            &["add", "-A"],
            &["commit", "-q", "-m", "base"],
            &["switch", "-q", "-c", "issue/7-cupom"],
        ] {
            projeto.git(argumentos);
        }
        projeto.escrever("tests/frete.rs", TESTE_DO_FRETE);
        projeto.escrever("tests/cupom.rs", TESTE_DO_CUPOM);
        projeto.git(&["add", "-A"]);
        projeto.git(&["commit", "-q", "-m", "testes"]);
        projeto.claude_falso(CLAUDE_QUE_RESPONDE);
        projeto
    }
    fn escrever(&self, caminho: &str, conteudo: &str) {
        let destino = self.raiz.join(caminho);
        fs::create_dir_all(destino.parent().unwrap()).unwrap();
        fs::write(destino, conteudo).unwrap();
    }
    fn git(&self, argumentos: &[&str]) {
        Command::new("git").args(argumentos).current_dir(&self.raiz).output().unwrap();
    }
    fn claude_falso(&self, script: &str) {
        let caminho = self.pasta_do_claude.join("claude");
        fs::write(&caminho, script).unwrap();
        Command::new("chmod").arg("+x").arg(&caminho).output().unwrap();
    }
    fn chamadas_ao_claude(&self) -> usize {
        fs::read_to_string(self.pasta_do_claude.join("chamadas"))
            .map(|texto| texto.lines().count())
            .unwrap_or(0)
    }
    // Roda o binário com o claude falso à frente do PATH; `com_claude` falso tira o claude do PATH.
    fn rodar(&self, argumentos: &[&str], entrada: &str, com_claude: bool) -> (i32, String) {
        let caminho = if com_claude {
            format!("{}:/usr/bin:/bin", self.pasta_do_claude.display())
        } else {
            "/usr/bin:/bin".to_string()
        };
        let mut processo = Command::new(env!("CARGO_BIN_EXE_watchdogs"))
            .args(argumentos)
            .current_dir(&self.raiz)
            .env("PATH", caminho)
            .env_remove("STACKLABS_WATCHDOGS")
            .env_remove("STACKLABS_PERMITIR_PUSH_MAIN")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        if let Err(erro) = processo.stdin.take().unwrap().write_all(entrada.as_bytes()) {
            assert_eq!(erro.kind(), std::io::ErrorKind::BrokenPipe, "falha ao escrever a entrada: {erro}");
        }
        let saida = processo.wait_with_output().unwrap();
        let tudo = format!("{}{}", String::from_utf8_lossy(&saida.stdout), String::from_utf8_lossy(&saida.stderr));
        (saida.status.code().unwrap_or(-1), tudo)
    }
    fn ci(&self) -> (i32, String) {
        self.rodar(&["ci", "--base", "main"], "", true)
    }
}

impl Drop for Projeto {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.raiz);
    }
}

// E1 e P5 nunca se afastam: E1 fica com o revisor (R3) e P5 com teste-cobre-risco (R1).
fn tabela_de_cobertura() -> String {
    let mut texto = String::from("| Princípio | Decisão | Onde ou por quê |\n|---|---|---|\n");
    for principio in watchdogs::validador::PRINCIPIOS {
        let linha = match principio {
            "E1" => "adotado | R3",
            "P5" => "adotado | R1",
            _ => "afastado | fora do escopo deste teste",
        };
        texto.push_str(&format!("| {principio} | {linha} |\n"));
    }
    texto
}

fn regras_com_revisao() -> serde_json::Value {
    json!([
        { "id": "R1", "principio": "P5", "verificacao": { "tipo": "teste-cobre-risco", "testes": ["tests/**"] } },
        { "id": "R2", "principio": "P5", "verificacao": { "tipo": "revisao-por-modelo", "modelo": "claude-haiku-5-5", "consultivo": true, "testes": ["tests/**"] } },
        { "id": "R3", "principio": "E1", "verificacao": { "tipo": "revisor" } }
    ])
}

// Risco: o teste que só cita o RK2 e não o exercita passa despercebido até o revisor.
#[test]
fn ci_mostra_o_teste_que_cita_o_risco_sem_exercitar() {
    let projeto = Projeto::novo(regras_com_revisao());
    let (codigo, saida) = projeto.ci();
    assert_eq!(codigo, 0, "a revisão é consultiva: {saida}");
    assert!(
        saida.contains("tests/cupom.rs:1") && saida.contains("RK2") && saida.contains("só confere o código do cupom, não a recusa"),
        "{saida}"
    );
    assert!(!saida.contains("tests/frete.rs"), "o teste que exercita o risco não é suspeito: {saida}");
    assert!(saida.contains("CI: APROVADO"), "{saida}");
}

// Risco: repetir o ci paga o modelo de novo, ou o veredito muda entre duas rodadas sobre o mesmo teste.
#[test]
fn segunda_rodada_usa_o_cache_sem_chamar_o_modelo() {
    let projeto = Projeto::novo(regras_com_revisao());
    let (_, primeira) = projeto.ci();
    assert_eq!(projeto.chamadas_ao_claude(), 2, "{primeira}");
    assert!(primeira.contains("2 chamada(s) ao modelo, 0 do cache"), "{primeira}");

    projeto.claude_falso("#!/bin/sh\necho chamada >> \"$(dirname \"$0\")/chamadas\"\necho '{\"is_error\":false,\"structured_output\":{\"exercita\":true,\"motivo\":\"mudou\"}}'\n");
    let (codigo, segunda) = projeto.ci();
    assert_eq!(codigo, 0);
    assert_eq!(projeto.chamadas_ao_claude(), 2, "a segunda rodada não chama o claude: {segunda}");
    assert!(segunda.contains("0 chamada(s) ao modelo, 2 do cache"), "{segunda}");
    assert!(segunda.contains("só confere o código do cupom, não a recusa"), "o veredito não muda: {segunda}");
}

// Risco: sem o claude, sem rede ou sem login, a revisão consultiva trava o push de quem não a tem.
#[test]
fn sem_claude_ou_com_claude_falhando_o_ci_avisa_e_aprova() {
    let projeto = Projeto::novo(regras_com_revisao());
    let (codigo, saida) = projeto.rodar(&["ci", "--base", "main"], "", false);
    assert_eq!(codigo, 0, "{saida}");
    assert!(saida.contains("o claude não está no PATH") && saida.contains("O ci segue sem ela"), "{saida}");
    assert!(saida.contains("CI: APROVADO"), "{saida}");

    projeto.claude_falso("#!/bin/sh\necho '{\"type\":\"result\",\"is_error\":true,\"result\":\"Failed to authenticate\"}'\n");
    let (codigo, saida) = projeto.ci();
    assert_eq!(codigo, 0, "{saida}");
    assert!(saida.contains("o claude respondeu com erro: Failed to authenticate"), "{saida}");
    assert!(saida.contains("2 teste(s) ficaram sem veredito") && saida.contains("CI: APROVADO"), "{saida}");

    projeto.claude_falso("#!/bin/sh\necho 'sem rede' >&2\nexit 1\n");
    let (codigo, saida) = projeto.ci();
    assert_eq!(codigo, 0, "{saida}");
    assert!(saida.contains("sem a resposta em JSON") && saida.contains("sem rede"), "{saida}");

    let (_, saida) = projeto.ci();
    assert!(saida.contains("0 do cache"), "falha não entra no cache: {saida}");
}

fn regras_com_lint_vermelho() -> serde_json::Value {
    let mut regras = regras_com_revisao();
    regras
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "R4", "principio": "E1", "verificacao": { "tipo": "comando", "executar": "echo 'lint: nome de uma letra'; exit 3" } }));
    regras
}

// Risco: código que quebra lint, typecheck ou regra do validador sobe para a PR sem ninguém ver.
#[test]
fn validador_vermelho_reprova_o_ci_e_o_pre_push() {
    let tabela_com_r4 = tabela_de_cobertura().replace("| E1 | adotado | R3 |", "| E1 | adotado | R3, R4 |");
    let projeto = Projeto::novo(regras_com_lint_vermelho());
    projeto.escrever("docs/regras-de-codigo.md", &tabela_com_r4);
    projeto.escrever("tests/soma.rs", "#[test]\nfn soma() {\n    assert_eq!(somar(1, 2), 3);\n}\n");

    let (codigo, saida) = projeto.ci();
    assert_eq!(codigo, 1, "{saida}");
    assert!(saida.contains("❌ R4 (E1) comando") && saida.contains("lint: nome de uma letra"), "{saida}");
    assert!(saida.contains("tests/soma.rs:1  o teste não cita o risco"), "{saida}");
    assert!(saida.trim_end().ends_with("CI: REPROVADO"), "{saida}");

    let (codigo, saida) = projeto.rodar(&["pre-push", "origin"], "refs/heads/issue/7-cupom abc refs/heads/issue/7-cupom def\n", true);
    assert_eq!(codigo, 1, "{saida}");
    assert!(saida.contains("[pre-push] BLOQUEADO") && saida.contains("CI: REPROVADO"), "{saida}");

    let (codigo, saida) = projeto.rodar(&["pre-push", "origin"], "refs/heads/issue/7-cupom abc refs/heads/main def\n", true);
    assert_eq!(codigo, 1);
    assert!(
        saida.contains("push direto para \"main\"") && !saida.contains("[ci]"),
        "a regra da main vem antes do ci: {saida}"
    );

    projeto.escrever(".claude/metodo.json", "{\"etapaDeAdocao\": \"observacao\"}");
    let (codigo, saida) = projeto.rodar(&["pre-push", "origin"], "refs/heads/issue/7-cupom abc refs/heads/issue/7-cupom def\n", true);
    assert_eq!(codigo, 0, "na observação o ci só avisa: {saida}");
    assert!(saida.contains("CI: REPROVADO") && saida.contains("etapa de observação"), "{saida}");
}

// Risco: o veredito sai com o nome errado ou vazio, e o revisor confere o teste errado.
#[test]
fn revisao_mostra_o_nome_do_testwidgets_na_mesma_linha_e_na_de_baixo() {
    let projeto = Projeto::novo(json!([
        { "id": "R1", "principio": "P5", "verificacao": { "tipo": "teste-cobre-risco", "testes": ["test/**"] } },
        { "id": "R2", "principio": "P5", "verificacao": { "tipo": "revisao-por-modelo", "modelo": "claude-haiku-5-5", "consultivo": true, "testes": ["test/**"] } },
        { "id": "R3", "principio": "E1", "verificacao": { "tipo": "revisor" } }
    ]));
    projeto.escrever(
        "test/frete_test.dart",
        "void main() {\n  testWidgets('RK1 mostra frete grátis', (tester) async {\n    expect(1, 1);\n  });\n\n  testWidgets(\n    'RK2 recusa cupom vencido em tela de checkout bem comprida',\n    (tester) async {\n      expect(2, 2);\n    },\n  );\n}\n",
    );
    projeto.claude_falso("#!/bin/sh\necho '{\"type\":\"result\",\"is_error\":false,\"structured_output\":{\"exercita\":false,\"motivo\":\"nao afirma o risco\"}}'\n");
    let (codigo, saida) = projeto.ci();
    assert_eq!(codigo, 0, "{saida}");
    assert!(saida.contains("test/frete_test.dart:2 RK1 mostra frete grátis (RK1)"), "{saida}");
    assert!(saida.contains("test/frete_test.dart:6 RK2 recusa cupom vencido em tela de checkout bem comprida (RK2)"), "{saida}");
}
