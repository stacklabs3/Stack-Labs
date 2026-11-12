// Grafo do método por consulta (ADR 0018): liga um símbolo, termo, arquivo ou issue ao que o projeto
// já escreve sobre ele (glossário, ADRs, riscos e testes, issues), a quem o chama ou muda junto e aos
// arquivos de código e testes que tratam dos termos da consulta.
// Tudo sai do repositório e do git a cada execução, sem modelo e sem índice guardado: nada envelhece.
// Uso: .claude/watchdogs/rodar contexto <símbolo | termo | caminho | #issue> [--limite <n>]
// Várias palavras viram uma consulta por nome, e as respostas saem juntas: a frase inteira não casa
// com nada, o nome solto casa com a definição, quem chama e o que muda junto.

use crate::comum::{caminho_eh_livre_sem_issue, ler_ajustes_do_projeto, AjustesDoProjeto};
use crate::riscos::{declaracao_padrao, ler_registro, testes_do_arquivo, TESTES_PADRAO};
use crate::validador::{casa_algum, glob_para_regex};
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

// Seções de arquivos (código e testes) mostram mais linhas que as outras: são o mapa da navegação.
const MULTIPLICADOR_DO_LIMITE_DE_ARQUIVOS: usize = 3;

// Arquivos de código cujas issues entram quando a consulta é texto livre, sem arquivo em foco.
const ARQUIVOS_PARA_AS_ISSUES: usize = 5;

// Testes citados por risco; os demais aparecem só na conta.
const TESTES_MOSTRADOS_POR_RISCO: usize = 3;

// Arquivo maior que isso é dado ou código gerado: fica fora da busca por termos.
const TAMANHO_MAXIMO_LIDO: u64 = 256 * 1024;

// Palavras de ligação com 3 letras ou mais: não dizem do que a consulta trata.
const PALAVRAS_DE_LIGACAO: [&str; 16] = [
    "com", "das", "dos", "nas", "nos", "num", "numa", "para", "pela", "pelo", "por", "que", "sem", "uma", "and", "the",
];

// O mapa do código do projeto: onde mora cada camada de uma fatia e o vizinho para imitar.
pub const MAPA_DO_CODIGO: &str = "docs/mapa-do-codigo.md";

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

// Minúsculas sem acento: "Cartão" casa `cartao-da-entrada.tsx`.
fn sem_acentos(texto: &str) -> String {
    texto
        .to_lowercase()
        .chars()
        .map(|letra| match letra {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            outra_letra => outra_letra,
        })
        .collect()
}

// Os termos que dizem do que a consulta trata, cada um com o radical sem acento.
struct TermoDaConsulta {
    palavra: String,
    radical: String,
}

fn termos_da_consulta(partes: &[String]) -> Vec<TermoDaConsulta> {
    let mut termos: Vec<TermoDaConsulta> = Vec::new();
    for parte in partes {
        let parte_sem_acento = sem_acentos(parte);
        if PALAVRAS_DE_LIGACAO.contains(&parte_sem_acento.as_str()) {
            continue;
        }
        let radical_da_parte = radical(&parte_sem_acento);
        if !termos.iter().any(|termo| termo.radical == radical_da_parte) {
            termos.push(TermoDaConsulta {
                palavra: parte_sem_acento,
                radical: radical_da_parte,
            });
        }
    }
    termos
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
    pub mapa_do_codigo: Option<String>,
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
    citados.extend(contexto_montado.mapa_do_codigo.clone());
    for ligacao in contexto_montado.secoes.iter().flat_map(|secao| &secao.ligacoes) {
        for local in ligacao.local.split(", ").filter(|local| !local.starts_with('#') && !local.starts_with('+')) {
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
        let pode_nomear = eh_issue || letras_do_nome >= 3;
        if pode_nomear && !nomes.iter().any(|existente| existente == nome) {
            nomes.push(nome.to_string());
        }
    }
    // Identificador (snake_case, camelCase, caminho, #issue) nomeia com certeza: havendo algum, cada um
    // vira uma consulta, e a palavra comum ("trata", "sessão") só traria ruído e tempo.
    let eh_identificador = |nome: &String| {
        let letras: Vec<char> = nome.chars().collect();
        nome.contains(['_', '/', '#', '.']) || letras.windows(2).any(|par| par[0].is_lowercase() && par[1].is_uppercase())
    };
    // Sem identificador, a frase é texto livre: os termos dela já levam ao código e aos testes inteira.
    if !nomes.iter().any(eh_identificador) {
        return vec![consulta.to_string()];
    }
    nomes.retain(eh_identificador);
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

    let termos_definidos = termos_do_glossario(raiz, &arquivos_do_repositorio, &partes);
    // Num texto livre, o termo do glossário que a consulta usa também liga a ADR e o risco que o citam.
    let nomes_dos_termos: Vec<String> = if numero_issue.is_none() && !eh_caminho {
        termos_definidos.iter().map(|termo| termo.rotulo.clone()).collect()
    } else {
        Vec::new()
    };
    // O risco vem do teste dos arquivos achados; o termo do glossário citado em teste ligaria quase todos.
    let citacoes_dos_testes = citacoes_procuradas.clone();
    citacoes_procuradas.extend(nomes_dos_termos.iter().cloned());

    // Os arquivos que tratam dos termos da consulta, menos os que já estão em foco.
    let limite_de_arquivos = limite_por_secao * MULTIPLICADOR_DO_LIMITE_DE_ARQUIVOS;
    let (codigo_dos_termos, testes_dos_termos) = if numero_issue.is_none() && !eh_caminho {
        let termos = termos_da_consulta(&partes);
        let ajustes = ler_ajustes_do_projeto(raiz);
        let candidatos: Vec<String> = arquivos_do_repositorio
            .iter()
            .filter(|caminho| !arquivos_em_foco.contains(caminho) && eh_codigo(caminho, &ajustes))
            .cloned()
            .collect();
        arquivos_dos_termos(raiz, &candidatos, &termos, limite_de_arquivos)
    } else {
        (Vec::new(), Vec::new())
    };
    let mut arquivos_dos_riscos = arquivos_em_foco.clone();
    arquivos_dos_riscos.extend(testes_dos_termos.iter().map(|teste| teste.local.clone()));

    let mut secoes = vec![Secao {
        titulo: "definido em",
        ligacoes: definicoes,
    }];
    let arquivos_para_as_issues: Vec<String> = if arquivos_em_foco.is_empty() {
        codigo_dos_termos
            .iter()
            .take(ARQUIVOS_PARA_AS_ISSUES)
            .map(|codigo| codigo.local.clone())
            .collect()
    } else {
        arquivos_em_foco.clone()
    };
    secoes.push(Secao {
        titulo: "código",
        ligacoes: codigo_dos_termos,
    });
    secoes.push(Secao {
        titulo: "testes",
        ligacoes: testes_dos_termos,
    });
    secoes.push(Secao {
        titulo: "termos do glossário",
        ligacoes: termos_definidos,
    });
    secoes.push(Secao {
        titulo: "decisões (ADR)",
        ligacoes: adrs_que_citam(raiz, &arquivos_do_repositorio, &citacoes_procuradas),
    });
    let mut riscos = riscos_ligados(raiz, &arquivos_do_repositorio, &citacoes_dos_testes, &arquivos_dos_riscos);
    riscos.extend(riscos_do_registro_que_citam(raiz, &nomes_dos_termos, &riscos));
    secoes.push(Secao {
        titulo: "riscos e testes",
        ligacoes: riscos,
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
            ligacoes: issues_dos_arquivos(raiz, &arquivos_para_as_issues),
        });
    }
    for secao in &mut secoes {
        let limite_da_secao = if matches!(secao.titulo, "código" | "testes") {
            limite_de_arquivos
        } else {
            limite_por_secao
        };
        secao.ligacoes.truncate(limite_da_secao);
    }
    let mapa_do_codigo = raiz.join(MAPA_DO_CODIGO).is_file().then(|| MAPA_DO_CODIGO.to_string());
    Contexto {
        consulta: consulta.to_string(),
        arquivos_em_foco,
        mapa_do_codigo,
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

// O nome do arquivo sem as extensões: `cartao-da-entrada.test.tsx` vira `cartao-da-entrada`.
fn nome_base(caminho: &str) -> String {
    let nome_arquivo = caminho.rsplit('/').next().unwrap_or(caminho);
    nome_arquivo.split('.').next().unwrap_or(nome_arquivo).to_string()
}

fn pasta_do_arquivo(caminho: &str) -> &str {
    caminho.rsplit_once('/').map(|(pasta, _)| pasta).unwrap_or("")
}

struct ArquivoDosTermos {
    caminho: String,
    termos_no_caminho: Vec<String>,
    termos_no_texto: Vec<String>,
}

impl ArquivoDosTermos {
    // O termo no caminho pesa mais que no texto: `telas/cartao-da-entrada.tsx` trata da entrada, um
    // arquivo que só cita `entrada` passa por ela.
    fn pontuacao(&self) -> usize {
        3 * self.termos_no_caminho.len() + self.termos_no_texto.len()
    }

    fn rotulo(&self) -> String {
        let mut partes_do_rotulo = Vec::new();
        if !self.termos_no_caminho.is_empty() {
            partes_do_rotulo.push(format!("nome: {}", self.termos_no_caminho.join(", ")));
        }
        let so_no_texto: Vec<&String> = self.termos_no_texto.iter().filter(|termo| !self.termos_no_caminho.contains(termo)).collect();
        if !so_no_texto.is_empty() {
            partes_do_rotulo.push(format!(
                "texto: {}",
                so_no_texto.iter().map(|termo| termo.as_str()).collect::<Vec<_>>().join(", ")
            ));
        }
        partes_do_rotulo.join("; ")
    }
}

// Arquivos de código e testes que tratam dos termos da consulta, os mais ligados primeiro. Sem termo no
// caminho, o texto precisa de dois termos (ou do único que a consulta tem). Cada teste vem ao lado do
// código que testa: mesmo nome na mesma pasta, ou o teste que importa o arquivo.
fn arquivos_dos_termos(raiz: &Path, candidatos: &[String], termos: &[TermoDaConsulta], limite_de_arquivos: usize) -> (Vec<Ligacao>, Vec<Ligacao>) {
    if termos.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let padroes_de_teste: Vec<Regex> = TESTES_PADRAO.iter().map(|padrao| glob_para_regex(padrao)).collect();
    let eh_teste = |caminho: &str| padroes_de_teste.iter().any(|padrao| padrao.is_match(caminho));
    let termos_exigidos_no_texto = termos.len().min(2);
    let mut achados: Vec<ArquivoDosTermos> = Vec::new();
    let mut textos_dos_testes: HashMap<String, String> = HashMap::new();
    for caminho in candidatos.iter().filter(|caminho| !eh_documento(caminho)) {
        let caminho_completo = raiz.join(caminho);
        let pequeno = fs::metadata(&caminho_completo).map(|dados| dados.len() <= TAMANHO_MAXIMO_LIDO).unwrap_or(false);
        if !pequeno {
            continue;
        }
        let Ok(texto_arquivo) = fs::read_to_string(&caminho_completo) else { continue };
        let caminho_sem_acento = sem_acentos(caminho);
        let texto_sem_acento = sem_acentos(&texto_arquivo);
        let termos_no_caminho: Vec<String> = termos
            .iter()
            .filter(|termo| caminho_sem_acento.contains(&termo.radical))
            .map(|termo| termo.palavra.clone())
            .collect();
        let termos_no_texto: Vec<String> = termos
            .iter()
            .filter(|termo| texto_sem_acento.contains(&termo.radical))
            .map(|termo| termo.palavra.clone())
            .collect();
        if eh_teste(caminho) {
            textos_dos_testes.insert(caminho.clone(), texto_arquivo);
        }
        if termos_no_caminho.is_empty() && termos_no_texto.len() < termos_exigidos_no_texto {
            continue;
        }
        achados.push(ArquivoDosTermos {
            caminho: caminho.clone(),
            termos_no_caminho,
            termos_no_texto,
        });
    }
    achados.sort_by(|primeiro, segundo| segundo.pontuacao().cmp(&primeiro.pontuacao()).then(primeiro.caminho.cmp(&segundo.caminho)));
    let (testes_achados, codigo_achado): (Vec<ArquivoDosTermos>, Vec<ArquivoDosTermos>) = achados.into_iter().partition(|achado| eh_teste(&achado.caminho));
    let codigo_mostrado: Vec<ArquivoDosTermos> = codigo_achado.into_iter().take(limite_de_arquivos).collect();

    // Para cada teste: o primeiro código mostrado que ele testa e os termos que ele mesmo tem.
    let mut codigo_testado: HashMap<String, (usize, String)> = HashMap::new();
    for (posicao_do_codigo, codigo) in codigo_mostrado.iter().enumerate() {
        let nome_do_codigo = nome_base(&codigo.caminho);
        let pasta_do_codigo = pasta_do_arquivo(&codigo.caminho);
        // `page`, `acao` e `index` existem em toda pasta: a importação só vale com a pasta junto.
        let pasta_e_nome = format!("{}/{nome_do_codigo}", pasta_do_codigo.rsplit('/').next().unwrap_or(""));
        let importacoes = [format!("{pasta_e_nome}\""), format!("{pasta_e_nome}'")];
        for (caminho_teste, texto_teste) in &textos_dos_testes {
            let mesmo_nome = nome_base(caminho_teste) == nome_do_codigo && pasta_do_arquivo(caminho_teste) == pasta_do_codigo;
            let importa = importacoes.iter().any(|importacao| texto_teste.contains(importacao.as_str()));
            if mesmo_nome || importa {
                codigo_testado
                    .entry(caminho_teste.clone())
                    .or_insert((posicao_do_codigo, codigo.caminho.clone()));
            }
        }
    }
    let mut testes_pontuados: Vec<(usize, usize, Ligacao)> = Vec::new();
    let mut caminhos_dos_testes = BTreeSet::new();
    for teste in testes_achados {
        caminhos_dos_testes.insert(teste.caminho.clone());
        let testado = codigo_testado.get(&teste.caminho);
        let posicao_do_codigo = testado.map(|(posicao, _)| *posicao).unwrap_or(usize::MAX);
        let mut rotulo = teste.rotulo();
        if let Some((_, caminho_do_codigo)) = testado {
            rotulo = format!("{rotulo}; testa {caminho_do_codigo}");
        }
        testes_pontuados.push((teste.pontuacao(), posicao_do_codigo, Ligacao { rotulo, local: teste.caminho }));
    }
    for (caminho_teste, (posicao_do_codigo, caminho_do_codigo)) in codigo_testado {
        if caminhos_dos_testes.insert(caminho_teste.clone()) {
            testes_pontuados.push((
                0,
                posicao_do_codigo,
                Ligacao {
                    rotulo: format!("testa {caminho_do_codigo}"),
                    local: caminho_teste,
                },
            ));
        }
    }
    testes_pontuados.sort_by(|primeiro, segundo| {
        segundo
            .0
            .cmp(&primeiro.0)
            .then(primeiro.1.cmp(&segundo.1))
            .then(primeiro.2.local.cmp(&segundo.2.local))
    });
    let testes: Vec<Ligacao> = testes_pontuados.into_iter().take(limite_de_arquivos).map(|(_, _, ligacao)| ligacao).collect();
    let codigo = codigo_mostrado
        .into_iter()
        .map(|achado| Ligacao {
            rotulo: achado.rotulo(),
            local: achado.caminho,
        })
        .collect();
    (codigo, testes)
}

// Riscos do registro docs/riscos.md cuja linha cita um termo do glossário da consulta, além dos que
// os testes já trouxeram.
fn riscos_do_registro_que_citam(raiz: &Path, nomes_dos_termos: &[String], riscos_dos_testes: &[Ligacao]) -> Vec<Ligacao> {
    if nomes_dos_termos.is_empty() {
        return Vec::new();
    }
    let caminho_registro = "docs/riscos.md";
    let texto_registro = fs::read_to_string(raiz.join(caminho_registro)).unwrap_or_default();
    let linhas_do_registro: Vec<&str> = texto_registro.lines().collect();
    let (riscos_do_registro, _) = ler_registro(&texto_registro);
    let termos_sem_acento: Vec<String> = nomes_dos_termos.iter().map(|nome| sem_acentos(nome)).collect();
    let mut riscos = Vec::new();
    for risco in riscos_do_registro {
        let ja_veio = riscos_dos_testes.iter().any(|ligacao| ligacao.rotulo.starts_with(&format!("{} ", risco.id)));
        let linha_registro = linhas_do_registro.get(risco.linha as usize - 1).copied().unwrap_or("");
        let linha_sem_acento = sem_acentos(linha_registro);
        let Some(termo_citado) = nomes_dos_termos
            .iter()
            .zip(&termos_sem_acento)
            .find(|(_, termo_sem_acento)| linha_sem_acento.contains(termo_sem_acento.as_str()))
            .map(|(nome, _)| nome)
        else {
            continue;
        };
        if ja_veio {
            continue;
        }
        let descricao = linha_registro.split('|').nth(2).unwrap_or("").trim();
        riscos.push(Ligacao {
            rotulo: format!("{} {descricao} (cita {termo_citado})", risco.id),
            local: format!("{caminho_registro}:{}", risco.linha),
        });
    }
    riscos
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
    let citacoes_minusculas: Vec<String> = citacoes_procuradas.iter().map(|citacao| sem_acentos(citacao)).collect();
    let mut adrs = Vec::new();
    for caminho in arquivos_do_repositorio
        .iter()
        .filter(|caminho| caminho.contains("adr/") && caminho.ends_with(".md"))
    {
        let Ok(texto_adr) = fs::read_to_string(raiz.join(caminho)) else { continue };
        let texto_minusculo = sem_acentos(&texto_adr);
        if citacoes_minusculas.iter().any(|citacao| texto_minusculo.contains(citacao)) {
            adrs.push(Ligacao {
                rotulo: primeira_linha_util(&texto_adr),
                local: caminho.clone(),
            });
        }
    }
    // A ADR que tem a consulta no título decide sobre ela; a que só a cita no corpo vem depois.
    let cita_no_titulo = |adr: &Ligacao| citacoes_minusculas.iter().any(|citacao| sem_acentos(&adr.rotulo).contains(citacao));
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
    // Os arquivos em foco vêm primeiro, na ordem em que a resposta os mostra: o risco deles abre a seção.
    let ordem_dos_testes = arquivos_em_foco
        .iter()
        .chain(arquivos_do_repositorio.iter().filter(|caminho| !arquivos_em_foco.contains(caminho)));
    for caminho in ordem_dos_testes.filter(|caminho| casa_algum(caminho, &padroes_de_teste)) {
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
            let todos_os_testes = testes_por_risco.remove(&id_risco).unwrap_or_default();
            let mut testes_do_risco: Vec<String> = todos_os_testes.iter().take(TESTES_MOSTRADOS_POR_RISCO).cloned().collect();
            if todos_os_testes.len() > TESTES_MOSTRADOS_POR_RISCO {
                testes_do_risco.push(format!("+{} testes", todos_os_testes.len() - TESTES_MOSTRADOS_POR_RISCO));
            }
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

// Código é o que se edita só com issue: docs/, prototipos/, .scratch/ e .claude/ ficam de fora.
fn eh_codigo(caminho_relativo: &str, ajustes: &AjustesDoProjeto) -> bool {
    !caminho_relativo.starts_with(".claude/") && !caminho_eh_livre_sem_issue(caminho_relativo, ajustes)
}

fn chamadas(raiz: &Path, simbolo: &str, definicoes: &[Ligacao]) -> Vec<Ligacao> {
    let linhas_de_definicao: BTreeSet<&str> = definicoes.iter().map(|definicao| definicao.local.as_str()).collect();
    let ajustes = ler_ajustes_do_projeto(raiz);
    let saida = git(raiz, &["grep", "-n", "-w", "--untracked", simbolo]).unwrap_or_default();
    let mut chamadas_encontradas = Vec::new();
    for linha in saida.lines() {
        let mut partes = linha.splitn(3, ':');
        let (Some(caminho), Some(numero_linha), Some(trecho)) = (partes.next(), partes.next(), partes.next()) else {
            continue;
        };
        let local = format!("{caminho}:{numero_linha}");
        if eh_documento(caminho) || !eh_codigo(caminho, &ajustes) || linhas_de_definicao.contains(local.as_str()) || !trecho.contains(&format!("{simbolo}(")) {
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
    let tem_ligacao = linhas.len() > 1 || contexto_montado.secoes.iter().any(|secao| !secao.ligacoes.is_empty());
    for secao in contexto_montado.secoes.iter().filter(|secao| !secao.ligacoes.is_empty()) {
        linhas.push(format!("  {}:", secao.titulo));
        for ligacao in &secao.ligacoes {
            linhas.push(format!("    {} · {}", ligacao.local, ligacao.rotulo));
        }
    }
    if !tem_ligacao {
        linhas.push("  nada ligado a esta consulta no repositório.".into());
    }
    if let Some(mapa_do_codigo) = &contexto_montado.mapa_do_codigo {
        linhas.push(format!(
            "  mapa do código: {mapa_do_codigo} · onde mora cada camada de uma fatia e o vizinho para imitar"
        ));
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
