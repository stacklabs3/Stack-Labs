// Testes do validador: cada checagem roda num repositório git de verdade, numa pasta temporária.
// Rodar: cargo test --release --manifest-path .claude/watchdogs/Cargo.toml
// Os testes do ast-grep rodam se ele estiver no PATH ou em $AST_GREP; sem ele, são pulados.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use watchdogs::validador::{
    arquivos_alterados_desde, checar_cobertura, glob_para_regex, ler_regras, termos_evitados_do_glossario, validar, Opcoes, PRINCIPIOS,
};

fn pasta_do_crate() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn ast_grep_disponivel() -> bool {
    let binario = std::env::var("AST_GREP").unwrap_or_else(|_| "ast-grep".into());
    Command::new(binario)
        .arg("--version")
        .output()
        .map(|saida| saida.status.success())
        .unwrap_or(false)
}

struct Repositorio {
    raiz: PathBuf,
}

impl Repositorio {
    fn novo(arquivos: &[(&str, &str)]) -> Self {
        let raiz = std::env::temp_dir().join(format!("validador-{}-{}", std::process::id(), rand_sufixo()));
        fs::create_dir_all(&raiz).unwrap();
        let repositorio = Repositorio { raiz };
        for (caminho, conteudo) in arquivos {
            repositorio.escrever(caminho, conteudo);
        }
        repositorio.git(&["init", "-q", "-b", "main"]);
        repositorio.git(&["config", "user.email", "teste@stacklabs"]);
        repositorio.git(&["config", "user.name", "Teste"]);
        repositorio.git(&["add", "-A"]);
        repositorio.git(&["commit", "-q", "--allow-empty", "-m", "chore: base"]);
        repositorio
    }
    fn escrever(&self, caminho: &str, conteudo: &str) {
        let destino = self.raiz.join(caminho);
        fs::create_dir_all(destino.parent().unwrap()).unwrap();
        fs::write(destino, conteudo).unwrap();
    }
    fn git(&self, argumentos: &[&str]) {
        Command::new("git").args(argumentos).current_dir(&self.raiz).output().unwrap();
    }
    fn commit(&self, mensagem: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", mensagem]);
    }
}

impl Drop for Repositorio {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.raiz);
    }
}

fn rand_sufixo() -> u128 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
}

fn tabela_com(linhas: &[(&str, &str)]) -> String {
    let mut texto = String::from("| Princípio | Decisão | Onde ou por quê |\n|---|---|---|\n");
    for principio in PRINCIPIOS {
        let linha = linhas
            .iter()
            .find(|(nome, _)| *nome == principio)
            .map(|(_, valor)| valor.to_string())
            .unwrap_or_else(|| "afastado | fora do escopo deste teste".into());
        texto.push_str(&format!("| {principio} | {linha} |\n"));
    }
    texto
}

fn opcoes(base: Option<&str>) -> Opcoes<'_> {
    Opcoes {
        base,
        completo: false,
        so_cobertura: false,
    }
}

fn violacoes_da(raiz: &Path, config: &Value, texto_md: &str, base: Option<&str>, id: &str) -> Vec<(Option<String>, Option<u64>, String)> {
    let relatorio = validar(raiz, config, texto_md, &opcoes(base)).unwrap_or_else(|erro| panic!("{}", erro.0));
    let resultado = relatorio.resultados.into_iter().find(|resultado| resultado.id == id).unwrap();
    resultado
        .violacoes
        .into_iter()
        .map(|violacao| (violacao.arquivo, violacao.linha, violacao.mensagem))
        .collect()
}

fn rodar_uma_regra(arquivos: &[(&str, &str)], verificacao: Value, principio: &str) -> Vec<(Option<String>, Option<u64>, String)> {
    let repositorio = Repositorio::novo(arquivos);
    let config = json!({ "versao": 1, "projeto": { "codigo": ["src/**"] }, "regras": [{ "id": "R1", "principio": principio, "verificacao": verificacao }] });
    violacoes_da(&repositorio.raiz, &config, &tabela_com(&[(principio, "adotado | R1")]), None, "R1")
}

fn regras_de(lista: Value) -> Vec<watchdogs::validador::Regra> {
    ler_regras(&json!({ "versao": 1, "regras": lista })).unwrap_or_else(|erro| panic!("{}", erro.0))
}

#[test]
fn glob_atravessa_pastas_e_escolhe_opcoes() {
    assert!(glob_para_regex("src/**/*.ts").is_match("src/a/b/c.ts"));
    assert!(glob_para_regex("src/**/*.ts").is_match("src/c.ts"));
    assert!(!glob_para_regex("src/*.ts").is_match("src/a/c.ts"));
    assert!(glob_para_regex("src/*.{ts,js}").is_match("src/c.js"));
}

#[test]
fn cobertura_aprova_os_22_decididos_e_ligados() {
    let regras = regras_de(json!([{ "id": "R1", "principio": "E1", "verificacao": { "tipo": "revisor" } }]));
    assert!(checar_cobertura(&tabela_com(&[("E1", "adotado | R1")]), &regras).is_empty());
}

#[test]
fn cobertura_reprova_faltando_decisao_invalida_inexistente_e_orfa() {
    let texto_md = tabela_com(&[("E1", "adotado | R1, R9"), ("E2", "talvez | R2")])
        .lines()
        .filter(|linha| !linha.starts_with("| D |"))
        .collect::<Vec<_>>()
        .join("\n");
    let regras = regras_de(json!([
        { "id": "R1", "principio": "E1", "verificacao": { "tipo": "revisor" } },
        { "id": "R5", "principio": "E3", "verificacao": { "tipo": "revisor" } }
    ]));
    let mensagens: Vec<String> = checar_cobertura(&texto_md, &regras).into_iter().map(|violacao| violacao.mensagem).collect();
    assert!(mensagens.iter().any(|mensagem| mensagem.starts_with("D sem linha")));
    assert!(mensagens.iter().any(|mensagem| mensagem.contains("decisão \"talvez\"")));
    assert!(mensagens.iter().any(|mensagem| mensagem.contains("R9, que não está")));
    assert!(mensagens.iter().any(|mensagem| mensagem.starts_with("R5 está no .json")));
}

#[test]
fn cobertura_reprova_regra_citada_noutro_principio() {
    let regras = regras_de(json!([{ "id": "R1", "principio": "E1", "verificacao": { "tipo": "revisor" } }]));
    let mensagens: Vec<String> = checar_cobertura(&tabela_com(&[("E2", "adotado | R1")]), &regras)
        .into_iter()
        .map(|violacao| violacao.mensagem)
        .collect();
    assert!(mensagens.iter().any(|mensagem| mensagem.contains("R1 nasce de E1")));
}

#[test]
fn termo_evitado_le_o_glossario_e_acha_o_nome_declarado() {
    assert_eq!(
        termos_evitados_do_glossario("_Evitar_: Épico, roadmap, Tarefa (quando não é), ok\n"),
        vec!["Épico", "roadmap", "Tarefa"]
    );
    let violacoes = rodar_uma_regra(
        &[
            ("GLOSSARY.md", "**Comanda**:\nO pedido da mesa.\n_Evitar_: Ticket, ordem de serviço\n"),
            (
                "src/comanda.ts",
                "export function abrirComanda() {}\nexport const ticketAberto = 1;\nclass OrdemDeServicoRepo {}\n// um ticket qualquer no comentário\n",
            ),
        ],
        json!({ "tipo": "termo-evitado" }),
        "E1",
    );
    assert_eq!(violacoes.iter().map(|violacao| violacao.1).collect::<Vec<_>>(), vec![Some(2), Some(3)]);
}

#[test]
fn todo_sem_issue_aceita_com_issue_e_reprova_sem() {
    let violacoes = rodar_uma_regra(
        &[("src/a.ts", "// TODO #12 trocar\n// TODO trocar depois\n// FIXME ver https://x/issues/3\n")],
        json!({ "tipo": "todo-sem-issue" }),
        "E6",
    );
    assert_eq!(violacoes.iter().map(|violacao| violacao.1).collect::<Vec<_>>(), vec![Some(2)]);
}

#[test]
fn teste_de_contrato_por_port_reprova_port_sem_contrato() {
    let violacoes = rodar_uma_regra(
        &[
            ("src/pagamento/portas/cobranca.ts", "export interface Cobranca {}\n"),
            ("src/pagamento/portas/cobranca.contrato.test.ts", ""),
            ("src/pedido/portas/repositorio.ts", "export interface Repositorio {}\n"),
        ],
        json!({ "tipo": "teste-de-contrato-por-port", "ports": "src/**/portas/*.ts", "contrato": "{dir}/{nome}.contrato.test.ts" }),
        "L",
    );
    assert_eq!(
        violacoes.into_iter().map(|violacao| violacao.0.unwrap()).collect::<Vec<_>>(),
        vec!["src/pedido/portas/repositorio.ts"]
    );
}

#[test]
fn lint_cobre_reprova_regra_delegada_que_falta() {
    let violacoes = rodar_uma_regra(
        &[("eslint.config.js", "export default [{ rules: { \"max-depth\": [\"error\", 3] } }];\n")],
        json!({ "tipo": "lint-cobre", "config": "eslint.config.js", "regras": ["max-depth", "max-params"] }),
        "E2",
    );
    assert_eq!(
        violacoes.into_iter().map(|violacao| violacao.2).collect::<Vec<_>>(),
        vec!["a regra de lint \"max-params\" não está configurada"]
    );
}

#[test]
fn mensagem_de_commit_confere_desde_a_base() {
    let repositorio = Repositorio::novo(&[("a.txt", "1")]);
    repositorio.git(&["switch", "-q", "-c", "issue/1-x"]);
    repositorio.escrever("a.txt", "2");
    repositorio.commit("feat: muda a");
    repositorio.escrever("a.txt", "3");
    repositorio.commit("ajustes");
    let config =
        json!({ "versao": 1, "regras": [{ "id": "R1", "principio": "P9", "verificacao": { "tipo": "mensagem-de-commit", "padrao": "^(feat|fix): .+" } }] });
    let violacoes = violacoes_da(&repositorio.raiz, &config, &tabela_com(&[("P9", "adotado | R1")]), Some("main"), "R1");
    assert_eq!(violacoes.len(), 1);
    assert!(violacoes[0].2.contains("\"ajustes\""));
}

#[test]
fn comando_saida_zero_aprova_e_outra_reprova() {
    assert!(rodar_uma_regra(&[], json!({ "tipo": "comando", "executar": "true" }), "E1").is_empty());
    let violacoes = rodar_uma_regra(&[], json!({ "tipo": "comando", "executar": "echo duplicado em a.ts; exit 3" }), "E1");
    assert!(violacoes[0].2.contains("saiu com 3") && violacoes[0].2.contains("duplicado em a.ts"));
}

#[test]
fn configuracao_invalida_e_erro_de_configuracao() {
    let repositorio = Repositorio::novo(&[]);
    for regra in [
        json!({ "id": "R1", "principio": "E1", "verificacao": { "tipo": "magia" } }),
        json!({ "id": "R1", "principio": "Z9", "verificacao": { "tipo": "revisor" } }),
    ] {
        assert!(validar(&repositorio.raiz, &json!({ "versao": 1, "regras": [regra] }), "", &opcoes(None)).is_err());
    }
}

#[test]
fn modelo_de_regras_de_codigo_json_e_valido() {
    let modelo: Value = serde_json::from_str(&fs::read_to_string(pasta_do_crate().join("modelos/regras-de-codigo.json")).unwrap()).unwrap();
    let repositorio = Repositorio::novo(&[]);
    let linhas: Vec<(String, String)> = modelo["regras"]
        .as_array()
        .unwrap()
        .iter()
        .map(|regra| {
            (
                regra["principio"].as_str().unwrap().to_string(),
                format!("adotado | {}", regra["id"].as_str().unwrap()),
            )
        })
        .collect();
    let linhas_ref: Vec<(&str, &str)> = linhas.iter().map(|(principio, valor)| (principio.as_str(), valor.as_str())).collect();
    let relatorio = validar(
        &repositorio.raiz,
        &modelo,
        &tabela_com(&linhas_ref),
        &Opcoes {
            base: None,
            completo: false,
            so_cobertura: true,
        },
    )
    .unwrap();
    assert!(relatorio.aprovado, "{:?}", relatorio.resultados[0].violacoes);
}

fn preencher_modelo(nome: &str, trocas: &[(&str, &str)]) -> String {
    trocas.iter().fold(
        fs::read_to_string(pasta_do_crate().join("modelos/ast-grep").join(nome)).unwrap(),
        |texto, (marcador, valor)| texto.replace(marcador, valor),
    )
}

#[test]
fn ast_grep_os_modelos_acham_o_que_devem() {
    if !ast_grep_disponivel() {
        eprintln!("ast-grep não instalado: teste pulado");
        return;
    }
    let repositorio = Repositorio::novo(&[
        (
            "regras/dominio.yml",
            &preencher_modelo(
                "dominio-sem-infra.yml",
                &[("<PASTA_DO_DOMINIO>", "src/dominio"), ("<PACOTES_DE_INFRA>", "prisma|@prisma/|express")],
            ),
        ),
        ("regras/catch.yml", &preencher_modelo("catch-vazio.yml", &[])),
        (
            "regras/ambiente.yml",
            &preencher_modelo("ambiente-so-na-montagem.yml", &[("<ARQUIVOS_DA_MONTAGEM>", "src/montagem/**")]),
        ),
        (
            "src/dominio/pedido.ts",
            "import { PrismaClient } from \"@prisma/client\";\nimport { somar } from \"./dinheiro\";\nconst express = require(\"express\");\n",
        ),
        (
            "src/infra/repo.js",
            "export function a() { try { g(); } catch (erro) { } }\nexport function b() { try { g(); } catch (erro) { tratar(erro); } }\n",
        ),
        ("src/montagem/index.ts", "export const chave = process.env.CHAVE;\n"),
        ("src/dominio/config.ts", "export const chave = process.env.CHAVE;\n"),
        ("py/dominio/servico.py", "try:\n    x()\nexcept Exception:\n    pass\n"),
    ]);
    let config = json!({ "versao": 1, "regras": [
        { "id": "R1", "principio": "D", "verificacao": { "tipo": "ast-grep", "regras": "regras/dominio.yml" } },
        { "id": "R2", "principio": "E4", "verificacao": { "tipo": "ast-grep", "regras": "regras/catch.yml" } },
        { "id": "R3", "principio": "P3", "verificacao": { "tipo": "ast-grep", "regras": "regras/ambiente.yml" } }
    ] });
    let texto_md = tabela_com(&[("D", "adotado | R1"), ("E4", "adotado | R2"), ("P3", "adotado | R3")]);
    let achados = |id: &str| {
        violacoes_da(&repositorio.raiz, &config, &texto_md, None, id)
            .into_iter()
            .map(|violacao| format!("{}:{}", violacao.0.unwrap(), violacao.1.unwrap()))
            .collect::<Vec<_>>()
    };
    assert_eq!(achados("R1"), vec!["src/dominio/pedido.ts:1", "src/dominio/pedido.ts:3"]);
    assert_eq!(achados("R2"), vec!["py/dominio/servico.py:3", "src/infra/repo.js:1"]);
    assert_eq!(achados("R3"), vec!["src/dominio/config.ts:1"]);
}

#[test]
fn ast_grep_regra_invalida_e_violacao() {
    if !ast_grep_disponivel() {
        eprintln!("ast-grep não instalado: teste pulado");
        return;
    }
    let violacoes = rodar_uma_regra(
        &[("regras/ruim.yml", "id: x\nlanguage: NaoExiste\nrule: { pattern: a }\n"), ("src/a.ts", "a;\n")],
        json!({ "tipo": "ast-grep", "regras": "regras/ruim.yml" }),
        "D",
    );
    assert_eq!(violacoes.len(), 1);
    assert!(violacoes[0].2.contains("ast-grep falhou"));
}

#[test]
fn incremental_le_so_o_que_mudou_desde_a_base() {
    let repositorio = Repositorio::novo(&[("src/antigo.ts", "// TODO sem issue, de antes\n")]);
    repositorio.git(&["switch", "-q", "-c", "issue/2-x"]);
    repositorio.escrever("src/novo.ts", "// TODO sem issue, novo\n");
    repositorio.commit("feat: novo");
    repositorio.escrever("src/sem-commit.ts", "// FIXME sem issue, ainda sem commit\n");
    let mut alterados = arquivos_alterados_desde(&repositorio.raiz, "main").unwrap();
    alterados.sort();
    assert_eq!(alterados, vec!["src/novo.ts", "src/sem-commit.ts"]);
    let config =
        json!({ "versao": 1, "projeto": { "codigo": ["src/**"] }, "regras": [{ "id": "R1", "principio": "E6", "verificacao": { "tipo": "todo-sem-issue" } }] });
    let texto_md = tabela_com(&[("E6", "adotado | R1")]);
    let arquivos_vistos = |completo: bool| {
        let relatorio = validar(
            &repositorio.raiz,
            &config,
            &texto_md,
            &Opcoes {
                base: Some("main"),
                completo,
                so_cobertura: false,
            },
        )
        .unwrap();
        let mut vistos: Vec<String> = relatorio.resultados[1]
            .violacoes
            .iter()
            .map(|violacao| violacao.arquivo.clone().unwrap())
            .collect();
        vistos.sort();
        (vistos, relatorio.escopo)
    };
    let (incremental, escopo) = arquivos_vistos(false);
    assert_eq!(incremental, vec!["src/novo.ts", "src/sem-commit.ts"]);
    assert_eq!(escopo, "2 arquivo(s) alterado(s) desde main");
    assert_eq!(arquivos_vistos(true).0, vec!["src/antigo.ts", "src/novo.ts", "src/sem-commit.ts"]);
}

#[test]
fn comando_recebe_so_os_arquivos_lidos() {
    let repositorio = Repositorio::novo(&[("src/a.ts", "1\n")]);
    repositorio.git(&["switch", "-q", "-c", "issue/3-x"]);
    repositorio.escrever("src/b c.ts", "2\n");
    repositorio.commit("feat: b");
    let config = json!({ "versao": 1, "projeto": { "codigo": ["src/**"] }, "regras": [{ "id": "R1", "principio": "E7", "verificacao": { "tipo": "comando", "executar": "printf '%s|' {arquivos}; exit 1" } }] });
    let texto_md = tabela_com(&[("E7", "adotado | R1")]);
    let violacoes = violacoes_da(&repositorio.raiz, &config, &texto_md, Some("main"), "R1");
    assert!(violacoes[0].2.ends_with("src/b c.ts|") && !violacoes[0].2.contains("src/a.ts"));
    repositorio.git(&["switch", "-q", "main"]);
    assert!(violacoes_da(&repositorio.raiz, &config, &texto_md, Some("main"), "R1").is_empty());
}

#[test]
fn linha_de_comando_imprime_o_parecer_e_sai_com_1() {
    let repositorio = Repositorio::novo(&[
        ("docs/regras-de-codigo.md", &tabela_com(&[("E6", "adotado | R1")])),
        ("src/a.ts", "// TODO sem issue\n"),
    ]);
    repositorio.escrever("docs/regras-de-codigo.json", &json!({ "versao": 1, "projeto": { "codigo": ["src/**"] }, "regras": [{ "id": "R1", "principio": "E6", "verificacao": { "tipo": "todo-sem-issue" } }] }).to_string());
    let saida = Command::new(env!("CARGO_BIN_EXE_watchdogs"))
        .arg("validar")
        .current_dir(&repositorio.raiz)
        .output()
        .unwrap();
    let texto = String::from_utf8_lossy(&saida.stdout);
    assert_eq!(saida.status.code(), Some(1));
    assert!(
        texto.contains("❌ R1 (E6) todo-sem-issue") && texto.contains("src/a.ts:1") && texto.trim_end().ends_with("VALIDADOR: REPROVADO"),
        "{texto}"
    );
}
