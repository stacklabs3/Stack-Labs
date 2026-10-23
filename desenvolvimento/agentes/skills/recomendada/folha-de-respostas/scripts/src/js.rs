//! O pedaço do JavaScript de que a folha depende, para o JSON lido valer o mesmo que valia no
//! script Node: o que conta como verdadeiro, como um valor vira texto, as comparações `<` e `>=`
//! e o `JSON.stringify` (números, ordem das chaves e escapes).
use serde_json::{Map, Value};
use std::cmp::Ordering;

/// O valor que um campo tem no JavaScript: ausente (`undefined`) ou um valor do JSON.
pub type Campo<'a> = Option<&'a Value>;

/// `objeto.nome`: só objetos têm campos vindos do JSON.
pub fn campo<'a>(valor: &'a Value, nome: &str) -> Campo<'a> {
    match valor {
        Value::Object(mapa) => mapa.get(nome),
        _ => None,
    }
}

/// `!!valor`: falso para ausente, null, false, 0 e texto vazio.
pub fn verdadeiro(valor: Campo) -> bool {
    match valor {
        None | Some(Value::Null) => false,
        Some(Value::Bool(logico)) => *logico,
        Some(Value::Number(numero)) => numero.as_f64().is_some_and(|n| n != 0.0),
        Some(Value::String(texto)) => !texto.is_empty(),
        Some(Value::Array(_)) | Some(Value::Object(_)) => true,
    }
}

/// `a || b`.
pub fn ou<'a>(primeiro: Campo<'a>, segundo: Campo<'a>) -> Campo<'a> {
    if verdadeiro(primeiro) {
        primeiro
    } else {
        segundo
    }
}

/// `String(valor)`.
pub fn texto_de(valor: Campo) -> String {
    match valor {
        None => "undefined".to_string(),
        Some(Value::Null) => "null".to_string(),
        Some(Value::Bool(logico)) => logico.to_string(),
        Some(Value::Number(numero)) => texto_do_numero(numero.as_f64().unwrap_or(f64::NAN)),
        Some(Value::String(texto)) => texto.clone(),
        Some(Value::Array(itens)) => itens
            .iter()
            .map(|item| match item {
                Value::Null => String::new(),
                outro => texto_de(Some(outro)),
            })
            .collect::<Vec<_>>()
            .join(","),
        Some(Value::Object(_)) => "[object Object]".to_string(),
    }
}

/// `Number.prototype.toString()`: os dígitos mais curtos que voltam ao mesmo número, com
/// expoente só abaixo de 1e-6 ou a partir de 1e21.
pub fn texto_do_numero(numero: f64) -> String {
    if numero.is_nan() {
        return "NaN".to_string();
    }
    if numero == 0.0 {
        return "0".to_string();
    }
    if numero.is_infinite() {
        return if numero > 0.0 {
            "Infinity"
        } else {
            "-Infinity"
        }
        .to_string();
    }
    let sinal = if numero < 0.0 { "-" } else { "" };
    let cientifico = format!("{:e}", numero.abs());
    let (mantissa, expoente) = cientifico.split_once('e').expect("formato {:e}");
    let digitos: String = mantissa.chars().filter(|c| *c != '.').collect();
    let expoente: i32 = expoente.parse().expect("expoente do {:e}");
    let quantidade = digitos.len() as i32;
    let n = expoente + 1;
    let corpo = if quantidade <= n && n <= 21 {
        format!("{digitos}{}", "0".repeat((n - quantidade) as usize))
    } else if 0 < n && n <= 21 {
        format!("{}.{}", &digitos[..n as usize], &digitos[n as usize..])
    } else if -6 < n && n <= 0 {
        format!("0.{}{digitos}", "0".repeat((-n) as usize))
    } else {
        let sinal_do_expoente = if n - 1 < 0 { '-' } else { '+' };
        let resto = if quantidade > 1 {
            format!(".{}", &digitos[1..])
        } else {
            String::new()
        };
        format!(
            "{}{resto}e{sinal_do_expoente}{}",
            &digitos[..1],
            (n - 1).abs()
        )
    };
    format!("{sinal}{corpo}")
}

/// `Number(texto)`: espaços nas pontas não contam, texto vazio é 0 e o que não é número é NaN.
fn numero_do_texto(texto: &str) -> f64 {
    let limpo = texto.trim_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}');
    if limpo.is_empty() {
        return 0.0;
    }
    for (prefixo, base) in [
        ("0x", 16),
        ("0X", 16),
        ("0o", 8),
        ("0O", 8),
        ("0b", 2),
        ("0B", 2),
    ] {
        if let Some(digitos) = limpo.strip_prefix(prefixo) {
            if digitos.is_empty() || !digitos.chars().all(|c| c.is_digit(base)) {
                return f64::NAN;
            }
            return digitos.chars().fold(0.0, |total, c| {
                total * base as f64 + c.to_digit(base).unwrap() as f64
            });
        }
    }
    let sem_sinal = limpo.strip_prefix(['+', '-']).unwrap_or(limpo);
    if sem_sinal == "Infinity" {
        return if limpo.starts_with('-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }
    if !decimal_valido(sem_sinal) {
        return f64::NAN;
    }
    limpo.parse().unwrap_or(f64::NAN)
}

/// `dígitos[.dígitos]` ou `.dígitos`, com expoente opcional.
fn decimal_valido(texto: &str) -> bool {
    let (base, expoente) = match texto.find(['e', 'E']) {
        Some(posicao) => (&texto[..posicao], Some(&texto[posicao + 1..])),
        None => (texto, None),
    };
    let (inteira, fracao) = base.split_once('.').unwrap_or((base, ""));
    let so_digitos = |parte: &str| parte.chars().all(|c| c.is_ascii_digit());
    if !so_digitos(inteira) || !so_digitos(fracao) || (inteira.is_empty() && fracao.is_empty()) {
        return false;
    }
    match expoente {
        None => true,
        Some(expoente) => {
            let digitos = expoente.strip_prefix(['+', '-']).unwrap_or(expoente);
            !digitos.is_empty() && so_digitos(digitos)
        }
    }
}

/// O valor primitivo de uma comparação: texto ou número.
enum Primitivo {
    Texto(String),
    Numero(f64),
}

fn primitivo(valor: Campo) -> Primitivo {
    match valor {
        None => Primitivo::Numero(f64::NAN),
        Some(Value::Null) => Primitivo::Numero(0.0),
        Some(Value::Bool(logico)) => Primitivo::Numero(if *logico { 1.0 } else { 0.0 }),
        Some(Value::Number(numero)) => Primitivo::Numero(numero.as_f64().unwrap_or(f64::NAN)),
        Some(outro) => Primitivo::Texto(texto_de(Some(outro))),
    }
}

fn numero_do_primitivo(valor: &Primitivo) -> f64 {
    match valor {
        Primitivo::Numero(numero) => *numero,
        Primitivo::Texto(texto) => numero_do_texto(texto),
    }
}

/// A comparação abstrata do JavaScript; `None` quando um dos lados é NaN.
fn comparar(esquerda: Campo, direita: Campo) -> Option<Ordering> {
    match (primitivo(esquerda), primitivo(direita)) {
        (Primitivo::Texto(a), Primitivo::Texto(b)) => Some(a.encode_utf16().cmp(b.encode_utf16())),
        (a, b) => numero_do_primitivo(&a).partial_cmp(&numero_do_primitivo(&b)),
    }
}

/// `esquerda < direita`.
pub fn menor(esquerda: Campo, direita: Campo) -> bool {
    comparar(esquerda, direita) == Some(Ordering::Less)
}

/// `esquerda >= direita`.
pub fn maior_ou_igual(esquerda: Campo, direita: Campo) -> bool {
    matches!(
        comparar(esquerda, direita),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// `valor.length`, para textos (em unidades UTF-16), listas e objetos com o campo `length`.
pub fn tamanho(valor: Campo) -> Option<Value> {
    match valor? {
        Value::String(texto) => Some(Value::from(texto.encode_utf16().count())),
        Value::Array(itens) => Some(Value::from(itens.len())),
        Value::Object(mapa) => mapa.get("length").cloned(),
        _ => None,
    }
}

/// A chave de um `Set` do JavaScript: valores primitivos iguais se encontram, listas e
/// objetos são sempre diferentes.
#[derive(PartialEq, Eq, Hash)]
pub enum ChaveDoConjunto {
    Ausente,
    Nulo,
    Logico(bool),
    Numero(u64),
    Texto(String),
    Unico(usize),
}

pub fn chave_do_conjunto(valor: Campo, posicao: usize) -> ChaveDoConjunto {
    match valor {
        None => ChaveDoConjunto::Ausente,
        Some(Value::Null) => ChaveDoConjunto::Nulo,
        Some(Value::Bool(logico)) => ChaveDoConjunto::Logico(*logico),
        Some(Value::Number(numero)) => {
            let numero = numero.as_f64().unwrap_or(f64::NAN);
            ChaveDoConjunto::Numero(if numero == 0.0 { 0 } else { numero.to_bits() })
        }
        Some(Value::String(texto)) => ChaveDoConjunto::Texto(texto.clone()),
        Some(_) => ChaveDoConjunto::Unico(posicao),
    }
}

/// `JSON.stringify(valor)`.
pub fn json(valor: &Value) -> String {
    let mut saida = String::new();
    escrever_json(valor, &mut saida);
    saida
}

fn escrever_json(valor: &Value, saida: &mut String) {
    match valor {
        Value::Null => saida.push_str("null"),
        Value::Bool(logico) => saida.push_str(if *logico { "true" } else { "false" }),
        Value::Number(numero) => {
            let numero = numero.as_f64().unwrap_or(f64::NAN);
            if numero.is_finite() {
                saida.push_str(&texto_do_numero(numero));
            } else {
                saida.push_str("null");
            }
        }
        Value::String(texto) => {
            saida.push_str(&serde_json::to_string(texto).expect("texto vira JSON"))
        }
        Value::Array(itens) => {
            saida.push('[');
            for (posicao, item) in itens.iter().enumerate() {
                if posicao > 0 {
                    saida.push(',');
                }
                escrever_json(item, saida);
            }
            saida.push(']');
        }
        Value::Object(mapa) => {
            saida.push('{');
            for (posicao, (chave, item)) in chaves_na_ordem_do_js(mapa).into_iter().enumerate() {
                if posicao > 0 {
                    saida.push(',');
                }
                saida.push_str(&serde_json::to_string(chave).expect("chave vira JSON"));
                saida.push(':');
                escrever_json(item, saida);
            }
            saida.push('}');
        }
    }
}

/// O JavaScript põe primeiro as chaves que são índices (`"0"`, `"12"`) em ordem crescente e
/// depois as outras na ordem em que apareceram.
fn chaves_na_ordem_do_js(mapa: &Map<String, Value>) -> Vec<(&String, &Value)> {
    let indice = |chave: &str| -> Option<u32> {
        let numero: u32 = chave.parse().ok()?;
        (numero != u32::MAX && numero.to_string() == chave).then_some(numero)
    };
    let mut indices: Vec<_> = mapa
        .iter()
        .filter_map(|(chave, item)| indice(chave).map(|numero| (numero, chave, item)))
        .collect();
    indices.sort_by_key(|(numero, _, _)| *numero);
    indices
        .into_iter()
        .map(|(_, chave, item)| (chave, item))
        .chain(mapa.iter().filter(|(chave, _)| indice(chave).is_none()))
        .collect()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn numeros_saem_como_no_javascript() {
        let casos = [
            (1.0, "1"),
            (100.0, "100"),
            (1.5e300, "1.5e+300"),
            (0.000001, "0.000001"),
            (1e-7, "1e-7"),
            (1.2345678901234569e23, "1.2345678901234569e+23"),
            (-0.0, "0"),
            (0.1, "0.1"),
            (2.5e-5, "0.000025"),
            (1e21, "1e+21"),
            (1e20, "100000000000000000000"),
            (-12.5, "-12.5"),
            (123.456, "123.456"),
        ];
        for (numero, esperado) in casos {
            assert_eq!(texto_do_numero(numero), esperado, "{numero}");
        }
    }

    #[test]
    fn texto_vira_numero_como_no_javascript() {
        assert_eq!(numero_do_texto(" 2 "), 2.0);
        assert_eq!(numero_do_texto(""), 0.0);
        assert_eq!(numero_do_texto("0x10"), 16.0);
        assert_eq!(numero_do_texto(".5"), 0.5);
        assert_eq!(numero_do_texto("-1e2"), -100.0);
        assert!(numero_do_texto("inf").is_nan());
        assert!(numero_do_texto("1x").is_nan());
        assert_eq!(numero_do_texto("-Infinity"), f64::NEG_INFINITY);
    }

    #[test]
    fn chaves_de_indice_vem_primeiro() {
        let valor: Value =
            serde_json::from_str(r#"{"b":1,"2":2,"a":3,"1":4,"01":5,"-1":6}"#).unwrap();
        assert_eq!(json(&valor), r#"{"1":4,"2":2,"b":1,"a":3,"01":5,"-1":6}"#);
    }
}
