// Trava dos comandos pesados (ADR 0024, emenda a 0016): build, dev server, suíte de testes, e2e,
// build nativo e o CI local passam por aqui. Duas camadas, ambas com flock (o sistema solta a trava
// quando o processo morre, sem PID para conferir):
//   1. a trava do projeto, `stacklabs-<projeto>.lock`: um comando pesado por vez em cada projeto;
//   2. uma vaga da máquina, `vaga-<n>.lock`: projetos diferentes rodam juntos até o número de vagas,
//      que sai da memória total (2 numa máquina de 8 GB).
// O comando roda com o heap do Node limitado (NODE_OPTIONS=--max-old-space-size), e o que ele chamar
// por dentro (o pre-push dentro de um `rodar pesado git push`) não espera de novo.
//
// Uso: .claude/watchdogs/rodar pesado <comando> [argumentos]   (um argumento só roda por `sh -c`)
// Ajustes: STACKLABS_VAGAS (vagas da máquina), STACKLABS_HEAP_MB (heap do Node, padrão 2048),
// STACKLABS_TRAVAS (pasta das travas, padrão ~/.stacklabs/travas).

use std::fs::{self, File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::comum::executar_git;

const VARIAVEL_DE_DENTRO: &str = "STACKLABS_PESADO";
const HEAP_PADRAO_MB: u64 = 2048;
// Memória que fica para o sistema, o editor e as sessões do Claude; cada vaga reserva o resto em fatias.
const RESERVA_DO_SISTEMA_MIB: u64 = 2048;
const MEMORIA_POR_VAGA_MIB: u64 = 2560;
const VAGAS_SEM_MEDIDA: usize = 2;
const INTERVALO_DE_ESPERA: Duration = Duration::from_millis(500);

pub struct Ajustes {
    pub pasta_das_travas: PathBuf,
    pub projeto: String,
    pub vagas: usize,
    pub heap_mb: u64,
}

// Vagas que a memória total comporta: 8 GB dão 2, 16 GB dão 5; nunca menos que 1.
pub fn vagas_para_memoria(memoria_total_mib: u64) -> usize {
    let vagas = memoria_total_mib.saturating_sub(RESERVA_DO_SISTEMA_MIB) / MEMORIA_POR_VAGA_MIB;
    usize::try_from(vagas).unwrap_or(1).max(1)
}

fn memoria_total_mib() -> Option<u64> {
    if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        let linha = meminfo.lines().find(|linha| linha.starts_with("MemTotal:"))?;
        let em_kib: u64 = linha.split_whitespace().nth(1)?.parse().ok()?;
        return Some(em_kib / 1024);
    }
    // macOS: hw.memsize em bytes.
    let saida = Command::new("sysctl").args(["-n", "hw.memsize"]).output().ok()?;
    let em_bytes: u64 = String::from_utf8_lossy(&saida.stdout).trim().parse().ok()?;
    Some(em_bytes / 1024 / 1024)
}

fn numero_da_variavel(nome: &str) -> Option<u64> {
    std::env::var(nome).ok().and_then(|valor| valor.trim().parse().ok()).filter(|valor| *valor > 0)
}

// O nome do projeto é a pasta da raiz do git (a worktree, quando houver), só com [A-Za-z0-9._-].
fn nome_do_projeto(pasta: &Path) -> String {
    let raiz = executar_git(pasta, &["rev-parse", "--show-toplevel"])
        .map(|saida| PathBuf::from(saida.trim()))
        .unwrap_or_else(|| pasta.to_path_buf());
    let nome: String = raiz
        .file_name()
        .map(|nome| nome.to_string_lossy().into_owned())
        .unwrap_or_else(|| "projeto".into())
        .chars()
        .map(|letra| if letra.is_ascii_alphanumeric() || "._-".contains(letra) { letra } else { '-' })
        .collect();
    if nome.is_empty() {
        "projeto".into()
    } else {
        nome
    }
}

pub fn ajustes_da_maquina(pasta: &Path) -> Ajustes {
    let pasta_das_travas = std::env::var_os("STACKLABS_TRAVAS")
        .filter(|valor| !valor.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|casa| PathBuf::from(casa).join(".stacklabs").join("travas")))
        .unwrap_or_else(|| std::env::temp_dir().join("stacklabs-travas"));
    let vagas = numero_da_variavel("STACKLABS_VAGAS")
        .and_then(|vagas| usize::try_from(vagas).ok())
        .unwrap_or_else(|| memoria_total_mib().map(vagas_para_memoria).unwrap_or(VAGAS_SEM_MEDIDA));
    Ajustes {
        pasta_das_travas,
        projeto: nome_do_projeto(pasta),
        vagas,
        heap_mb: numero_da_variavel("STACKLABS_HEAP_MB").unwrap_or(HEAP_PADRAO_MB),
    }
}

fn abrir(caminho: &Path) -> std::io::Result<File> {
    OpenOptions::new().create(true).truncate(false).write(true).open(caminho)
}

// As travas seguradas; soltam ao sair de escopo (e o sistema solta se o processo morrer).
pub struct Travas {
    _do_projeto: File,
    _vaga: File,
}

pub fn segurar(ajustes: &Ajustes) -> std::io::Result<Travas> {
    fs::create_dir_all(&ajustes.pasta_das_travas)?;
    let do_projeto = abrir(&ajustes.pasta_das_travas.join(format!("stacklabs-{}.lock", ajustes.projeto)))?;
    match do_projeto.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => {
            eprintln!("[pesado] {} já roda um comando pesado; esperando ele terminar.", ajustes.projeto);
            do_projeto.lock()?;
        }
        Err(TryLockError::Error(erro)) => return Err(erro),
    }
    let mut avisou = false;
    loop {
        for numero in 0..ajustes.vagas.max(1) {
            let vaga = abrir(&ajustes.pasta_das_travas.join(format!("vaga-{numero}.lock")))?;
            match vaga.try_lock() {
                Ok(()) => {
                    return Ok(Travas {
                        _do_projeto: do_projeto,
                        _vaga: vaga,
                    })
                }
                Err(TryLockError::WouldBlock) => {}
                Err(TryLockError::Error(erro)) => return Err(erro),
            }
        }
        if !avisou {
            eprintln!(
                "[pesado] as {} vagas da máquina estão ocupadas por outros projetos; esperando uma soltar.",
                ajustes.vagas
            );
            avisou = true;
        }
        std::thread::sleep(INTERVALO_DE_ESPERA);
    }
}

// NODE_OPTIONS com o teto do heap, sem trocar um teto que o comando já trouxe.
pub fn node_options_com_teto(atual: &str, heap_mb: u64) -> String {
    if atual.contains("--max-old-space-size") {
        atual.to_string()
    } else {
        format!("{atual} --max-old-space-size={heap_mb}").trim().to_string()
    }
}

pub fn ja_dentro_da_trava() -> bool {
    std::env::var_os(VARIAVEL_DE_DENTRO).is_some()
}

// Roda `tarefa` dentro das duas travas. Dentro de outra trava (variável herdada), roda direto.
pub fn com_trava<T>(pasta: &Path, tarefa: impl FnOnce() -> T) -> T {
    if ja_dentro_da_trava() {
        return tarefa();
    }
    let ajustes = ajustes_da_maquina(pasta);
    let travas = segurar(&ajustes);
    if let Err(erro) = &travas {
        eprintln!(
            "[pesado] não consegui a trava em {} ({erro}); seguindo sem ela.",
            ajustes.pasta_das_travas.display()
        );
    }
    std::env::set_var(VARIAVEL_DE_DENTRO, &ajustes.projeto);
    let resultado = tarefa();
    std::env::remove_var(VARIAVEL_DE_DENTRO);
    drop(travas);
    resultado
}

fn comando_do_sistema(argumentos: &[String]) -> Command {
    if let [linha_de_comando] = argumentos {
        let mut comando = if cfg!(windows) { Command::new("cmd") } else { Command::new("sh") };
        comando.args(if cfg!(windows) { ["/C", linha_de_comando] } else { ["-c", linha_de_comando] });
        comando
    } else {
        let mut comando = Command::new(&argumentos[0]);
        comando.args(&argumentos[1..]);
        comando
    }
}

pub fn rodar(argumentos: &[String]) -> i32 {
    let argumentos: Vec<String> = argumentos.iter().skip_while(|argumento| *argumento == "--").cloned().collect();
    if argumentos.is_empty() {
        eprintln!("uso: rodar pesado <comando> [argumentos]   (ex.: rodar pesado \"npm run build\")");
        return 2;
    }
    let pasta = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let heap_mb = numero_da_variavel("STACKLABS_HEAP_MB").unwrap_or(HEAP_PADRAO_MB);
    com_trava(&pasta, || {
        let node_options = node_options_com_teto(&std::env::var("NODE_OPTIONS").unwrap_or_default(), heap_mb);
        match comando_do_sistema(&argumentos).env("NODE_OPTIONS", node_options).status() {
            Ok(estado) => estado.code().unwrap_or(1),
            Err(erro) => {
                eprintln!("[pesado] {} não iniciou: {erro}", argumentos[0]);
                127
            }
        }
    })
}
