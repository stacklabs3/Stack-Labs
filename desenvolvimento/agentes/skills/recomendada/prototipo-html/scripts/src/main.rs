//! Tira os prints da conferência de uma rotina (ou de todas) do protótipo servido.
//!
//! Uso: sh ~/.claude/skills/prototipo-html/scripts/rodar <endereco> <id-da-rotina|todas> <pasta-de-saida> [idioma] [paralelo]
//!   ex.: sh ~/.claude/skills/prototipo-html/scripts/rodar http://localhost:8765 turno-do-caixa prototipos/meu-app/prints
//! O idioma padrão é pt-BR; o paralelo padrão é 6 abas. CHROME=<caminho> escolhe o navegador.
//!
//! A lista vem do próprio protótipo (?listarPrints=<rotina>). Cada print abre numa aba do
//! Chrome sem janela com a área de tela exata da combinação (CDP), espera a tela montar e
//! fotografa. O --screenshot do Chrome não serve: a janela dele sai 87 px mais baixa e com
//! no mínimo 500 px de largura, o que cortava e deslocava a moldura.
mod cdp;
mod node;

use cdp::Conexao;
use serde_json::{json, Value};
use std::collections::hash_map::RandomState;
use std::collections::VecDeque;
use std::hash::{BuildHasher, Hasher};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{exit, Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const FOLGA_DEPOIS_DE_MONTAR: Duration = Duration::from_millis(1500);
const LIMITE_PARA_MONTAR: Duration = Duration::from_millis(20000);
const CAMINHOS_DO_CHROME: [&str; 5] = [
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/usr/bin/google-chrome",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
];
const USO: &str = "Uso: sh ~/.claude/skills/prototipo-html/scripts/rodar <endereco> <id-da-rotina|todas> <pasta-de-saida> [idioma] [paralelo]";

// A tela está montada quando o palco do print tem conteúdo e as fontes carregaram.
const TELA_MONTADA: &str = "(() => { const palco = document.querySelector('.showcase-body--print'); return !!palco && palco.innerText.trim().length > 0 && document.fonts.status === 'loaded'; })()";

/// O que o script Node fazia num erro inesperado: avisa e sai com 1, sem fechar o Chrome.
fn falhar(mensagem: &str) -> ! {
    eprintln!("[tirar-prints] {mensagem}");
    exit(1);
}

fn dormir(tempo: Duration) {
    thread::sleep(tempo);
}

fn aleatorio() -> u64 {
    let mut misturador = RandomState::new().build_hasher();
    misturador.write_u128(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|tempo| tempo.as_nanos())
            .unwrap_or_default(),
    );
    misturador.finish()
}

/// `mkdtemp(<tmp>/prints-XXXXXX)`.
fn criar_perfil_do_chrome() -> PathBuf {
    const LETRAS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    loop {
        let mut sorteio = aleatorio();
        let sufixo: String = (0..6)
            .map(|_| {
                let letra = LETRAS[(sorteio % LETRAS.len() as u64) as usize] as char;
                sorteio /= LETRAS.len() as u64;
                letra
            })
            .collect();
        let pasta = std::env::temp_dir().join(format!("prints-{sufixo}"));
        match std::fs::create_dir(&pasta) {
            Ok(()) => return pasta,
            Err(erro) if erro.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(erro) => falhar(&format!("não deu para criar {}: {erro}", pasta.display())),
        }
    }
}

/// `chrome.kill()`: SIGTERM, para o Chrome fechar os processos dele.
fn encerrar_chrome(chrome: &mut Child) {
    #[cfg(unix)]
    {
        let terminou = Command::new("kill")
            .args(["-TERM", &chrome.id().to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
        if terminou {
            return;
        }
    }
    let _ = chrome.kill();
}

/// O `webSocketDebuggerUrl` de `GET /json/version`; erro quando o Chrome ainda não responde.
fn pedir_endereco_do_navegador(porta: u16) -> Result<Option<String>, ()> {
    let mut conexao = TcpStream::connect(("127.0.0.1", porta)).map_err(|_| ())?;
    conexao.set_read_timeout(Some(Duration::from_secs(30))).ok();
    write!(
        conexao,
        "GET /json/version HTTP/1.1\r\nHost: 127.0.0.1:{porta}\r\nAccept: */*\r\nConnection: close\r\n\r\n"
    )
    .map_err(|_| ())?;
    let corpo = ler_corpo_http(&mut conexao).map_err(|_| ())?;
    let versao: Value = serde_json::from_slice(&corpo).map_err(|_| ())?;
    Ok(versao
        .get("webSocketDebuggerUrl")
        .and_then(Value::as_str)
        .filter(|endereco| !endereco.is_empty())
        .map(str::to_string))
}

/// O corpo de uma resposta HTTP: até o `Content-Length` (o Chrome não fecha a conexão) ou,
/// sem ele, até o fim da conexão.
fn ler_corpo_http(conexao: &mut impl Read) -> std::io::Result<Vec<u8>> {
    let mut lido = Vec::new();
    let mut pedaco = [0u8; 4096];
    let fim_do_cabecalho = loop {
        if let Some(posicao) = lido.windows(4).position(|janela| janela == b"\r\n\r\n") {
            break posicao + 4;
        }
        let quantidade = conexao.read(&mut pedaco)?;
        if quantidade == 0 {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }
        lido.extend_from_slice(&pedaco[..quantidade]);
    };
    let cabecalho = String::from_utf8_lossy(&lido[..fim_do_cabecalho]).to_ascii_lowercase();
    let tamanho_do_corpo = cabecalho.lines().find_map(|linha| {
        let (nome, valor) = linha.split_once(':')?;
        (nome.trim() == "content-length").then(|| valor.trim().parse::<usize>().ok())?
    });
    let mut corpo = lido.split_off(fim_do_cabecalho);
    match tamanho_do_corpo {
        Some(tamanho) => {
            while corpo.len() < tamanho {
                let quantidade = conexao.read(&mut pedaco)?;
                if quantidade == 0 {
                    return Err(std::io::ErrorKind::UnexpectedEof.into());
                }
                corpo.extend_from_slice(&pedaco[..quantidade]);
            }
            corpo.truncate(tamanho);
        }
        None => {
            conexao.read_to_end(&mut corpo)?;
        }
    }
    Ok(corpo)
}

/// O valor de `resposta.result[campo]`, ou falha como a desestruturação do Node falharia.
fn resultado(resposta: Option<Value>, campo: &str, metodo: &str) -> Value {
    let Some(resposta) = resposta else {
        falhar(&format!("a conexão com o Chrome caiu durante {metodo}."));
    };
    match resposta.get("result") {
        Some(resultado @ Value::Object(_)) => resultado.get(campo).cloned().unwrap_or(Value::Null),
        _ => falhar(&format!("o Chrome recusou {metodo}: {resposta}")),
    }
}

/// `!!valor` do JavaScript.
fn verdadeiro(valor: Option<&Value>) -> bool {
    match valor {
        None | Some(Value::Null) => false,
        Some(Value::Bool(logico)) => *logico,
        Some(Value::Number(numero)) => numero.as_f64().is_some_and(|valor| valor != 0.0),
        Some(Value::String(texto)) => !texto.is_empty(),
        Some(_) => true,
    }
}

struct Prints {
    conexao: Conexao,
    endereco: String,
    idioma: String,
    pasta_de_saida: String,
}

impl Prints {
    fn abrir_aba(&self) -> String {
        let id_da_aba = resultado(
            self.conexao.enviar(
                "Target.createTarget",
                json!({ "url": "about:blank", "newWindow": true }),
                None,
            ),
            "targetId",
            "Target.createTarget",
        );
        let id_da_sessao = resultado(
            self.conexao.enviar(
                "Target.attachToTarget",
                json!({ "targetId": id_da_aba, "flatten": true }),
                None,
            ),
            "sessionId",
            "Target.attachToTarget",
        );
        let Some(id_da_sessao) = id_da_sessao.as_str().map(str::to_string) else {
            falhar("o Chrome não devolveu a sessão da aba.");
        };
        self.conexao
            .enviar("Page.enable", json!({}), Some(&id_da_sessao));
        self.conexao.enviar(
            "Emulation.setFocusEmulationEnabled",
            json!({ "enabled": true }),
            Some(&id_da_sessao),
        );
        id_da_sessao
    }

    fn avaliar(&self, id_da_sessao: &str, expressao: &str) -> Option<Value> {
        let resposta = self.conexao.enviar(
            "Runtime.evaluate",
            json!({ "expression": expressao, "returnByValue": true }),
            Some(id_da_sessao),
        )?;
        let interno = resposta.get("result")?.get("result")?;
        if !verdadeiro(Some(interno)) {
            return None;
        }
        interno.get("value").cloned()
    }

    fn esperar_ate(&self, id_da_sessao: &str, expressao: &str) -> bool {
        let inicio = Instant::now();
        while inicio.elapsed() < LIMITE_PARA_MONTAR {
            if verdadeiro(self.avaliar(id_da_sessao, expressao).as_ref()) {
                return true;
            }
            dormir(Duration::from_millis(150));
        }
        false
    }

    fn navegar(&self, id_da_sessao: &str, url: &str) {
        self.conexao
            .enviar("Page.navigate", json!({ "url": url }), Some(id_da_sessao));
    }

    fn tirar_um_print(&self, id_da_sessao: &str, print: &Value) {
        let campo = |nome: &str| print.get(nome);
        let (Some(Value::String(arquivo)), Some(Value::String(endereco_da_combinacao))) =
            (campo("arquivo"), campo("endereco"))
        else {
            falhar(&format!(
                "print sem \"arquivo\" ou \"endereco\" na lista: {print}"
            ));
        };
        let tamanho = |nome: &str, valor: Option<&Value>| match valor {
            Some(valor) => json!({ nome: valor }),
            None => json!({}),
        };
        let juntar = |mut base: Value, extra: Value| {
            if let (Some(base), Value::Object(extra)) = (base.as_object_mut(), extra) {
                base.extend(extra);
            }
            base
        };

        self.navegar(id_da_sessao, "about:blank");
        let metricas = juntar(
            juntar(
                json!({ "deviceScaleFactor": 1, "mobile": false }),
                tamanho("width", campo("largura")),
            ),
            tamanho("height", campo("altura")),
        );
        self.conexao.enviar(
            "Emulation.setDeviceMetricsOverride",
            metricas,
            Some(id_da_sessao),
        );
        let combinacao = node::trocar_primeiro(
            endereco_da_combinacao,
            "#",
            &format!("&idioma={}#", self.idioma),
        );
        self.navegar(id_da_sessao, &format!("{}/{combinacao}", self.endereco));
        let montou = self.esperar_ate(id_da_sessao, TELA_MONTADA);
        dormir(FOLGA_DEPOIS_DE_MONTAR);
        let recorte = juntar(
            juntar(
                json!({ "x": 0, "y": 0, "scale": 1 }),
                tamanho("width", campo("largura")),
            ),
            tamanho("height", campo("altura")),
        );
        let imagem_em_base64 = resultado(
            self.conexao.enviar(
                "Page.captureScreenshot",
                json!({ "format": "png", "clip": recorte }),
                Some(id_da_sessao),
            ),
            "data",
            "Page.captureScreenshot",
        );
        let Some(imagem_em_base64) = imagem_em_base64.as_str() else {
            falhar("o Chrome não devolveu a imagem do print.");
        };
        let destino = node::juntar_caminho(&[&self.pasta_de_saida, arquivo]);
        if let Some(pasta) = Path::new(&destino).parent() {
            std::fs::create_dir_all(pasta).unwrap_or_else(|erro| {
                falhar(&format!("não deu para criar {}: {erro}", pasta.display()))
            });
        }
        std::fs::write(&destino, node::decodificar_base64(imagem_em_base64))
            .unwrap_or_else(|erro| falhar(&format!("não deu para gravar {destino}: {erro}")));
        println!("{}{destino}", if montou { "" } else { "(sem montar) " });
    }
}

fn main() {
    let argumentos: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|argumento| argumento.to_string_lossy().into_owned())
        .collect();
    let argumento = |posicao: usize| argumentos.get(posicao).filter(|texto| !texto.is_empty());
    let (Some(endereco_do_prototipo), Some(id_da_rotina), Some(pasta_de_saida)) =
        (argumento(0), argumento(1), argumento(2))
    else {
        eprintln!("{USO}");
        exit(2);
    };
    if argumentos.len() > 5 {
        eprintln!("{USO}");
        exit(2);
    }
    let endereco = endereco_do_prototipo
        .strip_suffix('/')
        .unwrap_or(endereco_do_prototipo)
        .to_string();
    let idioma = argumento(3).cloned().unwrap_or_else(|| "pt-BR".to_string());
    let quantidade_em_paralelo = node::numero_do_texto(argumento(4).map_or("6", String::as_str));

    let candidatos_a_chrome: Vec<String> = std::env::var_os("CHROME")
        .map(|caminho| caminho.to_string_lossy().into_owned())
        .into_iter()
        .chain(CAMINHOS_DO_CHROME.iter().map(|caminho| caminho.to_string()))
        .filter(|caminho| !caminho.is_empty())
        .collect();
    let Some(caminho_do_chrome) = candidatos_a_chrome
        .iter()
        .find(|candidato| Path::new(candidato).exists())
    else {
        eprintln!(
            "Nenhum Chrome ou Chromium encontrado para tirar os prints (use CHROME=<caminho>)."
        );
        exit(1);
    };

    let porta_do_chrome = 9400 + (aleatorio() % 500) as u16;
    let perfil_do_chrome = criar_perfil_do_chrome();
    let mut chrome = Command::new(caminho_do_chrome)
        .args([
            "--headless=new".to_string(),
            "--disable-gpu".to_string(),
            "--hide-scrollbars".to_string(),
            format!("--remote-debugging-port={porta_do_chrome}"),
            // Abas em paralelo: nenhuma fica "em segundo plano" (sem isso a folha não abria a tempo).
            "--disable-background-timer-throttling".to_string(),
            "--disable-renderer-backgrounding".to_string(),
            "--disable-backgrounding-occluded-windows".to_string(),
            format!("--user-data-dir={}", perfil_do_chrome.display()),
            "about:blank".to_string(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|erro| falhar(&format!("não deu para abrir {caminho_do_chrome}: {erro}")));

    let mut endereco_do_navegador = None;
    for _tentativa in 0..100 {
        match pedir_endereco_do_navegador(porta_do_chrome) {
            Ok(Some(endereco)) => {
                endereco_do_navegador = Some(endereco);
                break;
            }
            Ok(None) => {}
            Err(()) => dormir(Duration::from_millis(100)),
        }
    }
    let Some(endereco_do_navegador) = endereco_do_navegador else {
        eprintln!("O Chrome não subiu.");
        exit(1);
    };

    let conexao = Conexao::abrir(&endereco_do_navegador).unwrap_or_else(|erro| falhar(&erro));
    let prints = Arc::new(Prints {
        conexao,
        endereco,
        idioma,
        pasta_de_saida: pasta_de_saida.clone(),
    });

    // A lista de prints da rotina.
    let aba_da_lista = prints.abrir_aba();
    prints.navegar(
        &aba_da_lista,
        &format!(
            "{}/?listarPrints={}",
            prints.endereco,
            node::codificar_componente(id_da_rotina)
        ),
    );
    prints.esperar_ate(
        &aba_da_lista,
        "!!document.getElementById('lista-de-prints')",
    );
    let texto_da_lista = match prints.avaliar(
        &aba_da_lista,
        "document.getElementById('lista-de-prints').textContent",
    ) {
        Some(Value::String(texto)) if !texto.is_empty() => texto,
        _ => "[]".to_string(),
    };
    let lista_de_prints: Value = serde_json::from_str(&texto_da_lista)
        .unwrap_or_else(|erro| falhar(&format!("a lista de prints não é JSON: {erro}")));
    let lista_de_prints = match lista_de_prints {
        Value::Array(lista) if !lista.is_empty() => lista,
        Value::Null => falhar("a lista de prints veio null."),
        Value::String(texto) if !texto.is_empty() => {
            falhar("a lista de prints veio como texto, não como lista.")
        }
        Value::Object(objeto) if verdadeiro(objeto.get("length")) => {
            falhar("a lista de prints veio como objeto, não como lista.")
        }
        _ => {
            eprintln!("Nenhum print para \"{id_da_rotina}\". O protótipo está servido e a rotina existe em visoes.jsx?");
            encerrar_chrome(&mut chrome);
            exit(1);
        }
    };

    let Some(quantidade_de_abas) = node::tamanho_de_lista(quantidade_em_paralelo) else {
        falhar("RangeError: Invalid array length (paralelo grande demais).");
    };
    let total_de_prints = lista_de_prints.len();
    // Uma aba a mais que os prints só ficaria parada: o resultado é o mesmo.
    let quantidade_de_abas = quantidade_de_abas.min(total_de_prints);
    let fila_de_prints = Arc::new(Mutex::new(VecDeque::from(lista_de_prints)));
    let trabalhadores: Vec<_> = (0..quantidade_de_abas)
        .map(|_| {
            let prints = Arc::clone(&prints);
            let fila_de_prints = Arc::clone(&fila_de_prints);
            thread::spawn(move || {
                let id_da_sessao = prints.abrir_aba();
                loop {
                    let Some(print) = fila_de_prints.lock().unwrap().pop_front() else {
                        break;
                    };
                    prints.tirar_um_print(&id_da_sessao, &print);
                }
            })
        })
        .collect();
    for trabalhador in trabalhadores {
        if trabalhador.join().is_err() {
            exit(1);
        }
    }
    eprintln!("{total_de_prints} prints em {pasta_de_saida}");

    prints.conexao.fechar();
    encerrar_chrome(&mut chrome);
    dormir(Duration::from_millis(300));
    let _ = std::fs::remove_dir_all(&perfil_do_chrome);
    let _ = chrome.try_wait();
}
