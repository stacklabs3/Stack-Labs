// Comandos pesados (ADR 0024, emenda a 0016): build, dev server, suíte de testes, e2e, build nativo e
// o CI local passam por aqui, e nada mais na máquina decide quem roda.
//
//   1. Trava do projeto, `stacklabs-<projeto>.lock`: um comando pesado por vez em cada projeto.
//   2. Vaga da máquina, `vaga-<n>.lock`: projetos diferentes rodam juntos até o número de vagas, que
//      sai da memória total (2 em 8 GB). Antes de entrar, espera a memória livre (MemAvailable)
//      chegar a metade da vaga, por até 2 minutos.
//   3. Limite de memória de verdade (Linux com systemd de usuário): o comando roda num scope com
//      MemoryHigh/MemoryMax do tamanho da vaga; quem passa dele morre sozinho, não a máquina. A sonda
//      confere no memory.max do próprio scope que o limite pegou; sem isso, avisa e segue sem ele.
//   4. O comando roda num grupo de processos próprio: SIGINT/SIGTERM/SIGHUP no `rodar pesado` chegam
//      a todos os processos dele, e o que sobrar quando ele terminar é encerrado. Nenhum dev server
//      fica vivo, fora da conta, depois que a trava solta.
//
// As travas usam flock: o sistema as solta quando o processo morre. O que roda dentro de um comando
// pesado (o pre-push dentro de um `rodar pesado git push`) não espera de novo.
//
// Uso: .claude/watchdogs/rodar pesado <comando> [argumentos]   (um argumento só roda por `sh -c`)
// Ajustes da máquina: STACKLABS_VAGAS, STACKLABS_MEMORIA_POR_VAGA_MB, STACKLABS_HEAP_MB (opcional,
// põe --max-old-space-size no NODE_OPTIONS), STACKLABS_LIMITE_DE_MEMORIA=desligado,
// STACKLABS_TRAVAS (pasta das travas, padrão ~/.stacklabs/travas).

use std::fs::{self, File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::comum::executar_git;

pub const VARIAVEL_DE_DENTRO: &str = "STACKLABS_PESADO";
// Memória que fica para o sistema, o editor e as sessões do Claude; o resto se divide em vagas.
const RESERVA_DO_SISTEMA_MIB: u64 = 2048;
const MEMORIA_MINIMA_POR_VAGA_MIB: u64 = 2560;
const VAGAS_SEM_MEDIDA: usize = 2;
const INTERVALO_DE_ESPERA: Duration = Duration::from_millis(500);
const ESPERA_MAXIMA_POR_MEMORIA: Duration = Duration::from_secs(120);
const PRAZO_DA_SONDA: Duration = Duration::from_secs(10);

pub struct Ajustes {
    pub pasta_das_travas: PathBuf,
    pub projeto: String,
    pub vagas: usize,
    // None quando a memória da máquina não pôde ser lida e ninguém fixou o orçamento.
    pub memoria_por_vaga_mib: Option<u64>,
    pub heap_mb: Option<u64>,
    pub limitar_memoria: bool,
}

// Vagas que a memória total comporta: 8 GB dão 2, 16 GB dão 5; nunca menos que 1.
pub fn vagas_para_memoria(memoria_total_mib: u64) -> usize {
    let vagas = memoria_total_mib.saturating_sub(RESERVA_DO_SISTEMA_MIB) / MEMORIA_MINIMA_POR_VAGA_MIB;
    usize::try_from(vagas).unwrap_or(1).max(1)
}

// O que sobra da reserva, dividido igual entre as vagas: 7,6 GiB em 2 vagas dão 2,8 GiB cada.
pub fn memoria_por_vaga(memoria_total_mib: u64, vagas: usize) -> u64 {
    let vagas = u64::try_from(vagas.max(1)).unwrap_or(1);
    (memoria_total_mib.saturating_sub(RESERVA_DO_SISTEMA_MIB) / vagas).max(1024)
}

fn linha_do_meminfo(campo: &str) -> Option<u64> {
    let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
    let linha = meminfo.lines().find(|linha| linha.starts_with(campo))?;
    let em_kib: u64 = linha.split_whitespace().nth(1)?.parse().ok()?;
    Some(em_kib / 1024)
}

fn memoria_total_mib() -> Option<u64> {
    if let Some(total) = linha_do_meminfo("MemTotal:") {
        return Some(total);
    }
    // macOS: hw.memsize em bytes.
    let saida = Command::new("sysctl").args(["-n", "hw.memsize"]).stderr(Stdio::null()).output().ok()?;
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
        .unwrap_or_default()
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
    let memoria_total = memoria_total_mib();
    let vagas = numero_da_variavel("STACKLABS_VAGAS")
        .and_then(|vagas| usize::try_from(vagas).ok())
        .unwrap_or_else(|| memoria_total.map(vagas_para_memoria).unwrap_or(VAGAS_SEM_MEDIDA));
    let memoria_por_vaga_mib = numero_da_variavel("STACKLABS_MEMORIA_POR_VAGA_MB").or_else(|| memoria_total.map(|total| memoria_por_vaga(total, vagas)));
    Ajustes {
        pasta_das_travas,
        projeto: nome_do_projeto(pasta),
        vagas,
        memoria_por_vaga_mib,
        heap_mb: numero_da_variavel("STACKLABS_HEAP_MB"),
        limitar_memoria: std::env::var("STACKLABS_LIMITE_DE_MEMORIA").as_deref() != Ok("desligado"),
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

// Memória livre para entrar: metade da vaga. A vaga é o teto do comando, não o uso típico; exigir o
// teto inteiro livre faria o segundo projeto esperar quase sempre numa máquina de 8 GB.
pub fn memoria_livre_para_entrar(memoria_por_vaga_mib: u64) -> u64 {
    memoria_por_vaga_mib / 2
}

// Espera a memória livre (MemAvailable) chegar à de entrada, por um tempo limitado: memória ocupada
// fora do método (navegador, editor) não pode travar o trabalho para sempre, e o scope segura o excesso.
fn esperar_memoria_livre(ajustes: &Ajustes) {
    let (Some(por_vaga), Some(_)) = (ajustes.memoria_por_vaga_mib, linha_do_meminfo("MemAvailable:")) else {
        return;
    };
    let necessaria = memoria_livre_para_entrar(por_vaga);
    let prazo = Instant::now() + ESPERA_MAXIMA_POR_MEMORIA;
    let mut avisou = false;
    while let Some(livre) = linha_do_meminfo("MemAvailable:").filter(|livre| *livre < necessaria) {
        if Instant::now() >= prazo {
            eprintln!("[pesado] a memória livre ({livre} MB) segue abaixo da de entrada ({necessaria} MB) depois de 2 minutos; seguindo assim mesmo.");
            return;
        }
        if !avisou {
            eprintln!("[pesado] memória livre {livre} MB, abaixo da de entrada ({necessaria} MB, metade da vaga); esperando até 2 minutos.");
            avisou = true;
        }
        std::thread::sleep(INTERVALO_DE_ESPERA);
    }
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
                    esperar_memoria_livre(ajustes);
                    return Ok(Travas {
                        _do_projeto: do_projeto,
                        _vaga: vaga,
                    });
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

// O comando como o sistema roda: um argumento só vai por `sh -c`, vários vão direto.
pub fn argumentos_do_sistema(argumentos: &[String]) -> Vec<String> {
    match argumentos {
        [linha_de_comando] if cfg!(windows) => vec!["cmd".into(), "/C".into(), linha_de_comando.clone()],
        [linha_de_comando] => vec!["sh".into(), "-c".into(), linha_de_comando.clone()],
        _ => argumentos.to_vec(),
    }
}

pub fn argumentos_do_scope(limite_mib: u64) -> Vec<String> {
    let alto_mib = limite_mib * 9 / 10;
    ["systemd-run", "--user", "--scope", "--quiet", "--collect", "-p"]
        .iter()
        .map(|parte| parte.to_string())
        .chain([format!("MemoryHigh={alto_mib}M"), "-p".into(), format!("MemoryMax={limite_mib}M"), "--".into()])
        .collect()
}

// A sonda roda um `cat memory.max` dentro de um scope igual ao do comando: só vale se o kernel
// mostrar o limite pedido (sem systemd de usuário ou sem o controlador de memória delegado, não mostra).
fn limite_confirmado(limite_mib: u64) -> Result<(), String> {
    let mut sonda = argumentos_do_scope(limite_mib);
    sonda.extend(["sh", "-c", "cat \"/sys/fs/cgroup$(sed -n 's/^0:://p' /proc/self/cgroup)/memory.max\""].map(String::from));
    let mut processo = Command::new(&sonda[0])
        .args(&sonda[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| "systemd-run não encontrado".to_string())?;
    let prazo = Instant::now() + PRAZO_DA_SONDA;
    while processo.try_wait().map_err(|erro| erro.to_string())?.is_none() {
        if Instant::now() >= prazo {
            let _ = processo.kill();
            let _ = processo.wait();
            return Err("systemd-run não respondeu".into());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let saida = processo.wait_with_output().map_err(|erro| erro.to_string())?;
    let lido = String::from_utf8_lossy(&saida.stdout).trim().to_string();
    let esperado = (limite_mib * 1024 * 1024).to_string();
    if saida.status.success() && lido == esperado {
        Ok(())
    } else {
        let erro = String::from_utf8_lossy(&saida.stderr).trim().chars().take(160).collect::<String>();
        Err(format!(
            "o scope não aplicou o limite (memory.max = \"{lido}\"{})",
            if erro.is_empty() { String::new() } else { format!("; {erro}") }
        ))
    }
}

fn limite_de_memoria(ajustes: &Ajustes) -> Option<u64> {
    if !cfg!(target_os = "linux") || !ajustes.limitar_memoria {
        return None;
    }
    let limite = ajustes.memoria_por_vaga_mib?;
    match limite_confirmado(limite) {
        Ok(()) => Some(limite),
        Err(motivo) => {
            eprintln!(
                "[pesado] sem limite de memória por vaga: {motivo}. O comando roda só com a trava. (Linux com systemd de usuário; docs/agents/orquestracao.md)"
            );
            None
        }
    }
}

#[cfg(unix)]
mod grupo {
    use std::os::unix::process::{CommandExt, ExitStatusExt};
    use std::process::{Child, Command, ExitStatus};
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::time::{Duration, Instant};

    static GRUPO_DO_FILHO: AtomicI32 = AtomicI32::new(0);
    static SINAL_RECEBIDO: AtomicI32 = AtomicI32::new(0);
    const SINAIS_REPASSADOS: [libc::c_int; 3] = [libc::SIGINT, libc::SIGTERM, libc::SIGHUP];
    const PRAZO_PARA_SAIR: Duration = Duration::from_secs(3);

    // Só usa funções seguras dentro de um tratador de sinal: guardar um inteiro e kill(2).
    extern "C" fn repassar(sinal: libc::c_int) {
        SINAL_RECEBIDO.store(sinal, Ordering::SeqCst);
        let grupo = GRUPO_DO_FILHO.load(Ordering::SeqCst);
        if grupo > 0 {
            // SAFETY: kill(2) é async-signal-safe; o grupo é o do filho, criado por nós.
            unsafe {
                libc::kill(-grupo, sinal);
            }
        }
    }

    fn instalar_repasse() {
        for sinal in SINAIS_REPASSADOS {
            // SAFETY: sigaction com um tratador extern "C" que só faz operações async-signal-safe.
            unsafe {
                let mut acao: libc::sigaction = std::mem::zeroed();
                acao.sa_sigaction = repassar as extern "C" fn(libc::c_int) as libc::sighandler_t;
                acao.sa_flags = libc::SA_RESTART;
                libc::sigemptyset(&mut acao.sa_mask);
                libc::sigaction(sinal, &acao, std::ptr::null_mut());
            }
        }
    }

    // No Linux, lê o /proc: kill(-grupo, 0) conta o zumbi, que já morreu e só espera quem o adotou
    // recolhê-lo (num container sem init de verdade, isso demora segundos).
    #[cfg(target_os = "linux")]
    fn grupo_vivo(grupo: i32) -> bool {
        let Ok(entradas) = std::fs::read_dir("/proc") else {
            // SAFETY: kill com sinal 0 só confere se o grupo existe.
            return unsafe { libc::kill(-grupo, 0) == 0 };
        };
        entradas.flatten().any(|entrada| {
            let Ok(estatistica) = std::fs::read_to_string(entrada.path().join("stat")) else {
                return false;
            };
            // "<pid> (<nome>) <estado> <ppid> <pgrp> ...": o nome pode ter espaço e parêntese.
            let depois_do_nome = estatistica.rsplit_once(')').map(|(_, resto)| resto).unwrap_or("");
            let mut campos = depois_do_nome.split_whitespace();
            let estado = campos.next();
            let grupo_do_processo = campos.nth(1).and_then(|campo| campo.parse::<i32>().ok());
            grupo_do_processo == Some(grupo) && estado != Some("Z")
        })
    }

    #[cfg(not(target_os = "linux"))]
    fn grupo_vivo(grupo: i32) -> bool {
        // SAFETY: kill com sinal 0 só confere se o grupo existe.
        unsafe { libc::kill(-grupo, 0) == 0 }
    }

    fn sinalizar(grupo: i32, sinal: libc::c_int) {
        // SAFETY: kill(2) no grupo que criamos.
        unsafe {
            libc::kill(-grupo, sinal);
        }
    }

    // Encerra o que ficou no grupo: SIGTERM, até 3 s para sair, depois SIGKILL. Devolve se havia sobra.
    fn encerrar_sobras(grupo: i32) -> bool {
        if !grupo_vivo(grupo) {
            return false;
        }
        sinalizar(grupo, libc::SIGTERM);
        let prazo = Instant::now() + PRAZO_PARA_SAIR;
        while grupo_vivo(grupo) && Instant::now() < prazo {
            std::thread::sleep(Duration::from_millis(50));
        }
        if grupo_vivo(grupo) {
            sinalizar(grupo, libc::SIGKILL);
        }
        true
    }

    pub struct Fim {
        pub estado: ExitStatus,
        pub sinal_recebido: Option<i32>,
        pub sobras_encerradas: bool,
    }

    pub fn rodar(comando: &mut Command) -> std::io::Result<Fim> {
        instalar_repasse();
        let mut filho: Child = comando.process_group(0).spawn()?;
        let grupo = i32::try_from(filho.id()).unwrap_or(0);
        GRUPO_DO_FILHO.store(grupo, Ordering::SeqCst);
        // Um sinal que chegou entre o spawn e o registro do grupo ainda precisa chegar ao filho.
        let pendente = SINAL_RECEBIDO.load(Ordering::SeqCst);
        if pendente != 0 && grupo > 0 {
            sinalizar(grupo, pendente);
        }
        let estado = filho.wait()?;
        let sobras_encerradas = grupo > 0 && encerrar_sobras(grupo);
        GRUPO_DO_FILHO.store(0, Ordering::SeqCst);
        let sinal_recebido = Some(SINAL_RECEBIDO.load(Ordering::SeqCst)).filter(|sinal| *sinal != 0);
        Ok(Fim {
            estado,
            sinal_recebido,
            sobras_encerradas,
        })
    }

    pub fn morto_por_sigkill(estado: &ExitStatus) -> bool {
        estado.signal() == Some(libc::SIGKILL)
    }

    pub fn codigo(fim: &Fim) -> i32 {
        if let Some(sinal) = fim.sinal_recebido {
            return 128 + sinal;
        }
        fim.estado.code().or_else(|| fim.estado.signal().map(|sinal| 128 + sinal)).unwrap_or(1)
    }
}

pub enum Saida {
    Herdada,
    // O pre-push escreve o relatório no stderr, como o git espera de um hook.
    NoErro,
}

fn redirecionar(comando: &mut Command, saida: Saida) {
    if let Saida::NoErro = saida {
        #[cfg(unix)]
        {
            use std::os::fd::AsFd;
            if let Ok(copia) = std::io::stderr().as_fd().try_clone_to_owned() {
                comando.stdout(Stdio::from(copia));
            }
        }
    }
}

// Roda um comando pesado do começo ao fim: travas, memória, grupo de processos. Devolve o código de
// saída (128 + sinal quando morto por um).
pub fn executar(argumentos: &[String], pasta: &Path, saida: Saida) -> i32 {
    let argumentos_base = argumentos_do_sistema(argumentos);
    if ja_dentro_da_trava() {
        let mut comando = Command::new(&argumentos_base[0]);
        comando.args(&argumentos_base[1..]).current_dir(pasta);
        redirecionar(&mut comando, saida);
        return match comando.status() {
            Ok(estado) => estado.code().unwrap_or(1),
            Err(erro) => {
                eprintln!("[pesado] {} não iniciou: {erro}", argumentos_base[0]);
                127
            }
        };
    }
    let ajustes = ajustes_da_maquina(pasta);
    let travas = segurar(&ajustes);
    if let Err(erro) = &travas {
        eprintln!(
            "[pesado] não consegui a trava em {} ({erro}); seguindo sem ela.",
            ajustes.pasta_das_travas.display()
        );
    }
    let limite = limite_de_memoria(&ajustes);
    let completos: Vec<String> = limite.map(argumentos_do_scope).unwrap_or_default().into_iter().chain(argumentos_base).collect();
    let mut comando = Command::new(&completos[0]);
    comando
        .args(&completos[1..])
        .current_dir(pasta)
        .stdin(Stdio::null())
        .env(VARIAVEL_DE_DENTRO, &ajustes.projeto);
    if let Some(heap_mb) = ajustes.heap_mb {
        comando.env(
            "NODE_OPTIONS",
            node_options_com_teto(&std::env::var("NODE_OPTIONS").unwrap_or_default(), heap_mb),
        );
    }
    redirecionar(&mut comando, saida);
    let codigo = rodar_e_relatar(&mut comando, &completos[0], limite);
    drop(travas);
    codigo
}

#[cfg(unix)]
fn rodar_e_relatar(comando: &mut Command, programa: &str, limite: Option<u64>) -> i32 {
    match grupo::rodar(comando) {
        Ok(fim) => {
            if fim.sobras_encerradas && fim.sinal_recebido.is_none() {
                eprintln!("[pesado] o comando deixou processos em segundo plano; encerrei-os para soltar a memória com a trava.");
            }
            if let (Some(limite_mib), true) = (limite, grupo::morto_por_sigkill(&fim.estado)) {
                eprintln!(
                    "[pesado] o comando foi morto (SIGKILL). Se passou do limite da vaga ({limite_mib} MB), reduza os workers dele \
                     (docs/agents/orquestracao.md) ou ajuste STACKLABS_MEMORIA_POR_VAGA_MB."
                );
            }
            grupo::codigo(&fim)
        }
        Err(erro) => {
            eprintln!("[pesado] {programa} não iniciou: {erro}");
            127
        }
    }
}

#[cfg(not(unix))]
fn rodar_e_relatar(comando: &mut Command, programa: &str, _limite: Option<u64>) -> i32 {
    match comando.status() {
        Ok(estado) => estado.code().unwrap_or(1),
        Err(erro) => {
            eprintln!("[pesado] {programa} não iniciou: {erro}");
            127
        }
    }
}

pub fn rodar(argumentos: &[String]) -> i32 {
    let argumentos: Vec<String> = argumentos.iter().skip_while(|argumento| *argumento == "--").cloned().collect();
    if argumentos.is_empty() {
        eprintln!("uso: rodar pesado <comando> [argumentos]   (ex.: rodar pesado \"npm run build\")");
        return 2;
    }
    let pasta = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    executar(&argumentos, &pasta, Saida::Herdada)
}
