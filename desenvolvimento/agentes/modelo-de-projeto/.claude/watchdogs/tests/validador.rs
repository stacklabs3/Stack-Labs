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

// O relógio do macOS tem resolução de microssegundo: o contador separa testes paralelos.
fn rand_sufixo() -> String {
    static PROXIMO: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let ordem = PROXIMO.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let instante = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    format!("{ordem}-{instante}")
}

// O P5 não se afasta e exige a checagem teste-cobre-risco: sem linha própria no teste, ele cita a
// regra R90, que regras_de acrescenta.
fn tabela_com(linhas: &[(&str, &str)]) -> String {
    let mut texto = String::from("| Princípio | Decisão | Onde ou por quê |\n|---|---|---|\n");
    for principio in PRINCIPIOS {
        let linha = linhas
            .iter()
            .find(|(nome, _)| *nome == principio)
            .map(|(_, valor)| valor.to_string())
            .unwrap_or_else(|| {
                if principio == "P5" {
                    "adotado | R90".into()
                } else {
                    "afastado | fora do escopo deste teste".into()
                }
            });
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
    let mut lista = lista;
    let regras = lista.as_array_mut().unwrap();
    if !regras.iter().any(|regra| regra["principio"] == "P5") {
        regras.push(json!({ "id": "R90", "principio": "P5", "verificacao": { "tipo": "teste-cobre-risco" } }));
    }
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
fn cobertura_reprova_e1_afastado_mesmo_com_o_porque() {
    let regras = regras_de(json!([{ "id": "R1", "principio": "E2", "verificacao": { "tipo": "revisor" } }]));
    let mensagens: Vec<String> = checar_cobertura(
        &tabela_com(&[("E1", "afastado | o time prefere nomes curtos"), ("E2", "adotado | R1")]),
        &regras,
    )
    .into_iter()
    .map(|violacao| violacao.mensagem)
    .collect();
    assert_eq!(mensagens, vec!["E1 é regra dura da constituição: adapte, mas não afaste".to_string()]);
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
fn modelos_de_regras_de_codigo_json_sao_validos() {
    for nome in [
        "regras-de-codigo.json",
        "regras-de-codigo.java.json",
        "regras-de-codigo.rust.json",
        "regras-de-codigo.dart.json",
    ] {
        let modelo: Value = serde_json::from_str(&fs::read_to_string(pasta_do_crate().join("modelos").join(nome)).unwrap()).unwrap();
        let repositorio = Repositorio::novo(&[]);
        // Uma linha por princípio ou prática, citando todas as regras que nascem dele.
        let mut linhas: Vec<(String, Vec<String>)> = Vec::new();
        for regra in modelo["regras"].as_array().unwrap() {
            let principio = regra["principio"].as_str().unwrap().to_string();
            let id = regra["id"].as_str().unwrap().to_string();
            match linhas.iter_mut().find(|(nome_da_linha, _)| *nome_da_linha == principio) {
                Some((_, ids)) => ids.push(id),
                None => linhas.push((principio, vec![id])),
            }
        }
        let valores: Vec<(String, String)> = linhas
            .iter()
            .map(|(principio, ids)| (principio.clone(), format!("adotado | {}", ids.join(", "))))
            .collect();
        let linhas_dos_22: Vec<(&str, &str)> = valores
            .iter()
            .filter(|(principio, _)| PRINCIPIOS.contains(&principio.as_str()))
            .map(|(principio, valor)| (principio.as_str(), valor.as_str()))
            .collect();
        let praticas: String = valores
            .iter()
            .filter(|(principio, _)| !PRINCIPIOS.contains(&principio.as_str()))
            .map(|(principio, valor)| format!("| {principio} | {valor} |\n"))
            .collect();
        let relatorio = validar(
            &repositorio.raiz,
            &modelo,
            &tabela_com_praticas(&linhas_dos_22, &praticas),
            &Opcoes {
                base: None,
                completo: false,
                so_cobertura: true,
            },
        )
        .unwrap_or_else(|erro| panic!("{nome}: {}", erro.0));
        assert!(relatorio.aprovado, "{nome}: {:?}", relatorio.resultados[0].violacoes);
    }
}

// Risco: o guia de Dart promete uma regra que quem copia o modelo não recebe.
#[test]
fn modelo_dart_tem_o_que_o_guia_cita() {
    let modelo: Value = serde_json::from_str(&fs::read_to_string(pasta_do_crate().join("modelos/regras-de-codigo.dart.json")).unwrap()).unwrap();
    let regras = modelo["regras"].as_array().unwrap();
    let delegadas: Vec<&str> = regras
        .iter()
        .filter(|regra| regra["verificacao"]["tipo"] == "lint-cobre")
        .flat_map(|regra| regra["verificacao"]["regras"].as_array().unwrap().iter().map(|nome| nome.as_str().unwrap()))
        .collect();
    for lint_do_guia in [
        "camel_case_types",
        "non_constant_identifier_names",
        "constant_identifier_names",
        "file_names",
        "avoid_positional_boolean_parameters",
        "use_rethrow_when_possible",
        "prefer_final_in_for_each",
        "avoid_print",
        "implementation_imports",
        "depend_on_referenced_packages",
        "always_declare_return_types",
        "annotate_overrides",
    ] {
        assert!(delegadas.contains(&lint_do_guia), "falta {lint_do_guia} no modelo de regras");
    }
    for pratica_do_revisor in ["DT5", "DT6"] {
        assert!(
            regras.iter().any(|regra| regra["principio"] == pratica_do_revisor && regra["verificacao"]["tipo"] == "revisor"),
            "{pratica_do_revisor} fica com o revisor"
        );
    }
    let gerado_de_mock = "**/*.mocks.dart";
    assert!(modelo["projeto"]["ignorar"].as_array().unwrap().iter().any(|padrao| padrao == gerado_de_mock));
    let analysis_options = fs::read_to_string(pasta_do_crate().join("modelos/lint/analysis_options.yaml")).unwrap();
    assert!(analysis_options.contains(&format!("- \"{gerado_de_mock}\"")));
    assert!(regras.iter().any(|regra| regra["texto"].as_str().unwrap().contains("flutter analyze --fatal-infos")));
    let sem_afirmacao = fs::read_to_string(pasta_do_crate().join("modelos/ast-grep/teste-sem-afirmacao.yml")).unwrap();
    assert!(sem_afirmacao.contains("goldenTest fica fora"));
}

#[test]
fn modelo_analysis_options_liga_toda_regra_do_modelo_dart() {
    let modelo: Value = serde_json::from_str(&fs::read_to_string(pasta_do_crate().join("modelos/regras-de-codigo.dart.json")).unwrap()).unwrap();
    let analysis_options = fs::read_to_string(pasta_do_crate().join("modelos/lint/analysis_options.yaml")).unwrap();
    let regras_delegadas: Vec<&str> = modelo["regras"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|regra| regra["verificacao"]["tipo"] == "lint-cobre")
        .flat_map(|regra| regra["verificacao"]["regras"].as_array().unwrap().iter().map(|nome| nome.as_str().unwrap()))
        .collect();
    let motivos = motivos_do_lint(
        &[("analysis_options.yaml", &analysis_options)],
        json!("analysis_options.yaml"),
        &regras_delegadas,
    );
    assert!(motivos.is_empty(), "{motivos:?}");
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
                &[("<PASTA_DO_DOMINIO>", "src/dominio"), ("<PACOTES_DE_INFRA>", "prisma|@prisma/|express|http")],
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
        ("regras/nome.yml", &preencher_modelo("nome-de-uma-letra.yml", &[])),
        (
            "src/dominio/viagem.dart",
            "import 'package:http/http.dart';\nimport 'dart:async';\nvoid a() { try { g(); } catch (erro) {} }\n\
             final lista = valores.map((v) => v * 2);\nfinal total = lista.length;\n",
        ),
    ]);
    let config = json!({ "versao": 1, "regras": [
        { "id": "R1", "principio": "D", "verificacao": { "tipo": "ast-grep", "regras": "regras/dominio.yml" } },
        { "id": "R2", "principio": "E4", "verificacao": { "tipo": "ast-grep", "regras": "regras/catch.yml" } },
        { "id": "R3", "principio": "P3", "verificacao": { "tipo": "ast-grep", "regras": "regras/ambiente.yml" } },
        { "id": "R4", "principio": "E1", "verificacao": { "tipo": "ast-grep", "regras": "regras/nome.yml" } }
    ] });
    let texto_md = tabela_com(&[("D", "adotado | R1"), ("E4", "adotado | R2"), ("P3", "adotado | R3"), ("E1", "adotado | R4")]);
    let achados = |id: &str| {
        violacoes_da(&repositorio.raiz, &config, &texto_md, None, id)
            .into_iter()
            .map(|violacao| format!("{}:{}", violacao.0.unwrap(), violacao.1.unwrap()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        achados("R1"),
        vec!["src/dominio/pedido.ts:1", "src/dominio/pedido.ts:3", "src/dominio/viagem.dart:1"]
    );
    assert_eq!(
        achados("R2"),
        vec!["py/dominio/servico.py:3", "src/dominio/viagem.dart:3", "src/infra/repo.js:1"]
    );
    // "void a()" e "(v) =>": função e lambda contam; o uso de "v" e "valores" não.
    assert_eq!(achados("R4"), vec!["src/dominio/viagem.dart:3", "src/dominio/viagem.dart:4"]);
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

// ---------- lint-cobre falha fechado ----------

fn motivos_do_lint(arquivos: &[(&str, &str)], config: Value, regras: &[&str]) -> Vec<String> {
    rodar_uma_regra(arquivos, json!({ "tipo": "lint-cobre", "config": config, "regras": regras }), "E4")
        .into_iter()
        .map(|violacao| violacao.2)
        .collect()
}

#[test]
fn lint_cobre_cargo_toml_exige_deny_ou_forbid() {
    let cargo_toml = "[package]\nname = \"x\"\n\n[lints.rust]\nunsafe_code = \"forbid\"\n\n[lints.clippy]\n\
        pedantic = { level = \"deny\", priority = -1 }\nexpect_used = { level = \"deny\", priority = 1 }\nunwrap_used = \"allow\"\n\
        todo = \"warn\"\npanic_in_result_fn = \"deny\"\n# dbg_macro = \"deny\"\n";
    let motivos = motivos_do_lint(
        &[("Cargo.toml", cargo_toml)],
        json!("Cargo.toml"),
        &[
            "unsafe_code",
            "pedantic",
            "expect_used",
            "clippy::expect_used",
            "unwrap_used",
            "todo",
            "panic",
            "dbg_macro",
            "rust::expect_used",
        ],
    );
    assert_eq!(
        motivos,
        vec![
            "a regra de lint \"unwrap_used\" está configurada como \"allow\", que não bloqueia",
            "a regra de lint \"todo\" está configurada como \"warn\", que não bloqueia",
            "a regra de lint \"panic\" não está configurada",
            "a regra de lint \"dbg_macro\" só aparece comentada",
            "a regra de lint \"rust::expect_used\" não está configurada",
        ]
    );
}

#[test]
fn lint_cobre_workspace_lints_e_limite_do_clippy_toml() {
    let motivos = motivos_do_lint(
        &[
            ("Cargo.toml", "[workspace]\nmembers = []\n\n[workspace.lints.clippy]\nunwrap_used = \"deny\"\n"),
            ("clippy.toml", "too-many-lines-threshold = 60\nallow-unwrap-in-tests = false\n"),
        ],
        json!(["Cargo.toml", "clippy.toml"]),
        &[
            "unwrap_used",
            "too-many-lines-threshold",
            "allow-unwrap-in-tests",
            "excessive-nesting-threshold",
        ],
    );
    assert_eq!(
        motivos,
        vec![
            "a regra de lint \"allow-unwrap-in-tests\" está configurada como false, que não bloqueia",
            "a regra de lint \"excessive-nesting-threshold\" não está configurada",
        ]
    );
}

#[test]
fn lint_cobre_analysis_options_le_lista_mapa_severidade_e_linguagem() {
    let analysis_options = "include: package:very_good_analysis/analysis_options.yaml\n\nanalyzer:\n  language:\n    strict-casts: true\n\
        \x20   strict-inference: false\n  errors:\n    empty_catches: error\n    discarded_futures: ignore\n    unawaited_futures: info\n\n\
        linter:\n  rules:\n    - avoid_print\n    # - prefer_final_locals\n    - \"use_build_context_synchronously\"\n";
    let analysis_options_em_mapa = "linter:\n  rules:\n    prefer_const_constructors: true\n    public_member_api_docs: false\n";
    let motivos = motivos_do_lint(
        &[
            ("analysis_options.yaml", analysis_options),
            ("pacote/analysis_options.yaml", analysis_options_em_mapa),
        ],
        json!(["analysis_options.yaml", "pacote/analysis_options.yaml"]),
        &[
            "strict-casts",
            "strict-inference",
            "empty_catches",
            "discarded_futures",
            "unawaited_futures",
            "avoid_print",
            "prefer_final_locals",
            "use_build_context_synchronously",
            "prefer_const_constructors",
            "public_member_api_docs",
            "avoid_dynamic_calls",
        ],
    );
    assert_eq!(
        motivos,
        vec![
            "a regra de lint \"strict-inference\" está configurada como \"false\", que não bloqueia",
            "a regra de lint \"discarded_futures\" está configurada como \"ignore\", que não bloqueia",
            "a regra de lint \"unawaited_futures\" está configurada como \"info\", que não bloqueia",
            "a regra de lint \"prefer_final_locals\" só aparece comentada",
            "a regra de lint \"public_member_api_docs\" está configurada como \"false\", que não bloqueia",
            // Só o preset do include: liga, e o preset não aparece no arquivo.
            "a regra de lint \"avoid_dynamic_calls\" não está configurada",
        ]
    );
}

// Roda o binário num projeto que delega avoid_print ao lint, com a config declarada dada.
fn validar_lint_dart(arquivos: &[(&str, &str)], config: Value) -> (i32, String) {
    validar_lint_dart_com_nao_versionados(arquivos, &[], config)
}

fn validar_lint_dart_com_nao_versionados(arquivos: &[(&str, &str)], nao_versionados: &[(&str, &str)], config: Value) -> (i32, String) {
    let repositorio = Repositorio::novo(arquivos);
    for (caminho, conteudo) in nao_versionados {
        repositorio.escrever(caminho, conteudo);
    }
    let regras = json!({ "versao": 1, "regras": [{ "id": "R1", "principio": "E4", "verificacao": { "tipo": "lint-cobre", "config": config, "regras": ["avoid_print"] } },
        { "id": "R2", "principio": "E1", "verificacao": { "tipo": "revisor" } },
        { "id": "R90", "principio": "P5", "verificacao": { "tipo": "teste-cobre-risco" } }
    ] });
    repositorio.escrever("docs/riscos.md", "| Risco | O que dá errado | Dano | Onde | Origem | Situação |\n|---|---|---|---|---|---|\n");
    repositorio.escrever("docs/regras-de-codigo.json", &regras.to_string());
    repositorio.escrever("docs/regras-de-codigo.md", &tabela_com(&[("E4", "adotado | R1"), ("E1", "adotado | R2")]));
    let saida = Command::new(env!("CARGO_BIN_EXE_watchdogs"))
        .arg("validar")
        .current_dir(&repositorio.raiz)
        .output()
        .unwrap();
    (saida.status.code().unwrap_or(-1), String::from_utf8_lossy(&saida.stdout).into_owned())
}

const ANALYSIS_OPTIONS_LIGADO: &str = "linter:\n  rules:\n    - avoid_print\n";
const ANALYSIS_OPTIONS_DESLIGADO: &str = "linter:\n  rules:\n    avoid_print: false\n";

// Risco: um analysis_options.yaml numa subpasta cala o lint daquela pasta e o lint-cobre aprova.
#[test]
fn lint_cobre_reprova_analysis_options_aninhado_fora_da_lista() {
    let (codigo, saida) = validar_lint_dart(
        &[("analysis_options.yaml", ANALYSIS_OPTIONS_LIGADO), ("lib/sub/analysis_options.yaml", ANALYSIS_OPTIONS_DESLIGADO)],
        json!("analysis_options.yaml"),
    );
    assert_eq!(codigo, 1, "{saida}");
    assert!(saida.contains("lib/sub/analysis_options.yaml"), "{saida}");

    // Subpasta com regras próprias: a pasta da config é a do app, e o aninhado nela reprova.
    let (codigo, saida) = validar_lint_dart(
        &[("app/analysis_options.yaml", ANALYSIS_OPTIONS_LIGADO), ("app/lib/analysis_options.yaml", ANALYSIS_OPTIONS_DESLIGADO)],
        json!("app/analysis_options.yaml"),
    );
    assert_eq!(codigo, 1, "{saida}");
    assert!(saida.contains("app/lib/analysis_options.yaml"), "{saida}");
}

// Risco: reprovar o que o projeto já declarou, ou o que está fora da pasta da config.
#[test]
fn lint_cobre_aceita_analysis_options_listado_ou_fora_da_pasta() {
    let (codigo, saida) = validar_lint_dart(
        &[("analysis_options.yaml", ANALYSIS_OPTIONS_LIGADO), ("pacote/analysis_options.yaml", ANALYSIS_OPTIONS_LIGADO)],
        json!(["analysis_options.yaml", "pacote/analysis_options.yaml"]),
    );
    assert_eq!(codigo, 0, "{saida}");

    let (codigo, saida) = validar_lint_dart(
        &[("app/analysis_options.yaml", ANALYSIS_OPTIONS_LIGADO), ("outro/analysis_options.yaml", ANALYSIS_OPTIONS_DESLIGADO)],
        json!("app/analysis_options.yaml"),
    );
    assert_eq!(codigo, 0, "{saida}");
}

// Risco: código gerado ignorado pelo git (contrato_gerado/ do excurfly) reprovar o projeto.
#[test]
fn lint_cobre_nao_dispara_com_analysis_options_ignorado_pelo_git() {
    let (codigo, saida) = validar_lint_dart(
        &[
            (".gitignore", "contrato_gerado/\n"),
            ("analysis_options.yaml", ANALYSIS_OPTIONS_LIGADO),
            ("contrato_gerado/analysis_options.yaml", ANALYSIS_OPTIONS_DESLIGADO),
        ],
        json!("analysis_options.yaml"),
    );
    assert_eq!(codigo, 0, "{saida}");
}

// Risco: reprovar antes do commit um arquivo que o projeto ainda não versionou (a regra é sobre o versionado).
#[test]
fn lint_cobre_nao_dispara_com_analysis_options_ainda_sem_commit() {
    let (codigo, saida) = validar_lint_dart_com_nao_versionados(
        &[("analysis_options.yaml", ANALYSIS_OPTIONS_LIGADO)],
        &[("lib/sub/analysis_options.yaml", ANALYSIS_OPTIONS_DESLIGADO)],
        json!("analysis_options.yaml"),
    );
    assert_eq!(codigo, 0, "{saida}");
}

// Risco: a mesma violação repetida em cada regra lint-cobre esconde as outras violações da saída.
#[test]
fn lint_cobre_mostra_o_analysis_options_aninhado_uma_vez_por_execucao() {
    let repositorio = Repositorio::novo(&[
        ("analysis_options.yaml", "linter:\n  rules:\n    - avoid_print\n    - empty_catches\n"),
        ("lib/sub/analysis_options.yaml", ANALYSIS_OPTIONS_DESLIGADO),
        ("docs/riscos.md", "| Risco | O que dá errado | Dano | Onde | Origem | Situação |\n|---|---|---|---|---|---|\n"),
    ]);
    let regras = json!({ "versao": 1, "regras": [
        { "id": "R1", "principio": "E4", "verificacao": { "tipo": "lint-cobre", "config": "analysis_options.yaml", "regras": ["empty_catches"] } },
        { "id": "R3", "principio": "E6", "verificacao": { "tipo": "lint-cobre", "config": "analysis_options.yaml", "regras": ["avoid_print"] } },
        { "id": "R2", "principio": "E1", "verificacao": { "tipo": "revisor" } },
        { "id": "R90", "principio": "P5", "verificacao": { "tipo": "teste-cobre-risco" } }
    ] });
    repositorio.escrever("docs/regras-de-codigo.json", &regras.to_string());
    repositorio.escrever("docs/regras-de-codigo.md", &tabela_com(&[("E4", "adotado | R1"), ("E6", "adotado | R3"), ("E1", "adotado | R2")]));
    let saida = Command::new(env!("CARGO_BIN_EXE_watchdogs")).arg("validar").current_dir(&repositorio.raiz).output().unwrap();
    let texto = String::from_utf8_lossy(&saida.stdout);
    assert_eq!(saida.status.code(), Some(1), "{texto}");
    assert_eq!(texto.matches("lib/sub/analysis_options.yaml").count(), 1, "{texto}");
}

#[test]
fn lint_cobre_toml_invalido_reprova() {
    let motivos = motivos_do_lint(
        &[("Cargo.toml", "[lints.clippy\nunwrap_used = \"deny\"\n")],
        json!("Cargo.toml"),
        &["unwrap_used"],
    );
    assert_eq!(motivos.len(), 1);
    assert!(motivos[0].contains("não foi conferida: não é TOML válido"), "{motivos:?}");
}

#[test]
fn lint_cobre_checkstyle_ignora_comentario_e_severity_ignore() {
    let checkstyle = r#"<?xml version="1.0"?>
<module name="Checker">
  <module name="TreeWalker">
    <module name="ParameterNumber"><property name="max" value="3"/></module>
    <module name="MagicNumberX"/>
    <!-- <module name="MagicNumber"/> -->
    <module name="NestedIfDepth">
      <property name="severity" value="ignore"/>
    </module>
    <module name="com.puppycrawl.tools.checkstyle.checks.coding.IllegalCatchCheck"/>
    <module name="AvoidStarImport"/>
  </module>
</module>
"#;
    let motivos = motivos_do_lint(
        &[("config/checkstyle.xml", checkstyle)],
        json!("config/checkstyle.xml"),
        &[
            "ParameterNumber",
            "MagicNumber",
            "NestedIfDepth",
            "IllegalCatch",
            "AvoidStarImport",
            "TreeWalker",
            "Number",
        ],
    );
    assert_eq!(
        motivos,
        vec![
            "a regra de lint \"MagicNumber\" só aparece comentada",
            "a regra de lint \"NestedIfDepth\" está configurada como severity \"ignore\", que não bloqueia",
            "a regra de lint \"Number\" não está configurada",
        ]
    );
}

#[test]
fn lint_cobre_pmd_aceita_ref_e_respeita_exclude() {
    let pmd = r#"<ruleset name="casa">
  <rule ref="category/java/design.xml/ExcessiveParameterList"/>
  <rule ref="category/java/errorprone.xml">
    <exclude name="AvoidCatchingGenericException"/>
  </rule>
  <!-- <rule ref="category/java/design.xml/GodClass"/> -->
  <rule ref="category/java/design.xml/CognitiveComplexityX"/>
</ruleset>
"#;
    let motivos = motivos_do_lint(
        &[("pmd.xml", pmd)],
        json!("pmd.xml"),
        &["ExcessiveParameterList", "AvoidCatchingGenericException", "GodClass", "CognitiveComplexity"],
    );
    assert_eq!(
        motivos,
        vec![
            "a regra de lint \"AvoidCatchingGenericException\" está configurada como <exclude>, que não bloqueia",
            "a regra de lint \"GodClass\" só aparece comentada",
            "a regra de lint \"CognitiveComplexity\" não está configurada",
        ]
    );
}

#[test]
fn lint_cobre_error_prone_exige_error() {
    let pom = r#"<project><build><plugins><plugin><configuration><compilerArgs>
  <arg>-Xplugin:ErrorProne -Xep:EmptyCatch:ERROR -Xep:NullAway:ERROR -Xep:UnusedVariable:WARN -Xep:MissingOverride -XepOpt:NullAway:OnlyNullMarked=true</arg>
  <!-- <arg>-Xep:CatchAndPrintStackTrace:ERROR</arg> -->
</compilerArgs></configuration></plugin></plugins></build></project>
"#;
    let motivos = motivos_do_lint(
        &[("pom.xml", pom)],
        json!("pom.xml"),
        &[
            "EmptyCatch",
            "-Xep:EmptyCatch:ERROR",
            "NullAway:ERROR",
            "UnusedVariable",
            "MissingOverride",
            "CatchAndPrintStackTrace",
            "Empty",
        ],
    );
    assert_eq!(
        motivos,
        vec![
            "a regra de lint \"UnusedVariable\" está configurada como -Xep:UnusedVariable:WARN, que não bloqueia",
            "a regra de lint \"MissingOverride\" está configurada como -Xep:MissingOverride sem :ERROR, que não bloqueia",
            "a regra de lint \"CatchAndPrintStackTrace\" só aparece comentada",
            "a regra de lint \"Empty\" não está configurada",
        ]
    );
}

#[test]
fn lint_cobre_json_com_comentarios_e_valor_desligado() {
    let tsconfig = "{\n  // base da casa\n  \"compilerOptions\": {\n    \"strict\": true,\n    \"noImplicitAny\": false,\n    \
        \"url\": \"https://x//y\",\n    /* \"noUncheckedIndexedAccess\": true, */\n    \"exactOptionalPropertyTypes\": true,\n  },\n}\n";
    let eslintrc = r#"{ "rules": { "max-depth": ["error", 3], "max-params": "off", "complexity": 0, "no-console": ["off"], "@typescript-eslint/no-explicit-any": "error" } }"#;
    let motivos = motivos_do_lint(
        &[("tsconfig.json", tsconfig), (".eslintrc.json", eslintrc)],
        json!(["tsconfig.json", ".eslintrc.json"]),
        &[
            "strict",
            "noImplicitAny",
            "noUncheckedIndexedAccess",
            "exactOptionalPropertyTypes",
            "max-depth",
            "max-params",
            "complexity",
            "no-console",
            "@typescript-eslint/no-explicit-any",
            "no-explicit-any",
        ],
    );
    assert_eq!(
        motivos,
        vec![
            "a regra de lint \"noImplicitAny\" está configurada como false, que não bloqueia",
            "a regra de lint \"noUncheckedIndexedAccess\" só aparece comentada",
            "a regra de lint \"max-params\" está configurada como \"off\", que não bloqueia",
            "a regra de lint \"complexity\" está configurada como 0, que não bloqueia",
            "a regra de lint \"no-console\" está configurada como \"off\", que não bloqueia",
            "a regra de lint \"no-explicit-any\" não está configurada",
        ]
    );
}

#[test]
fn lint_cobre_outros_arquivos_por_identificador_fora_de_comentario() {
    let eslint = "import tseslint from \"typescript-eslint\";\n// \"no-console\": \"error\",\n/* \"eqeqeq\": \"error\" */\n\
        export default [...tseslint.configs.strictTypeChecked, { rules: {\n  \"max-depth\": [\"error\", 3],\n  \"max-params\": \"off\",\n  \
        'complexity': 'off',\n  \"max-lines-per-function\": [\"error\", 40],\n  \"url\": \"https://exemplo/no-console\",\n} }];\n";
    let motivos = motivos_do_lint(
        &[("eslint.config.js", eslint)],
        json!("eslint.config.js"),
        &[
            "max-depth",
            "max-params",
            "complexity",
            "max-lines",
            "no-console",
            "eqeqeq",
            "strictTypeChecked",
        ],
    );
    assert_eq!(
        motivos,
        vec![
            "a regra de lint \"max-params\" está configurada como \"off\", que não bloqueia",
            "a regra de lint \"complexity\" está configurada como 'off', que não bloqueia",
            "a regra de lint \"max-lines\" não está configurada",
            "a regra de lint \"no-console\" só aparece comentada",
            "a regra de lint \"eqeqeq\" só aparece comentada",
        ]
    );
}

#[test]
fn lint_cobre_basta_um_arquivo_cobrir() {
    let motivos = motivos_do_lint(
        &[
            ("Cargo.toml", "[lints.clippy]\nunwrap_used = \"allow\"\n"),
            ("outro/Cargo.toml", "[lints.clippy]\nunwrap_used = \"deny\"\n"),
        ],
        json!(["Cargo.toml", "outro/Cargo.toml"]),
        &["unwrap_used"],
    );
    assert!(motivos.is_empty(), "{motivos:?}");
}

// ---------- teste-de-contrato-por-port no layout Maven/Gradle ----------

#[test]
fn teste_de_contrato_por_port_no_layout_maven() {
    let violacoes = rodar_uma_regra(
        &[
            ("src/main/java/br/x/dominio/portas/Pedidos.java", "public interface Pedidos {}\n"),
            ("src/test/java/br/x/dominio/portas/PedidosContratoTest.java", "class PedidosContratoTest {}\n"),
            ("src/main/java/br/x/dominio/portas/Cobranca.java", "public interface Cobranca {}\n"),
            ("src/main/java/br/x/dominio/portas/CobrancaContratoTest.java", "class CobrancaContratoTest {}\n"),
            ("modulo/src/main/kotlin/Estoque.kt", "interface Estoque\n"),
            ("modulo/src/test/kotlin/EstoqueContratoTest.kt", "class EstoqueContratoTest\n"),
        ],
        json!({
            "tipo": "teste-de-contrato-por-port",
            "ports": ["src/main/java/**/portas/*.java", "modulo/src/main/kotlin/*.kt"],
            "ignorar": ["**/*Test.java"],
            "contrato": "{dir_de_teste}/{nome}ContratoTest.{ext}"
        }),
        "L",
    );
    // "{ext}" não é marcador: fica literal e nenhum contrato existe com ele.
    assert_eq!(violacoes.len(), 3);
    let violacoes = rodar_uma_regra(
        &[
            ("src/main/java/br/x/dominio/portas/Pedidos.java", "public interface Pedidos {}\n"),
            ("src/test/java/br/x/dominio/portas/PedidosContratoTest.java", "class PedidosContratoTest {}\n"),
            ("src/main/java/br/x/dominio/portas/Cobranca.java", "public interface Cobranca {}\n"),
            ("src/main/java/br/x/dominio/portas/CobrancaContratoTest.java", "class CobrancaContratoTest {}\n"),
        ],
        json!({
            "tipo": "teste-de-contrato-por-port",
            "ports": "src/main/java/**/portas/*.java",
            "ignorar": ["**/*Test.java"],
            "contrato": "{dir_de_teste}/{nome}ContratoTest.java"
        }),
        "L",
    );
    assert_eq!(
        violacoes.into_iter().map(|violacao| (violacao.0.unwrap(), violacao.2)).collect::<Vec<_>>(),
        vec![(
            "src/main/java/br/x/dominio/portas/Cobranca.java".to_string(),
            "port sem teste de contrato em src/test/java/br/x/dominio/portas/CobrancaContratoTest.java".to_string()
        )]
    );
}

#[test]
fn teste_de_contrato_dir_de_teste_sem_src_main_fica_igual_ao_dir() {
    let violacoes = rodar_uma_regra(
        &[
            ("src/pagamento/portas/cobranca.ts", "export interface Cobranca {}\n"),
            ("src/pagamento/portas/cobranca.contrato.test.ts", ""),
            ("modulo/src/main/kotlin/Estoque.kt", "interface Estoque\n"),
            ("modulo/src/test/kotlin/EstoqueContratoTest.kt", ""),
            ("xsrc/main/Caixa.kt", "interface Caixa\n"),
        ],
        json!({ "tipo": "teste-de-contrato-por-port", "ports": ["src/**/portas/*.ts", "modulo/**/*.kt", "xsrc/**/*.kt"], "ignorar": ["**/*.test.*", "**/*Test.kt"], "contrato": "{dir_de_teste}/{nome}.contrato.test.ts" }),
        "L",
    );
    let mensagens: Vec<String> = violacoes.into_iter().map(|violacao| violacao.2).collect();
    assert_eq!(
        mensagens,
        vec![
            "port sem teste de contrato em modulo/src/test/kotlin/Estoque.contrato.test.ts",
            "port sem teste de contrato em xsrc/main/Caixa.contrato.test.ts"
        ]
    );
}

// ---------- termo-evitado em Rust ----------

#[test]
fn termo_evitado_em_rust_ve_mut_static_mod_e_as_outras_declaracoes() {
    let violacoes = rodar_uma_regra(
        &[
            ("GLOSSARY.md", "**Cliente**:\nQuem compra.\n_Evitar_: Usuario\n"),
            (
                "src/lib.rs",
                "let mut usuario_novo = 1;\nstatic USUARIO_PADRAO: u8 = 0;\nstatic mut USUARIO_GLOBAL: u8 = 0;\nmod usuario_repo;\npub(crate) const USUARIO_MAX: u8 = 9;\n\
                 fn criar_usuario() {}\nstruct UsuarioX;\nenum TipoDeUsuario {}\ntrait UsuarioRepo {}\ntype UsuarioId = u64;\nunion UsuarioBits { a: u8 }\n\
                 let r#usuario_bruto = 2;\nlet cliente = 1;\nlet mut cliente_novo = usuario();\nmacro_rules! usuario_macro { () => {} }\n",
            ),
        ],
        json!({ "tipo": "termo-evitado" }),
        "E1",
    );
    assert_eq!(
        violacoes.iter().map(|violacao| violacao.1.unwrap()).collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 15]
    );
}

#[test]
fn termo_evitado_em_dart_ve_final_tipado_late_mixin_e_extension() {
    let violacoes = rodar_uma_regra(
        &[
            ("GLOSSARY.md", "_Evitar_: Usuario\n"),
            (
                "src/a.dart",
                "class Conta extends UsuarioBase {}\nfinal usuarioAtivo = 1;\nfinal Map<String, int> usuarioPorId = {};\n\
                 late final String nomeDoUsuario;\nmixin UsuarioMixin {}\nextension type UsuarioId(int valor) {}\nconst limite = 3;\n",
            ),
        ],
        json!({ "tipo": "termo-evitado" }),
        "E1",
    );
    let linhas: Vec<u64> = violacoes.into_iter().map(|violacao| violacao.1.unwrap()).collect();
    // "class Conta extends UsuarioBase" declara Conta: o tipo opcional só vale depois de final/const/var/late.
    assert_eq!(linhas, vec![2, 3, 4, 5, 6]);
}

#[test]
fn termo_evitado_em_ts_java_e_python_segue_igual() {
    let violacoes = rodar_uma_regra(
        &[
            ("GLOSSARY.md", "_Evitar_: Usuario, void, final\n"),
            (
                "src/a.ts",
                "let usuarioAtivo = 1;\nlet mut = 2;\nclass Usuario {}\nstatic readonly usuarioFixo = 1;\nconst x = usuario;\n",
            ),
            (
                "src/A.java",
                "public static void criarUsuario() {}\nprivate static final int USUARIO_MAX = 1;\nrecord UsuarioDto() {}\n",
            ),
            ("src/b.py", "def criar_usuario():\n    pass\nclass UsuarioRepo:\n    pass\n"),
        ],
        json!({ "tipo": "termo-evitado" }),
        "E1",
    );
    let achados: Vec<String> = violacoes
        .into_iter()
        .map(|violacao| format!("{}:{}", violacao.0.unwrap(), violacao.1.unwrap()))
        .collect();
    // Como antes: "static" não declara nome fora de Rust (nem "void" nem "final" viram nome), e "let mut = 2"
    // em JS declara "mut", que não é termo evitado.
    assert_eq!(achados, vec!["src/A.java:3", "src/a.ts:1", "src/a.ts:3", "src/b.py:1", "src/b.py:3"]);
}

// ---------- práticas da linguagem fora dos 22 ----------

fn tabela_com_praticas(linhas_dos_22: &[(&str, &str)], praticas: &str) -> String {
    format!(
        "{}\n## Práticas da linguagem\n\n| Prática | Decisão | Onde ou por quê |\n|---|---|---|\n{praticas}",
        tabela_com(linhas_dos_22)
    )
}

#[test]
fn pratica_da_linguagem_aceita_com_linha_na_tabela_de_praticas() {
    let regras = regras_de(json!([
        { "id": "R1", "principio": "E1", "verificacao": { "tipo": "revisor" } },
        { "id": "R12", "principio": "RS1", "verificacao": { "tipo": "revisor" } },
        { "id": "R13", "principio": "TS2", "verificacao": { "tipo": "revisor" } },
        { "id": "R14", "principio": "JV10", "verificacao": { "tipo": "revisor" } }
    ]));
    let texto_md = tabela_com_praticas(
        &[("E1", "adotado | R1")],
        "| RS1 | adotado | R12 |\n| TS2 | garantido pelo lint | R13 |\n| JV10 | adaptado | R14 |\n| RS9 | afastado | sem regra, não exigida |\n",
    );
    assert!(checar_cobertura(&texto_md, &regras).is_empty(), "{:?}", checar_cobertura(&texto_md, &regras));
}

#[test]
fn pratica_sem_linha_ou_citada_so_nos_22_reprova() {
    let regras = regras_de(json!([
        { "id": "R1", "principio": "E1", "verificacao": { "tipo": "revisor" } },
        { "id": "R12", "principio": "RS1", "verificacao": { "tipo": "revisor" } }
    ]));
    let sem_linha: Vec<String> = checar_cobertura(&tabela_com(&[("E1", "adotado | R1")]), &regras)
        .into_iter()
        .map(|violacao| violacao.mensagem)
        .collect();
    assert_eq!(sem_linha, vec!["R12 nasce da prática RS1 e nenhuma linha da tabela de práticas a cita"]);
    let so_nos_22: Vec<String> = checar_cobertura(&tabela_com(&[("E1", "adotado | R1, R12")]), &regras)
        .into_iter()
        .map(|violacao| violacao.mensagem)
        .collect();
    assert_eq!(
        so_nos_22,
        vec![
            "R12 nasce de RS1 no .json, mas a tabela a cita em E1",
            "R12 nasce da prática RS1 e nenhuma linha da tabela de práticas a cita"
        ]
    );
    let decisao_ruim: Vec<String> = checar_cobertura(&tabela_com_praticas(&[("E1", "adotado | R1")], "| RS1 | talvez | R12 |\n"), &regras)
        .into_iter()
        .map(|violacao| violacao.mensagem)
        .collect();
    assert_eq!(
        decisao_ruim,
        vec![
            "RS1: decisão \"talvez\" não é uma de: adotado, adaptado, afastado, garantido pelo lint",
            "R12 nasce da prática RS1 e nenhuma linha da tabela de práticas a cita"
        ]
    );
}

#[test]
fn id_de_pratica_invalido_e_erro_de_configuracao() {
    let repositorio = Repositorio::novo(&[]);
    for principio in ["XX1", "RS", "rs1", "RS1a"] {
        let config = json!({ "versao": 1, "regras": [{ "id": "R1", "principio": principio, "verificacao": { "tipo": "revisor" } }] });
        let Err(erro) = validar(&repositorio.raiz, &config, "", &opcoes(None)) else {
            panic!("{principio} deveria ser erro de configuração")
        };
        assert!(erro.0.contains("TS<n>, JV<n>, RS<n> ou DT<n>"), "{}", erro.0);
    }
}

// ---------- mensagem do ast-grep ausente ----------

#[test]
fn ast_grep_ausente_sugere_versao_fixa() {
    let violacoes = rodar_uma_regra(
        &[("regras/r.yml", "id: x\n"), ("src/a.ts", "a;\n")],
        json!({ "tipo": "ast-grep", "regras": "regras/r.yml", "binario": "ast-grep-que-nao-existe" }),
        "D",
    );
    assert_eq!(violacoes.len(), 1);
    assert!(
        violacoes[0].2.contains("npm i -g @ast-grep/cli@0.51.0") && violacoes[0].2.contains("cargo install ast-grep --version 0.51.0 --locked"),
        "{}",
        violacoes[0].2
    );
}

// ---------- P5: todo teste cobre um risco do registro ----------

const REGISTRO: &str = "# Riscos\n\n| Risco | O que dá errado | Dano | Onde | Origem | Situação |\n|---|---|---|---|---|---|\n\
| RK1 | Cobrar duas vezes o mesmo pedido | estorno e cliente perdido | pagamento | #12 | |\n\
| RK2 | Cupom vencido aceito | margem perdida | cupom | docs/adr/0003-cupom.md | |\n\
| RK3 | Mesa fechada recebe pedido | pedido perdido | comanda | #15 | pendente #31 |\n";

fn riscos_cobertos(arquivos: &[(&str, &str)]) -> Vec<String> {
    rodar_uma_regra(arquivos, json!({ "tipo": "teste-cobre-risco" }), "P5")
        .into_iter()
        .map(|(arquivo, linha, mensagem)| format!("{}:{} {mensagem}", arquivo.unwrap_or_default(), linha.unwrap_or(0)))
        .collect()
}

#[test]
fn teste_cobre_risco_aceita_risco_no_nome_no_comentario_e_no_bloco() {
    let violacoes = riscos_cobertos(&[
        ("docs/riscos.md", REGISTRO),
        (
            "src/pagamento.test.ts",
            "describe(\"RK1: cobrança única\", () => {\n  it(\"cobra uma vez por pedido\", () => {\n    expect(cobrar(pedido)).toBe(1);\n  });\n});\n",
        ),
        (
            "src/cupom.rs",
            "fn aplicar() {}\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn rk2_recusa_cupom_vencido() {\n        assert!(aplicar().is_err());\n    }\n}\n",
        ),
        (
            "src/test/java/CupomTest.java",
            "class CupomTest {\n  // RK2: cupom vencido\n  @Test\n  void recusaVencido() { assertThrows(Erro.class, () -> aplicar()); }\n}\n",
        ),
    ]);
    assert!(violacoes.is_empty(), "{violacoes:?}");
}

#[test]
fn teste_cobre_risco_le_test_e_testwidgets_de_dart() {
    let violacoes = riscos_cobertos(&[
        ("docs/riscos.md", REGISTRO),
        (
            "test/cupom_test.dart",
            "void main() {\n  group('RK2: cupom vencido', () {\n    test('recusa', () {\n      expect(aplicar(), isFalse);\n    });\n  });\n  \
             // RK1\n  testWidgets('cobra uma vez', (tester) async {\n    expect(find.text('1'), findsOneWidget);\n  });\n  \
             testWidgets('mostra o total', (tester) async {\n    expect(find.text('2'), findsOneWidget);\n  });\n  \
             goldenTest(\n    'RK1 recibo',\n    fileName: 'recibo',\n    builder: () => const Recibo(),\n  );\n}\n",
        ),
    ]);
    assert_eq!(
        violacoes,
        vec!["test/cupom_test.dart:11 o teste não cita o risco que cobre (RK<n> de docs/riscos.md); sem risco, o teste não entra"]
    );
}

#[test]
fn teste_cobre_risco_reprova_teste_sem_risco_e_risco_inexistente() {
    let violacoes = riscos_cobertos(&[
        ("docs/riscos.md", REGISTRO),
        (
            "src/pagamento.test.ts",
            "// RK1\nit(\"cobra uma vez\", () => { expect(cobrar()).toBe(1); });\nit(\"soma\", () => { expect(somar(1, 2)).toBe(3); });\nit(\"RK9 algo\", () => { expect(1).toBe(1); });\n",
        ),
        ("src/cupom.rs", "#[test]\nfn rk2_recusa() { assert!(true); }\n"),
    ]);
    assert_eq!(
        violacoes,
        vec![
            "src/pagamento.test.ts:3 o teste não cita o risco que cobre (RK<n> de docs/riscos.md); sem risco, o teste não entra",
            "src/pagamento.test.ts:4 o teste cita RK9, que não está em docs/riscos.md",
        ]
    );
}

#[test]
fn teste_cobre_risco_exige_teste_origem_e_issue_da_pendencia() {
    let registro = "| RK1 | Cobrar duas vezes | estorno | pagamento | #12 | |\n| RK2 | Cupom vencido | margem | cupom | o time acha | |\n\
                    | RK3 | Mesa fechada | pedido perdido | comanda | #15 | pendente |\n| RK1 | Repetido | x | y | #1 | |\n";
    let violacoes = riscos_cobertos(&[
        ("docs/riscos.md", registro),
        ("src/cupom.test.ts", "it(\"RK2 recusa\", () => { expect(aplicar()).toBe(false); });\n"),
    ]);
    assert_eq!(
        violacoes,
        vec![
            "docs/riscos.md:2 RK2 sem origem: cite a issue, o ticket ou a ADR em que o humano aprovou o risco",
            "docs/riscos.md:3 RK3 pendente sem a issue que vai cobri-lo (pendente #<n>)",
            "docs/riscos.md:4 RK1 aparece duas vezes no registro",
            "docs/riscos.md:1 RK1 não tem teste que o cite; escreva o teste ou marque pendente #<issue que vai cobri-lo>",
            "docs/riscos.md:4 RK1 não tem teste que o cite; escreva o teste ou marque pendente #<issue que vai cobri-lo>",
        ]
    );
    assert_eq!(
        riscos_cobertos(&[("src/a.test.ts", "it(\"RK1\", () => {});\n")]),
        vec![":0 registro de riscos docs/riscos.md não existe (docs/agents/riscos.md diz como nasce)"]
    );
}

#[test]
fn teste_cobre_risco_nao_confunde_rk_no_meio_da_palavra() {
    assert_eq!(
        watchdogs::riscos::riscos_citados("TestRK3Recusa rk4_cobra (RK5) work1 MARK2 frk6"),
        vec!["RK3", "RK4", "RK5"]
    );
}

#[test]
fn teste_cobre_risco_com_base_confere_so_os_testes_alterados_e_o_registro_inteiro() {
    let repositorio = Repositorio::novo(&[
        ("docs/riscos.md", REGISTRO),
        (
            "src/antigo.test.ts",
            "it(\"sem risco, de antes\", () => { expect(somar(1, 2)).toBe(3); });\nit(\"RK2\", () => { expect(aplicar()).toBe(false); });\n",
        ),
    ]);
    repositorio.escrever("src/novo.test.ts", "it(\"novo sem risco\", () => { expect(somar(1, 2)).toBe(3); });\n");
    repositorio.commit("test: novo");
    let config = json!({ "versao": 1, "projeto": { "codigo": ["src/**"] }, "regras": [{ "id": "R1", "principio": "P5", "verificacao": { "tipo": "teste-cobre-risco" } }] });
    let violacoes: Vec<String> = violacoes_da(&repositorio.raiz, &config, &tabela_com(&[("P5", "adotado | R1")]), Some("HEAD~1"), "R1")
        .into_iter()
        .map(|(arquivo, linha, _)| format!("{}:{}", arquivo.unwrap_or_default(), linha.unwrap_or(0)))
        .collect();
    assert_eq!(violacoes, vec!["src/novo.test.ts:1", "docs/riscos.md:5"]);
}

#[test]
fn cobertura_reprova_p5_afastado_ou_sem_a_checagem_de_risco() {
    let regras = regras_de(json!([
        { "id": "R1", "principio": "E1", "verificacao": { "tipo": "revisor" } },
        { "id": "R2", "principio": "P5", "verificacao": { "tipo": "revisor" } }
    ]));
    let mensagens = |texto_md: String| {
        checar_cobertura(&texto_md, &regras)
            .into_iter()
            .map(|violacao| violacao.mensagem)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        mensagens(tabela_com(&[("E1", "adotado | R1"), ("P5", "afastado | o time não testa")])),
        vec![
            "P5 é regra dura da constituição: adapte, mas não afaste",
            "R2 está no .json e nenhuma linha da tabela de cobertura a cita"
        ]
    );
    assert_eq!(
        mensagens(tabela_com(&[("E1", "adotado | R1"), ("P5", "adotado | R2")])),
        vec!["P5 precisa citar uma regra com a checagem teste-cobre-risco (regra da constituição)"]
    );
}

#[test]
fn ast_grep_modelos_de_teste_acham_teste_sem_valor() {
    if !ast_grep_disponivel() {
        eprintln!("ast-grep não instalado: teste pulado");
        return;
    }
    let repositorio = Repositorio::novo(&[
        ("regras/sem.yml", &preencher_modelo("teste-sem-afirmacao.yml", &[])),
        ("regras/trivial.yml", &preencher_modelo("afirmacao-trivial.yml", &[])),
        ("regras/desligado.yml", &preencher_modelo("teste-desligado.yml", &[])),
        (
            "src/cobranca.test.ts",
            "describe(\"RK1\", () => {\n  it(\"cobra\", () => { expect(cobrar(10)).toBe(10); });\n  it(\"sem afirmação\", () => { cobrar(10); });\n  \
             it(\"trivial\", () => { expect(total).toBe(total); expect(true).toBeTruthy(); });\n  it.skip(\"desligado\", () => { expect(cobrar(1)).toBe(1); });\n  \
             test.each([[1, 2]])(\"dobra %i\", (entrada, saida) => { expect(dobrar(entrada)).toBe(saida); });\n});\n",
        ),
        (
            "src/cupom.rs",
            "#[cfg(test)]\nmod tests {\n    #[test]\n    fn rk2_recusa() { assert!(aplicar().is_err()); }\n    #[test]\n    fn rk2_sem() { aplicar(); }\n    \
             #[test]\n    #[should_panic(expected = \"vencido\")]\n    fn rk2_entra_em_panico() { aplicar(); }\n    #[test]\n    fn rk2_trivial() { assert_eq!(total, total); }\n    \
             #[test]\n    #[ignore]\n    fn rk2_desligado() { assert!(aplicar().is_err()); }\n    fn auxiliar() { aplicar(); }\n}\n",
        ),
        (
            "src/test/java/CupomTest.java",
            "class CupomTest {\n  @Test\n  void recusa() { assertThrows(Erro.class, () -> aplicar()); }\n  @Test\n  void sem() { aplicar(); }\n  \
             @Test\n  void trivial() { assertTrue(true); }\n  @Disabled\n  @Test\n  void desligado() { verify(gateway).cobrar(1); }\n}\n",
        ),
    ]);
    let config = json!({ "versao": 1, "regras": [
        { "id": "R1", "principio": "P5", "verificacao": { "tipo": "ast-grep", "regras": "regras/sem.yml" } },
        { "id": "R2", "principio": "P5", "verificacao": { "tipo": "ast-grep", "regras": "regras/trivial.yml" } },
        { "id": "R3", "principio": "P5", "verificacao": { "tipo": "ast-grep", "regras": "regras/desligado.yml" } }
    ] });
    let texto_md = tabela_com(&[("P5", "adotado | R1, R2, R3")]);
    let achados = |id: &str| {
        violacoes_da(&repositorio.raiz, &config, &texto_md, None, id)
            .into_iter()
            .map(|violacao| format!("{}:{}", violacao.0.unwrap(), violacao.1.unwrap()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        achados("R1"),
        vec!["src/cobranca.test.ts:3", "src/cupom.rs:6", "src/test/java/CupomTest.java:4"]
    );
    assert_eq!(
        achados("R2"),
        vec![
            "src/cobranca.test.ts:4",
            "src/cobranca.test.ts:4",
            "src/cupom.rs:11",
            "src/test/java/CupomTest.java:7"
        ]
    );
    assert_eq!(
        achados("R3"),
        vec!["src/cobranca.test.ts:5", "src/cupom.rs:13", "src/test/java/CupomTest.java:8"]
    );
}

// ---------- revisão por modelo: consultiva e fora da cobertura (ADR 0015) ----------

// Risco: a revisão por modelo vira portão sem o piloto medido, ou passa a contar como a checagem do P5.
#[test]
fn revisao_por_modelo_so_entra_consultiva_e_fora_da_cobertura() {
    let sem_consultivo = json!({ "versao": 1, "regras": [{ "id": "R91", "principio": "P5", "verificacao": { "tipo": "revisao-por-modelo" } }] });
    let erro = ler_regras(&sem_consultivo).err().expect("sem consultivo é configuração inválida").0;
    assert!(erro.contains("R91") && erro.contains("\"consultivo\": true"), "{erro}");

    let revisao = json!({ "id": "R91", "principio": "P5", "verificacao": { "tipo": "revisao-por-modelo", "modelo": "claude-haiku-5-5", "consultivo": true } });
    let regras = regras_de(json!([
        { "id": "R1", "principio": "E1", "verificacao": { "tipo": "revisor" } },
        { "id": "R90", "principio": "P5", "verificacao": { "tipo": "teste-cobre-risco" } },
        revisao
    ]));
    assert!(
        checar_cobertura(&tabela_com(&[("E1", "adotado | R1")]), &regras).is_empty(),
        "a regra consultiva não precisa de linha na tabela"
    );
    let violacoes = checar_cobertura(&tabela_com(&[("E1", "adotado | R1"), ("P5", "adotado | R91")]), &regras);
    assert!(
        violacoes
            .iter()
            .any(|violacao| violacao.mensagem.contains("P5 precisa citar uma regra com a checagem teste-cobre-risco")),
        "a revisão por modelo não substitui o teste-cobre-risco: {violacoes:?}"
    );
}
