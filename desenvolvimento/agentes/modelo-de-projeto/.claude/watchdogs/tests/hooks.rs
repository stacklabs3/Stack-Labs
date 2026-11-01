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
        // O relógio do macOS tem resolução de microssegundo: o contador separa testes paralelos.
        static PROXIMO: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let ordem = PROXIMO.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let sufixo = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let raiz = std::env::temp_dir().join(format!("hooks-{}-{ordem}-{sufixo}", std::process::id()));
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
    fn leitura(&self, ferramenta: &str, parametros: Value) -> (i32, String, String) {
        self.rodar(
            &["hook", "guarda-contexto"],
            &json!({ "session_id": "s1", "tool_name": ferramenta, "tool_input": parametros }).to_string(),
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

#[test]
fn vigia_teste_cobra_risco_e_afirmacao_no_teste_editado() {
    let projeto = Projeto::novo("issue/7-x");
    let editar = |caminho: &str| {
        projeto.rodar(
            &["hook", "vigia-teste"],
            &json!({ "session_id": "s1", "tool_input": { "file_path": projeto.raiz.join(caminho).to_string_lossy() } }).to_string(),
            &[],
        )
    };
    let escrever = |caminho: &str, conteudo: &str| {
        let destino = projeto.raiz.join(caminho);
        fs::create_dir_all(destino.parent().unwrap()).unwrap();
        fs::write(destino, conteudo).unwrap();
    };
    escrever("src/cupom.test.ts", "it(\"soma\", () => { expect(somar(1, 2)).toBe(3); });\n");
    assert_eq!(
        editar("src/cupom.test.ts"),
        (0, String::new(), String::new()),
        "sem regras de código decididas, passa"
    );

    escrever(
        "docs/regras-de-codigo.json",
        &json!({ "versao": 1, "regras": [{ "id": "R1", "principio": "P5", "verificacao": { "tipo": "teste-cobre-risco" } }] }).to_string(),
    );
    escrever(
        "docs/riscos.md",
        "| RK1 | Cupom vencido aceito | margem | cupom | #12 | |\n| RK2 | Outro | x | y | #13 | |\n",
    );
    let (codigo, saida, _) = editar("src/cupom.test.ts");
    assert_eq!(codigo, 0);
    let resposta: Value = serde_json::from_str(&saida).unwrap();
    assert_eq!(resposta["decision"], "block");
    let motivo = resposta["reason"].as_str().unwrap();
    assert!(motivo.contains("linha 1: o teste não cita o risco"), "{motivo}");
    assert!(!motivo.contains("RK2"), "risco sem teste é do validador, não do hook: {motivo}");

    escrever(
        "src/cupom.test.ts",
        "// RK1\nit(\"recusa cupom vencido\", () => { expect(aplicar(vencido)).toBe(false); });\n",
    );
    assert_eq!(editar("src/cupom.test.ts").1, "");
    assert_eq!(editar("src/cupom.ts").1, "", "código que não é teste passa");

    escrever(".claude/metodo.json", "{\"etapaDeAdocao\": \"observacao\"}");
    escrever("src/cupom.test.ts", "it(\"soma\", () => { expect(somar(1, 2)).toBe(3); });\n");
    let (_, saida, _) = editar("src/cupom.test.ts");
    assert!(
        saida.contains("additionalContext") && !saida.contains("\"decision\""),
        "na observação só avisa: {saida}"
    );
}

fn mensagem_do_modelo(id_mensagem: &str, modelo: &str, entrada: u64, leitura_cache: u64, saida: u64) -> String {
    json!({ "type": "assistant", "message": { "id": id_mensagem, "model": modelo, "usage": {
        "input_tokens": entrada, "cache_creation_input_tokens": 0, "cache_read_input_tokens": leitura_cache, "output_tokens": saida } } })
    .to_string()
}

#[test]
fn custo_soma_cada_mensagem_uma_vez_na_issue_ativa_e_avisa_no_orcamento() {
    let projeto = Projeto::novo("issue/7-x");
    projeto.estado(json!({ "issuesAtivas": { "principal": { "numero": 7, "titulo": "Cupom" } } }));
    fs::create_dir_all(projeto.raiz.join(".claude")).unwrap();
    fs::write(projeto.raiz.join(".claude/metodo.json"), json!({ "orcamentoPorIssue": 2_000_000 }).to_string()).unwrap();
    let transcript = projeto.raiz.join("t.jsonl");
    // A mesma mensagem aparece em duas linhas: conta uma vez (1M + 0,1 × 1M + 5 × 100k = 1,6M).
    let linha_repetida = mensagem_do_modelo("m1", "claude-opus-5-5", 1_000_000, 1_000_000, 100_000);
    fs::write(&transcript, format!("{linha_repetida}\n{linha_repetida}\n")).unwrap();
    let parada = json!({ "session_id": "s1", "transcript_path": transcript }).to_string();
    let (codigo, saida, _) = projeto.rodar(&["hook", "vigia-custo"], &parada, &[]);
    assert_eq!(codigo, 0);
    assert!(saida.is_empty(), "abaixo do orçamento não avisa: {saida}");
    let custo: Value = serde_json::from_str(&fs::read_to_string(projeto.raiz.join(".claude/state/custos/issue-7.json")).unwrap()).unwrap();
    assert_eq!(custo["tokensEquivalentes"], 1_600_000);

    // O turno seguinte só soma o que é novo, e o primeiro a cruzar o orçamento avisa o humano.
    let mut texto_transcript = fs::read_to_string(&transcript).unwrap();
    texto_transcript.push_str(&format!("{}\n", mensagem_do_modelo("m2", "claude-haiku-5-5", 500_000, 0, 0)));
    fs::write(&transcript, texto_transcript).unwrap();
    let (_, saida, _) = projeto.rodar(&["hook", "vigia-custo"], &parada, &[]);
    assert!(saida.contains("passou do orçamento"), "{saida}");
    let custo: Value = serde_json::from_str(&fs::read_to_string(projeto.raiz.join(".claude/state/custos/issue-7.json")).unwrap()).unwrap();
    assert_eq!(custo["tokensEquivalentes"], 2_100_000);
    assert_eq!(custo["porModelo"]["claude-haiku-5-5"]["entrada"], 500_000);

    let prompt = json!({ "session_id": "s1", "transcript_path": transcript, "prompt": "segue" }).to_string();
    assert!(projeto.rodar(&["hook", "vigia-prompt"], &prompt, &[]).1.contains("já custou 2.1M"));
    let (_, relatorio, _) = projeto.rodar(&["custo", "7"], "", &[]);
    assert!(relatorio.contains("Issue #7 Cupom: 2.1M") && relatorio.contains("105%"), "{relatorio}");
}

#[test]
fn custo_sem_issue_ativa_nao_grava_nada() {
    let projeto = Projeto::novo("main");
    let transcript = projeto.raiz.join("t.jsonl");
    fs::write(&transcript, format!("{}\n", mensagem_do_modelo("m1", "claude-opus-5-5", 9_000_000, 0, 0))).unwrap();
    let parada = json!({ "session_id": "s1", "transcript_path": transcript }).to_string();
    assert_eq!(projeto.rodar(&["hook", "vigia-custo"], &parada, &[]), (0, String::new(), String::new()));
    assert!(!projeto.raiz.join(".claude/state/custos").exists());
}

#[test]
fn saida_grande_pede_filtro_e_pequena_passa_calada() {
    let projeto = Projeto::novo("main");
    let saida_ferramenta = |tamanho: usize| json!({ "session_id": "s1", "tool_name": "Bash", "tool_response": { "stdout": "x".repeat(tamanho) } }).to_string();
    assert!(projeto
        .rodar(&["hook", "vigia-saida"], &saida_ferramenta(25_000), &[])
        .1
        .contains("filtre a saída com grep"));
    assert!(projeto.rodar(&["hook", "vigia-saida"], &saida_ferramenta(500), &[]).1.is_empty());
}

#[test]
fn subagente_de_busca_sem_modelo_recebe_lembrete() {
    let projeto = Projeto::novo("main");
    let lancamento = |entrada_agente: Value| json!({ "session_id": "s1", "tool_name": "Agent", "tool_input": entrada_agente }).to_string();
    assert!(projeto
        .rodar(&["hook", "vigia-agente"], &lancamento(json!({ "subagent_type": "Explore" })), &[])
        .1
        .contains("haiku"));
    assert!(projeto
        .rodar(
            &["hook", "vigia-agente"],
            &lancamento(json!({ "subagent_type": "Explore", "model": "haiku" })),
            &[]
        )
        .1
        .is_empty());
    assert!(projeto
        .rodar(&["hook", "vigia-agente"], &lancamento(json!({ "subagent_type": "general-purpose" })), &[])
        .1
        .is_empty());
}

#[test]
fn ajustes_do_projeto_liberam_caminho_e_branch_e_mudam_a_smart_zone() {
    let projeto = Projeto::novo("spike/teste");
    fs::create_dir_all(projeto.raiz.join(".claude")).unwrap();
    fs::write(
        projeto.raiz.join(".claude/metodo.json"),
        json!({ "ajustes": { "caminhosLivres": ["infra/"], "branchesDescartaveis": ["spike/"], "contexto": { "aviso": 100_000, "duro": 200_000 } } })
            .to_string(),
    )
    .unwrap();
    assert_eq!(projeto.edicao("src/a.ts").0, 0, "branch spike/ é descartável no projeto");
    assert!(projeto.edicao(".claude/watchdogs/src/main.rs").2.contains("infraestrutura dos watchdogs"));

    let transcript = projeto.raiz.join("t.jsonl");
    fs::write(&transcript, format!("{}\n", json!({ "message": { "usage": { "input_tokens": 250_000 } } }))).unwrap();
    let entrada = json!({ "session_id": "s1", "transcript_path": transcript, "prompt": "x" }).to_string();
    assert!(projeto
        .rodar(&["hook", "vigia-prompt"], &entrada, &[])
        .1
        .contains("passou da smart zone (200k)"));
}

#[test]
fn caminho_livre_do_projeto_dispensa_issue_ativa() {
    let projeto = Projeto::novo("issue/7-x");
    fs::create_dir_all(projeto.raiz.join(".claude")).unwrap();
    fs::write(
        projeto.raiz.join(".claude/metodo.json"),
        json!({ "ajustes": { "caminhosLivres": ["infra/"] } }).to_string(),
    )
    .unwrap();
    assert_eq!(projeto.edicao("infra/main.tf").0, 0);
    assert_eq!(projeto.edicao("src/a.ts").0, 2);
}

fn projeto_com_codigo() -> Projeto {
    let projeto = Projeto::novo("issue/1-x");
    fs::create_dir_all(projeto.raiz.join("src")).unwrap();
    fs::write(projeto.raiz.join("src/cupom.rs"), "pub fn aplicar_cupom() {}\n").unwrap();
    fs::write(projeto.raiz.join("src/pedido.rs"), "pub fn finalizar_pedido() {}\n").unwrap();
    projeto
}

const CONSULTA_DO_CUPOM: &str = "sh .claude/watchdogs/rodar contexto aplicar_cupom";

#[test]
fn leitura_de_codigo_sem_consulta_bloqueia_e_mostra_o_comando() {
    let projeto = projeto_com_codigo();
    for (ferramenta, parametros) in [
        ("Read", json!({ "file_path": "src/cupom.rs" })),
        ("Grep", json!({ "pattern": "cupom" })),
        ("Grep", json!({ "pattern": "cupom", "path": "src" })),
        ("Glob", json!({ "pattern": "**/*.rs" })),
    ] {
        let (codigo_saida, _, motivo) = projeto.leitura(ferramenta, parametros);
        assert_eq!(codigo_saida, 2, "{ferramenta} passou sem consulta");
        assert!(motivo.contains("rodar contexto"), "sem o comando: {motivo}");
    }
}

#[test]
fn consulta_libera_a_leitura_e_avisa_uma_vez_fora_dos_citados() {
    let projeto = projeto_com_codigo();
    assert_eq!(projeto.bash(CONSULTA_DO_CUPOM).0, 0);
    let (codigo_citado, aviso_citado, _) = projeto.leitura("Read", json!({ "file_path": "src/cupom.rs" }));
    assert_eq!((codigo_citado, aviso_citado.as_str()), (0, ""));
    let (codigo_fora, aviso_fora, _) = projeto.leitura("Read", json!({ "file_path": "src/pedido.rs" }));
    assert_eq!(codigo_fora, 0);
    assert!(aviso_fora.contains("fora do que a consulta citou"), "sem aviso: {aviso_fora}");
    assert_eq!(projeto.leitura("Read", json!({ "file_path": "src/pedido.rs" })).1, "");
}

#[test]
fn resposta_vazia_libera_a_leitura_sem_aviso() {
    let projeto = projeto_com_codigo();
    assert_eq!(projeto.bash("sh .claude/watchdogs/rodar contexto reembolso_parcial").0, 0);
    let (codigo_saida, aviso, _) = projeto.leitura("Read", json!({ "file_path": "src/cupom.rs" }));
    assert_eq!((codigo_saida, aviso.as_str()), (0, ""));
}

#[test]
fn caminhos_livres_passam_sem_consulta() {
    let projeto = projeto_com_codigo();
    fs::create_dir_all(projeto.raiz.join("docs")).unwrap();
    for parametros in [
        json!({ "file_path": "AGENTS.md" }),
        json!({ "file_path": "CLAUDE.md" }),
        json!({ "file_path": "docs/agents/watchdogs.md" }),
        json!({ "file_path": ".scratch/nota.ts" }),
        json!({ "file_path": ".claude/skills/pr/SKILL.md" }),
        json!({ "file_path": "package.json" }),
        json!({ "file_path": "vite.config.ts" }),
        json!({ "file_path": "/tmp/fora-do-projeto.rs" }),
    ] {
        assert_eq!(projeto.leitura("Read", parametros.clone()).0, 0, "barrou {parametros}");
    }
    assert_eq!(projeto.leitura("Glob", json!({ "pattern": "docs/**/*" })).0, 0);
    assert_eq!(projeto.leitura("Grep", json!({ "pattern": "x", "path": "docs" })).0, 0);
}

#[test]
fn subagente_precisa_da_propria_consulta() {
    let projeto = projeto_com_codigo();
    assert_eq!(projeto.bash(CONSULTA_DO_CUPOM).0, 0);
    let entrada = json!({ "session_id": "s1", "agent_id": "a1", "tool_name": "Read", "tool_input": { "file_path": "src/cupom.rs" } });
    assert_eq!(projeto.rodar(&["hook", "guarda-contexto"], &entrada.to_string(), &[]).0, 2);
}

#[test]
fn bash_trata_cat_sed_e_grep_em_codigo_como_leitura() {
    let projeto = projeto_com_codigo();
    for comando in [
        "cat src/cupom.rs",
        "sed -n 1,5p src/cupom.rs",
        "grep -n cupom src/cupom.rs",
        "grep -rn cupom",
        "git status && cat src/*.rs",
    ] {
        let (codigo_saida, _, motivo) = projeto.bash(comando);
        assert_eq!(codigo_saida, 2, "passou sem consulta: {comando}");
        assert!(motivo.contains("rodar contexto"), "sem o comando: {motivo}");
    }
    for comando in ["cat AGENTS.md", "git log | grep cupom", "grep -n cupom docs/a.md", "cat > src/novo.rs <<FIM"] {
        assert_eq!(projeto.bash(comando).0, 0, "barrou: {comando}");
    }
    assert_eq!(projeto.bash(&format!("{CONSULTA_DO_CUPOM} && cat src/cupom.rs")).0, 0);
    assert_eq!(projeto.bash("cat src/pedido.rs").0, 0);
}

#[test]
fn etapa_de_observacao_nao_exige_consulta() {
    let projeto = projeto_com_codigo();
    fs::create_dir_all(projeto.raiz.join(".claude")).unwrap();
    fs::write(projeto.raiz.join(".claude/metodo.json"), json!({ "etapaDeAdocao": "observacao" }).to_string()).unwrap();
    assert_eq!(projeto.leitura("Read", json!({ "file_path": "src/cupom.rs" })).0, 0);
    assert_eq!(projeto.bash("cat src/cupom.rs").0, 0);
}
