// Testes do `rodar contexto`: um repositório de teste com glossário, ADR, registro de riscos, um
// teste que cita o risco e histórico em que dois arquivos mudam juntos sob a issue #7.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Repositorio {
    raiz: PathBuf,
}

impl Repositorio {
    fn novo() -> Self {
        static PROXIMO: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let ordem = PROXIMO.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let sufixo = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let raiz = std::env::temp_dir().join(format!("contexto-{}-{ordem}-{sufixo}", std::process::id()));
        fs::create_dir_all(&raiz).unwrap();
        let repositorio = Repositorio { raiz };
        for argumentos in [
            &["init", "-q", "-b", "main"][..],
            &["config", "user.email", "t@t"],
            &["config", "user.name", "t"],
        ] {
            repositorio.git(argumentos);
        }
        repositorio
    }
    fn git(&self, argumentos: &[&str]) {
        Command::new("git").args(argumentos).current_dir(&self.raiz).output().unwrap();
    }
    fn escrever(&self, caminho: &str, conteudo: &str) {
        let destino = self.raiz.join(caminho);
        fs::create_dir_all(destino.parent().unwrap()).unwrap();
        fs::write(destino, conteudo).unwrap();
    }
    fn commitar(&self, mensagem: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", mensagem]);
    }
    fn contexto(&self, consulta: &str) -> String {
        let saida = Command::new(env!("CARGO_BIN_EXE_watchdogs"))
            .args(["contexto", consulta])
            .current_dir(&self.raiz)
            .output()
            .unwrap();
        assert!(saida.status.success(), "contexto falhou: {}", String::from_utf8_lossy(&saida.stderr));
        String::from_utf8_lossy(&saida.stdout).into()
    }
}

impl Drop for Repositorio {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.raiz);
    }
}

fn projeto_do_cupom() -> Repositorio {
    let repositorio = Repositorio::novo();
    repositorio.escrever(
        "GLOSSARY.md",
        "## Language\n\n**Cupom**:\nDesconto aplicado ao pedido.\n\n**Mesa**:\nOnde o cliente senta.\n",
    );
    repositorio.escrever(
        "docs/adr/0003-cupom.md",
        "# Cupom não acumula\n\nSó um cupom por pedido: `aplicar_cupom` recusa o segundo.\n",
    );
    repositorio.escrever("docs/riscos.md", "| ID | Risco | Dano | Área | Origem | Situação |\n|---|---|---|---|---|---|\n| RK2 | Cupom vencido aceito | margem perdida | cupom | docs/adr/0003-cupom.md | |\n");
    repositorio.escrever("src/cupom.rs", "pub fn aplicar_cupom(valor_pedido: u64) -> u64 {\n    valor_pedido - 10\n}\n");
    repositorio.escrever("src/pedido.rs", "pub fn finalizar_pedido() -> u64 {\n    aplicar_cupom(100)\n}\n");
    repositorio.commitar("Cupom por pedido (#7)");
    repositorio.escrever(
        "src/cupom.rs",
        "pub fn aplicar_cupom(valor_pedido: u64) -> u64 {\n    valor_pedido.saturating_sub(10)\n}\n",
    );
    repositorio.escrever("src/pedido.rs", "pub fn finalizar_pedido() -> u64 {\n    aplicar_cupom(100) + 0\n}\n");
    repositorio.escrever(
        "tests/cupom.rs",
        "// RK2: cupom vencido não desconta\n#[test]\nfn cupom_vencido_nao_desconta() {\n    assert_eq!(aplicar_cupom(5), 0);\n}\n",
    );
    repositorio.commitar("Cupom não fica negativo (#9)");
    repositorio
}

#[test]
fn simbolo_traz_definicao_termo_decisao_risco_chamada_e_historico() {
    let repositorio = projeto_do_cupom();
    let contexto = repositorio.contexto("aplicar_cupom");
    for esperado in [
        "src/cupom.rs:1 · pub fn aplicar_cupom",
        "GLOSSARY.md:3 · Cupom",
        "docs/adr/0003-cupom.md · Cupom não acumula",
        "tests/cupom.rs:2 · RK2 Cupom vencido aceito (docs/riscos.md:3)",
        "src/pedido.rs:2 · aplicar_cupom(100) + 0",
        "src/pedido.rs · 2 de 2 commits, 100% dos commits de qualquer um dos dois",
        "#9 · Cupom não fica negativo (#9)",
        "#7 · Cupom por pedido (#7)",
    ] {
        assert!(contexto.contains(esperado), "faltou \"{esperado}\" em:\n{contexto}");
    }
    assert!(!contexto.contains("Mesa"), "termo sem relação entrou:\n{contexto}");
}

#[test]
fn issue_traz_os_arquivos_dos_commits_que_a_citam() {
    let repositorio = projeto_do_cupom();
    let contexto = repositorio.contexto("#7");
    let linha_de_arquivos = contexto.lines().find(|linha| linha.trim_start().starts_with("arquivos:")).unwrap_or("");
    for esperado in ["src/cupom.rs", "src/pedido.rs", "GLOSSARY.md"] {
        assert!(linha_de_arquivos.contains(esperado), "faltou {esperado} em: {linha_de_arquivos}");
    }
    assert!(!linha_de_arquivos.contains("tests/cupom.rs"), "arquivo da #9 entrou na #7: {linha_de_arquivos}");
}

#[test]
fn consulta_sem_ligacao_diz_que_nao_achou() {
    let repositorio = projeto_do_cupom();
    assert!(repositorio.contexto("reembolso_parcial").contains("nada ligado a esta consulta"));
    assert!(Path::new(&repositorio.raiz).exists());
}

#[test]
fn skill_escondida_pelo_gitignore_aparece_pelo_nome() {
    let repositorio = projeto_do_cupom();
    repositorio.escrever(".gitignore", ".claude/skills/\n");
    repositorio.escrever(".claude/skills/orquestrar-issues/SKILL.md", "# Orquestrar\n");
    let contexto = repositorio.contexto("orquestrar-issues");
    assert!(
        contexto.contains(".claude/skills/orquestrar-issues/SKILL.md"),
        "skill não apareceu:\n{contexto}"
    );
}
