//! Gera a folha de respostas .html de uma rodada de perguntas a partir de um JSON.
//! Uso: sh ~/.claude/skills/folha-de-respostas/scripts/rodar <perguntas.json> <saida.html>
//! O formato do JSON está no SKILL.md da skill folha-de-respostas.
mod js;

use js::{campo, chave_do_conjunto, json, ou, texto_de, verdadeiro};
use serde_json::Value;
use std::collections::HashSet;
use std::process::exit;

const MOLDE_DA_FOLHA: &str = include_str!("molde-da-folha.html");

fn parar(mensagem: &str) -> ! {
    eprintln!("{mensagem}");
    exit(1);
}

fn escapar_html(texto: &str) -> String {
    let mut saida = String::with_capacity(texto.len());
    for caractere in texto.chars() {
        match caractere {
            '&' => saida.push_str("&amp;"),
            '<' => saida.push_str("&lt;"),
            '>' => saida.push_str("&gt;"),
            '"' => saida.push_str("&quot;"),
            outro => saida.push(outro),
        }
    }
    saida
}

/// O id começa com letra e tem só letras, números, `_` ou `-` (ASCII).
fn id_valido(id: &str) -> bool {
    let mut caracteres = id.chars();
    caracteres.next().is_some_and(|c| c.is_ascii_alphabetic())
        && caracteres.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Os problemas que impedem a folha; vazio quando o JSON está bom.
fn problemas_da_rodada(rodada: &Value) -> Vec<String> {
    if rodada.is_null() {
        parar("A folha não foi gerada: o JSON precisa ser um objeto, não null.");
    }
    let mut problemas = Vec::new();
    for campo_obrigatorio in ["titulo", "chave", "cabecalhoDaCopia", "perguntas"] {
        if !verdadeiro(campo(rodada, campo_obrigatorio)) {
            problemas.push(format!("falta o campo \"{campo_obrigatorio}\""));
        }
    }
    let perguntas = match campo(rodada, "perguntas") {
        Some(Value::Array(perguntas)) => perguntas.as_slice(),
        outro if !verdadeiro(outro) => &[],
        _ => parar("A folha não foi gerada: \"perguntas\" precisa ser uma lista."),
    };
    let mut ids_vistos = HashSet::new();
    for (posicao, pergunta) in perguntas.iter().enumerate() {
        if pergunta.is_null() {
            parar(&format!(
                "A folha não foi gerada: a pergunta na posição {posicao} é null."
            ));
        }
        if verdadeiro(campo(pergunta, "bloco")) {
            continue;
        }
        let id = campo(pergunta, "id");
        let onde = if verdadeiro(id) {
            texto_de(id)
        } else {
            format!("pergunta na posição {posicao}")
        };
        if !verdadeiro(id) {
            problemas.push(format!("{onde}: falta \"id\""));
        }
        if !ids_vistos.insert(chave_do_conjunto(id, posicao)) {
            problemas.push(format!("{onde}: id repetido"));
        }
        let id_como_texto = if verdadeiro(id) {
            texto_de(id)
        } else {
            String::new()
        };
        if !id_valido(&id_como_texto) {
            problemas.push(format!(
                "{onde}: o id precisa começar com letra e ter só letras, números, _ ou -"
            ));
        }
        if !verdadeiro(campo(pergunta, "titulo")) {
            problemas.push(format!("{onde}: falta \"titulo\""));
        }
        let opcoes = campo(pergunta, "opcoes");
        if !matches!(opcoes, Some(Value::Array(lista)) if (2..=5).contains(&lista.len())) {
            problemas.push(format!("{onde}: \"opcoes\" precisa ter de 2 a 5 itens"));
        }
        if let Some(rec) = campo(pergunta, "rec") {
            let zero = Value::from(0);
            let vazia = Value::Array(Vec::new());
            let quantidade_de_opcoes = js::tamanho(ou(opcoes, Some(&vazia)));
            if js::menor(Some(rec), Some(&zero))
                || js::maior_ou_igual(Some(rec), quantidade_de_opcoes.as_ref())
            {
                problemas.push(format!(
                    "{onde}: \"rec\" aponta para uma opção que não existe"
                ));
            }
        }
    }
    problemas
}

fn gerar_html(rodada: &Value) -> (String, usize) {
    let perguntas = campo(rodada, "perguntas").cloned().unwrap_or(Value::Null);
    let mut dados = serde_json::Map::new();
    dados.insert("perguntas".into(), perguntas.clone());
    for nome in ["chave", "cabecalhoDaCopia"] {
        if let Some(valor) = campo(rodada, nome) {
            dados.insert(nome.into(), valor.clone());
        }
    }
    // O JSON vai dentro de <script>: "</" não pode fechar a tag antes da hora.
    let dados_no_script = json(&Value::Object(dados)).replace("</", "<\\/");
    let paragrafos_da_intro = match campo(rodada, "intro") {
        Some(Value::Array(paragrafos)) => paragrafos
            .iter()
            .map(|paragrafo| {
                format!(
                    "<p class=\"intro\">{}</p>",
                    escapar_html(&texto_de(Some(paragrafo)))
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        outro if !verdadeiro(outro) => String::new(),
        _ => parar("A folha não foi gerada: \"intro\" precisa ser uma lista."),
    };
    let total_de_perguntas = perguntas
        .as_array()
        .map(|lista| {
            lista
                .iter()
                .filter(|pergunta| !verdadeiro(campo(pergunta, "bloco")))
                .count()
        })
        .unwrap_or(0);
    let titulo = campo(rodada, "titulo");
    let trocas = [
        (
            "{{TITULO_DA_ABA}}",
            escapar_html(&texto_de(ou(campo(rodada, "tituloDaAba"), titulo))),
        ),
        ("{{TITULO}}", escapar_html(&texto_de(titulo))),
        ("{{INTRO}}", paragrafos_da_intro),
        ("{{TOTAL_DE_PERGUNTAS}}", total_de_perguntas.to_string()),
        ("{{DADOS}}", dados_no_script),
    ];
    // Cada marca aparece uma vez no molde; o texto da rodada entra depois de achada a marca,
    // então nada que venha do JSON é confundido com uma marca.
    let mut html = String::with_capacity(MOLDE_DA_FOLHA.len() * 2);
    let mut resto = MOLDE_DA_FOLHA;
    for (marca, valor) in trocas {
        let (antes, depois) = resto.split_once(marca).expect("a marca está no molde");
        html.push_str(antes);
        html.push_str(&valor);
        resto = depois;
    }
    html.push_str(resto);
    (html, total_de_perguntas)
}

fn main() {
    let argumentos: Vec<String> = std::env::args().skip(1).collect();
    let (caminho_do_json, caminho_da_saida) = match argumentos.as_slice() {
        [json, saida, ..] if !json.is_empty() && !saida.is_empty() => (json, saida),
        _ => parar("Uso: sh ~/.claude/skills/folha-de-respostas/scripts/rodar <perguntas.json> <saida.html>"),
    };

    let bytes = std::fs::read(caminho_do_json)
        .unwrap_or_else(|erro| parar(&format!("Não deu para ler {caminho_do_json}: {erro}")));
    let rodada: Value = serde_json::from_str(&String::from_utf8_lossy(&bytes))
        .unwrap_or_else(|erro| parar(&format!("{caminho_do_json} não é um JSON válido: {erro}")));

    let problemas = problemas_da_rodada(&rodada);
    if !problemas.is_empty() {
        parar(&format!(
            "A folha não foi gerada:\n- {}",
            problemas.join("\n- ")
        ));
    }

    let (html, total_de_perguntas) = gerar_html(&rodada);
    std::fs::write(caminho_da_saida, html)
        .unwrap_or_else(|erro| parar(&format!("Não deu para gravar {caminho_da_saida}: {erro}")));
    println!("Folha gerada: {caminho_da_saida} ({total_de_perguntas} perguntas)");
}
