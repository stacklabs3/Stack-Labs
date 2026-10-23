//! A conexão com o Chrome pelo protocolo de depuração (CDP): uma conexão WebSocket só, com as
//! abas em sessões (`flatten`), como no script Node. Uma thread cuida do socket: manda o que as
//! abas pedem e entrega cada resposta a quem esperava aquele `id`.
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tungstenite::protocol::WebSocketConfig;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

/// De quanto em quanto tempo a thread do socket larga a leitura para mandar os pedidos.
const INTERVALO_DO_SOCKET: Duration = Duration::from_millis(2);

enum Pedido {
    Mensagem(String),
    Fechar,
}

/// Quem espera resposta, por `id`; `fechada` quando a conexão acabou.
#[derive(Default)]
struct Espera {
    fechada: bool,
    por_id: HashMap<u64, Sender<Value>>,
}

type Pendentes = Arc<Mutex<Espera>>;

pub struct Conexao {
    pedidos: Mutex<Sender<Pedido>>,
    pendentes: Pendentes,
    proximo_id: AtomicU64,
}

impl Conexao {
    pub fn abrir(endereco_do_navegador: &str) -> Result<Conexao, String> {
        // Sem limite de tamanho: um print grande em base64 passa dos 16 MB de quadro padrão.
        let configuracao = WebSocketConfig::default()
            .max_message_size(None)
            .max_frame_size(None);
        let (socket, _) =
            tungstenite::client::connect_with_config(endereco_do_navegador, Some(configuracao), 3)
                .map_err(|erro| {
                    format!("não deu para conectar em {endereco_do_navegador}: {erro}")
                })?;
        if let MaybeTlsStream::Plain(tcp) = socket.get_ref() {
            tcp.set_read_timeout(Some(INTERVALO_DO_SOCKET))
                .map_err(|erro| erro.to_string())?;
            tcp.set_nodelay(true).ok();
        }
        let (enviar_pedido, pedidos) = mpsc::channel();
        let pendentes: Pendentes = Arc::default();
        let pendentes_da_thread = Arc::clone(&pendentes);
        thread::spawn(move || cuidar_do_socket(socket, pedidos, pendentes_da_thread));
        Ok(Conexao {
            pedidos: Mutex::new(enviar_pedido),
            pendentes,
            proximo_id: AtomicU64::new(1),
        })
    }

    /// Manda um comando e espera a resposta inteira (`{ id, result }` ou `{ id, error }`).
    /// `None` quando a conexão caiu antes da resposta.
    pub fn enviar(
        &self,
        metodo: &str,
        parametros: Value,
        id_da_sessao: Option<&str>,
    ) -> Option<Value> {
        let id = self.proximo_id.fetch_add(1, Ordering::Relaxed);
        let (resolver, resposta) = mpsc::channel();
        {
            let mut espera = self.pendentes.lock().unwrap();
            if espera.fechada {
                return None;
            }
            espera.por_id.insert(id, resolver);
        }
        let mut mensagem = json!({ "id": id, "method": metodo, "params": parametros });
        if let Some(id_da_sessao) = id_da_sessao {
            mensagem["sessionId"] = Value::from(id_da_sessao);
        }
        self.pedidos
            .lock()
            .unwrap()
            .send(Pedido::Mensagem(mensagem.to_string()))
            .ok()?;
        resposta.recv().ok()
    }

    pub fn fechar(&self) {
        let _ = self.pedidos.lock().unwrap().send(Pedido::Fechar);
    }
}

fn cuidar_do_socket(
    mut socket: WebSocket<MaybeTlsStream<TcpStream>>,
    pedidos: Receiver<Pedido>,
    pendentes: Pendentes,
) {
    loop {
        loop {
            match pedidos.try_recv() {
                Ok(Pedido::Mensagem(texto)) => {
                    if socket.send(Message::text(texto)).is_err() {
                        return encerrar(&pendentes);
                    }
                }
                Ok(Pedido::Fechar) | Err(mpsc::TryRecvError::Disconnected) => {
                    let _ = socket.close(None);
                    let _ = socket.flush();
                    return encerrar(&pendentes);
                }
                Err(mpsc::TryRecvError::Empty) => break,
            }
        }
        match socket.read() {
            Ok(Message::Text(texto)) => entregar(texto.as_str(), &pendentes),
            Ok(Message::Binary(bytes)) => entregar(&String::from_utf8_lossy(&bytes), &pendentes),
            Ok(_) => {}
            Err(tungstenite::Error::Io(erro))
                if matches!(erro.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(_) => return encerrar(&pendentes),
        }
    }
}

fn entregar(texto: &str, pendentes: &Pendentes) {
    let Ok(mensagem) = serde_json::from_str::<Value>(texto) else {
        return;
    };
    let Some(id) = mensagem.get("id").and_then(Value::as_u64) else {
        return;
    };
    if let Some(resolver) = pendentes.lock().unwrap().por_id.remove(&id) {
        let _ = resolver.send(mensagem);
    }
}

/// A conexão acabou: quem ainda espera resposta recebe `None`.
fn encerrar(pendentes: &Pendentes) {
    let mut espera = pendentes.lock().unwrap();
    espera.fechada = true;
    espera.por_id.clear();
}
