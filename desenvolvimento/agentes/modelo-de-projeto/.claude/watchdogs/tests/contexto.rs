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

// Risco: o agente passa uma frase, nada casa e ele lê o código às cegas.
#[test]
fn frase_vira_uma_consulta_por_nome() {
    let repositorio = projeto_do_cupom();
    let contexto = repositorio.contexto("como aplicar_cupom trata o reembolso_parcial");
    assert!(contexto.contains("consulta de 2 nomes (aplicar_cupom, reembolso_parcial)"), "{contexto}");
    assert!(
        contexto.contains("src/cupom.rs:1 · pub fn aplicar_cupom"),
        "o nome solto acha a definição:\n{contexto}"
    );
    assert!(contexto.contains("sem ligação no repositório: reembolso_parcial"), "{contexto}");
    assert!(repositorio.contexto("trata o reembolso_parcial").contains("nada ligado a esta consulta"));
    assert!(
        !repositorio.contexto("cupom da mesa").contains("consulta de"),
        "frase sem identificador é texto livre, inteira"
    );
}

// Risco: `#n` volta vazio para a issue que ainda está na branch dela ou numa branch de integração.
#[test]
fn issue_acha_a_branch_dela_e_o_merge_fora_da_main() {
    let repositorio = projeto_do_cupom();
    repositorio.git(&["switch", "-q", "-c", "issue/12-mesa"]);
    repositorio.escrever("src/mesa.rs", "pub fn abrir_mesa() {}\n");
    repositorio.commitar("feat: mesa abre sozinha");
    repositorio.git(&["switch", "-q", "main"]);
    let contexto = repositorio.contexto("#12");
    assert!(contexto.contains("src/mesa.rs"), "a branch issue/12-* não entrou:\n{contexto}");

    repositorio.git(&["switch", "-q", "-c", "integracao/salao"]);
    repositorio.git(&["switch", "-q", "-c", "outra"]);
    repositorio.escrever("src/conta.rs", "pub fn fechar_conta() {}\n");
    repositorio.commitar("feat: fecha a conta");
    repositorio.git(&["switch", "-q", "integracao/salao"]);
    repositorio.git(&["merge", "-q", "--no-ff", "outra", "-m", "Merge issue/13-conta (#13)"]);
    repositorio.git(&["switch", "-q", "main"]);
    let contexto = repositorio.contexto("#13");
    assert!(contexto.contains("src/conta.rs"), "o merge na integração não entrou:\n{contexto}");
}

// A consulta da issue 176 do Organiza-ai devolvia só dois termos do glossário; o implementador fez 47
// buscas antes da primeira edição. Agora ela leva aos arquivos, aos testes, à ADR e ao risco.
fn projeto_da_fila() -> Repositorio {
    let repositorio = Repositorio::novo();
    repositorio.escrever(
        "GLOSSARY.md",
        "## Language\n\n**Entrada**:\nO lugar do Cliente na Fila.\n\n**Preferência de ambiente**:\nSalão ou varanda.\n\n**Mesa**:\nOnde o cliente senta.\n",
    );
    repositorio.escrever(
        "docs/adr/0007-ambiente.md",
        "# Ambiente é preferência, não promessa\n\nA preferência de ambiente vai para a Equipe.\n",
    );
    repositorio.escrever(
        "docs/riscos.md",
        "| Risco | O que dá errado | Dano | Onde | Origem | Situação |\n|---|---|---|---|---|---|\n| RK43 | A Preferência de ambiente do Cliente não chega à Equipe | mesa errada | Fila | #176 | |\n| RK9 | Publicação segue depois de uma falha | deploy quebrado | Publicar | #80 | |\n",
    );
    repositorio.escrever("docs/mapa-do-codigo.md", "# Mapa do código\n");
    repositorio.escrever("docs/conferencia/176.md", "preferencia de ambiente no cartao da entrada\n");
    repositorio.escrever("prototipos/fila/cartao-da-entrada.jsx", "export const preferencia = 'ambiente';\n");
    repositorio.escrever(
        "src/modulos/fila/telas/cartao-da-entrada.tsx",
        "import { PreferenciaDeAmbiente } from './preferencia-de-ambiente';\nexport function CartaoDaEntrada() { return <PreferenciaDeAmbiente />; }\n",
    );
    repositorio.escrever(
        "src/modulos/fila/telas/preferencia-de-ambiente.tsx",
        "export function PreferenciaDeAmbiente() { return 'salão'; }\n",
    );
    repositorio.escrever("src/modulos/fila/estado.ts", "export const estadoDaFila = 'aberta';\n");
    repositorio.escrever(
        "src/app/equipe/fila/preferencia-de-ambiente.test.tsx",
        "import { CartaoDaEntrada } from '@/modulos/fila/telas/cartao-da-entrada';\n// RK43\nit('mostra a preferência de ambiente no cartão da entrada', () => {});\n",
    );
    repositorio.escrever("comandos/publicar.test.ts", "// RK9\nit('para na primeira falha', () => {});\n");
    repositorio.commitar("feat(fila): preferência de ambiente no cartão (#176)");
    repositorio
}

#[test]
fn texto_livre_traz_codigo_testes_adr_risco_e_mapa_do_codigo() {
    let repositorio = projeto_da_fila();
    let contexto = repositorio.contexto("preferencia de ambiente cartao da entrada");
    for esperado in [
        "  código:",
        "src/modulos/fila/telas/cartao-da-entrada.tsx · nome: cartao, entrada",
        "src/modulos/fila/telas/preferencia-de-ambiente.tsx · nome: preferencia, ambiente",
        "  testes:",
        "src/app/equipe/fila/preferencia-de-ambiente.test.tsx · nome: preferencia, ambiente",
        "testa src/modulos/fila/telas/cartao-da-entrada.tsx",
        "GLOSSARY.md:6 · Preferência de ambiente",
        "docs/adr/0007-ambiente.md · Ambiente é preferência, não promessa",
        "src/app/equipe/fila/preferencia-de-ambiente.test.tsx:3 · RK43 A Preferência de ambiente do Cliente não chega à Equipe",
        "#176 · feat(fila): preferência de ambiente no cartão (#176)",
        "mapa do código: docs/mapa-do-codigo.md",
    ] {
        assert!(contexto.contains(esperado), "faltou \"{esperado}\" em:\n{contexto}");
    }
    for fora in ["docs/conferencia/176.md", "prototipos/fila", "src/modulos/fila/estado.ts", "RK9", "Mesa"] {
        assert!(!contexto.contains(fora), "entrou \"{fora}\", que não trata da consulta:\n{contexto}");
    }
}

#[test]
fn secoes_de_arquivos_respeitam_o_limite() {
    let repositorio = projeto_da_fila();
    for numero in 0..40 {
        repositorio.escrever(&format!("src/modulos/fila/entrada-{numero}.ts"), "export const valor = 1;\n");
    }
    repositorio.commitar("muitas entradas");
    let contexto = repositorio.contexto("entrada na fila");
    let linhas_de_codigo = contexto
        .lines()
        .skip_while(|linha| linha.trim() != "código:")
        .skip(1)
        .take_while(|linha| linha.starts_with("    "))
        .count();
    assert_eq!(linhas_de_codigo, 15, "a seção de código passou do limite:\n{contexto}");
}
