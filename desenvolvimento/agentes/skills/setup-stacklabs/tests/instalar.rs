//! Casos do instalador numa pasta temporária, com um Stack-Labs e um mattpocock/skills de mentira.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static CONTADOR: AtomicUsize = AtomicUsize::new(0);

struct Cenario {
    raiz: PathBuf,
    stacklabs: PathBuf,
    matt: PathBuf,
    projeto: PathBuf,
}

impl Drop for Cenario {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.raiz);
    }
}

fn escrever(caminho: &Path, conteudo: &str) {
    fs::create_dir_all(caminho.parent().unwrap()).unwrap();
    fs::write(caminho, conteudo).unwrap();
}

fn tornar_executavel(caminho: &Path) {
    fs::set_permissions(caminho, fs::Permissions::from_mode(0o755)).unwrap();
}

/// `rodar` dos watchdogs de mentira: sai com o código pedido, sem compilar nada.
fn lancador_de_mentira(codigo: i32) -> String {
    format!("#!/bin/sh\n[ \"$1\" = --version ] || exit 9\necho 'erro de compilação' >&2\nexit {codigo}\n")
}

impl Cenario {
    fn novo() -> Self {
        let numero = CONTADOR.fetch_add(1, Ordering::SeqCst);
        let raiz =
            std::env::temp_dir().join(format!("teste-instalar-{}-{numero}", std::process::id()));
        let _ = fs::remove_dir_all(&raiz);
        let stacklabs = raiz.join("stacklabs");
        let matt = raiz.join("matt");
        let projeto = raiz.join("projeto");

        let modelo = stacklabs.join("desenvolvimento/agentes/modelo-de-projeto");
        escrever(&modelo.join("AGENTS.md"), "# AGENTS do modelo\n");
        escrever(&modelo.join("CLAUDE.md"), "@AGENTS.md\n");
        escrever(&modelo.join(".gitignore"), "target/\n");
        escrever(&modelo.join(".claude/settings.json"), "{}\n");
        escrever(&modelo.join("docs/agents/watchdogs.md"), "# watchdogs\n");
        escrever(
            &modelo.join(".claude/githooks/pre-push"),
            "#!/bin/sh\nexit 0\n",
        );
        tornar_executavel(&modelo.join(".claude/githooks/pre-push"));
        escrever(
            &modelo.join(".claude/watchdogs/Cargo.toml"),
            "[package]\nname = \"watchdogs\"\n",
        );
        escrever(
            &modelo.join(".claude/watchdogs/src/main.rs"),
            "fn main() {}\n",
        );
        escrever(
            &modelo.join(".claude/watchdogs/rodar"),
            &lancador_de_mentira(0),
        );
        escrever(
            &modelo.join(".claude/watchdogs/target/release/watchdogs"),
            "binário",
        );

        let skills = stacklabs.join("desenvolvimento/agentes/skills");
        escrever(&skills.join("setup-stacklabs/SKILL.md"), "setup\n");
        escrever(
            &skills.join("setup-stacklabs/target/release/instalar"),
            "binário",
        );
        escrever(&skills.join("auditar-metodo/SKILL.md"), "auditar\n");
        escrever(&skills.join("recomendada/nova-sessao/SKILL.md"), "nova\n");
        escrever(
            &skills.join("recomendada/grill-me/SKILL.md"),
            "cópia da casa da skill do Matt\n",
        );

        escrever(
            &matt.join(".claude-plugin/plugin.json"),
            r#"{ "version": "1.2.3", "skills": ["./skills/grill-me", "./skills/tdd", "./skills/wayfinder", "./skills/implement-spec"] }"#,
        );
        escrever(&matt.join("skills/grill-me/SKILL.md"), "grill-me\n");
        escrever(&matt.join("skills/tdd/SKILL.md"), "tdd\n");
        escrever(
            &matt.join("skills/wayfinder/SKILL.md"),
            "---\nname: wayfinder\ndisable-model-invocation: true\n---\ncorpo\n",
        );
        escrever(
            &matt.join("skills/implement-spec/SKILL.md"),
            "---\nname: implement-spec\ndisable-model-invocation: true\n---\n\nIntro.\n\nCommunication to and from subagents should be sparse. Pointers.\n\n## Steps\n",
        );

        fs::create_dir_all(&projeto).unwrap();
        let iniciar = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&projeto)
            .status()
            .unwrap();
        assert!(iniciar.success());
        Cenario {
            raiz,
            stacklabs,
            matt,
            projeto,
        }
    }

    fn modelo(&self) -> PathBuf {
        self.stacklabs
            .join("desenvolvimento/agentes/modelo-de-projeto")
    }

    fn rodar(&self, argumentos: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_instalar"))
            .args(["--destino", self.projeto.to_str().unwrap(), "--sem-labels"])
            .args(argumentos)
            .env("STACKLABS_ORIGEM", &self.stacklabs)
            .env("MATT_ORIGEM", &self.matt)
            .output()
            .unwrap()
    }

    fn relatorio(&self, argumentos: &[&str]) -> String {
        let saida = self.rodar(argumentos);
        assert!(
            saida.status.success(),
            "falhou: {}",
            String::from_utf8_lossy(&saida.stderr)
        );
        String::from_utf8(saida.stdout).unwrap()
    }

    fn hooks_path(&self) -> String {
        let saida = Command::new("git")
            .args(["config", "--get", "core.hooksPath"])
            .current_dir(&self.projeto)
            .output()
            .unwrap();
        String::from_utf8_lossy(&saida.stdout).trim().to_string()
    }

    fn com_watchdogs_antigos(&self) {
        escrever(
            &self.projeto.join(".claude/hooks/guarda-bash.mjs"),
            "// antigo\n",
        );
        escrever(
            &self.projeto.join(".claude/validador/validar.mjs"),
            "// antigo\n",
        );
    }
}

fn linhas_que_comecam(relatorio: &str, prefixo: &str) -> Vec<String> {
    relatorio
        .lines()
        .filter(|linha| linha.starts_with(prefixo))
        .map(str::to_string)
        .collect()
}

#[test]
fn instalacao_copia_o_modelo_as_skills_e_ativa_o_pre_push() {
    let cenario = Cenario::novo();
    let relatorio = cenario.relatorio(&[]);

    assert!(relatorio.starts_with(&format!(
        "\nMétodo StackLabs em {}\n",
        cenario.projeto.display()
    )));
    assert!(relatorio.contains(
        "Fontes: Stack-Labs sem versão (desconhecido) · mattpocock/skills desconhecido (v1.2.3)"
    ));
    for arquivo in [
        "AGENTS.md",
        "CLAUDE.md",
        ".gitignore",
        ".claude/settings.json",
        ".claude/githooks/pre-push",
        ".claude/watchdogs/rodar",
    ] {
        assert!(
            relatorio.contains(&format!("COPIADO   {arquivo}\n")),
            "{arquivo} não copiado:\n{relatorio}"
        );
        assert!(cenario.projeto.join(arquivo).is_file());
    }
    let modo = fs::metadata(cenario.projeto.join(".claude/githooks/pre-push"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(modo & 0o111, 0o111, "o pre-push perdeu o modo executável");
    let data_do_modelo = fs::metadata(cenario.modelo().join("AGENTS.md"))
        .unwrap()
        .modified()
        .unwrap();
    let data_copiada = fs::metadata(cenario.projeto.join("AGENTS.md"))
        .unwrap()
        .modified()
        .unwrap();
    assert_eq!(
        data_do_modelo, data_copiada,
        "a cópia do modelo não manteve a data"
    );

    assert!(relatorio.contains("COMPILADO .claude/watchdogs\n"));
    assert_eq!(
        linhas_que_comecam(&relatorio, "SKILL"),
        [
            "SKILL     grill-me (Matt)",
            "SKILL     tdd (Matt)",
            "SKILL     wayfinder (Matt)",
            "SKILL     implement-spec (Matt)",
            "SKILL     nova-sessao (casa)",
            "SKILL     auditar-metodo (casa)",
            "SKILL     setup-stacklabs (casa)",
        ]
    );
    assert_eq!(
        fs::read_to_string(cenario.projeto.join(".claude/skills/grill-me/SKILL.md")).unwrap(),
        "grill-me\n"
    );
    assert!(relatorio.contains("PRE-PUSH  ativado (core.hooksPath)"));
    assert_eq!(cenario.hooks_path(), ".claude/githooks");
    assert!(
        cenario.projeto.join("docs/adr").is_dir() && cenario.projeto.join("docs/research").is_dir()
    );

    let origem: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(cenario.projeto.join(".claude/skills/ORIGEM.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(origem["matt"]["versao"], "1.2.3");
    assert_eq!(
        origem["skillsDoMatt"],
        serde_json::json!(["grill-me", "tdd", "wayfinder", "implement-spec"])
    );
    assert_eq!(
        origem["skillsDaCasa"],
        serde_json::json!(["nova-sessao", "auditar-metodo", "setup-stacklabs"])
    );
    let instalado_em = origem["instaladoEm"].as_str().unwrap();
    assert_eq!(instalado_em.len(), 24);
    assert!(instalado_em.ends_with('Z') && instalado_em.as_bytes()[10] == b'T');

    let segunda_vez = cenario.relatorio(&[]);
    assert!(
        segunda_vez.ends_with("Nada a fazer: o projeto já está em dia.\n"),
        "{segunda_vez}"
    );
}

#[test]
fn pastas_target_nunca_sao_copiadas() {
    let cenario = Cenario::novo();
    let relatorio = cenario.relatorio(&[]);
    assert!(!relatorio.contains("target"), "{relatorio}");
    assert!(!cenario.projeto.join(".claude/watchdogs/target").exists());
    assert!(cenario
        .projeto
        .join(".claude/skills/setup-stacklabs/SKILL.md")
        .exists());
    assert!(!cenario
        .projeto
        .join(".claude/skills/setup-stacklabs/target")
        .exists());

    // Uma compilação no projeto não faz a skill parecer diferente da fonte.
    escrever(
        &cenario
            .projeto
            .join(".claude/skills/setup-stacklabs/target/release/instalar"),
        "outro",
    );
    assert!(cenario.relatorio(&[]).contains("Nada a fazer"));
}

#[test]
fn arquivo_proprio_do_projeto_vira_stacklabs_novo() {
    let cenario = Cenario::novo();
    escrever(&cenario.projeto.join("AGENTS.md"), "# AGENTS do projeto\n");
    let relatorio = cenario.relatorio(&[]);
    assert!(relatorio.contains("MESCLAR   AGENTS.md (modelo novo em AGENTS.md.stacklabs-novo)\n"));
    assert!(!relatorio.contains("COPIADO   AGENTS.md"));
    assert_eq!(
        fs::read_to_string(cenario.projeto.join("AGENTS.md")).unwrap(),
        "# AGENTS do projeto\n"
    );
    assert_eq!(
        fs::read_to_string(cenario.projeto.join("AGENTS.md.stacklabs-novo")).unwrap(),
        "# AGENTS do modelo\n"
    );
}

#[test]
fn watchdogs_antigos_saem_e_os_novos_compilam_sem_flag_nenhuma() {
    let cenario = Cenario::novo();
    cenario.com_watchdogs_antigos();
    let relatorio = cenario.relatorio(&[]);
    assert!(relatorio
        .contains("REMOVIDA  .claude/hooks (watchdogs antigos: agora em .claude/watchdogs)\n"));
    assert!(relatorio
        .contains("REMOVIDA  .claude/validador (watchdogs antigos: agora em .claude/watchdogs)\n"));
    assert!(relatorio.contains("COPIADO   .claude/watchdogs/rodar\n"));
    assert!(relatorio.contains("COMPILADO .claude/watchdogs\n"));
    assert!(relatorio.contains("PRE-PUSH  ativado (core.hooksPath)"));
    assert!(!relatorio.contains("PULADO"));
    assert!(!cenario.projeto.join(".claude/hooks").exists());
    assert!(!cenario.projeto.join(".claude/validador").exists());
    assert!(cenario.projeto.join(".claude/watchdogs/rodar").exists());
}

#[test]
fn atualizacao_troca_watchdogs_existentes_e_tira_o_que_saiu_da_fonte() {
    let cenario = Cenario::novo();
    cenario.relatorio(&[]);
    escrever(&cenario.projeto.join(".claude/watchdogs/src/sobra.rs"), "// saiu da fonte\n");
    escrever(&cenario.modelo().join(".claude/watchdogs/src/main.rs"), "fn main() { /* novo */ }\n");
    let relatorio = cenario.relatorio(&[]);
    assert!(relatorio.contains("COPIADO   .claude/watchdogs/src/main.rs\n"), "{relatorio}");
    assert!(relatorio.contains("REMOVIDO  .claude/watchdogs/src/sobra.rs (saiu da fonte)\n"), "{relatorio}");
    assert!(!cenario.projeto.join(".claude/watchdogs/src/sobra.rs").exists());
    assert!(cenario.relatorio(&[]).contains("Nada a fazer"));
}

#[test]
fn fonte_que_nao_compila_nao_toca_nos_watchdogs_do_projeto() {
    let cenario = Cenario::novo();
    cenario.relatorio(&[]);
    let lancador_instalado = fs::read_to_string(cenario.projeto.join(".claude/watchdogs/rodar")).unwrap();
    escrever(&cenario.modelo().join(".claude/watchdogs/rodar"), &lancador_de_mentira(2));
    escrever(&cenario.modelo().join(".claude/settings.json"), "{ \"hooks\": { \"Stop\": [] } }\n");
    let relatorio = cenario.relatorio(&[]);
    assert!(relatorio.contains("os watchdogs do projeto ficaram como estavam\n"), "{relatorio}");
    assert_eq!(fs::read_to_string(cenario.projeto.join(".claude/watchdogs/rodar")).unwrap(), lancador_instalado);
    assert_eq!(fs::read_to_string(cenario.projeto.join(".claude/settings.json")).unwrap(), "{}\n");
}

#[test]
fn settings_e_gitignore_do_projeto_sao_mesclados_sem_stacklabs_novo() {
    let cenario = Cenario::novo();
    escrever(
        &cenario.modelo().join(".claude/settings.json"),
        r#"{ "hooks": { "Stop": [{ "hooks": [{ "type": "command", "command": "sh .claude/watchdogs/rodar hook novo" }] }] } }"#,
    );
    escrever(
        &cenario.projeto.join(".claude/settings.json"),
        r#"{ "permissions": { "allow": ["Bash(ls)"] }, "hooks": { "Stop": [{ "hooks": [{ "type": "command", "command": "sh .claude/watchdogs/rodar hook velho" }, { "type": "command", "command": "sh meu-hook" }] }] } }"#,
    );
    escrever(&cenario.projeto.join(".claude/settings.json.stacklabs-novo"), "{}\n");
    escrever(&cenario.projeto.join(".gitignore"), "node_modules/");
    let relatorio = cenario.relatorio(&[]);
    assert!(relatorio.contains("MESCLADO  .claude/settings.json"), "{relatorio}");
    assert!(relatorio.contains("MESCLADO  .gitignore (1 linhas novas)"), "{relatorio}");
    let settings: serde_json::Value = serde_json::from_str(&fs::read_to_string(cenario.projeto.join(".claude/settings.json")).unwrap()).unwrap();
    assert_eq!(settings["permissions"]["allow"][0], "Bash(ls)");
    let grupos = settings["hooks"]["Stop"].as_array().unwrap();
    assert_eq!(grupos.len(), 2);
    assert_eq!(grupos[0]["hooks"][0]["command"], "sh .claude/watchdogs/rodar hook novo");
    assert_eq!(grupos[1]["hooks"].as_array().unwrap().len(), 1);
    assert_eq!(grupos[1]["hooks"][0]["command"], "sh meu-hook");
    assert!(!cenario.projeto.join(".claude/settings.json.stacklabs-novo").exists());
    assert_eq!(fs::read_to_string(cenario.projeto.join(".gitignore")).unwrap(), "node_modules/\ntarget/\n");
    assert!(cenario.relatorio(&[]).contains("Nada a fazer"));
}

#[test]
fn versao_anterior_a_instalada_sai_com_erro() {
    let cenario = Cenario::novo();
    escrever(&cenario.stacklabs.join("desenvolvimento/VERSAO"), "1.0.0\n");
    escrever(
        &cenario.projeto.join(".claude/skills/ORIGEM.json"),
        r#"{ "stacklabs": { "versao": "2.0.0" }, "skillsDoMatt": [], "skillsDaCasa": [] }"#,
    );
    let saida = cenario.rodar(&["--versao", "1.0.0"]);
    assert!(!saida.status.success());
    assert!(String::from_utf8_lossy(&saida.stderr).contains("--permitir-regressao"));
    assert!(cenario.rodar(&["--versao", "1.0.0", "--permitir-regressao"]).status.success());
}

#[test]
fn conferir_skill_atualiza_a_pasta_e_mantem_a_compilacao() {
    let cenario = Cenario::novo();
    escrever(&cenario.stacklabs.join("desenvolvimento/VERSAO"), "9.9.9\n");
    escrever(&cenario.stacklabs.join("desenvolvimento/agentes/skills/adotar-metodo-stack-labs/SKILL.md"), "adotar\n");
    let pasta_das_skills = cenario.raiz.join("skills-da-maquina");
    escrever(&pasta_das_skills.join("setup-stacklabs/SKILL.md"), "velha\n");
    escrever(&pasta_das_skills.join("setup-stacklabs/arquivo-que-saiu.md"), "velho\n");
    escrever(&pasta_das_skills.join("setup-stacklabs/target/release/instalar"), "binário\n");
    let conferir = || {
        let saida = Command::new(env!("CARGO_BIN_EXE_instalar"))
            .args(["--conferir-skill", "--pasta-das-skills", pasta_das_skills.to_str().unwrap()])
            .env("STACKLABS_ORIGEM", &cenario.stacklabs)
            .output()
            .unwrap();
        assert!(saida.status.success());
        String::from_utf8(saida.stdout).unwrap()
    };
    let primeira = conferir();
    assert!(primeira.contains("SKILL ATUALIZADA setup-stacklabs (método v9.9.9)"), "{primeira}");
    assert!(primeira.contains("SKILL ATUALIZADA auditar-metodo"), "{primeira}");
    assert_eq!(fs::read_to_string(pasta_das_skills.join("setup-stacklabs/SKILL.md")).unwrap(), "setup\n");
    assert!(!pasta_das_skills.join("setup-stacklabs/arquivo-que-saiu.md").exists());
    assert!(pasta_das_skills.join("setup-stacklabs/target/release/instalar").exists());
    assert!(conferir().contains("SKILL EM DIA (método v9.9.9)"));
}

#[test]
fn conferir_skill_sem_acesso_a_fonte_avisa_e_nao_falha() {
    let cenario = Cenario::novo();
    let pasta_das_skills = cenario.raiz.join("skills-da-maquina");
    fs::create_dir_all(&pasta_das_skills).unwrap();
    let saida = Command::new(env!("CARGO_BIN_EXE_instalar"))
        .args(["--conferir-skill", "--pasta-das-skills", pasta_das_skills.to_str().unwrap()])
        .env_remove("STACKLABS_ORIGEM")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_ALLOW_PROTOCOL", "file")
        .output()
        .unwrap();
    assert!(saida.status.success());
    assert!(String::from_utf8_lossy(&saida.stdout).contains("SKILL NÃO CONFERIDA"));
}

#[test]
fn verificar_aprova_a_instalacao_e_reprova_o_que_falta() {
    let cenario = Cenario::novo();
    cenario.relatorio(&[]);
    let verificar = || {
        Command::new(env!("CARGO_BIN_EXE_instalar"))
            .args(["--verificar", "--destino", cenario.projeto.to_str().unwrap()])
            .output()
            .unwrap()
    };
    let aprovada = verificar();
    assert!(aprovada.status.success(), "{}", String::from_utf8_lossy(&aprovada.stdout));
    assert!(String::from_utf8_lossy(&aprovada.stdout).contains("VERIFICADO"));
    fs::remove_file(cenario.projeto.join(".claude/skills/tdd/SKILL.md")).unwrap();
    let reprovada = verificar();
    assert!(!reprovada.status.success());
    assert!(String::from_utf8_lossy(&reprovada.stdout).contains("REPROVADO .claude/skills/tdd não tem SKILL.md"));
}

/// Projeto com o método instalado na `main` e uma atualização commitada em `chore/atualizar-metodo`,
/// com um `gh` de mentira que anota as chamadas em `gh.log`.
fn cenario_de_pr_de_atualizacao() -> Cenario {
    let cenario = Cenario::novo();
    let git_no_projeto = |argumentos: &[&str]| {
        let saida = Command::new("git")
            .args(["-c", "user.name=teste", "-c", "user.email=teste@teste"])
            .args(argumentos)
            .current_dir(&cenario.projeto)
            .output()
            .unwrap();
        assert!(saida.status.success(), "git {argumentos:?}: {}", String::from_utf8_lossy(&saida.stderr));
    };
    let remoto = cenario.raiz.join("remoto.git");
    let criar_remoto = Command::new("git").args(["init", "-q", "--bare", remoto.to_str().unwrap()]).status().unwrap();
    assert!(criar_remoto.success());
    git_no_projeto(&["checkout", "-q", "-b", "main"]);
    git_no_projeto(&["remote", "add", "origin", remoto.to_str().unwrap()]);
    cenario.relatorio(&[]);
    escrever(&cenario.projeto.join("AGENTS.md"), "# AGENTS\n\n## Regras deste projeto\n\nRP1\n");
    git_no_projeto(&["add", "-A"]);
    git_no_projeto(&["commit", "-q", "-m", "base"]);
    git_no_projeto(&["push", "-q", "origin", "main"]);
    git_no_projeto(&["checkout", "-q", "-b", "chore/atualizar-metodo"]);
    escrever(&cenario.modelo().join(".claude/watchdogs/src/main.rs"), "fn main() { /* novo */ }\n");
    cenario.relatorio(&[]);
    git_no_projeto(&["add", "-A"]);
    git_no_projeto(&["commit", "-q", "-m", "atualiza"]);
    git_no_projeto(&["fetch", "-q", "origin"]);

    let pasta_do_gh = cenario.raiz.join("bin");
    escrever(
        &pasta_do_gh.join("gh"),
        r#"#!/bin/sh
echo "$*" >> "$GH_LOG"
case "$1 $2" in
  "pr view") printf '{"state":"OPEN","headRefName":"chore/atualizar-metodo","headRefOid":"%s","baseRefName":"main"}' "$(git rev-parse HEAD)" ;;
  "pr diff") git diff --name-only origin/main...HEAD ;;
  "pr checks") [ -z "$GH_CI_VERMELHO" ] || { echo 'ci vermelho' >&2; exit 1; } ;;
esac
"#,
    );
    tornar_executavel(&pasta_do_gh.join("gh"));
    cenario
}

fn concluir_pr(cenario: &Cenario, ci_vermelho: bool) -> (Option<i32>, String, String) {
    let log_do_gh = cenario.raiz.join("gh.log");
    let caminho = format!("{}:{}", cenario.raiz.join("bin").display(), std::env::var("PATH").unwrap());
    let mut comando = Command::new(env!("CARGO_BIN_EXE_instalar"));
    comando
        .args(["--concluir-pr", "7", "--destino", cenario.projeto.to_str().unwrap()])
        .current_dir(&cenario.projeto)
        .env("PATH", caminho)
        .env("GH_LOG", &log_do_gh)
        .env("STACKLABS_SEM_ESPERA", "1")
        .env("STACKLABS_ORIGEM", &cenario.stacklabs)
        .env("MATT_ORIGEM", &cenario.matt);
    if ci_vermelho {
        comando.env("GH_CI_VERMELHO", "1");
    }
    let saida = comando.output().unwrap();
    (
        saida.status.code(),
        String::from_utf8_lossy(&saida.stdout).into_owned(),
        fs::read_to_string(&log_do_gh).unwrap_or_default(),
    )
}

#[test]
fn concluir_pr_identica_a_fonte_rotula_e_faz_o_merge() {
    let cenario = cenario_de_pr_de_atualizacao();
    let (codigo, saida, chamadas) = concluir_pr(&cenario, false);
    assert_eq!(codigo, Some(0), "{saida}");
    assert!(saida.contains("MESCLADA  PR #7"), "{saida}");
    assert!(chamadas.contains("pr edit 7 --add-label altera-watchdogs"), "{chamadas}");
    assert!(chamadas.contains("pr merge 7 --merge"), "{chamadas}");
}

fn commitar_tudo(cenario: &Cenario) {
    for argumentos in [vec!["add", "-A"], vec!["-c", "user.name=teste", "-c", "user.email=teste@teste", "commit", "-q", "-m", "mexe"]] {
        assert!(Command::new("git").args(&argumentos).current_dir(&cenario.projeto).status().unwrap().success());
    }
}

fn assert_ficou_para_humano(cenario: &Cenario, motivo: &str) {
    let (codigo, saida, chamadas) = concluir_pr(cenario, false);
    assert_eq!(codigo, Some(3), "{saida}");
    assert!(saida.contains("FICA PARA UM HUMANO"), "{saida}");
    assert!(saida.contains(motivo), "{saida}");
    assert!(!chamadas.contains("--add-label") && !chamadas.contains("pr merge"), "{chamadas}");
}

#[test]
fn concluir_pr_com_watchdog_alterado_fica_para_humano() {
    let cenario = cenario_de_pr_de_atualizacao();
    escrever(&cenario.projeto.join(".claude/watchdogs/src/main.rs"), "fn main() { /* desligado */ }\n");
    commitar_tudo(&cenario);
    assert_ficou_para_humano(&cenario, "difere da fonte oficial: COPIADO   .claude/watchdogs/src/main.rs");
}

#[test]
fn concluir_pr_com_arquivo_de_fora_fica_para_humano() {
    let cenario = cenario_de_pr_de_atualizacao();
    escrever(&cenario.projeto.join("src/app.rs"), "// código de produção\n");
    commitar_tudo(&cenario);
    assert_ficou_para_humano(&cenario, "src/app.rs não vem do instalador");
}

#[test]
fn concluir_pr_com_secao_do_projeto_alterada_fica_para_humano() {
    let cenario = cenario_de_pr_de_atualizacao();
    escrever(&cenario.projeto.join("AGENTS.md"), "# AGENTS do modelo\n\n\n## Regras deste projeto\n\nRP1 alterada\n");
    commitar_tudo(&cenario);
    assert_ficou_para_humano(&cenario, "a seção ## Regras deste projeto");
}

#[test]
fn concluir_pr_com_hook_a_mais_no_settings_fica_para_humano() {
    let cenario = cenario_de_pr_de_atualizacao();
    escrever(&cenario.projeto.join(".claude/settings.json"), r#"{ "disableAllHooks": true }"#);
    commitar_tudo(&cenario);
    assert_ficou_para_humano(&cenario, ".claude/settings.json traz mais");
}

#[test]
fn concluir_pr_com_ci_vermelho_fica_para_humano_sem_merge() {
    let cenario = cenario_de_pr_de_atualizacao();
    let (codigo, saida, chamadas) = concluir_pr(&cenario, true);
    assert_eq!(codigo, Some(3), "{saida}");
    assert!(saida.contains("o CI não ficou verde"), "{saida}");
    assert!(!chamadas.contains("pr merge"), "{chamadas}");
}

#[test]
fn falha_da_compilacao_vai_para_o_relatorio_e_a_instalacao_segue() {
    let cenario = Cenario::novo();
    escrever(
        &cenario.modelo().join(".claude/watchdogs/rodar"),
        &lancador_de_mentira(2),
    );
    let relatorio = cenario.relatorio(&[]);
    assert!(
        relatorio.contains(
            "FALHA     .claude/watchdogs não compilou (sh .claude/watchdogs/rodar --version saiu com 2): erro de compilação\n"
        ),
        "{relatorio}"
    );
    assert!(relatorio.contains("SKILL     tdd (Matt)"));
}

#[test]
fn simulacao_nao_escreve_nem_compila() {
    let cenario = Cenario::novo();
    cenario.com_watchdogs_antigos();
    let relatorio = cenario.relatorio(&["--simular", "--adocao"]);
    assert!(relatorio.contains("(simulação: nada foi escrito)\n"));
    assert!(relatorio.contains("ADOÇÃO    .claude/metodo.json criado na etapa observacao\n"));
    assert!(relatorio.contains("REMOVIDA  .claude/hooks"));
    assert!(relatorio.contains("COPIADO   .claude/watchdogs/rodar\n"));
    assert!(!relatorio.contains("COMPILADO") && !relatorio.contains("FALHA"));
    let mut restante: Vec<_> = fs::read_dir(cenario.projeto.join(".claude"))
        .unwrap()
        .map(|entrada| entrada.unwrap().file_name())
        .collect();
    restante.sort();
    assert_eq!(restante, ["hooks", "validador"]);
    assert!(!cenario.projeto.join("AGENTS.md").exists());
    assert_eq!(cenario.hooks_path(), "");
}

#[test]
fn adocao_grava_a_etapa_de_observacao() {
    let cenario = Cenario::novo();
    let relatorio = cenario.relatorio(&["--adocao"]);
    assert!(relatorio.contains("ADOÇÃO    .claude/metodo.json criado na etapa observacao\n"));
    let metodo: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(cenario.projeto.join(".claude/metodo.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(metodo["etapaDeAdocao"], "observacao");
    assert!(!cenario.relatorio(&["--adocao"]).contains("ADOÇÃO"));
}

#[test]
fn skill_que_saiu_da_fonte_sai_do_projeto() {
    let cenario = Cenario::novo();
    cenario.relatorio(&[]);
    fs::remove_dir_all(
        cenario
            .stacklabs
            .join("desenvolvimento/agentes/skills/auditar-metodo"),
    )
    .unwrap();
    let relatorio = cenario.relatorio(&[]);
    assert!(relatorio.contains("REMOVIDA  skill auditar-metodo (saiu da fonte)\n"));
    assert!(!cenario
        .projeto
        .join(".claude/skills/auditar-metodo")
        .exists());
}

#[test]
fn opcao_desconhecida_sai_com_erro() {
    let cenario = Cenario::novo();
    let saida = cenario.rodar(&["--outra"]);
    assert_eq!(saida.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&saida.stderr),
        "opção desconhecida: --outra\n"
    );
}

#[test]
fn skills_do_matt_chegam_com_os_remendos_do_metodo() {
    let cenario = Cenario::novo();
    cenario.relatorio(&[]);
    let skills = cenario.projeto.join(".claude/skills");

    let wayfinder = fs::read_to_string(skills.join("wayfinder/SKILL.md")).unwrap();
    assert!(
        !wayfinder.contains("disable-model-invocation"),
        "{wayfinder}"
    );
    assert!(wayfinder.contains("name: wayfinder"));

    let implement_spec = fs::read_to_string(skills.join("implement-spec/SKILL.md")).unwrap();
    assert!(
        implement_spec.contains("disable-model-invocation: true"),
        "a trava do implement-spec fica"
    );
    let remendo = implement_spec
        .find("run **one at a time**")
        .expect("remendo ausente");
    assert!(implement_spec.find("Communication to and from").unwrap() < remendo);
    assert!(remendo < implement_spec.find("## Steps").unwrap());
    assert_eq!(implement_spec.matches("one at a time").count(), 1);

    let segunda_vez = cenario.relatorio(&[]);
    assert!(
        segunda_vez.ends_with("Nada a fazer: o projeto já está em dia.\n"),
        "{segunda_vez}"
    );
}

#[test]
fn agents_com_a_secao_do_projeto_recebe_o_modelo_novo_e_mantem_a_secao() {
    let cenario = Cenario::novo();
    escrever(
        &cenario.modelo().join("AGENTS.md"),
        "# AGENTS do modelo v2\n\n## Regras deste projeto\n\n(vazia no modelo)\n",
    );
    escrever(
        &cenario.projeto.join("AGENTS.md"),
        "# AGENTS do modelo v1\n\n## Regras deste projeto\n\n### Acrescentadas\n\n- **P1.** Toda tela tem modo escuro.\n",
    );
    let relatorio = cenario.relatorio(&[]);
    assert!(relatorio.contains(
        "MESCLADO  AGENTS.md (modelo novo com a seção ## Regras deste projeto do projeto)\n"
    ));
    assert!(!cenario.projeto.join("AGENTS.md.stacklabs-novo").exists());
    assert_eq!(
        fs::read_to_string(cenario.projeto.join("AGENTS.md")).unwrap(),
        "# AGENTS do modelo v2\n\n## Regras deste projeto\n\n### Acrescentadas\n\n- **P1.** Toda tela tem modo escuro.\n"
    );
    assert!(!cenario.relatorio(&[]).contains("MESCLADO"));
}

#[test]
fn versao_do_metodo_vai_para_a_origem_e_a_mudanca_para_o_relatorio() {
    let cenario = Cenario::novo();
    escrever(&cenario.stacklabs.join("desenvolvimento/VERSAO"), "1.0.0\n");
    let primeira = cenario.relatorio(&[]);
    assert!(primeira.contains("Stack-Labs v1.0.0 ("));
    assert!(linhas_que_comecam(&primeira, "VERSÃO").is_empty());
    let origem: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(cenario.projeto.join(".claude/skills/ORIGEM.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(origem["stacklabs"]["versao"], "1.0.0");

    escrever(&cenario.stacklabs.join("desenvolvimento/VERSAO"), "1.1.0\n");
    assert_eq!(
        linhas_que_comecam(&cenario.relatorio(&[]), "VERSÃO"),
        ["VERSÃO    método 1.0.0 → 1.1.0 (o que mudou: desenvolvimento/CHANGELOG.md)"]
    );
}

#[test]
fn versao_pedida_diferente_da_fonte_sai_com_erro() {
    let cenario = Cenario::novo();
    escrever(&cenario.stacklabs.join("desenvolvimento/VERSAO"), "1.0.0\n");
    let saida = cenario.rodar(&["--versao", "v2.0.0"]);
    assert!(!saida.status.success());
    assert!(String::from_utf8_lossy(&saida.stderr)
        .contains("pediu a versão 2.0.0, mas a fonte do Stack-Labs está na 1.0.0"));
}

#[test]
fn falha_das_labels_vai_para_o_relatorio_e_a_instalacao_termina() {
    let cenario = Cenario::novo();
    let pasta_do_gh = cenario.raiz.join("bin");
    escrever(
        &pasta_do_gh.join("gh"),
        "#!/bin/sh\necho 'not logged in' >&2\nexit 1\n",
    );
    tornar_executavel(&pasta_do_gh.join("gh"));
    let caminho = format!(
        "{}:{}",
        pasta_do_gh.display(),
        std::env::var("PATH").unwrap()
    );
    let saida = Command::new(env!("CARGO_BIN_EXE_instalar"))
        .args(["--destino", cenario.projeto.to_str().unwrap()])
        .env("STACKLABS_ORIGEM", &cenario.stacklabs)
        .env("MATT_ORIGEM", &cenario.matt)
        .env("PATH", caminho)
        .output()
        .unwrap();
    assert!(
        saida.status.success(),
        "{}",
        String::from_utf8_lossy(&saida.stderr)
    );
    assert!(String::from_utf8(saida.stdout).unwrap().contains(
        "FALHA     labels do método não criadas (gh label create wayfinder:map: not logged in)"
    ));
    assert!(cenario.projeto.join(".claude/skills/ORIGEM.json").exists());
}

// Risco: a atualização copia o fonte novo com as datas da origem (num clone local, mais antigas que o
// binário do projeto), o `rodar` não recompila e o binário velho segue rodando com o relatório dizendo
// COMPILADO.
#[test]
fn atualizar_os_watchdogs_apaga_a_compilacao_antiga_antes_de_compilar() {
    let cenario = Cenario::novo();
    cenario.relatorio(&[]);
    let binario_do_projeto = cenario
        .projeto
        .join(".claude/watchdogs/target/release/watchdogs");
    escrever(&binario_do_projeto, "binário da versão anterior");

    // Sem mudança nos watchdogs, o binário fica.
    cenario.relatorio(&[]);
    assert!(
        binario_do_projeto.exists(),
        "nada mudou: não há o que recompilar"
    );

    escrever(
        &cenario.modelo().join(".claude/watchdogs/src/main.rs"),
        "fn main() { println!(\"nova\"); }\n",
    );
    let relatorio = cenario.relatorio(&[]);
    assert!(
        relatorio.contains("COPIADO   .claude/watchdogs/src/main.rs\n"),
        "{relatorio}"
    );
    assert!(
        relatorio.contains("COMPILADO .claude/watchdogs\n"),
        "{relatorio}"
    );
    assert!(
        !binario_do_projeto.exists(),
        "o binário velho saiu; o rodar compila o novo"
    );
}
