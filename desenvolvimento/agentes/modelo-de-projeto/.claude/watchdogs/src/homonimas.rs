// Skills pessoais que encobrem as do projeto (ADR 0028). No Claude Code, `~/.claude/skills/<nome>`
// vence `.claude/skills/<nome>` do projeto: a sessão rodaria a cópia pessoal, não a do método.
//
// A mesma regra mora no instalador (`setup-stacklabs/src/main.rs`, seção "skills pessoais
// homônimas"), que não pode depender deste crate. As duas cópias seguem os mesmos casos de teste:
// mudou aqui, mude lá.

use std::fs;
use std::path::{Path, PathBuf};

/// Moram na pasta pessoal por design e se atualizam pelo `--conferir-skill`: nunca contam.
pub const SKILLS_DE_MAQUINA: [&str; 3] = ["setup-stacklabs", "adotar-metodo-stack-labs", "auditar-metodo"];

/// Pastas e arquivos que não fazem parte do conteúdo de uma skill.
const IGNORADOS_NA_COMPARACAO: [&str; 2] = ["target", ".DS_Store"];

/// O que uma entrada de dentro da skill guarda: os bytes de um arquivo ou o alvo de um link.
#[derive(PartialEq, PartialOrd, Eq, Ord)]
enum ConteudoDaEntrada {
    Arquivo(Vec<u8>),
    Link(PathBuf),
}

/// `$CLAUDE_CONFIG_DIR/skills`, ou `$HOME/.claude/skills` sem a variável.
pub fn pasta_das_skills_pessoais() -> Option<PathBuf> {
    if let Some(pasta_de_configuracao) =
        std::env::var_os("CLAUDE_CONFIG_DIR").filter(|caminho_da_configuracao| !caminho_da_configuracao.is_empty())
    {
        return Some(PathBuf::from(pasta_de_configuracao).join("skills"));
    }
    std::env::var_os("HOME")
        .filter(|caminho_do_usuario| !caminho_do_usuario.is_empty())
        .map(|pasta_do_usuario| PathBuf::from(pasta_do_usuario).join(".claude").join("skills"))
}

/// A pasta da skill é pasta mesmo ou um link para uma pasta: o Claude Code carrega as duas.
fn eh_pasta_de_skill(caminho_da_skill: &Path) -> bool {
    fs::metadata(caminho_da_skill).is_ok_and(|dados_da_skill| dados_da_skill.is_dir())
}

/// Os arquivos da skill, sem seguir link de dentro dela (um link para a pasta de cima entraria em laço).
fn arquivos_da_skill(pasta_da_skill: &Path) -> Option<Vec<(String, ConteudoDaEntrada)>> {
    fn visitar(pasta_atual: &Path, prefixo: &str, arquivos: &mut Vec<(String, ConteudoDaEntrada)>) -> Option<()> {
        for entrada in fs::read_dir(pasta_atual).ok()? {
            let entrada = entrada.ok()?;
            let nome_da_entrada = entrada.file_name().to_string_lossy().into_owned();
            if IGNORADOS_NA_COMPARACAO.contains(&nome_da_entrada.as_str()) {
                continue;
            }
            let caminho_relativo = format!("{prefixo}{nome_da_entrada}");
            let tipo_da_entrada = entrada.file_type().ok()?;
            if tipo_da_entrada.is_symlink() {
                arquivos.push((caminho_relativo, ConteudoDaEntrada::Link(fs::read_link(entrada.path()).ok()?)));
            } else if tipo_da_entrada.is_dir() {
                visitar(&entrada.path(), &format!("{caminho_relativo}/"), arquivos)?;
            } else {
                arquivos.push((caminho_relativo, ConteudoDaEntrada::Arquivo(fs::read(entrada.path()).ok()?)));
            }
        }
        Some(())
    }
    let mut arquivos = Vec::new();
    visitar(pasta_da_skill, "", &mut arquivos)?;
    arquivos.sort();
    Some(arquivos)
}

/// Iguais quando têm as mesmas entradas com o mesmo conteúdo, fora `target/` e `.DS_Store`.
/// Ilegível conta como diferente.
pub fn skills_iguais(pasta_a: &Path, pasta_b: &Path) -> bool {
    match (arquivos_da_skill(pasta_a), arquivos_da_skill(pasta_b)) {
        (Some(arquivos_a), Some(arquivos_b)) => arquivos_a == arquivos_b,
        _ => false,
    }
}

/// A skill do projeto com esse nome está encoberta por uma pessoal de conteúdo diferente.
fn esta_encoberta(pasta_das_skills_do_projeto: &Path, pasta_pessoal: &Path, nome_da_skill: &str) -> bool {
    let pasta_do_projeto = pasta_das_skills_do_projeto.join(nome_da_skill);
    let pasta_homonima = pasta_pessoal.join(nome_da_skill);
    !SKILLS_DE_MAQUINA.contains(&nome_da_skill)
        && eh_pasta_de_skill(&pasta_do_projeto)
        && eh_pasta_de_skill(&pasta_homonima)
        && !skills_iguais(&pasta_do_projeto, &pasta_homonima)
}

/// As skills do projeto que uma pessoal de mesmo nome e conteúdo diferente encobre, em ordem.
pub fn skills_encobertas_em(pasta_das_skills_do_projeto: &Path, pasta_pessoal: &Path) -> Vec<String> {
    let Ok(entradas) = fs::read_dir(pasta_das_skills_do_projeto) else {
        return Vec::new();
    };
    let mut encobertas: Vec<String> = entradas
        .filter_map(Result::ok)
        .map(|entrada| entrada.file_name().to_string_lossy().into_owned())
        .filter(|nome_da_skill| esta_encoberta(pasta_das_skills_do_projeto, pasta_pessoal, nome_da_skill))
        .collect();
    encobertas.sort();
    encobertas
}

pub fn skills_encobertas(diretorio_projeto: &Path) -> Vec<String> {
    match pasta_das_skills_pessoais() {
        Some(pasta_pessoal) => skills_encobertas_em(&diretorio_projeto.join(".claude").join("skills"), &pasta_pessoal),
        None => Vec::new(),
    }
}

/// O nome como chega do Skill tool ou do prompt (`/tdd`, `tdd`); `plugin:skill` não é da pasta pessoal.
/// Compara só a skill pedida, não todas as do projeto.
pub fn skill_esta_encoberta(diretorio_projeto: &Path, nome_digitado: &str) -> Option<String> {
    let nome_da_skill = nome_digitado.trim().trim_start_matches('/');
    if nome_da_skill.is_empty() || nome_da_skill.contains(':') || nome_da_skill.contains('/') {
        return None;
    }
    let pasta_pessoal = pasta_das_skills_pessoais()?;
    esta_encoberta(&diretorio_projeto.join(".claude").join("skills"), &pasta_pessoal, nome_da_skill)
        .then(|| nome_da_skill.to_string())
}

pub fn como_resolver(nomes_das_skills: &[String]) -> String {
    let pasta_pessoal = pasta_das_skills_pessoais()
        .map(|pasta_pessoal_resolvida| pasta_pessoal_resolvida.display().to_string())
        .unwrap_or_else(|| "~/.claude/skills".into());
    let instalador = format!("{pasta_pessoal}/setup-stacklabs/rodar");
    let como_tirar = if Path::new(&instalador).is_file() {
        format!(
            "Tire a pessoal do caminho com `sh {instalador} --mover-homonimas` (move para backup, nunca apaga; se ele não conhecer a opção, atualize antes com `sh {instalador} --conferir-skill`)"
        )
    } else {
        format!(
            "A skill de máquina `setup-stacklabs` não está em {pasta_pessoal}/: instale-a como manda o AGENTS.md do Stack-Labs e rode `sh {instalador} --mover-homonimas` (move para backup, nunca apaga)"
        )
    };
    format!(
        "A cópia pessoal em {pasta_pessoal}/ vence a do projeto e difere dela: {}. {como_tirar}, ou resolva no repositório da pasta pessoal.",
        nomes_das_skills.join(", ")
    )
}
