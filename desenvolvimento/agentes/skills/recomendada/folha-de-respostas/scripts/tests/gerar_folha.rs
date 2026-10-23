//! Os casos de tests/casos: cada .html e cada .txt foi gerado pelo gerar-folha.mjs original,
//! e o binário precisa sair igual, byte a byte.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn casos() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/casos")
}

fn pasta_temporaria(nome: &str) -> PathBuf {
    let pasta = std::env::temp_dir().join(format!("gerar-folha-{}-{nome}", std::process::id()));
    std::fs::create_dir_all(&pasta).unwrap();
    pasta
}

fn gerar(argumentos: &[&Path]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_gerar-folha"))
        .args(argumentos)
        .output()
        .unwrap()
}

fn confere_folha(nome: &str, total: usize) {
    let saida = pasta_temporaria(nome).join("folha.html");
    let resultado = gerar(&[&casos().join(format!("{nome}.json")), &saida]);
    assert!(resultado.status.success());
    assert_eq!(
        String::from_utf8(resultado.stdout).unwrap(),
        format!("Folha gerada: {} ({total} perguntas)\n", saida.display())
    );
    let esperado = std::fs::read(casos().join(format!("{nome}.html"))).unwrap();
    assert!(
        std::fs::read(&saida).unwrap() == esperado,
        "{nome}: o HTML difere do gerado pelo Node"
    );
}

fn confere_recusa(nome: &str) {
    let saida = pasta_temporaria(nome).join("folha.html");
    let resultado = gerar(&[&casos().join(format!("{nome}.json")), &saida]);
    assert_eq!(resultado.status.code(), Some(1));
    assert!(resultado.stdout.is_empty());
    let esperado = std::fs::read_to_string(casos().join(format!("{nome}.txt"))).unwrap();
    assert_eq!(String::from_utf8(resultado.stderr).unwrap(), esperado);
    assert!(!saida.exists());
}

#[test]
fn exemplo_do_skill_md() {
    confere_folha("exemplo-doc", 1);
}

#[test]
fn escapes_numeros_e_ordem_das_chaves() {
    confere_folha("escapes", 2);
}

#[test]
fn recusa_com_cada_problema() {
    confere_recusa("erros");
}

#[test]
fn recusa_ids_repetidos_e_ausentes() {
    confere_recusa("repetidos");
}

#[test]
fn sem_argumentos_mostra_o_uso() {
    let resultado = gerar(&[]);
    assert_eq!(resultado.status.code(), Some(1));
    assert!(String::from_utf8(resultado.stderr)
        .unwrap()
        .starts_with("Uso: "));
}

#[test]
fn arquivo_que_nao_existe() {
    let pasta = pasta_temporaria("inexistente");
    let resultado = gerar(&[&pasta.join("nao-existe.json"), &pasta.join("folha.html")]);
    assert_eq!(resultado.status.code(), Some(1));
}
