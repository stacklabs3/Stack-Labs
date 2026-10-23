//! Os argumentos e, quando há um Chrome em CHROME=<caminho>, os prints de verdade contra o
//! protótipo mínimo de tests/prototipo-minimo (servido aqui mesmo, numa porta livre).
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;

fn tirar_prints(argumentos: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tirar-prints"))
        .args(argumentos)
        .output()
        .unwrap()
}

fn texto(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn sem_argumentos_mostra_o_uso() {
    let resultado = tirar_prints(&[]);
    assert_eq!(resultado.status.code(), Some(2));
    assert!(texto(&resultado.stderr).starts_with("Uso: "));
}

#[test]
fn argumentos_demais_mostram_o_uso() {
    let resultado = tirar_prints(&["http://x", "a", "b", "c", "d", "e"]);
    assert_eq!(resultado.status.code(), Some(2));
}

#[test]
fn argumento_vazio_conta_como_ausente() {
    let resultado = tirar_prints(&["http://x", "", "b"]);
    assert_eq!(resultado.status.code(), Some(2));
}

/// Serve o index.html do protótipo mínimo para qualquer caminho e devolve o endereço.
fn servir_prototipo() -> String {
    let pagina = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/prototipo-minimo/index.html"),
    )
    .unwrap();
    let servidor = TcpListener::bind("127.0.0.1:0").unwrap();
    let endereco = format!("http://127.0.0.1:{}", servidor.local_addr().unwrap().port());
    thread::spawn(move || {
        for conexao in servidor.incoming().flatten() {
            let pagina = pagina.clone();
            thread::spawn(move || {
                let mut conexao = conexao;
                let mut pedido = Vec::new();
                let mut pedaco = [0u8; 1024];
                while !pedido.windows(4).any(|janela| janela == b"\r\n\r\n") {
                    match conexao.read(&mut pedaco) {
                        Ok(0) | Err(_) => return,
                        Ok(quantidade) => pedido.extend_from_slice(&pedaco[..quantidade]),
                    }
                }
                let cabecalho = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    pagina.len()
                );
                let _ = conexao.write_all(cabecalho.as_bytes());
                let _ = conexao.write_all(&pagina);
            });
        }
    });
    endereco
}

/// Largura e altura do PNG, lidas do cabeçalho IHDR.
fn tamanho_do_png(caminho: &Path) -> (u32, u32) {
    let bytes = std::fs::read(caminho).unwrap();
    assert_eq!(
        &bytes[..8],
        b"\x89PNG\r\n\x1a\n",
        "{} não é PNG",
        caminho.display()
    );
    let numero = |inicio: usize| u32::from_be_bytes(bytes[inicio..inicio + 4].try_into().unwrap());
    (numero(16), numero(20))
}

fn pasta_temporaria(nome: &str) -> PathBuf {
    let pasta = std::env::temp_dir().join(format!("tirar-prints-{}-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&pasta);
    pasta
}

#[test]
fn tira_os_prints_com_o_chrome() {
    if std::env::var_os("CHROME").is_none() {
        eprintln!("CHROME não definido: o teste com o Chrome foi pulado.");
        return;
    }
    let endereco = servir_prototipo();
    let pasta = pasta_temporaria("todas");
    let pasta_texto = pasta.to_str().unwrap();
    let resultado = tirar_prints(&[&format!("{endereco}/"), "todas", pasta_texto, "en", "3"]);
    assert_eq!(
        resultado.status.code(),
        Some(0),
        "{}",
        texto(&resultado.stderr)
    );
    assert_eq!(
        texto(&resultado.stderr),
        format!("6 prints em {pasta_texto}\n")
    );
    let esperados = [
        (
            "turno-do-caixa/01-abrir--celular--light--default.png",
            (438, 892),
        ),
        (
            "turno-do-caixa/01-abrir--celular--dark--default.png",
            (438, 892),
        ),
        (
            "turno-do-caixa/02-fechar--computador--light--default.png",
            (1280, 800),
        ),
        (
            "turno-do-caixa/02-fechar--computador--dark--default.png",
            (1280, 800),
        ),
        (
            "cadastro/opcao-Q1-B/01-novo--tablet--light--default.png",
            (868, 1228),
        ),
        (
            "cadastro/opcao-Q1-B/01-novo--tablet--dark--default.png",
            (868, 1228),
        ),
    ];
    let mut linhas: Vec<String> = texto(&resultado.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    linhas.sort();
    let mut linhas_esperadas: Vec<String> = esperados
        .iter()
        .map(|(arquivo, _)| format!("{pasta_texto}/{arquivo}"))
        .collect();
    linhas_esperadas.sort();
    assert_eq!(linhas, linhas_esperadas);
    for (arquivo, tamanho) in esperados {
        assert_eq!(tamanho_do_png(&pasta.join(arquivo)), tamanho, "{arquivo}");
    }
}

#[test]
fn rotina_que_nao_existe_com_o_chrome() {
    if std::env::var_os("CHROME").is_none() {
        return;
    }
    let endereco = servir_prototipo();
    let pasta = pasta_temporaria("nenhum");
    let resultado = tirar_prints(&[&endereco, "nao-existe", pasta.to_str().unwrap()]);
    assert_eq!(resultado.status.code(), Some(1));
    assert_eq!(
        texto(&resultado.stderr),
        "Nenhum print para \"nao-existe\". O protótipo está servido e a rotina existe em visoes.jsx?\n"
    );
    assert!(!pasta.exists());
}
