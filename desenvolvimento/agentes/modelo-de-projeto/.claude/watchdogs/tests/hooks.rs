// Testes dos hooks: cada um roda o binário como o Claude Code roda, com a entrada JSON no stdin,
// num repositório de teste e com um `gh` falso no PATH que devolve respostas preparadas.

use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

struct Projeto {
    raiz: PathBuf,
    pasta_do_gh: PathBuf,
}

impl Projeto {
    fn novo(branch: &str) -> Self {
        let sufixo = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let raiz = std::env::temp_dir().join(format!("hooks-{}-{sufixo}", std::process::id()));
        let pasta_do_gh = raiz.join(".gh-falso");
        fs::create_dir_all(&pasta_do_gh).unwrap();
        let projeto = Projeto { raiz, pasta_do_gh };
        for argumentos in [
            &["init", "-q", "-b", "main"][..],
            &["config", "user.email", "t@t"],
            &["config", "user.name", "t"],
            &["remote", "add", "origin", "git@github.com:org/proj.git"],
            &["commit", "-q", "--allow-empty", "-m", "base"],
        ] {
            projeto.git(argumentos);
        }
        if branch != "main" {
            projeto.git(&["switch", "-q", "-c", branch]);
        }
        projeto.respostas_do_gh(&[]);
        projeto
    }
    fn git(&self, argumentos: &[&str]) {
        Command::new("git").args(argumentos).current_dir(&self.raiz).output().unwrap();
    }
    // O gh falso responde pelo texto exato dos argumentos; o resto falha como gh sem rede.
    fn respostas_do_gh(&self, respostas: &[(String, String)]) {
        let mut script = String::from("#!/bin/sh\ncase \"$*\" in\n");
        for (argumentos, saida) in respostas {
            script.push_str(&format!("  '{}') cat <<'FIM'\n{saida}\nFIM\n  ;;\n", argumentos.replace('\'', "'\\''")));
        }
        script.push_str("  *) echo \"gh falso: sem resposta para $*\" >&2; exit 1 ;;\nesac\n");
        let caminho = self.pasta_do_gh.join("gh");
        fs::write(&caminho, script).unwrap();
        Command::new("chmod").arg("+x").arg(&caminho).output().unwrap();
    }
    fn estado(&self, valor: Value) {
        let pasta = self.raiz.join(".claude/state/sessoes");
        fs::create_dir_all(&pasta).unwrap();
        fs::write(pasta.join("s1.json"), valor.to_string()).unwrap();
    }
    fn ler_estado(&self) -> Value {
        serde_json::from_str(&fs::read_to_string(self.raiz.join(".claude/state/sessoes/s1.json")).unwrap()).unwrap()
    }
    fn rodar(&self, argumentos: &[&str], entrada: &str, variaveis: &[(&str, &str)]) -> (i32, String, String) {
        let caminho = format!("{}:{}", self.pasta_do_gh.display(), std::env::var("PATH").unwrap());
        let mut comando = Command::new(env!("CARGO_BIN_EXE_watchdogs"));
        comando
            .args(argumentos)
            .current_dir(&self.raiz)
            .env("PATH", caminho)
            .env("CLAUDE_PROJECT_DIR", &self.raiz)
            .env_remove("STACKLABS_WATCHDOGS")
            .env_remove("STACKLABS_PERMITIR_PUSH_MAIN");
        for (nome, valor) in variaveis {
            comando.env(nome, valor);
        }
        let mut processo = comando.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
        // O hook pode decidir e sair antes de ler a entrada (o pre-push lê só o que precisa): o cano
        // fechado não é falha do teste.
        if let Err(erro) = processo.stdin.take().unwrap().write_all(entrada.as_bytes()) {
            assert_eq!(erro.kind(), std::io::ErrorKind::BrokenPipe, "falha ao escrever a entrada do hook: {erro}");
        }
        let saida = processo.wait_with_output().unwrap();
        (
            saida.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&saida.stdout).into(),
            String::from_utf8_lossy(&saida.stderr).into(),
        )
    }
    fn bash(&self, comando: &str) -> (i32, String, String) {
        self.rodar(
            &["hook", "guarda-bash"],
            &json!({ "session_id": "s1", "tool_input": { "command": comando } }).to_string(),
            &[],
        )
    }
    fn edicao(&self, caminho: &str) -> (i32, String, String) {
        self.rodar(
            &["hook", "guarda-edicao"],
            &json!({ "session_id": "s1", "tool_input": { "file_path": caminho } }).to_string(),
            &[],
        )
    }
}

impl Drop for Projeto {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.raiz);
    }
}

fn issue(numero: i64, estado: &str, labels: &[&str], responsaveis: &[&str], comentarios: i64, corpo: &str) -> (String, String) {
    let labels: Vec<Value> = labels.iter().map(|nome| json!({ "name": nome })).collect();
    let responsaveis: Vec<Value> = responsaveis.iter().map(|login| json!({ "login": login })).collect();
    let corpo_json = json!({ "number": numero, "title": format!("Issue {numero}"), "state": estado, "body": corpo, "labels": labels, "assignees": responsaveis, "comments": comentarios });
    (format!("api repos/org/proj/issues/{numero}"), corpo_json.to_string())
}

fn mapas(estado: &str) -> Vec<(String, String)> {
    ["prototipo", "arquitetura", "fases", "regras-de-codigo", "infra"]
        .iter()
        .enumerate()
        .map(|(indice, tipo)| {
            let argumentos = format!("issue list --label mapa-inicial:{tipo} --label wayfinder:map --state all --json number,title,state");
            (
                argumentos,
                json!([{ "number": 100 + indice, "title": format!("Mapa {tipo}"), "state": estado }]).to_string(),
            )
        })
        .collect()
}

fn comentarios(numero: i64, corpos: &[&str]) -> (String, String) {
    (
        format!("api --paginate repos/org/proj/issues/{numero}/comments --jq [.[].body]"),
        json!(corpos).to_string(),
    )
}

#[test]
fn bash_barra_o_que_desliga_os_watchdogs() {
    let projeto = Projeto::novo("main");
    for comando in [
        "rm .claude/watchdogs/rodar",
        "sed -i s/a/b/ .claude/settings.json",
        "git commit --no-verify -m x",
        "git config core.hooksPath /tmp",
        "git push --force",
        "STACKLABS_PERMITIR_PUSH_MAIN=1 git push",
        "sh rodar --atualizar-watchdogs",
    ] {
        let (codigo, _, erro) = projeto.bash(comando);
        assert_eq!(codigo, 2, "{comando} deveria ser barrado");
        assert!(erro.contains("[watchdog guarda-bash] BLOQUEADO"), "{erro}");
    }
    for comando in [
        "ls -la",
        "git config --get core.hooksPath",
        "cat .claude/settings.json > /dev/null",
        ".claude/watchdogs/rodar validar --base origin/main",
    ] {
        assert_eq!(projeto.bash(comando).0, 0, "{comando} deveria passar");
    }
}

#[test]
fn ativar_issue_pronta_grava_o_estado() {
    let projeto = Projeto::novo("main");
    let mut respostas = vec![issue(7, "open", &["ready-for-agent"], &[], 0, "")];
    respostas.extend(mapas("CLOSED"));
    projeto.respostas_do_gh(&respostas);
    assert_eq!(projeto.bash(".claude/watchdogs/rodar ativar-issue 7").0, 0);
    let estado = projeto.ler_estado();
    assert_eq!(estado["issuesAtivas"]["principal"]["numero"], 7);
    assert!(estado["issuesAtivas"]["principal"]["ativadaEm"].as_str().unwrap().ends_with('Z'));
}

#[test]
fn ativar_issue_barra_sem_label_com_mapa_aberto_e_segunda_issue() {
    let projeto = Projeto::novo("main");
    projeto.respostas_do_gh(&[issue(7, "open", &["bug"], &[], 0, "")]);
    assert!(projeto.bash("rodar ativar-issue 7").2.contains("não tem ready-for-agent nem ready-for-human"));

    let mut respostas = vec![issue(7, "open", &["ready-for-agent"], &[], 0, "")];
    respostas.extend(mapas("OPEN"));
    projeto.respostas_do_gh(&respostas);
    assert!(projeto.bash("rodar ativar-issue 7").2.contains("ainda estão abertos"));

    projeto.estado(json!({ "issuesAtivas": { "principal": { "numero": 3, "titulo": "Outra" } }, "ticketsWayfinderResolvidos": [] }));
    assert!(projeto.bash("rodar ativar-issue 7").2.contains("já trabalha na issue #3"));
    assert!(projeto.bash("rodar ativar-issue 7 && rodar ativar-issue 8").2.contains("mais de uma issue"));
}

#[test]
fn fechar_ticket_exige_claim_comentario_e_fontes() {
    let projeto = Projeto::novo("main");
    projeto.respostas_do_gh(&[issue(9, "open", &["wayfinder:grilling"], &[], 0, "")]);
    assert!(projeto.bash("gh issue close 9 -c x").2.contains("nunca foi reivindicado"));

    projeto.respostas_do_gh(&[issue(9, "open", &["wayfinder:grilling"], &["eu"], 0, ""), comentarios(9, &["sem fonte"])]);
    assert!(projeto.bash("gh issue close 9").2.contains("sem comentário de resolução"));
    assert!(projeto.bash("gh issue close 9 --comment 'decidido'").2.contains("sem \"## Fontes\""));
    assert_eq!(projeto.bash("gh issue close 9 --comment 'ok\n## Fontes\n- https://exemplo.dev'").0, 0);
    assert_eq!(projeto.ler_estado()["ticketsWayfinderResolvidos"][0]["numero"], 9);
}

#[test]
fn um_ticket_de_decisao_por_sessao() {
    let projeto = Projeto::novo("main");
    projeto.estado(json!({ "issuesAtivas": {}, "ticketsWayfinderResolvidos": [{ "numero": 5, "titulo": "Antes", "tipo": "grilling" }] }));
    projeto.respostas_do_gh(&[issue(9, "open", &["wayfinder:task"], &["eu"], 1, ""), comentarios(9, &["## Fontes\nhttps://a"])]);
    assert!(projeto.bash("gh issue close 9").2.contains("já resolveu \"Antes\" (#5)"));
    projeto.respostas_do_gh(&[
        issue(9, "open", &["wayfinder:research"], &["eu"], 1, ""),
        comentarios(9, &["## Fontes\nhttps://a"]),
    ]);
    assert_eq!(projeto.bash("gh issue close 9").0, 0, "pesquisa é a exceção");
}

#[test]
fn commit_na_main_e_barrado_e_na_branch_passa() {
    assert!(Projeto::novo("main").bash("git commit -m x").2.contains("commit direto na \"main\""));
    assert_eq!(Projeto::novo("issue/7-x").bash("git commit -m x").0, 0);
}

#[test]
fn observacao_so_protege_watchdogs_e_main() {
    let projeto = Projeto::novo("main");
    fs::create_dir_all(projeto.raiz.join(".claude")).unwrap();
    fs::write(projeto.raiz.join(".claude/metodo.json"), r#"{"etapaDeAdocao":"observacao"}"#).unwrap();
    assert_eq!(projeto.bash("rodar ativar-issue 7").0, 0);
    assert_eq!(projeto.bash("git push --no-verify").0, 2);
}

#[test]
fn edicao_protege_watchdogs_e_exige_issue_ativa() {
    let projeto = Projeto::novo("issue/7-x");
    assert!(projeto.edicao(".claude/watchdogs/src/main.rs").2.contains("infraestrutura dos watchdogs"));
    assert_eq!(projeto.edicao("docs/regras-de-codigo.json").0, 0);
    assert_eq!(projeto.edicao("HANDOFF-app.md").0, 0);
    assert_eq!(projeto.edicao("/tmp/fora-do-repo.md").0, 0);
    let (codigo, _, erro) = projeto.edicao("src/a.ts");
    assert_eq!(codigo, 2);
    assert!(erro.contains("Ative com: .claude/watchdogs/rodar ativar-issue <número>"), "{erro}");
    projeto.estado(json!({ "issuesAtivas": { "principal": { "numero": 7, "titulo": "F" } }, "ticketsWayfinderResolvidos": [] }));
    assert_eq!(projeto.edicao("src/a.ts").0, 0);
}

#[test]
fn smart_zone_avisa_e_depois_barra() {
    let projeto = Projeto::novo("issue/7-x");
    let transcript = projeto.raiz.join("t.jsonl");
    fs::write(&transcript, format!("{}\n", json!({ "message": { "usage": { "input_tokens": 560_000 } } }))).unwrap();
    let entrada = json!({ "session_id": "s1", "transcript_path": transcript, "prompt": "x" }).to_string();
    assert!(projeto
        .rodar(&["hook", "vigia-prompt"], &entrada, &[])
        .1
        .contains("passou da smart zone (550k)"));
    let edicao = json!({ "session_id": "s1", "transcript_path": transcript, "tool_input": { "file_path": "docs/a.md" } }).to_string();
    assert!(projeto.rodar(&["hook", "guarda-edicao"], &edicao, &[]).2.contains("acima da smart zone"));
}

#[test]
fn grilling_injeta_a_regra_de_linguagem() {
    let projeto = Projeto::novo("main");
    let (_, saida, _) = projeto.rodar(
        &["hook", "vigia-skill"],
        &json!({ "session_id": "s1", "tool_input": { "skill": "mattpocock:wayfinder" } }).to_string(),
        &[],
    );
    assert!(saida.contains("linguagem-grilling"));
    let (_, saida, _) = projeto.rodar(
        &["hook", "vigia-prompt"],
        &json!({ "session_id": "s1", "prompt": "a resposta" }).to_string(),
        &[],
    );
    assert!(saida.contains("linguagem-grilling"), "o grilling ficou marcado na sessão");
    assert_eq!(
        projeto
            .rodar(
                &["hook", "vigia-skill"],
                &json!({ "session_id": "s2", "tool_input": { "skill": "tdd" } }).to_string(),
                &[]
            )
            .1,
        ""
    );
}

#[test]
fn parada_cobra_o_ponteiro_no_mapa() {
    let projeto = Projeto::novo("main");
    let pendente = json!({ "issuesAtivas": {}, "ticketsWayfinderResolvidos": [{ "numero": 9, "titulo": "Ticket 9", "tipo": "grilling", "ponteiroNoMapaVerificado": false }] });
    projeto.estado(pendente.clone());
    projeto.respostas_do_gh(&[
        issue(9, "closed", &["wayfinder:grilling"], &[], 0, "Part of #100"),
        issue(100, "open", &["wayfinder:map"], &[], 0, "## Decisions so far\n\n## Not yet\n#9"),
    ]);
    let (_, saida, _) = projeto.rodar(&["hook", "vigia-parada"], r#"{"session_id":"s1"}"#, &[]);
    assert!(
        saida.contains("\"decision\":\"block\"") && saida.contains("- [Ticket 9](link): <gist"),
        "{saida}"
    );

    projeto.respostas_do_gh(&[
        issue(9, "closed", &["wayfinder:grilling"], &[], 0, "Part of #100"),
        issue(
            100,
            "open",
            &["wayfinder:map"],
            &[],
            0,
            "## Decisions so far\n- [Ticket 9](https://github.com/org/proj/issues/9): x",
        ),
    ]);
    assert_eq!(projeto.rodar(&["hook", "vigia-parada"], r#"{"session_id":"s1"}"#, &[]).1, "");
    assert_eq!(projeto.ler_estado()["ticketsWayfinderResolvidos"][0]["ponteiroNoMapaVerificado"], true);
}

#[test]
fn inicio_mostra_setup_e_mapas() {
    let projeto = Projeto::novo("main");
    let (_, saida, _) = projeto.rodar(&["hook", "sessao-inicio"], r#"{"session_id":"s1"}"#, &[]);
    let contexto: Value = serde_json::from_str(&saida).unwrap();
    let texto = contexto["hookSpecificOutput"]["additionalContext"].as_str().unwrap();
    assert!(texto.contains("❌ Setup incompleto") && texto.contains("falha ao consultar o GitHub") && texto.contains("Branch atual: main"));
    let (_, saida, _) = projeto.rodar(&["hook", "sessao-inicio"], r#"{"session_id":"s1"}"#, &[("STACKLABS_WATCHDOGS", "desligado")]);
    assert!(saida.contains("Watchdogs DESLIGADOS"));
}

#[test]
fn pre_push_protege_a_main() {
    let projeto = Projeto::novo("main");
    let (codigo, _, erro) = projeto.rodar(&["pre-push", "origin"], "refs/heads/x abc refs/heads/main def\n", &[]);
    assert_eq!(codigo, 1);
    assert!(erro.contains("push direto para \"main\""));
    assert_eq!(projeto.rodar(&["pre-push", "origin"], "refs/heads/x abc refs/heads/issue/1-x def\n", &[]).0, 0);
    assert_eq!(
        projeto
            .rodar(
                &["pre-push", "origin"],
                "refs/heads/x abc refs/heads/master def\n",
                &[("STACKLABS_PERMITIR_PUSH_MAIN", "1")]
            )
            .0,
        0
    );
}
