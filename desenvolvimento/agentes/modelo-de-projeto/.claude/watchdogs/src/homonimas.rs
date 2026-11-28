// Skills pessoais que encobrem as do projeto (ADR 0028). No Claude Code, `~/.claude/skills/<nome>`
// vence `.claude/skills/<nome>` do projeto: a sessão rodaria a cópia pessoal, não a do método.

use std::fs;
use std::path::{Path, PathBuf};

/// Moram na pasta pessoal por design e se atualizam pelo `--conferir-skill`: nunca contam.
pub const SKILLS_DE_MAQUINA: [&str; 3] = ["setup-stacklabs", "adotar-metodo-stack-labs", "auditar-metodo"];

/// Pastas e arquivos que não fazem parte do conteúdo de uma skill.
const IGNORADOS_NA_COMPARACAO: [&str; 2] = ["target", ".DS_Store"];

/// `$CLAUDE_CONFIG_DIR/skills`, ou `$HOME/.claude/skills` sem a variável.
pub fn pasta_das_skills_pessoais() -> Option<PathBuf> {
    if let Some(pasta_de_configuracao) = std::env::var_os("CLAUDE_CONFIG_DIR").filter(|valor| !valor.is_empty()) {
        return Some(PathBuf::from(pasta_de_configuracao).join("skills"));
    }
    std::env::var_os("HOME")
        .filter(|valor| !valor.is_empty())
        .map(|pasta_do_usuario| PathBuf::from(pasta_do_usuario).join(".claude").join("skills"))
}

fn arquivos_da_skill(pasta_da_skill: &Path) -> Option<Vec<(String, Vec<u8>)>> {
    fn visitar(pasta_atual: &Path, prefixo: &str, arquivos: &mut Vec<(String, Vec<u8>)>) -> Option<()> {
        for entrada in fs::read_dir(pasta_atual).ok()? {
            let entrada = entrada.ok()?;
            let nome_da_entrada = entrada.file_name().to_string_lossy().into_owned();
            if IGNORADOS_NA_COMPARACAO.contains(&nome_da_entrada.as_str()) {
                continue;
            }
            let caminho_relativo = format!("{prefixo}{nome_da_entrada}");
            let caminho_da_entrada = entrada.path();
            if caminho_da_entrada.is_dir() {
                visitar(&caminho_da_entrada, &format!("{caminho_relativo}/"), arquivos)?;
            } else {
                arquivos.push((caminho_relativo, fs::read(&caminho_da_entrada).ok()?));
            }
        }
        Some(())
    }
    let mut arquivos = Vec::new();
    visitar(pasta_da_skill, "", &mut arquivos)?;
    arquivos.sort();
    Some(arquivos)
}

/// Iguais quando têm os mesmos arquivos com os mesmos bytes, fora `target/`. Ilegível conta como diferente.
pub fn skills_iguais(pasta_a: &Path, pasta_b: &Path) -> bool {
    match (arquivos_da_skill(pasta_a), arquivos_da_skill(pasta_b)) {
        (Some(arquivos_a), Some(arquivos_b)) => arquivos_a == arquivos_b,
        _ => false,
    }
}

/// As skills do projeto que uma pessoal de mesmo nome e conteúdo diferente encobre, em ordem.
pub fn skills_encobertas_em(pasta_das_skills_do_projeto: &Path, pasta_pessoal: &Path) -> Vec<String> {
    let Ok(entradas) = fs::read_dir(pasta_das_skills_do_projeto) else {
        return Vec::new();
    };
    let mut encobertas: Vec<String> = entradas
        .filter_map(Result::ok)
        .filter(|entrada| entrada.path().is_dir())
        .map(|entrada| entrada.file_name().to_string_lossy().into_owned())
        .filter(|nome_da_skill| !SKILLS_DE_MAQUINA.contains(&nome_da_skill.as_str()))
        .filter(|nome_da_skill| {
            let pasta_homonima = pasta_pessoal.join(nome_da_skill);
            pasta_homonima.is_dir() && !skills_iguais(&pasta_das_skills_do_projeto.join(nome_da_skill), &pasta_homonima)
        })
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
pub fn skill_esta_encoberta(diretorio_projeto: &Path, nome_digitado: &str) -> Option<String> {
    let nome_da_skill = nome_digitado.trim().trim_start_matches('/');
    if nome_da_skill.is_empty() || nome_da_skill.contains(':') || nome_da_skill.contains('/') {
        return None;
    }
    skills_encobertas(diretorio_projeto).into_iter().find(|encoberta| encoberta == nome_da_skill)
}

pub fn como_resolver(nomes_das_skills: &[String]) -> String {
    let pasta_pessoal = pasta_das_skills_pessoais()
        .map(|pasta| pasta.display().to_string())
        .unwrap_or_else(|| "~/.claude/skills".into());
    format!(
        "A cópia pessoal em {pasta_pessoal}/ vence a do projeto e difere dela: {}. Tire a pessoal do caminho com `sh {pasta_pessoal}/setup-stacklabs/rodar --mover-homonimas` (move para backup, nunca apaga) ou resolva no repositório da pasta pessoal.",
        nomes_das_skills.join(", ")
    )
}
