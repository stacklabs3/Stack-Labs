// Testes do subcomando pesado: o binário roda com a pasta das travas e o número de vagas do teste, e
// o teste segura à mão a trava de um projeto ou uma vaga para ver quem espera e quem passa. Os sinais
// são de verdade (kill -TERM), e o limite de memória usa um systemd-run falso que anota o que recebeu.

use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use watchdogs::trava::{
    argumentos_do_scope, arquivo_da_trava_do_projeto, memoria_livre_para_entrar, memoria_por_vaga, node_options_com_teto, projeto_da_pasta, vagas_para_memoria,
};

const PRAZO_DE_ESPERA: Duration = Duration::from_millis(1500);

fn pasta_nova(prefixo: &str) -> PathBuf {
    static PROXIMO: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let ordem = PROXIMO.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let sufixo = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let pasta = std::env::temp_dir().join(format!("{prefixo}-{}-{ordem}-{sufixo}", std::process::id()));
    fs::create_dir_all(&pasta).unwrap();
    pasta
}

fn trava_do_projeto(travas: &Path, pasta: &Path) -> PathBuf {
    travas.join(arquivo_da_trava_do_projeto(&projeto_da_pasta(pasta)))
}

// Um projeto (pasta sem git: o nome vem da pasta) que roda `rodar pesado` com as travas em `travas`.
// A vaga de 64 MB deixa a espera por memória livre fora dos testes de trava.
fn pesado_com(projeto: &Path, travas: &Path, vagas: usize, comando: &str, variaveis: &[(&str, &str)]) -> Child {
    let mut processo = Command::new(env!("CARGO_BIN_EXE_watchdogs"));
    processo
        .args(["pesado", comando])
        .current_dir(projeto)
        .env("STACKLABS_TRAVAS", travas)
        .env("STACKLABS_VAGAS", vagas.to_string())
        .env("STACKLABS_MEMORIA_POR_VAGA_MB", "64")
        .env("STACKLABS_LIMITE_DE_MEMORIA", "desligado")
        .env_remove("STACKLABS_PESADO")
        .env_remove("STACKLABS_HEAP_MB")
        .env_remove("NODE_OPTIONS")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (chave, valor) in variaveis {
        processo.env(chave, valor);
    }
    processo.spawn().unwrap()
}

fn pesado(projeto: &Path, travas: &Path, vagas: usize, comando: &str) -> Child {
    pesado_com(projeto, travas, vagas, comando, &[])
}

fn segurar(caminho: &Path) -> File {
    fs::create_dir_all(caminho.parent().unwrap()).unwrap();
    let arquivo = OpenOptions::new().create(true).truncate(false).write(true).open(caminho).unwrap();
    arquivo.lock().unwrap();
    arquivo
}

// Espera o processo até o prazo; None se ele ainda roda.
fn terminou_ate(processo: &mut Child, prazo: Duration) -> Option<i32> {
    let limite = Instant::now() + prazo;
    while Instant::now() < limite {
        if let Some(estado) = processo.try_wait().unwrap() {
            return Some(estado.code().unwrap_or(-1));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    None
}

// Processo vivo cuja linha de comando é exatamente `sleep <segundos>`; espera até 2 s ele sumir.
#[cfg(target_os = "linux")]
fn sleep_vivo(segundos: u32) -> bool {
    let procurado = format!("sleep\0{segundos}\0");
    let limite = Instant::now() + Duration::from_secs(2);
    loop {
        let vivo = fs::read_dir("/proc")
            .map(|entradas| {
                entradas.flatten().any(|entrada| {
                    fs::read(entrada.path().join("cmdline"))
                        .map(|linha| linha == procurado.as_bytes())
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        if !vivo || Instant::now() >= limite {
            return vivo;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(target_os = "linux")]
fn enviar_sigterm(processo: &Child) {
    Command::new("kill").args(["-TERM", &processo.id().to_string()]).status().unwrap();
}

#[test]
fn roda_o_comando_e_devolve_a_saida_dele_sem_mexer_no_heap() {
    let projeto = pasta_nova("excurfly");
    let travas = pasta_nova("travas");
    let saida = pesado(&projeto, &travas, 2, "echo \"[$NODE_OPTIONS]|$STACKLABS_PESADO\"; exit 3")
        .wait_with_output()
        .unwrap();
    assert_eq!(saida.status.code(), Some(3));
    let impresso = String::from_utf8_lossy(&saida.stdout);
    let (heap, dono) = impresso.trim().split_once('|').unwrap();
    assert_eq!(heap, "[]", "sem STACKLABS_HEAP_MB, o heap não muda");
    assert!(dono.starts_with(&format!("{}:", projeto_da_pasta(&projeto))), "{dono}");
    assert!(trava_do_projeto(&travas, &projeto).exists());
}

#[test]
fn varios_argumentos_rodam_direto_sem_shell() {
    let projeto = pasta_nova("excurfly");
    let travas = pasta_nova("travas");
    let saida = Command::new(env!("CARGO_BIN_EXE_watchdogs"))
        .args(["pesado", "--", "printf", "%s|", "a b", "$HOME"])
        .current_dir(&projeto)
        .env("STACKLABS_TRAVAS", &travas)
        .env("STACKLABS_MEMORIA_POR_VAGA_MB", "64")
        .env("STACKLABS_LIMITE_DE_MEMORIA", "desligado")
        .env_remove("STACKLABS_PESADO")
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&saida.stdout), "a b|$HOME|", "sem shell, nada se expande nem se quebra");
}

#[test]
fn o_teto_do_heap_so_entra_quando_a_maquina_pede() {
    let projeto = pasta_nova("excurfly");
    let travas = pasta_nova("travas");
    let saida = pesado_com(&projeto, &travas, 2, "echo \"$NODE_OPTIONS\"", &[("STACKLABS_HEAP_MB", "1536")])
        .wait_with_output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&saida.stdout).trim(), "--max-old-space-size=1536");
    assert_eq!(node_options_com_teto("--inspect", 2048), "--inspect --max-old-space-size=2048");
    assert_eq!(node_options_com_teto("--max-old-space-size=1024", 2048), "--max-old-space-size=1024");
}

#[test]
fn dois_projetos_rodam_juntos_quando_ha_vaga() {
    let travas = pasta_nova("travas");
    let excurfly = pasta_nova("excurfly");
    let organiza_ai = pasta_nova("organiza-ai");
    let _excurfly_ocupado = segurar(&trava_do_projeto(&travas, &excurfly));
    let _vaga_do_excurfly = segurar(&travas.join("vaga-0.lock"));
    let mut do_organiza_ai = pesado(&organiza_ai, &travas, 2, "true");
    assert_eq!(terminou_ate(&mut do_organiza_ai, PRAZO_DE_ESPERA), Some(0));
}

#[test]
fn o_mesmo_projeto_espera_o_comando_pesado_anterior() {
    let travas = pasta_nova("travas");
    let excurfly = pasta_nova("excurfly");
    let trava_do_excurfly = segurar(&trava_do_projeto(&travas, &excurfly));
    let mut segundo = pesado(&excurfly, &travas, 2, "true");
    assert_eq!(terminou_ate(&mut segundo, PRAZO_DE_ESPERA), None);
    drop(trava_do_excurfly);
    assert_eq!(terminou_ate(&mut segundo, PRAZO_DE_ESPERA), Some(0));
}

#[test]
fn quem_espera_ve_o_comando_que_segura_a_trava() {
    let travas = pasta_nova("travas");
    let excurfly = pasta_nova("excurfly");
    let mut primeiro = pesado(&excurfly, &travas, 2, "sleep 3");
    let caminho = trava_do_projeto(&travas, &excurfly);
    let limite = Instant::now() + PRAZO_DE_ESPERA;
    while !fs::read_to_string(&caminho).unwrap_or_default().contains("sleep 3") {
        assert!(Instant::now() < limite, "o dono não anotou o comando na trava");
        std::thread::sleep(Duration::from_millis(50));
    }
    let segundo = pesado(&excurfly, &travas, 2, "true");
    let saida = segundo.wait_with_output().unwrap();
    let avisos = String::from_utf8_lossy(&saida.stderr);
    assert!(avisos.contains(&format!("(pid {}: sleep 3)", primeiro.id())), "{avisos}");
    primeiro.wait().unwrap();
}

#[test]
fn sem_vaga_livre_o_outro_projeto_espera_uma_soltar() {
    let travas = pasta_nova("travas");
    let organiza_ai = pasta_nova("organiza-ai");
    let vaga_ocupada = segurar(&travas.join("vaga-0.lock"));
    let mut do_organiza_ai = pesado(&organiza_ai, &travas, 1, "true");
    assert_eq!(terminou_ate(&mut do_organiza_ai, PRAZO_DE_ESPERA), None);
    drop(vaga_ocupada);
    assert_eq!(terminou_ate(&mut do_organiza_ai, PRAZO_DE_ESPERA), Some(0));
}

#[test]
fn comando_pesado_dentro_de_outro_nao_espera_a_propria_trava() {
    let travas = pasta_nova("travas");
    let excurfly = pasta_nova("excurfly");
    let watchdogs = env!("CARGO_BIN_EXE_watchdogs");
    let mut externo = pesado(&excurfly, &travas, 1, &format!("'{watchdogs}' pesado true"));
    assert_eq!(terminou_ate(&mut externo, PRAZO_DE_ESPERA), Some(0));
}

// Risco do órfão: o agente derruba o `rodar pesado` e o dev server segue comendo memória sem trava.
#[cfg(target_os = "linux")]
#[test]
fn sigterm_no_rodar_pesado_derruba_todos_os_processos_do_comando() {
    let travas = pasta_nova("travas");
    let excurfly = pasta_nova("excurfly");
    let mut dono = pesado(&excurfly, &travas, 1, "sleep 3101 & sleep 3102");
    std::thread::sleep(Duration::from_millis(400));
    assert!(sleep_vivo(3101) && sleep_vivo(3102), "o comando subiu");
    enviar_sigterm(&dono);
    assert_eq!(terminou_ate(&mut dono, PRAZO_DE_ESPERA), Some(143), "sai com 128 + SIGTERM");
    assert!(!sleep_vivo(3101), "o processo em segundo plano morreu junto");
    assert!(!sleep_vivo(3102), "o processo principal morreu junto");
    let mut seguinte = pesado(&excurfly, &travas, 1, "true");
    assert_eq!(terminou_ate(&mut seguinte, PRAZO_DE_ESPERA), Some(0), "a trava soltou");
}

#[cfg(target_os = "linux")]
#[test]
fn processo_que_o_comando_deixa_para_tras_e_encerrado_com_a_trava() {
    let travas = pasta_nova("travas");
    let excurfly = pasta_nova("excurfly");
    let saida = pesado(&excurfly, &travas, 1, "sleep 3103 & echo subiu").wait_with_output().unwrap();
    assert_eq!(saida.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&saida.stderr).contains("deixou processos em segundo plano"));
    assert!(!sleep_vivo(3103), "nada fica vivo fora da trava");
}

// SIGKILL não se intercepta: a trava solta (flock), mas o filho fica. Limite conhecido (ADR 0025).
#[cfg(target_os = "linux")]
#[test]
fn sigkill_no_dono_solta_a_trava() {
    let travas = pasta_nova("travas");
    let excurfly = pasta_nova("excurfly");
    let mut dono = pesado(&excurfly, &travas, 1, "sleep 3104");
    std::thread::sleep(Duration::from_millis(300));
    dono.kill().unwrap();
    dono.wait().unwrap();
    let mut seguinte = pesado(&excurfly, &travas, 1, "true");
    assert_eq!(terminou_ate(&mut seguinte, PRAZO_DE_ESPERA), Some(0));
    Command::new("pkill").args(["-x", "-f", "sleep 3104"]).status().unwrap();
}

// systemd-run falso: anota os argumentos, responde a sonda com $RESPOSTA_DA_SONDA e roda o resto.
#[cfg(target_os = "linux")]
fn systemd_run_falso(resposta_da_sonda: &str) -> (PathBuf, PathBuf) {
    let pasta = pasta_nova("systemd-falso");
    let registro = pasta.join("chamadas");
    let script = format!(
        "#!/bin/sh\necho \"$*\" >> '{}'\nwhile [ \"$1\" != \"--\" ]; do shift; done; shift\ncase \"$*\" in *memory.max*) echo '{resposta_da_sonda}'; exit 0 ;; esac\nexec \"$@\"\n",
        registro.display()
    );
    let caminho = pasta.join("systemd-run");
    fs::write(&caminho, script).unwrap();
    Command::new("chmod").arg("+x").arg(&caminho).status().unwrap();
    (pasta, registro)
}

#[cfg(target_os = "linux")]
fn com_systemd_falso(pasta_do_falso: &Path) -> Vec<(&'static str, String)> {
    vec![
        ("PATH", format!("{}:{}", pasta_do_falso.display(), std::env::var("PATH").unwrap())),
        ("STACKLABS_LIMITE_DE_MEMORIA", "ligado".into()),
        ("STACKLABS_MEMORIA_POR_VAGA_MB", "1500".into()),
    ]
}

#[cfg(target_os = "linux")]
#[test]
fn com_systemd_o_comando_roda_num_scope_com_o_tamanho_da_vaga() {
    let travas = pasta_nova("travas");
    let excurfly = pasta_nova("excurfly");
    let (pasta_do_falso, registro) = systemd_run_falso(&(1500u64 * 1024 * 1024).to_string());
    let variaveis = com_systemd_falso(&pasta_do_falso);
    let variaveis: Vec<(&str, &str)> = variaveis.iter().map(|(chave, valor)| (*chave, valor.as_str())).collect();
    let saida = pesado_com(&excurfly, &travas, 1, "echo rodou", &variaveis).wait_with_output().unwrap();
    assert_eq!(String::from_utf8_lossy(&saida.stdout).trim(), "rodou");
    let chamadas = fs::read_to_string(&registro).unwrap();
    let linhas: Vec<&str> = chamadas.lines().collect();
    assert_eq!(linhas.len(), 2, "a sonda e o comando: {chamadas}");
    assert!(linhas[0].contains("MemoryMax=1500M") && linhas[0].contains("memory.max"), "{chamadas}");
    assert_eq!(
        linhas[1],
        "--user --scope --quiet --collect -p MemoryHigh=1350M -p MemoryMax=1500M -- sh -c echo rodou"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn sem_o_limite_confirmado_pelo_kernel_avisa_e_roda_so_com_a_trava() {
    let travas = pasta_nova("travas");
    let excurfly = pasta_nova("excurfly");
    let (pasta_do_falso, registro) = systemd_run_falso("max");
    let variaveis = com_systemd_falso(&pasta_do_falso);
    let variaveis: Vec<(&str, &str)> = variaveis.iter().map(|(chave, valor)| (*chave, valor.as_str())).collect();
    let saida = pesado_com(&excurfly, &travas, 1, "echo rodou", &variaveis).wait_with_output().unwrap();
    assert_eq!(saida.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&saida.stdout).trim(), "rodou");
    assert!(String::from_utf8_lossy(&saida.stderr).contains("sem limite de memória por vaga"));
    assert_eq!(fs::read_to_string(&registro).unwrap().lines().count(), 1, "só a sonda passou pelo systemd-run");
}

#[test]
fn as_vagas_e_os_orcamentos_saem_da_memoria_total() {
    assert_eq!(vagas_para_memoria(4 * 1024), 1);
    assert_eq!(vagas_para_memoria(7_800), 2);
    assert_eq!(vagas_para_memoria(16 * 1024), 5);
    assert_eq!(vagas_para_memoria(0), 1);
    assert_eq!(memoria_por_vaga(7_800, 2), 2_876);
    assert_eq!(memoria_por_vaga(16 * 1024, 5), 2_867);
    assert_eq!(memoria_por_vaga(1_000, 4), 1024, "nunca abaixo de 1 GB");
    assert_eq!(memoria_livre_para_entrar(2_876), 1_438);
    assert_eq!(argumentos_do_scope(2_876)[6..9], ["MemoryHigh=2588M", "-p", "MemoryMax=2876M"]);
}

#[test]
fn pastas_com_o_mesmo_nome_em_lugares_diferentes_nao_dividem_a_trava() {
    let app_do_excurfly = pasta_nova("excurfly").join("app");
    let app_do_organiza_ai = pasta_nova("organiza-ai").join("app");
    fs::create_dir_all(&app_do_excurfly).unwrap();
    fs::create_dir_all(&app_do_organiza_ai).unwrap();
    assert!(projeto_da_pasta(&app_do_excurfly).starts_with("app-"));
    assert_ne!(projeto_da_pasta(&app_do_excurfly), projeto_da_pasta(&app_do_organiza_ai));
}

// Risco: um comando aninhado em outro projeto espera uma vaga que o de fora segura: espera sem fim.
#[test]
fn aninhado_em_outro_projeto_pega_a_trava_dele_e_nao_outra_vaga() {
    let travas = pasta_nova("travas");
    let excurfly = pasta_nova("excurfly");
    let organiza_ai = pasta_nova("organiza-ai");
    let watchdogs = env!("CARGO_BIN_EXE_watchdogs");
    let comando = format!("cd '{}' && '{watchdogs}' pesado 'echo dentro'", organiza_ai.display());
    let mut externo = pesado(&excurfly, &travas, 1, &comando);
    assert_eq!(
        terminou_ate(&mut externo, PRAZO_DE_ESPERA),
        Some(0),
        "com 1 vaga, o de dentro não espera a vaga do de fora"
    );
    assert!(trava_do_projeto(&travas, &organiza_ai).exists(), "o de dentro pegou a trava do próprio projeto");
}

// Risco: a variável herdada de um dono que já morreu faz um comando pular todas as travas.
#[test]
fn variavel_de_um_dono_morto_nao_dispensa_as_travas() {
    let travas = pasta_nova("travas");
    let excurfly = pasta_nova("excurfly");
    let vaga_ocupada = segurar(&travas.join("vaga-0.lock"));
    let dono_morto = format!("{}:999999", projeto_da_pasta(&excurfly));
    let mut comando = pesado_com(&excurfly, &travas, 1, "true", &[("STACKLABS_PESADO", &dono_morto)]);
    assert_eq!(terminou_ate(&mut comando, PRAZO_DE_ESPERA), None, "espera a vaga como qualquer outro");
    drop(vaga_ocupada);
    assert_eq!(terminou_ate(&mut comando, PRAZO_DE_ESPERA), Some(0));
}

// Risco: sem terminal (o agente), um prompt de senha do git ou um pager para o comando para sempre,
// segurando a trava e a vaga.
#[cfg(target_os = "linux")]
#[test]
fn sem_terminal_o_comando_nao_para_esperando_o_teclado() {
    let travas = pasta_nova("travas");
    let excurfly = pasta_nova("excurfly");
    let mut comando = pesado(&excurfly, &travas, 1, "if read resposta </dev/tty; then echo leu; else echo sem-terminal; fi");
    assert_eq!(terminou_ate(&mut comando, PRAZO_DE_ESPERA), Some(0));
    let saida = comando.wait_with_output().unwrap();
    assert_eq!(String::from_utf8_lossy(&saida.stdout).trim(), "sem-terminal");
}

// Risco: no terminal de uma pessoa, o comando que lê o teclado para em segundo plano (SIGTTIN).
#[cfg(target_os = "linux")]
#[test]
fn no_terminal_o_comando_le_o_teclado() {
    if Command::new("script").arg("--version").output().is_err() {
        return; // sem util-linux: o pty não se cria aqui
    }
    let travas = pasta_nova("travas");
    let excurfly = pasta_nova("excurfly");
    let watchdogs = env!("CARGO_BIN_EXE_watchdogs");
    let dentro_do_pty = format!(
        "cd '{}' && STACKLABS_TRAVAS='{}' STACKLABS_MEMORIA_POR_VAGA_MB=64 STACKLABS_LIMITE_DE_MEMORIA=desligado '{watchdogs}' pesado 'read resposta </dev/tty; echo \"leu $resposta\"'",
        excurfly.display(),
        travas.display()
    );
    let mut processo = Command::new("script")
        .args(["-qec", &dentro_do_pty, "/dev/null"])
        .env_remove("STACKLABS_PESADO")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_millis(500));
    use std::io::Write;
    processo.stdin.as_mut().unwrap().write_all(b"sim\n").unwrap();
    assert_eq!(terminou_ate(&mut processo, Duration::from_secs(5)), Some(0), "não ficou parado em SIGTTIN");
    let saida = processo.wait_with_output().unwrap();
    assert!(
        String::from_utf8_lossy(&saida.stdout).contains("leu sim"),
        "{}",
        String::from_utf8_lossy(&saida.stdout)
    );
}
