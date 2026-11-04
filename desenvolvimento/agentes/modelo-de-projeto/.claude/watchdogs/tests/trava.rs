// Testes do subcomando pesado: o binário roda com a pasta das travas e o número de vagas do teste, e
// o teste segura à mão a trava de um projeto ou uma vaga para ver quem espera e quem passa.

use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use watchdogs::trava::{node_options_com_teto, vagas_para_memoria};

const PRAZO_DE_ESPERA: Duration = Duration::from_millis(1500);

fn pasta_nova(prefixo: &str) -> PathBuf {
    static PROXIMO: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let ordem = PROXIMO.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let sufixo = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let pasta = std::env::temp_dir().join(format!("{prefixo}-{}-{ordem}-{sufixo}", std::process::id()));
    fs::create_dir_all(&pasta).unwrap();
    pasta
}

// Um projeto (pasta sem git: o nome vem da pasta) que roda `rodar pesado` com as travas em `travas`.
fn pesado(projeto: &Path, travas: &Path, vagas: usize, comando: &str) -> Child {
    Command::new(env!("CARGO_BIN_EXE_watchdogs"))
        .args(["pesado", comando])
        .current_dir(projeto)
        .env("STACKLABS_TRAVAS", travas)
        .env("STACKLABS_VAGAS", vagas.to_string())
        .env_remove("STACKLABS_PESADO")
        .env_remove("NODE_OPTIONS")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
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

#[test]
fn roda_o_comando_com_o_heap_limitado_e_devolve_a_saida_dele() {
    let projeto = pasta_nova("excurfly");
    let travas = pasta_nova("travas");
    let processo = pesado(&projeto, &travas, 2, "echo \"$NODE_OPTIONS|$STACKLABS_PESADO\"; exit 3");
    let saida = processo.wait_with_output().unwrap();
    assert_eq!(saida.status.code(), Some(3));
    let impresso = String::from_utf8_lossy(&saida.stdout);
    assert!(impresso.contains("--max-old-space-size=2048|"), "{impresso}");
    assert!(travas
        .join(format!("stacklabs-{}.lock", projeto.file_name().unwrap().to_string_lossy()))
        .exists());
}

#[test]
fn dois_projetos_rodam_juntos_quando_ha_vaga() {
    let travas = pasta_nova("travas");
    let excurfly = pasta_nova("excurfly");
    let organiza_ai = pasta_nova("organiza-ai");
    let _excurfly_ocupado = segurar(&travas.join(format!("stacklabs-{}.lock", excurfly.file_name().unwrap().to_string_lossy())));
    let _vaga_do_excurfly = segurar(&travas.join("vaga-0.lock"));
    let mut do_organiza_ai = pesado(&organiza_ai, &travas, 2, "true");
    assert_eq!(terminou_ate(&mut do_organiza_ai, PRAZO_DE_ESPERA), Some(0));
}

#[test]
fn o_mesmo_projeto_espera_o_comando_pesado_anterior() {
    let travas = pasta_nova("travas");
    let excurfly = pasta_nova("excurfly");
    let trava_do_excurfly = segurar(&travas.join(format!("stacklabs-{}.lock", excurfly.file_name().unwrap().to_string_lossy())));
    let mut segundo = pesado(&excurfly, &travas, 2, "true");
    assert_eq!(terminou_ate(&mut segundo, PRAZO_DE_ESPERA), None);
    drop(trava_do_excurfly);
    assert_eq!(terminou_ate(&mut segundo, PRAZO_DE_ESPERA), Some(0));
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

#[test]
fn a_trava_solta_quando_o_dono_morre() {
    let travas = pasta_nova("travas");
    let excurfly = pasta_nova("excurfly");
    let mut dono = pesado(&excurfly, &travas, 1, "sleep 30");
    std::thread::sleep(Duration::from_millis(300));
    dono.kill().unwrap();
    dono.wait().unwrap();
    let mut seguinte = pesado(&excurfly, &travas, 1, "true");
    assert_eq!(terminou_ate(&mut seguinte, PRAZO_DE_ESPERA), Some(0));
}

#[test]
fn as_vagas_saem_da_memoria_total() {
    assert_eq!(vagas_para_memoria(4 * 1024), 1);
    assert_eq!(vagas_para_memoria(7_800), 2);
    assert_eq!(vagas_para_memoria(16 * 1024), 5);
    assert_eq!(vagas_para_memoria(0), 1);
}

#[test]
fn o_teto_do_heap_respeita_o_que_o_comando_ja_trouxe() {
    assert_eq!(node_options_com_teto("", 2048), "--max-old-space-size=2048");
    assert_eq!(node_options_com_teto("--inspect", 2048), "--inspect --max-old-space-size=2048");
    assert_eq!(node_options_com_teto("--max-old-space-size=1024", 2048), "--max-old-space-size=1024");
}
