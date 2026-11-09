// O que um comando do Bash grava (ADR 0027 do método). O guarda-bash decide pelo alvo da escrita, não
// por um caminho citado em qualquer lugar do comando: `rodar ci > "$TMPDIR/log"` lê os watchdogs e grava
// no $TMPDIR, e um heredoc que só cita `.claude/agents` no texto não grava lá. O vigia-codigo e o
// vigia-teste usam os mesmos alvos para conferir o código editado por `sed -i`, heredoc ou `python`.
//
// É uma leitura de shell de propósito pequena: aspas, escapes, redirecionamentos, heredocs, `$(...)`,
// `sh -c` e os verbos que gravam. O que não dá para saber (o alvo do `xargs rm`, o caminho montado
// dentro de um script) vai em `alvo_desconhecido` e em `scripts_que_gravam`, para quem decide ser
// conservador com eles.

use regex::Regex;
use std::path::{Path, PathBuf};

pub struct Escrita {
    pub alvo: String,
    // As pastas dos `cd` que vieram antes, na ordem: o alvo relativo vale a partir delas.
    pub pastas_do_cd: Vec<String>,
}

#[derive(Default)]
pub struct Analise {
    pub escritas: Vec<Escrita>,
    // Scripts de python, node, ruby ou perl que chamam uma função que grava arquivo.
    pub scripts_que_gravam: Vec<String>,
    // Algum comando grava num alvo que só aparece na execução (`xargs rm`, `find -exec rm`).
    pub alvo_desconhecido: bool,
    // O comando sem o corpo dos heredocs: o texto que o shell lê como comando.
    pub texto_sem_heredocs: String,
}

const INTERPRETADORES: [&str; 9] = ["python", "python3", "node", "ruby", "perl", "bun", "deno", "php", "osascript"];
const DESTINOS_INOFENSIVOS: [&str; 4] = ["/dev/null", "/dev/stdout", "/dev/stderr", "/dev/tty"];

fn regex(padrao: &str) -> Regex {
    Regex::new(padrao).expect("regex válida")
}

pub fn analisar(comando: &str) -> Analise {
    let mut analise = Analise::default();
    let (texto_sem_heredocs, corpos_dos_heredocs) = separar_heredocs(comando);
    let mut pastas_do_cd = Vec::new();
    analisar_texto(&texto_sem_heredocs, &corpos_dos_heredocs, &mut pastas_do_cd, &mut analise, 0);
    analise.texto_sem_heredocs = texto_sem_heredocs;
    analise
}

// Os alvos resolvidos contra a pasta de trabalho: `$CLAUDE_PROJECT_DIR`, `$PWD` e `~` expandidos.
pub fn caminhos_gravados(comando: &str, pasta_de_trabalho: &Path, diretorio_projeto: &Path) -> Vec<PathBuf> {
    let pasta_pessoal = std::env::var("HOME").unwrap_or_default();
    let mut caminhos: Vec<PathBuf> = Vec::new();
    for escrita in analisar(comando).escritas {
        let mut pasta = pasta_de_trabalho.to_path_buf();
        for pasta_do_cd in &escrita.pastas_do_cd {
            pasta = pasta.join(expandir(pasta_do_cd, diretorio_projeto, &pasta, &pasta_pessoal));
        }
        let caminho = pasta.join(expandir(&escrita.alvo, diretorio_projeto, &pasta, &pasta_pessoal));
        if !caminhos.contains(&caminho) {
            caminhos.push(caminho);
        }
    }
    caminhos
}

// Os arquivos do projeto que a ferramenta do hook acabou de gravar, relativos à raiz: o `file_path` do
// Write e do Edit, ou os alvos de um Bash que ainda existem como arquivo (o `rm` não deixa o que conferir).
pub fn arquivos_gravados_pela_ferramenta(entrada_hook: &serde_json::Value, diretorio_projeto: &Path) -> Vec<String> {
    let caminho_da_edicao = entrada_hook["tool_input"]["file_path"].as_str().unwrap_or("");
    if !caminho_da_edicao.is_empty() {
        return crate::comum::caminho_relativo_ao_projeto(diretorio_projeto, caminho_da_edicao)
            .into_iter()
            .collect();
    }
    let Some(comando) = entrada_hook["tool_input"]["command"].as_str() else {
        return Vec::new();
    };
    let pasta_de_trabalho = match entrada_hook["cwd"].as_str() {
        Some(pasta) if !pasta.is_empty() => PathBuf::from(pasta),
        _ => diretorio_projeto.to_path_buf(),
    };
    let mut arquivos = Vec::new();
    for caminho in caminhos_gravados(comando, &pasta_de_trabalho, diretorio_projeto) {
        let Some(relativo) = crate::comum::caminho_relativo_ao_projeto(diretorio_projeto, &caminho.to_string_lossy()) else {
            continue;
        };
        if !relativo.is_empty() && diretorio_projeto.join(&relativo).is_file() && !arquivos.contains(&relativo) {
            arquivos.push(relativo);
        }
    }
    arquivos
}

fn expandir(caminho: &str, diretorio_projeto: &Path, pasta_atual: &Path, pasta_pessoal: &str) -> String {
    let projeto = diretorio_projeto.to_string_lossy();
    let atual = pasta_atual.to_string_lossy();
    let expandido = caminho
        .replace("${CLAUDE_PROJECT_DIR}", &projeto)
        .replace("$CLAUDE_PROJECT_DIR", &projeto)
        .replace("${PWD}", &atual)
        .replace("$PWD", &atual)
        .replace("${HOME}", pasta_pessoal)
        .replace("$HOME", pasta_pessoal);
    match expandido.strip_prefix('~') {
        Some(resto) if resto.is_empty() || resto.starts_with('/') => format!("{pasta_pessoal}{resto}"),
        _ => expandido,
    }
}

// Tira o corpo de cada heredoc do texto e o devolve à parte, na ordem em que os `<<` aparecem.
fn separar_heredocs(comando: &str) -> (String, Vec<String>) {
    let abertura = regex(r#"<<(-?)\s*(?:'([^']+)'|"([^"]+)"|\\?([A-Za-z_][\w.-]*))"#);
    let mut texto_sem_heredocs = String::new();
    let mut corpos: Vec<String> = Vec::new();
    // Heredocs abertos na linha de comando, na ordem: o delimitador e se o `<<-` tira os tabs.
    let mut pendentes: std::collections::VecDeque<(String, bool, usize)> = std::collections::VecDeque::new();
    for linha in comando.split('\n') {
        if let Some((delimitador, tira_tabs, indice_do_corpo)) = pendentes.front().cloned() {
            let comparada = if tira_tabs { linha.trim_start_matches('\t') } else { linha };
            if comparada.trim_end_matches('\r') == delimitador {
                pendentes.pop_front();
            } else {
                corpos[indice_do_corpo].push_str(linha);
                corpos[indice_do_corpo].push('\n');
            }
            continue;
        }
        for partes in abertura.captures_iter(linha) {
            let inicio = partes.get(0).map(|parte| parte.start()).unwrap_or(0);
            let fim = partes.get(0).map(|parte| parte.end()).unwrap_or(0);
            // `<<<` é here-string: não abre corpo.
            if linha[..inicio].ends_with('<') || linha[inicio + 2..].starts_with('<') {
                continue;
            }
            let _ = fim;
            let delimitador = partes
                .get(2)
                .or_else(|| partes.get(3))
                .or_else(|| partes.get(4))
                .map(|parte| parte.as_str().to_string())
                .unwrap_or_default();
            corpos.push(String::new());
            pendentes.push_back((delimitador, &partes[1] == "-", corpos.len() - 1));
        }
        texto_sem_heredocs.push_str(linha);
        texto_sem_heredocs.push('\n');
    }
    (texto_sem_heredocs.trim_end_matches('\n').to_string(), corpos)
}

#[derive(Debug, Clone, PartialEq)]
enum Pedaco {
    Palavra(String),
    Operador(String),
    // Fim de um comando simples: `;`, `&&`, `||`, `|`, `&`, quebra de linha, parênteses.
    Separador(String),
}

struct Leitura {
    pedacos: Vec<Pedaco>,
    // O texto de dentro de `$(...)` e de crases: roda como comando também.
    subcomandos: Vec<String>,
}

fn ler_pedacos(texto: &str) -> Leitura {
    let letras: Vec<char> = texto.chars().collect();
    let mut leitura = Leitura {
        pedacos: Vec::new(),
        subcomandos: Vec::new(),
    };
    let mut palavra = String::new();
    let mut palavra_comecou = false;
    let mut posicao = 0;
    let fechar_palavra = |palavra: &mut String, palavra_comecou: &mut bool, pedacos: &mut Vec<Pedaco>| {
        if *palavra_comecou {
            pedacos.push(Pedaco::Palavra(std::mem::take(palavra)));
            *palavra_comecou = false;
        }
    };
    while posicao < letras.len() {
        let letra = letras[posicao];
        match letra {
            '\\' => {
                if let Some(proxima) = letras.get(posicao + 1) {
                    if *proxima != '\n' {
                        palavra.push(*proxima);
                        palavra_comecou = true;
                    }
                }
                posicao += 2;
            }
            '\'' => {
                palavra_comecou = true;
                posicao += 1;
                while posicao < letras.len() && letras[posicao] != '\'' {
                    palavra.push(letras[posicao]);
                    posicao += 1;
                }
                posicao += 1;
            }
            '"' => {
                palavra_comecou = true;
                posicao += 1;
                while posicao < letras.len() && letras[posicao] != '"' {
                    if letras[posicao] == '\\' && posicao + 1 < letras.len() {
                        palavra.push(letras[posicao + 1]);
                        posicao += 2;
                        continue;
                    }
                    if letras[posicao] == '$' && letras.get(posicao + 1) == Some(&'(') {
                        posicao = ler_subcomando(&letras, posicao, &mut palavra, &mut leitura.subcomandos);
                        continue;
                    }
                    palavra.push(letras[posicao]);
                    posicao += 1;
                }
                posicao += 1;
            }
            '$' if letras.get(posicao + 1) == Some(&'(') => {
                palavra_comecou = true;
                posicao = ler_subcomando(&letras, posicao, &mut palavra, &mut leitura.subcomandos);
            }
            '`' => {
                palavra_comecou = true;
                let mut interno = String::new();
                posicao += 1;
                while posicao < letras.len() && letras[posicao] != '`' {
                    interno.push(letras[posicao]);
                    posicao += 1;
                }
                posicao += 1;
                palavra.push_str(&interno);
                leitura.subcomandos.push(interno);
            }
            ' ' | '\t' | '\r' => {
                fechar_palavra(&mut palavra, &mut palavra_comecou, &mut leitura.pedacos);
                posicao += 1;
            }
            '\n' | ';' | '(' | ')' | '{' | '}' if !(matches!(letra, '{' | '}') && palavra_comecou) => {
                fechar_palavra(&mut palavra, &mut palavra_comecou, &mut leitura.pedacos);
                leitura.pedacos.push(Pedaco::Separador(letra.to_string()));
                posicao += 1;
            }
            '|' | '&' if !(letra == '&' && letras.get(posicao + 1) == Some(&'>')) => {
                fechar_palavra(&mut palavra, &mut palavra_comecou, &mut leitura.pedacos);
                let dobrado = letras.get(posicao + 1) == Some(&letra);
                leitura
                    .pedacos
                    .push(Pedaco::Separador(if dobrado { format!("{letra}{letra}") } else { letra.to_string() }));
                posicao += if dobrado { 2 } else { 1 };
            }
            '>' | '<' | '&' => {
                // `2>` e `&>`: o número ou o & antes do sinal fazem parte do operador.
                let mut operador = String::new();
                if palavra_comecou && (palavra == "&" || (!palavra.is_empty() && palavra.chars().all(|algarismo| algarismo.is_ascii_digit()))) {
                    operador = std::mem::take(&mut palavra);
                    palavra_comecou = false;
                } else {
                    fechar_palavra(&mut palavra, &mut palavra_comecou, &mut leitura.pedacos);
                }
                if letra == '&' {
                    operador.push('&');
                    posicao += 1;
                }
                let sinal = letras[posicao];
                operador.push(sinal);
                posicao += 1;
                while posicao < letras.len() && (letras[posicao] == sinal || letras[posicao] == '|' && sinal == '>' || letras[posicao] == '-' && sinal == '<') {
                    operador.push(letras[posicao]);
                    posicao += 1;
                }
                // `>&2`, `2>&1`, `<&-`: duplicação de descritor, sem arquivo.
                if letras.get(posicao) == Some(&'&') {
                    operador.push('&');
                    posicao += 1;
                    while posicao < letras.len() && (letras[posicao].is_ascii_digit() || letras[posicao] == '-') {
                        posicao += 1;
                    }
                }
                leitura.pedacos.push(Pedaco::Operador(operador));
            }
            _ => {
                palavra.push(letra);
                palavra_comecou = true;
                posicao += 1;
            }
        }
    }
    fechar_palavra(&mut palavra, &mut palavra_comecou, &mut leitura.pedacos);
    leitura
}

// Lê `$(...)` a partir do `$`, com parênteses aninhados; devolve a posição depois do `)`.
fn ler_subcomando(letras: &[char], inicio: usize, palavra: &mut String, subcomandos: &mut Vec<String>) -> usize {
    let mut posicao = inicio + 2;
    let mut profundidade = 1;
    let mut interno = String::new();
    while posicao < letras.len() {
        match letras[posicao] {
            '(' => profundidade += 1,
            ')' => {
                profundidade -= 1;
                if profundidade == 0 {
                    break;
                }
            }
            _ => {}
        }
        interno.push(letras[posicao]);
        posicao += 1;
    }
    palavra.push_str(&interno);
    subcomandos.push(interno);
    posicao + 1
}

struct ComandoSimples {
    palavras: Vec<String>,
    redirecoes: Vec<(String, String)>,
    corpos: Vec<String>,
}

fn analisar_texto(texto: &str, corpos_dos_heredocs: &[String], pastas_do_cd: &mut Vec<String>, analise: &mut Analise, profundidade: usize) {
    // `sh -c "sh -c ..."` sem fim não existe; o limite só evita laço.
    if profundidade > 8 {
        analise.alvo_desconhecido = true;
        return;
    }
    let leitura = ler_pedacos(texto);
    let mut proximo_corpo = 0;
    let mut atual = ComandoSimples {
        palavras: Vec::new(),
        redirecoes: Vec::new(),
        corpos: Vec::new(),
    };
    let mut pedacos = leitura.pedacos.into_iter().peekable();
    let mut comandos: Vec<ComandoSimples> = Vec::new();
    while let Some(pedaco) = pedacos.next() {
        match pedaco {
            Pedaco::Palavra(palavra) => atual.palavras.push(palavra),
            Pedaco::Operador(operador) => {
                if operador.ends_with('&') {
                    continue;
                }
                let alvo = match pedacos.peek() {
                    Some(Pedaco::Palavra(_)) => match pedacos.next() {
                        Some(Pedaco::Palavra(alvo)) => alvo,
                        _ => String::new(),
                    },
                    _ => String::new(),
                };
                if operador.starts_with("<<") && !operador.starts_with("<<<") {
                    if let Some(corpo) = corpos_dos_heredocs.get(proximo_corpo) {
                        atual.corpos.push(corpo.clone());
                    }
                    proximo_corpo += 1;
                } else {
                    atual.redirecoes.push((operador, alvo));
                }
            }
            Pedaco::Separador(_) => {
                comandos.push(std::mem::replace(
                    &mut atual,
                    ComandoSimples {
                        palavras: Vec::new(),
                        redirecoes: Vec::new(),
                        corpos: Vec::new(),
                    },
                ));
            }
        }
    }
    comandos.push(atual);
    for comando in comandos {
        analisar_comando(comando, pastas_do_cd, analise, profundidade);
    }
    for subcomando in leitura.subcomandos {
        let mut pastas_do_subcomando = pastas_do_cd.clone();
        analisar_texto(&subcomando, &[], &mut pastas_do_subcomando, analise, profundidade + 1);
    }
}

fn eh_opcao(palavra: &str) -> bool {
    palavra.len() > 1 && palavra.starts_with('-')
}

fn nao_opcoes(argumentos: &[String]) -> Vec<String> {
    let mut depois_do_fim_das_opcoes = false;
    argumentos
        .iter()
        .filter(|argumento| {
            if depois_do_fim_das_opcoes {
                return true;
            }
            if argumento.as_str() == "--" {
                depois_do_fim_das_opcoes = true;
                return false;
            }
            !eh_opcao(argumento)
        })
        .cloned()
        .collect()
}

fn analisar_comando(comando: ComandoSimples, pastas_do_cd: &mut Vec<String>, analise: &mut Analise, profundidade: usize) {
    let gravar = |alvo: &str, analise: &mut Analise, pastas_do_cd: &[String]| {
        if !alvo.is_empty() && !DESTINOS_INOFENSIVOS.contains(&alvo) {
            analise.escritas.push(Escrita {
                alvo: alvo.to_string(),
                pastas_do_cd: pastas_do_cd.to_vec(),
            });
        }
    };
    for (operador, alvo) in &comando.redirecoes {
        if operador.contains('>') {
            gravar(alvo, analise, pastas_do_cd);
        }
    }
    // Atribuições e prefixos que só mudam o ambiente: o verbo vem depois deles.
    let prefixos = ["sudo", "command", "env", "nohup", "time", "exec", "nice", "builtin"];
    let mut palavras: &[String] = &comando.palavras;
    while let Some(primeira) = palavras.first() {
        let eh_atribuicao = regex(r"^[A-Za-z_]\w*=").is_match(primeira);
        if eh_atribuicao || prefixos.contains(&primeira.as_str()) || (palavras.len() > 1 && eh_opcao(primeira) && comando.palavras[0] == "env") {
            palavras = &palavras[1..];
        } else {
            break;
        }
    }
    let Some(primeira) = palavras.first() else { return };
    let verbo = primeira.rsplit('/').next().unwrap_or(primeira).to_string();
    let argumentos = &palavras[1..];
    let posicionais = nao_opcoes(argumentos);
    let tem_opcao = |teste: &dyn Fn(&str) -> bool| argumentos.iter().any(|argumento| eh_opcao(argumento) && teste(argumento));
    match verbo.as_str() {
        "cd" => {
            if let Some(pasta) = posicionais.first() {
                pastas_do_cd.push(pasta.clone());
            }
        }
        "rm" | "rmdir" | "unlink" | "shred" | "touch" | "mv" | "tee" | "patch" => {
            for alvo in &posicionais {
                gravar(alvo, analise, pastas_do_cd);
            }
        }
        "truncate" => {
            let mut proximo_eh_tamanho = false;
            for argumento in argumentos {
                if proximo_eh_tamanho {
                    proximo_eh_tamanho = false;
                } else if argumento == "-s" || argumento == "-r" {
                    proximo_eh_tamanho = true;
                } else if !eh_opcao(argumento) {
                    gravar(argumento, analise, pastas_do_cd);
                }
            }
        }
        "chmod" | "chown" | "chgrp" => {
            for alvo in posicionais.iter().skip(1) {
                gravar(alvo, analise, pastas_do_cd);
            }
        }
        "cp" | "ln" | "install" | "rsync" | "ditto" => {
            let mut destino_por_opcao = None;
            for (indice, argumento) in argumentos.iter().enumerate() {
                if argumento == "-t" {
                    destino_por_opcao = argumentos.get(indice + 1).cloned();
                } else if let Some(pasta) = argumento.strip_prefix("--target-directory=") {
                    destino_por_opcao = Some(pasta.to_string());
                }
            }
            if let Some(destino) = destino_por_opcao.or_else(|| posicionais.last().cloned().filter(|_| posicionais.len() > 1)) {
                gravar(&destino, analise, pastas_do_cd);
            }
        }
        "dd" => {
            for argumento in argumentos {
                if let Some(destino) = argumento.strip_prefix("of=") {
                    gravar(destino, analise, pastas_do_cd);
                }
            }
        }
        "sed" | "gsed" => {
            let no_lugar = tem_opcao(&|opcao| opcao.starts_with("--in-place") || (!opcao.starts_with("--") && opcao.contains('i')));
            if no_lugar {
                let script_por_opcao = tem_opcao(&|opcao| matches!(opcao, "-e" | "-f" | "--expression" | "--file") || opcao.starts_with("--expression="));
                let mut proximo_eh_script = false;
                let mut arquivos = Vec::new();
                for (indice, argumento) in argumentos.iter().enumerate() {
                    // O sed do macOS pede o sufixo do backup logo depois do -i: `-i ''` ou `-i .bak`.
                    let eh_sufixo_do_backup =
                        indice > 0 && argumentos[indice - 1] == "-i" && (argumento.is_empty() || (argumento.starts_with('.') && !argumento.contains('/')));
                    if proximo_eh_script {
                        proximo_eh_script = false;
                    } else if eh_sufixo_do_backup {
                    } else if matches!(argumento.as_str(), "-e" | "-f" | "--expression" | "--file") {
                        proximo_eh_script = true;
                    } else if !eh_opcao(argumento) {
                        arquivos.push(argumento.clone());
                    }
                }
                let pular = usize::from(!script_por_opcao);
                for alvo in arquivos.iter().skip(pular) {
                    gravar(alvo, analise, pastas_do_cd);
                }
            }
        }
        "git" => {
            let mut resto = argumentos;
            while let Some(opcao) = resto.first().filter(|palavra| eh_opcao(palavra)) {
                resto = if opcao == "-C" || opcao == "-c" {
                    resto.get(2..).unwrap_or(&[])
                } else {
                    &resto[1..]
                };
            }
            let Some(subcomando) = resto.first() else { return };
            if matches!(subcomando.as_str(), "checkout" | "restore" | "rm" | "mv" | "stash" | "clean") {
                for alvo in nao_opcoes(&resto[1..]) {
                    gravar(&alvo, analise, pastas_do_cd);
                }
            }
        }
        "find" => {
            let apaga = argumentos.iter().any(|argumento| argumento == "-delete");
            let executa_quem_grava = argumentos
                .windows(2)
                .any(|par| matches!(par[0].as_str(), "-exec" | "-execdir" | "-ok") && verbo_grava(&par[1]));
            if apaga || executa_quem_grava {
                for raiz in argumentos
                    .iter()
                    .take_while(|argumento| !argumento.starts_with('-') && !matches!(argumento.as_str(), "(" | "!"))
                {
                    gravar(raiz, analise, pastas_do_cd);
                }
            }
        }
        "xargs" | "parallel" => {
            if argumentos.iter().any(|argumento| verbo_grava(argumento)) {
                analise.alvo_desconhecido = true;
            }
        }
        "sh" | "bash" | "zsh" | "dash" | "ksh"
            if posicionais
                .first()
                .is_some_and(|script| script.ends_with("watchdogs/rodar") && !script.contains(char::is_whitespace)) =>
        {
            if posicionais.get(1).map(String::as_str) == Some("pesado") {
                for script in posicionais.iter().skip(2) {
                    analisar_texto(script, &[], &mut pastas_do_cd.clone(), analise, profundidade + 1);
                }
            }
        }
        "sh" | "bash" | "zsh" | "dash" | "ksh" => {
            if let Some(indice) = argumentos
                .iter()
                .position(|argumento| argumento == "-c" || (eh_opcao(argumento) && argumento.ends_with('c') && !argumento.starts_with("--")))
            {
                if let Some(script) = argumentos.get(indice + 1) {
                    analisar_texto(&script.clone(), &[], &mut pastas_do_cd.clone(), analise, profundidade + 1);
                }
            }
            for corpo in &comando.corpos {
                let (texto, corpos_internos) = separar_heredocs(corpo);
                analisar_texto(&texto, &corpos_internos, &mut pastas_do_cd.clone(), analise, profundidade + 1);
            }
        }
        "eval" => {
            analisar_texto(&argumentos.join(" "), &[], &mut pastas_do_cd.clone(), analise, profundidade + 1);
        }
        "rodar" if argumentos.first().map(String::as_str) == Some("pesado") => {
            for script in argumentos.iter().skip(1) {
                analisar_texto(script, &[], &mut pastas_do_cd.clone(), analise, profundidade + 1);
            }
        }
        _ if eh_interpretador(&verbo) => {
            let edita_no_lugar = verbo == "perl" && tem_opcao(&|opcao| !opcao.starts_with("--") && opcao.contains('i'));
            let mut scripts: Vec<String> = comando.corpos.clone();
            let mut proximo_eh_script = false;
            let mut arquivos_do_perl = Vec::new();
            for argumento in argumentos {
                if proximo_eh_script {
                    scripts.push(argumento.clone());
                    proximo_eh_script = false;
                } else if matches!(argumento.as_str(), "-c" | "-e" | "-E" | "--eval") || (eh_opcao(argumento) && argumento.ends_with('e') && verbo == "perl") {
                    proximo_eh_script = true;
                } else if !eh_opcao(argumento) {
                    arquivos_do_perl.push(argumento.clone());
                }
            }
            if edita_no_lugar {
                for alvo in &arquivos_do_perl {
                    gravar(alvo, analise, pastas_do_cd);
                }
            }
            for script in scripts {
                if let Some(alvos) = alvos_do_script(&script) {
                    for alvo in alvos {
                        gravar(&alvo, analise, pastas_do_cd);
                    }
                    analise.scripts_que_gravam.push(script);
                }
            }
        }
        _ => {}
    }
}

fn eh_interpretador(verbo: &str) -> bool {
    INTERPRETADORES.contains(&verbo) || regex(r"^python3?(\.\d+)?$").is_match(verbo)
}

fn verbo_grava(palavra: &str) -> bool {
    let verbo = palavra.rsplit('/').next().unwrap_or(palavra);
    matches!(
        verbo,
        "rm" | "rmdir" | "unlink" | "shred" | "touch" | "mv" | "cp" | "ln" | "tee" | "truncate" | "chmod" | "chown" | "sed" | "perl" | "install" | "dd"
    )
}

// Um script que chama função que grava devolve os textos entre aspas das linhas dessas chamadas: é ali
// que fica o caminho escrito à mão. None: o script não grava arquivo.
fn alvos_do_script(script: &str) -> Option<Vec<String>> {
    let chamada_que_grava = regex(
        r#"open\([^)]*,\s*(?:mode\s*=\s*)?['"][^'"]*[wax+]|\.write_text\(|\.write_bytes\(|\.unlink\(|\.rename\(|\.touch\(|\.mkdir\(|\bshutil\.|\bos\.(?:remove|unlink|rename|replace|rmdir|removedirs|chmod|truncate|makedirs)\(|writeFile|appendFile|rmSync|unlinkSync|renameSync|copyFile|\bfs\.(?:rm|cp|writeFile|appendFile|rename|unlink|truncate)|File\.(?:write|open\([^)]*['"][wa])|FileUtils\.|Bun\.write\(|Deno\.(?:writeTextFile|remove|rename)\("#,
    );
    let texto_entre_aspas = regex(r#"'([^'\n]+)'|"([^"\n]+)"|`([^`\n]+)`"#);
    let mut alvos = Vec::new();
    let mut grava = false;
    for linha in script.lines().filter(|linha| chamada_que_grava.is_match(linha)) {
        grava = true;
        for partes in texto_entre_aspas.captures_iter(linha) {
            let citado = partes
                .get(1)
                .or_else(|| partes.get(2))
                .or_else(|| partes.get(3))
                .map(|parte| parte.as_str())
                .unwrap_or("");
            if citado.contains('/') || citado.contains('.') {
                alvos.push(citado.to_string());
            }
        }
    }
    grava.then_some(alvos)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn alvos(comando: &str) -> Vec<String> {
        analisar(comando).escritas.into_iter().map(|escrita| escrita.alvo).collect()
    }

    #[test]
    fn redirecionamento_grava_no_destino_e_nao_no_que_le() {
        assert_eq!(
            alvos(r#"sh .claude/watchdogs/rodar ci --base origin/dev > "$TMPDIR/ci.log" 2>&1"#),
            vec!["$TMPDIR/ci.log"]
        );
        assert_eq!(alvos("cat .claude/settings.json > /dev/null"), Vec::<String>::new());
        assert_eq!(alvos("echo oi >> a.txt 2>/dev/null >&2"), vec!["a.txt"]);
        assert_eq!(alvos("echo oi &> b.txt"), vec!["b.txt"]);
    }

    #[test]
    fn heredoc_so_cita_o_texto_e_grava_no_destino_do_cat() {
        let comando = "cat > .scratch/brief.md <<'EOF'\nleia .claude/agents/implementador.md\nrm -rf .claude/watchdogs\nEOF\necho fim";
        assert_eq!(alvos(comando), vec![".scratch/brief.md"]);
        assert!(!analisar(comando).texto_sem_heredocs.contains(".claude/agents"));
    }

    #[test]
    fn verbos_que_gravam_devolvem_o_alvo() {
        assert_eq!(alvos("cp a.txt b.txt c/"), vec!["c/"]);
        assert_eq!(alvos("mv velho.txt novo.txt"), vec!["velho.txt", "novo.txt"]);
        assert_eq!(alvos("sed -i '' 's/a/b/' src/x.ts src/y.ts"), vec!["src/x.ts", "src/y.ts"]);
        assert_eq!(alvos("sed -i -e s/a/b/ src/x.ts"), vec!["src/x.ts"]);
        assert_eq!(alvos("sed -n 1,20p src/x.ts"), Vec::<String>::new());
        assert_eq!(alvos("echo x | tee -a log.txt"), vec!["log.txt"]);
        assert_eq!(alvos("cd pacote && rm -f dist/a.js"), vec!["dist/a.js"]);
        assert_eq!(analisar("cd pacote && rm -f dist/a.js").escritas[0].pastas_do_cd, vec!["pacote"]);
        assert_eq!(alvos("git checkout -- src/a.ts"), vec!["src/a.ts"]);
        assert_eq!(alvos("git -C ../outra log --oneline"), Vec::<String>::new());
        assert_eq!(alvos("find .claude/watchdogs -name '*.rs' -delete"), vec![".claude/watchdogs"]);
        assert_eq!(alvos("chmod +x scripts/a.sh"), vec!["scripts/a.sh"]);
    }

    #[test]
    fn comando_dentro_de_comando_tambem_conta() {
        assert_eq!(alvos(r#"bash -c "rm .claude/watchdogs/rodar""#), vec![".claude/watchdogs/rodar"]);
        assert_eq!(alvos("echo $(rm x.txt)"), vec!["x.txt"]);
        assert_eq!(alvos(r#"sh .claude/watchdogs/rodar pesado "npm test > saida.txt""#), vec!["saida.txt"]);
        assert!(analisar("ls | xargs rm").alvo_desconhecido);
    }

    #[test]
    fn script_que_grava_devolve_o_caminho_citado_na_chamada() {
        let comando = "python3 - <<'EOF'\nimport json\nestado = json.load(open('.claude/orquestracao/fila.json'))\nopen('.claude/orquestracao/fila.json', 'w').write(json.dumps(estado))\nEOF";
        let analise = analisar(comando);
        assert_eq!(
            analise.escritas.iter().map(|escrita| escrita.alvo.as_str()).collect::<Vec<_>>(),
            vec![".claude/orquestracao/fila.json"]
        );
        assert_eq!(analise.scripts_que_gravam.len(), 1);
        let so_le = analisar("python3 -c \"print(open('src/a.py').read())\"");
        assert!(so_le.escritas.is_empty() && so_le.scripts_que_gravam.is_empty());
        assert_eq!(alvos("node -e \"require('fs').writeFileSync('src/a.ts', x)\""), vec!["src/a.ts"]);
    }
}
