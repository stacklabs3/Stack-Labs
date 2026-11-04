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
fn sem_a_flag_watchdog_e_pulado_e_os_antigos_ficam() {
    let cenario = Cenario::novo();
    cenario.com_watchdogs_antigos();
    let relatorio = cenario.relatorio(&[]);
    let sufixo = "(watchdog ausente ou desatualizado: um humano roda com --atualizar-watchdogs)";
    for arquivo in [
        ".claude/settings.json",
        ".claude/githooks/pre-push",
        ".claude/watchdogs/rodar",
    ] {
        assert!(
            relatorio.contains(&format!("PULADO    {arquivo} {sufixo}\n")),
            "{arquivo}:\n{relatorio}"
        );
    }
    assert!(relatorio.contains("COPIADO   AGENTS.md\n"));
    assert!(!relatorio.contains("REMOVIDA") && !relatorio.contains("COMPILADO"));
    assert!(relatorio
        .contains("PRE-PUSH  inativo: um humano roda git config core.hooksPath .claude/githooks"));
    assert!(cenario
        .projeto
        .join(".claude/hooks/guarda-bash.mjs")
        .exists());
    assert!(cenario
        .projeto
        .join(".claude/validador/validar.mjs")
        .exists());
    assert!(!cenario.projeto.join(".claude/watchdogs").exists());
}

#[test]
fn com_a_flag_os_watchdogs_antigos_saem_e_os_novos_compilam() {
    let cenario = Cenario::novo();
    cenario.com_watchdogs_antigos();
    let relatorio = cenario.relatorio(&["--atualizar-watchdogs"]);
    assert!(relatorio
        .contains("REMOVIDA  .claude/hooks (watchdogs antigos: agora em .claude/watchdogs)\n"));
    assert!(relatorio
        .contains("REMOVIDA  .claude/validador (watchdogs antigos: agora em .claude/watchdogs)\n"));
    assert!(relatorio.contains("COPIADO   .claude/watchdogs/rodar\n"));
    assert!(relatorio.contains("COMPILADO .claude/watchdogs\n"));
    assert!(relatorio.contains("PRE-PUSH  ativado (core.hooksPath)"));
    assert!(!cenario.projeto.join(".claude/hooks").exists());
    assert!(!cenario.projeto.join(".claude/validador").exists());
    assert!(cenario.projeto.join(".claude/watchdogs/rodar").exists());
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
    let relatorio = cenario.relatorio(&["--simular", "--atualizar-watchdogs", "--adocao"]);
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
    cenario.relatorio(&["--atualizar-watchdogs"]);
    assert!(
        binario_do_projeto.exists(),
        "nada mudou: não há o que recompilar"
    );

    escrever(
        &cenario.modelo().join(".claude/watchdogs/src/main.rs"),
        "fn main() { println!(\"nova\"); }\n",
    );
    let relatorio = cenario.relatorio(&["--atualizar-watchdogs"]);
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
