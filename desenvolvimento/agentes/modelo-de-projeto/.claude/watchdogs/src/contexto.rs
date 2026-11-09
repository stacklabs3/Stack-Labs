// Grafo do método por consulta (ADR 0018): liga um símbolo, termo, arquivo ou issue ao que o projeto
// já escreve sobre ele (glossário, ADRs, riscos e testes, issues) e a quem o chama ou muda junto.
// Tudo sai do repositório e do git a cada execução, sem modelo e sem índice guardado: nada envelhece.
// Uso: .claude/watchdogs/rodar contexto <símbolo | termo | caminho | #issue> [--limite <n>]
// Várias palavras viram uma consulta por nome, e as respostas saem juntas: a frase inteira não casa
// com nada, o nome solto casa com a definição, quem chama e o que muda junto.

use crate::riscos::{declaracao_padrao, ler_registro, testes_do_arquivo, TESTES_PADRAO};
use crate::validador::casa_algum;
use regex::Regex;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

pub const LIMITE_PADRAO_POR_SECAO: usize = 5;

// Commits olhados para as issues dos arquivos em foco.
const COMMITS_DE_HISTORICO: &str = "300";

// Janela do "muda junto": os últimos commits do repositório inteiro, para comparar arquivos na mesma base.
const COMMITS_DA_JANELA_DE_MUDANCA: &str = "2000";
const MAXIMO_ARQUIVOS_POR_COMMIT: usize = 50;

// Arquivos em foco mostrados na resposta; os demais entram na conta e aparecem só no total.
const ARQUIVOS_EM_FOCO_MOSTRADOS: usize = 10;

// Extensões de documento: entram como glossário, ADR e risco, nunca como definição ou chamada.
const EXTENSOES_DE_DOCUMENTO: [&str; 4] = ["md", "txt", "csv", "json"];

fn git(raiz: &Path, argumentos: &[&str]) -> Option<String> {
    let saida = Command::new("git")
        .args(argumentos)
        .current_dir(raiz)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    saida.status.success().then(|| String::from_utf8_lossy(&saida.stdout).to_string())
}

fn eh_documento(caminho_relativo: &str) -> bool {
    let extensao = caminho_relativo.rsplit_once('.').map(|(_, extensao)| extensao).unwrap_or("");
    EXTENSOES_DE_DOCUMENTO.contains(&extensao)
}

// Partes da consulta para casar com termos do glossário: aplicar_cupom, aplicarCupom e
// "aplicar cupom" viram ["aplicar", "cupom"].
fn partes_da_consulta(consulta: &str) -> Vec<String> {
    let mut com_separadores = String::new();
    let mut letra_anterior_minuscula = false;
    for letra in consulta.chars() {
        if letra.is_uppercase() && letra_anterior_minuscula {
            com_separadores.push(' ');
        }
        letra_anterior_minuscula = letra.is_lowercase();
        com_separadores.push(letra);
    }
    com_separadores
        .split(|letra: char| !letra.is_alphanumeric())
        .filter(|parte| parte.chars().count() >= 3)
        .map(str::to_lowercase)
        .collect()
}

// Prefixo de 5 letras como radical: "cupom" casa "cupons", "pedido" casa "pedidos".
fn radical(palavra: &str) -> String {
    palavra.chars().take(5).collect()
}

pub struct Ligacao {
    pub rotulo: String,
    pub local: String,
}

pub struct Secao {
    pub titulo: &'static str,
    pub ligacoes: Vec<Ligacao>,
}

pub struct Contexto {
    pub consulta: String,
    pub arquivos_em_foco: Vec<String>,
    pub secoes: Vec<Secao>,
}

// As skills do projeto entram mesmo quando o .gitignore as esconde do git.
fn somar_arquivos_da_pasta(raiz: &Path, pasta_relativa: &str, arquivos: &mut Vec<String>) {
    let Ok(entradas) = fs::read_dir(raiz.join(pasta_relativa)) else { return };
    for entrada in entradas.flatten() {
        let caminho = format!("{pasta_relativa}/{}", entrada.file_name().to_string_lossy());
        if entrada.path().is_dir() {
            somar_arquivos_da_pasta(raiz, &caminho, arquivos);
        } else if !arquivos.contains(&caminho) {
            arquivos.push(caminho);
        }
    }
}

// Arquivo cujo nome ou pasta é a própria consulta: `orquestrar-issues` acha a pasta da skill.
fn arquivos_com_o_nome(arquivos_do_repositorio: &[String], consulta: &str) -> Vec<String> {
    let consulta_minuscula = consulta.trim_end_matches('/').to_lowercase();
    arquivos_do_repositorio
        .iter()
        .filter(|caminho| {
            let caminho_minusculo = caminho.to_lowercase();
            let nome_sem_extensao = Path::new(&caminho_minusculo)
                .file_stem()
                .map(|nome| nome.to_string_lossy().to_string())
                .unwrap_or_default();
            nome_sem_extensao == consulta_minuscula
                || caminho_minusculo.split('/').any(|parte| parte == consulta_minuscula)
                || caminho_minusculo.starts_with(&format!("{consulta_minuscula}/"))
        })
        .cloned()
        .collect()
}

// Os arquivos que a resposta cita, sem linha: o guarda-contexto avisa quando a leitura sai deles.
pub fn arquivos_citados(contexto_montado: &Contexto) -> Vec<String> {
    let mut citados = contexto_montado.arquivos_em_foco.clone();
    for ligacao in contexto_montado.secoes.iter().flat_map(|secao| &secao.ligacoes) {
        for local in ligacao.local.split(", ").filter(|local| !local.starts_with('#')) {
            let caminho = local.split(':').next().unwrap_or("").to_string();
            if !citados.contains(&caminho) {
                citados.push(caminho);
            }
        }
    }
    citados
}

fn listar_arquivos_do_repositorio(raiz: &Path) -> Vec<String> {
    let mut arquivos_do_repositorio: Vec<String> = git(raiz, &["ls-files", "--cached", "--others", "--exclude-standard"])
        .unwrap_or_default()
        .lines()
        .filter(|caminho| !caminho.is_empty() && raiz.join(caminho).is_file())
        .map(String::from)
        .collect();
    somar_arquivos_da_pasta(raiz, ".claude/skills", &mut arquivos_do_repositorio);
    arquivos_do_repositorio
}

// Palavras que não nomeiam nada no código: ficam de fora quando a consulta é uma frase.
const PALAVRAS_SEM_NOME: [&str; 40] = [
    "com", "para", "pra", "que", "como", "quando", "onde", "qual", "quais", "uma", "uns", "umas", "dos", "das", "nos", "nas", "pelo", "pela", "sem", "por",
    "mais", "não", "nao", "seu", "sua", "isso", "este", "esta", "esse", "essa", "the", "and", "for", "with", "from", "how", "what", "where", "when", "into",
];

// Mais nomes que isto numa consulta é uma frase longa: as respostas viram ruído.
const MAXIMO_DE_NOMES_POR_CONSULTA: usize = 6;

// Os nomes de uma consulta: ela inteira quando é um nome, um caminho ou uma issue; numa frase, cada
// palavra que pode nomear algo (identificador, caminho, #issue, termo do glossário).
pub fn nomes_da_consulta(consulta: &str, arquivos_do_repositorio: &[String]) -> Vec<String> {
    let consulta = consulta.trim();
    if !consulta.contains(char::is_whitespace) || arquivos_do_repositorio.iter().any(|caminho| caminho == consulta) {
        return vec![consulta.to_string()];
    }
    let mut nomes: Vec<String> = Vec::new();
    for palavra in consulta.split_whitespace() {
        let nome = palavra
            .trim_matches(|letra: char| matches!(letra, '"' | '\'' | '`' | ',' | ';' | ':' | '(' | ')' | '?' | '!'))
            .trim_end_matches('.');
        let eh_issue = nome
            .strip_prefix('#')
            .is_some_and(|numero| !numero.is_empty() && numero.chars().all(|letra| letra.is_ascii_digit()));
        let letras_do_nome = nome.chars().filter(|letra| letra.is_alphanumeric()).count();
        let pode_nomear = eh_issue || (letras_do_nome >= 3 && !PALAVRAS_SEM_NOME.contains(&nome.to_lowercase().as_str()));
        if pode_nomear && !nomes.iter().any(|existente| existente == nome) {
            nomes.push(nome.to_string());
        }
    }
    // Identificador (snake_case, camelCase, caminho, #issue) nomeia com certeza: havendo algum, a
    // palavra comum ("trata", "sessão") só traria ruído e tempo.
    let eh_identificador = |nome: &String| {
        let letras: Vec<char> = nome.chars().collect();
        nome.contains(['_', '/', '#', '.']) || letras.windows(2).any(|par| par[0].is_lowercase() && par[1].is_uppercase())
    };
    if nomes.iter().any(eh_identificador) {
        nomes.retain(eh_identificador);
    }
    nomes.truncate(MAXIMO_DE_NOMES_POR_CONSULTA);
    if nomes.is_empty() {
        nomes.push(consulta.to_string());
    }
    nomes
}

// Uma resposta por nome da consulta (`nomes_da_consulta`), com a lista de arquivos lida uma vez só.
pub fn montar_contextos(raiz: &Path, consulta: &str, limite_por_secao: usize) -> Vec<Contexto> {
    let arquivos_do_repositorio = listar_arquivos_do_repositorio(raiz);
    nomes_da_consulta(consulta, &arquivos_do_repositorio)
        .iter()
        .map(|nome| montar_contexto_com_arquivos(raiz, nome, limite_por_secao, arquivos_do_repositorio.clone()))
        .collect()
}

pub fn montar_contexto(raiz: &Path, consulta: &str, limite_por_secao: usize) -> Contexto {
    montar_contexto_com_arquivos(raiz, consulta, limite_por_secao, listar_arquivos_do_repositorio(raiz))
}

fn montar_contexto_com_arquivos(raiz: &Path, consulta: &str, limite_por_secao: usize, arquivos_do_repositorio: Vec<String>) -> Contexto {
    let numero_issue = consulta
        .strip_prefix('#')
        .filter(|numero| !numero.is_empty() && numero.chars().all(|letra| letra.is_ascii_digit()));
    let eh_caminho = arquivos_do_repositorio.iter().any(|caminho| caminho == consulta);
    let eh_simbolo = numero_issue.is_none() && !eh_caminho && Regex::new(r"^[A-Za-z_][\w]*$").expect("regex válida").is_match(consulta);
    // Num caminho, só o nome do arquivo diz do que ele trata: as pastas casam com tudo.
    let texto_para_termos = if eh_caminho {
        Path::new(consulta)
            .file_stem()
            .map(|nome| nome.to_string_lossy().to_string())
            .unwrap_or_default()
    } else {
        consulta.to_string()
    };
    let partes = partes_da_consulta(&texto_para_termos);

    let definicoes = if eh_simbolo {
        achar_definicoes(raiz, consulta, &arquivos_do_repositorio)
    } else {
        Vec::new()
    };
    let arquivos_em_foco: Vec<String> = if let Some(numero) = numero_issue {
        arquivos_da_issue(raiz, numero)
    } else if eh_caminho {
        vec![consulta.to_string()]
    } else {
        let mut arquivos_das_definicoes: Vec<String> = Vec::new();
        for definicao in &definicoes {
            let caminho = definicao.local.split(':').next().unwrap_or("").to_string();
            if !arquivos_das_definicoes.contains(&caminho) {
                arquivos_das_definicoes.push(caminho);
            }
        }
        for caminho in arquivos_com_o_nome(&arquivos_do_repositorio, consulta) {
            if !arquivos_das_definicoes.contains(&caminho) {
                arquivos_das_definicoes.push(caminho);
            }
        }
        arquivos_das_definicoes
    };
    // Nome de arquivo só liga um documento ao arquivo quando é único no repositório: SKILL.md,
    // README.md e mod.rs aparecem em todo lugar e ligariam tudo a tudo.
    let nome_do_arquivo = |caminho: &str| {
        Path::new(caminho)
            .file_name()
            .map(|nome| nome.to_string_lossy().to_string())
            .unwrap_or_default()
    };
    let mut arquivos_por_nome: HashMap<String, usize> = HashMap::new();
    for caminho in &arquivos_do_repositorio {
        *arquivos_por_nome.entry(nome_do_arquivo(caminho)).or_default() += 1;
    }
    let nomes_em_foco: Vec<String> = arquivos_em_foco
        .iter()
        .map(|caminho| nome_do_arquivo(caminho))
        .filter(|nome| arquivos_por_nome.get(nome) == Some(&1))
        .collect();
    // O que um documento precisa citar para estar ligado à consulta: o próprio texto ou um arquivo em foco.
    let mut citacoes_procuradas: Vec<String> = nomes_em_foco.clone();
    if numero_issue.is_none() && !eh_caminho {
        citacoes_procuradas.push(consulta.to_string());
    }

    let mut secoes = vec![Secao {
        titulo: "definido em",
        ligacoes: definicoes,
    }];
    secoes.push(Secao {
        titulo: "termos do glossário",
        ligacoes: termos_do_glossario(raiz, &arquivos_do_repositorio, &partes),
    });
    secoes.push(Secao {
        titulo: "decisões (ADR)",
        ligacoes: adrs_que_citam(raiz, &arquivos_do_repositorio, &citacoes_procuradas),
    });
    secoes.push(Secao {
        titulo: "riscos e testes",
        ligacoes: riscos_ligados(raiz, &arquivos_do_repositorio, &citacoes_procuradas, &arquivos_em_foco),
    });
    if eh_simbolo {
        secoes.push(Secao {
            titulo: "chamado por",
            ligacoes: chamadas(raiz, consulta, &secoes[0].ligacoes),
        });
    }
    secoes.push(Secao {
        titulo: "muda junto com",
        ligacoes: mudam_junto(raiz, &arquivos_em_foco),
    });
    if numero_issue.is_none() {
        secoes.push(Secao {
            titulo: "issues",
            ligacoes: issues_dos_arquivos(raiz, &arquivos_em_foco),
        });
    }
    for secao in &mut secoes {
        secao.ligacoes.truncate(limite_por_secao);
    }
    Contexto {
        consulta: consulta.to_string(),
        arquivos_em_foco,
        secoes,
    }
}

// Definição do símbolo nas linguagens da casa: a palavra que declara, depois o nome.
fn achar_definicoes(raiz: &Path, simbolo: &str, arquivos_do_repositorio: &[String]) -> Vec<Ligacao> {
    let declaracao = Regex::new(&format!(
        r"^\s*(?:pub(?:\([^)]*\))?\s+|export\s+(?:default\s+)?|public\s+|private\s+|protected\s+|static\s+|async\s+|abstract\s+|final\s+)*(?:fn|function|def|class|struct|enum|trait|interface|type|impl|const|let|var|val|mod|object|record)\s+{}\b",
        regex::escape(simbolo)
    ))
    .expect("regex válida");
    let mut definicoes = Vec::new();
    for caminho in arquivos_do_repositorio.iter().filter(|caminho| !eh_documento(caminho)) {
        let Ok(texto_arquivo) = fs::read_to_string(raiz.join(caminho)) else {
            continue;
        };
        if !texto_arquivo.contains(simbolo) {
            continue;
        }
        for (indice, linha) in texto_arquivo.lines().enumerate() {
            if declaracao.is_match(linha) {
                definicoes.push(Ligacao {
                    rotulo: linha.trim().chars().take(100).collect(),
                    local: format!("{caminho}:{}", indice + 1),
                });
            }
        }
    }
    definicoes
}

// Arquivos da issue: os dos commits que a citam em qualquer branch (o merge da PR com "Closes #n", o
// "(#n)" do squash, o merge da issue numa branch de integração), cada merge comparado com o primeiro pai,
// e os da branch `issue/<n>-*` que ainda não entrou, comparada com a base de onde saiu.
fn arquivos_da_issue(raiz: &Path, numero_issue: &str) -> Vec<String> {
    let padrao_issue = format!("#{numero_issue}([^0-9]|$)");
    let mut saida = git(
        raiz,
        &[
            "log",
            "--all",
            "-E",
            "--grep",
            &padrao_issue,
            "--name-only",
            "--format=",
            "--diff-merges=first-parent",
        ],
    )
    .unwrap_or_default();
    let padrao_da_branch = format!("issue/{numero_issue}-*");
    let branches_da_issue = git(
        raiz,
        &[
            "for-each-ref",
            "--format=%(refname)",
            &format!("refs/heads/{padrao_da_branch}"),
            &format!("refs/remotes/*/{padrao_da_branch}"),
        ],
    )
    .unwrap_or_default();
    let base_padrao = ["refs/remotes/origin/HEAD", "main", "master"]
        .into_iter()
        .find(|base| git(raiz, &["rev-parse", "--verify", "--quiet", base]).is_some());
    for branch in branches_da_issue.lines().filter(|branch| !branch.is_empty()) {
        let Some(base) = base_padrao.and_then(|base| git(raiz, &["merge-base", base, branch])) else {
            continue;
        };
        saida.push_str(&git(raiz, &["diff", "--name-only", base.trim(), branch]).unwrap_or_default());
    }
    // Sem exigir o arquivo no disco: a branch da issue pode não estar no checkout de quem consulta.
    let mut arquivos_da_issue: Vec<String> = Vec::new();
    for caminho in saida.lines().filter(|caminho| !caminho.is_empty()) {
        if !arquivos_da_issue.iter().any(|existente| existente == caminho) {
            arquivos_da_issue.push(caminho.to_string());
        }
    }
    arquivos_da_issue
}

// Termos do GLOSSARY.md (linhas "**Termo**:") que partilham um radical com a consulta.
fn termos_do_glossario(raiz: &Path, arquivos_do_repositorio: &[String], partes: &[String]) -> Vec<Ligacao> {
    let radicais_da_consulta: BTreeSet<String> = partes.iter().map(|parte| radical(parte)).collect();
    let termo_definido = Regex::new(r"^\*\*([^*]+)\*\*:").expect("regex válida");
    let mut termos = Vec::new();
    for caminho in arquivos_do_repositorio.iter().filter(|caminho| caminho.ends_with("GLOSSARY.md")) {
        let Ok(texto_glossario) = fs::read_to_string(raiz.join(caminho)) else {
            continue;
        };
        for (indice, linha) in texto_glossario.lines().enumerate() {
            let Some(captura) = termo_definido.captures(linha) else { continue };
            let termo = captura[1].trim();
            let casa = partes_da_consulta(termo)
                .iter()
                .any(|parte_do_termo| radicais_da_consulta.contains(&radical(parte_do_termo)));
            if casa {
                termos.push(Ligacao {
                    rotulo: termo.to_string(),
                    local: format!("{caminho}:{}", indice + 1),
                });
            }
        }
    }
    termos
}

fn primeira_linha_util(texto: &str) -> String {
    texto
        .lines()
        .find(|linha| !linha.trim().is_empty())
        .unwrap_or("")
        .trim_start_matches('#')
        .trim()
        .chars()
        .take(100)
        .collect()
}

fn adrs_que_citam(raiz: &Path, arquivos_do_repositorio: &[String], citacoes_procuradas: &[String]) -> Vec<Ligacao> {
    if citacoes_procuradas.is_empty() {
        return Vec::new();
    }
    let citacoes_minusculas: Vec<String> = citacoes_procuradas.iter().map(|citacao| citacao.to_lowercase()).collect();
    let mut adrs = Vec::new();
    for caminho in arquivos_do_repositorio
        .iter()
        .filter(|caminho| caminho.contains("adr/") && caminho.ends_with(".md"))
    {
        let Ok(texto_adr) = fs::read_to_string(raiz.join(caminho)) else { continue };
        let texto_minusculo = texto_adr.to_lowercase();
        if citacoes_minusculas.iter().any(|citacao| texto_minusculo.contains(citacao)) {
            adrs.push(Ligacao {
                rotulo: primeira_linha_util(&texto_adr),
                local: caminho.clone(),
            });
        }
    }
    // A ADR que tem a consulta no título decide sobre ela; a que só a cita no corpo vem depois.
    let cita_no_titulo = |adr: &Ligacao| citacoes_minusculas.iter().any(|citacao| adr.rotulo.to_lowercase().contains(citacao));
    adrs.sort_by_key(|adr| !cita_no_titulo(adr));
    adrs
}

// Riscos citados pelos testes que tocam a consulta (testes nos arquivos em foco ou que a citam),
// com a descrição do registro docs/riscos.md.
fn riscos_ligados(raiz: &Path, arquivos_do_repositorio: &[String], citacoes_procuradas: &[String], arquivos_em_foco: &[String]) -> Vec<Ligacao> {
    let padroes_de_teste: Vec<String> = TESTES_PADRAO.iter().map(|padrao| padrao.to_string()).collect();
    let caminho_registro = "docs/riscos.md";
    let texto_registro = fs::read_to_string(raiz.join(caminho_registro)).unwrap_or_default();
    let linhas_do_registro: Vec<&str> = texto_registro.lines().collect();
    let (riscos_do_registro, _) = ler_registro(&texto_registro);
    let descricao_do_risco = |id_risco: &str| -> Option<(String, u64)> {
        let risco = riscos_do_registro.iter().find(|risco| risco.id == id_risco)?;
        let linha_registro = linhas_do_registro.get(risco.linha as usize - 1).copied().unwrap_or("");
        let descricao = linha_registro.split('|').nth(2).unwrap_or("").trim().to_string();
        Some((descricao, risco.linha))
    };
    let mut testes_por_risco: HashMap<String, Vec<String>> = HashMap::new();
    let mut ordem_dos_riscos: Vec<String> = Vec::new();
    for caminho in arquivos_do_repositorio.iter().filter(|caminho| casa_algum(caminho, &padroes_de_teste)) {
        let Some(declaracao) = declaracao_padrao(caminho) else { continue };
        let Ok(texto_teste) = fs::read_to_string(raiz.join(caminho)) else { continue };
        let toca_a_consulta = arquivos_em_foco.contains(caminho) || citacoes_procuradas.iter().any(|citacao| texto_teste.contains(citacao.as_str()));
        if !toca_a_consulta {
            continue;
        }
        for teste in testes_do_arquivo(&texto_teste, declaracao) {
            for id_risco in teste.riscos {
                if !ordem_dos_riscos.contains(&id_risco) {
                    ordem_dos_riscos.push(id_risco.clone());
                }
                testes_por_risco.entry(id_risco).or_default().push(format!("{caminho}:{}", teste.linha));
            }
        }
    }
    ordem_dos_riscos
        .into_iter()
        .map(|id_risco| {
            let testes_do_risco = testes_por_risco.remove(&id_risco).unwrap_or_default();
            match descricao_do_risco(&id_risco) {
                Some((descricao, linha)) => Ligacao {
                    rotulo: format!("{id_risco} {descricao} ({caminho_registro}:{linha})"),
                    local: testes_do_risco.join(", "),
                },
                None if texto_registro.is_empty() => Ligacao {
                    rotulo: format!("{id_risco} (sem {caminho_registro} neste repositório)"),
                    local: testes_do_risco.join(", "),
                },
                None => Ligacao {
                    rotulo: format!("{id_risco} fora do registro {caminho_registro}"),
                    local: testes_do_risco.join(", "),
                },
            }
        })
        .collect()
}

fn chamadas(raiz: &Path, simbolo: &str, definicoes: &[Ligacao]) -> Vec<Ligacao> {
    let linhas_de_definicao: BTreeSet<&str> = definicoes.iter().map(|definicao| definicao.local.as_str()).collect();
    let saida = git(raiz, &["grep", "-n", "-w", "--untracked", simbolo]).unwrap_or_default();
    let mut chamadas_encontradas = Vec::new();
    for linha in saida.lines() {
        let mut partes = linha.splitn(3, ':');
        let (Some(caminho), Some(numero_linha), Some(trecho)) = (partes.next(), partes.next(), partes.next()) else {
            continue;
        };
        let local = format!("{caminho}:{numero_linha}");
        if eh_documento(caminho) || linhas_de_definicao.contains(local.as_str()) || !trecho.contains(&format!("{simbolo}(")) {
            continue;
        }
        chamadas_encontradas.push(Ligacao {
            rotulo: trecho.trim().chars().take(100).collect(),
            local,
        });
    }
    chamadas_encontradas
}

// Arquivos que mudam junto com os arquivos em foco, pelo índice de Jaccard sobre a mesma janela de
// commits do repositório: commits juntos ÷ commits de qualquer um dos dois. Assim, arquivo que muda
// em todo commit (CLAUDE.md, openapi.json) não sobe só por ser frequente. Commit com mais de
// MAXIMO_ARQUIVOS_POR_COMMIT arquivos é mudança em massa e fica de fora.
fn mudam_junto(raiz: &Path, arquivos_em_foco: &[String]) -> Vec<Ligacao> {
    // Foco largo (a issue que mexeu em 200 arquivos) muda junto com o repositório inteiro: não diz nada.
    if arquivos_em_foco.is_empty() || arquivos_em_foco.len() > ARQUIVOS_EM_FOCO_MOSTRADOS {
        return Vec::new();
    }
    let saida = git(raiz, &["log", "-n", COMMITS_DA_JANELA_DE_MUDANCA, "--name-only", "--format=format:@@commit"]).unwrap_or_default();
    let mut commits_por_arquivo: HashMap<String, usize> = HashMap::new();
    let mut commits_juntos_por_arquivo: HashMap<String, usize> = HashMap::new();
    let mut commits_do_foco = 0;
    for bloco_do_commit in saida.split("@@commit") {
        let arquivos_do_commit: BTreeSet<&str> = bloco_do_commit.lines().filter(|caminho| !caminho.is_empty()).collect();
        if arquivos_do_commit.is_empty() || arquivos_do_commit.len() > MAXIMO_ARQUIVOS_POR_COMMIT {
            continue;
        }
        let toca_o_foco = arquivos_em_foco.iter().any(|em_foco| arquivos_do_commit.contains(em_foco.as_str()));
        if toca_o_foco {
            commits_do_foco += 1;
        }
        for caminho in arquivos_do_commit {
            *commits_por_arquivo.entry(caminho.to_string()).or_default() += 1;
            if toca_o_foco {
                *commits_juntos_por_arquivo.entry(caminho.to_string()).or_default() += 1;
            }
        }
    }
    let mut pontuados: Vec<(String, usize, f64)> = commits_juntos_por_arquivo
        .into_iter()
        .filter(|(caminho, commits_juntos)| *commits_juntos >= 2 && !arquivos_em_foco.iter().any(|em_foco| em_foco == caminho) && raiz.join(caminho).is_file())
        .map(|(caminho, commits_juntos)| {
            let commits_do_outro = commits_por_arquivo.get(&caminho).copied().unwrap_or(commits_juntos);
            let jaccard = commits_juntos as f64 / (commits_do_foco + commits_do_outro - commits_juntos) as f64;
            (caminho, commits_juntos, jaccard)
        })
        .collect();
    pontuados.sort_by(|primeiro, segundo| segundo.2.total_cmp(&primeiro.2).then(primeiro.0.cmp(&segundo.0)));
    pontuados
        .into_iter()
        .map(|(caminho, commits_juntos, jaccard)| Ligacao {
            rotulo: format!(
                "{commits_juntos} de {commits_do_foco} commits, {:.0}% dos commits de qualquer um dos dois",
                jaccard * 100.0
            ),
            local: caminho,
        })
        .collect()
}

fn issues_dos_arquivos(raiz: &Path, arquivos_em_foco: &[String]) -> Vec<Ligacao> {
    if arquivos_em_foco.is_empty() {
        return Vec::new();
    }
    let mut argumentos = vec!["log", "-n", COMMITS_DE_HISTORICO, "--format=%s%n%b@@fim"];
    argumentos.push("--");
    argumentos.extend(arquivos_em_foco.iter().map(String::as_str));
    let saida = git(raiz, &argumentos).unwrap_or_default();
    let citacao_de_issue = Regex::new(r"#(\d+)").expect("regex válida");
    let mut issues = Vec::new();
    let mut numeros_vistos: BTreeSet<String> = BTreeSet::new();
    for mensagem in saida.split("@@fim") {
        let assunto = mensagem.trim().lines().next().unwrap_or("").to_string();
        for captura in citacao_de_issue.captures_iter(mensagem) {
            if numeros_vistos.insert(captura[1].to_string()) {
                issues.push(Ligacao {
                    rotulo: assunto.chars().take(90).collect(),
                    local: format!("#{}", &captura[1]),
                });
            }
        }
    }
    issues
}

// Uma resposta por nome; os nomes sem ligação saem numa linha só, no fim.
pub fn formatar_varios(contextos: &[Contexto]) -> String {
    if let [unico] = contextos {
        return formatar(unico);
    }
    let nada_ligado =
        |contexto_montado: &Contexto| contexto_montado.arquivos_em_foco.is_empty() && contexto_montado.secoes.iter().all(|secao| secao.ligacoes.is_empty());
    let nomes: Vec<&str> = contextos.iter().map(|contexto_montado| contexto_montado.consulta.as_str()).collect();
    let mut blocos = vec![format!(
        "consulta de {} nomes ({}): uma resposta por nome. Da próxima vez, um nome por consulta.",
        contextos.len(),
        nomes.join(", ")
    )];
    blocos.extend(contextos.iter().filter(|contexto_montado| !nada_ligado(contexto_montado)).map(formatar));
    let sem_ligacao: Vec<&str> = contextos
        .iter()
        .filter(|contexto_montado| nada_ligado(contexto_montado))
        .map(|contexto_montado| contexto_montado.consulta.as_str())
        .collect();
    if sem_ligacao.len() == contextos.len() {
        blocos.push("  nada ligado a esta consulta no repositório.".into());
    } else if !sem_ligacao.is_empty() {
        blocos.push(format!("sem ligação no repositório: {}", sem_ligacao.join(", ")));
    }
    blocos.join("\n\n")
}

pub fn formatar(contexto_montado: &Contexto) -> String {
    let mut linhas = vec![format!("contexto de {}", contexto_montado.consulta)];
    if !contexto_montado.arquivos_em_foco.is_empty() {
        let quantidade_em_foco = contexto_montado.arquivos_em_foco.len();
        let mostrados = contexto_montado
            .arquivos_em_foco
            .iter()
            .take(ARQUIVOS_EM_FOCO_MOSTRADOS)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let restantes = quantidade_em_foco.saturating_sub(ARQUIVOS_EM_FOCO_MOSTRADOS);
        let sufixo = if restantes > 0 {
            format!(" (+{restantes}, {quantidade_em_foco} no total)")
        } else {
            String::new()
        };
        linhas.push(format!("  arquivos: {mostrados}{sufixo}"));
    }
    for secao in contexto_montado.secoes.iter().filter(|secao| !secao.ligacoes.is_empty()) {
        linhas.push(format!("  {}:", secao.titulo));
        for ligacao in &secao.ligacoes {
            linhas.push(format!("    {} · {}", ligacao.local, ligacao.rotulo));
        }
    }
    if linhas.len() == 1 {
        linhas.push("  nada ligado a esta consulta no repositório.".into());
    }
    linhas.join("\n")
}

pub fn rodar(argumentos: &[String]) -> i32 {
    let mut consulta_partes: Vec<String> = Vec::new();
    let mut limite_por_secao = LIMITE_PADRAO_POR_SECAO;
    let mut argumentos_restantes = argumentos.iter();
    while let Some(argumento) = argumentos_restantes.next() {
        if argumento == "--limite" {
            match argumentos_restantes.next().and_then(|valor| valor.parse().ok()) {
                Some(limite) => limite_por_secao = limite,
                None => {
                    eprintln!("--limite pede um número");
                    return 2;
                }
            }
        } else {
            consulta_partes.push(argumento.clone());
        }
    }
    let consulta = consulta_partes.join(" ");
    if consulta.trim().is_empty() {
        eprintln!("uso: .claude/watchdogs/rodar contexto <símbolo | termo | caminho | #issue> [--limite <n>]");
        return 2;
    }
    let raiz = git(Path::new("."), &["rev-parse", "--show-toplevel"]).map(|saida| saida.trim().to_string());
    let Some(raiz) = raiz else {
        eprintln!("o contexto roda dentro de um repositório git");
        return 2;
    };
    println!("{}", formatar_varios(&montar_contextos(Path::new(&raiz), consulta.trim(), limite_por_secao)));
    0
}
